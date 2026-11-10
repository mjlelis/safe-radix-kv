# safe-radix-kv

Port and formal verification of Redis's production Radix Tree (**Rax**, Salvatore Sanfilippo) to 100% Safe Rust (`#![forbid(unsafe_code)]`), validated with **Miri** and **Differential Fuzzing**.

[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Safety](https://img.shields.io/badge/unsafe_code-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)
[![Miri](https://img.shields.io/badge/verified_by-Miri-brightgreen.svg)](https://github.com/rust-lang/miri)
[![Fuzzer](https://img.shields.io/badge/differential_fuzzing-libFuzzer-blueviolet.svg)](https://github.com/rust-fuzz/cargo-fuzz)

---

## Table of Contents
- [Executive Summary & Problem Statement](#executive-summary--problem-statement)
- [Real-World Production Context (AI & In-Memory Systems)](#real-world-production-context-ai--in-memory-systems)
- [The C2Rust Dilemma & The Lifting Pipeline](#the-c2rust-dilemma--the-lifting-pipeline)
- [Structural Lifting: From C Pointers to Safe Rust](#structural-lifting-from-c-pointers-to-safe-rust)
- [Verification & Quality Architecture](#verification--quality-architecture)
  - [Miri Formal Verification (Stacked Borrows)](#miri-formal-verification-stacked-borrows)
  - [Differential Fuzzing (C ASan vs Safe Rust)](#differential-fuzzing-c-asan-vs-safe-rust)
- [Repository Structure](#repository-structure)
- [Getting Started & Verification](#getting-started--verification)
  - [Building and Running Safe Rust Tests](#building-and-running-safe-rust-tests)
  - [Running Miri Undefined Behavior Audit](#running-miri-undefined-behavior-audit)
  - [Running Differential Fuzzing](#running-differential-fuzzing)

---

## Executive Summary & Problem Statement

Radix Trees (compact prefix tries) are the foundational indexing structures behind in-memory databases, lexical search engines, and LLM inference engines. Salvatore Sanfilippo's **`rax`** is the battle-tested ANSI C radix tree powering Redis streams, cluster routing, and command dispatching. 

While extraordinarily memory-efficient and fast, `rax.c` achieves performance through techniques that are notorious vectors for memory corruption:
- **Bitfield-packed node headers** (`iskey:1`, `isnull:1`, `iscompr:1`, `size:29`).
- **Flexible array members** storing dynamically sized arrays of characters followed directly by unaligned pointer tables.
- **Manual dynamic reallocations** (`malloc`, `realloc`, `free`) with pointer arithmetic and `void*` payloads.

Converting such pointer-dense, self-referential structures to Rust is considered one of the hardest challenges in systems engineering (the central goal of initiatives like DARPA TRACTOR).

---

## Real-World Production Context (AI & In-Memory Systems)

This architecture directly addresses two core production workloads in state-of-the-art AI infrastructure:

1. **Hybrid Search Engines (BM25 + Dense Vectors):** Vector databases like **Qdrant** and **Vespa** combine approximate nearest neighbor graphs (HNSW) with memory-resident inverted indexes. Radix trees provide sub-millisecond prefix lookup for sparse token vocabularies.
2. **LLM KV-Cache & RadixAttention:** Advanced LLM serving engines (such as **vLLM** and **SGLang**) use radix trees to manage token KV-cache hierarchies across concurrent requests. Common prefix trees allow instant context reuse across system prompts and conversation branches.

---

## The C2Rust Dilemma & The Lifting Pipeline

Automated transpilers like `c2rust` mechanically map C syntax to Rust, but produce code that is **100% `unsafe`**, full of raw pointers (`*mut T`), `libc::malloc`, and explicit pointer casts. This provides zero memory safety benefits.

`safe-radix-kv` showcases the end-to-end **Lifting Pipeline**: translating transpiled unsafe code into idiomatic, compile-time verified Safe Rust under `#![forbid(unsafe_code)]`.

```mermaid
flowchart TD
    subgraph Stage1["1. Ground Truth (C Baseline)"]
        CRef["c_reference/rax.c<br/>(Redis Production Source)"]
        CASan["Clang + AddressSanitizer<br/>(-fsanitize=address,undefined)"]
        CRef --> CASan
    end

    subgraph Stage2["2. AST Interception & Transpilation"]
        Bear["bear -- make<br/>(compile_commands.json)"]
        C2Rust["c2rust transpile"]
        RawAST["c2rust_raw.rs<br/>(100% Unsafe Rust AST)"]
        CASan --> Bear
        Bear --> C2Rust
        C2Rust --> RawAST
    end

    subgraph Stage3["3. Structural Lifting (Safe Architecture)"]
        LiftTypes["Deconstruct Unions & Pointers<br/>*mut raxNode -> Option Box RadixNode<br/>Bitfield Flags -> Algebraic Enum NodeKind"]
        SafeCore["rust_port/src/lib.rs<br/>#![forbid(unsafe_code)]"]
        RawAST --> LiftTypes
        LiftTypes --> SafeCore
    end

    subgraph Stage4["4. Formal Verification & Fuzzing"]
        Miri["Miri VM Audit<br/>(Stacked Borrows & Aliasing)"]
        DiffFuzz["cargo-fuzz / libFuzzer<br/>(C ASan vs Safe Rust)"]
        SafeCore --> Miri
        SafeCore --> DiffFuzz
        CASan --> DiffFuzz
    end
```

---

## Structural Lifting: From C Pointers to Safe Rust

The table below contrasts the low-level C memory layout of Redis's `rax` against the lifted Safe Rust design:

| Redis `rax` (C) | Lifted `SafeRadixTree` (Safe Rust) | Architectural Advantage |
| :--- | :--- | :--- |
| `raxNode *` (Raw nullable pointer) | `Option<Box<RadixNode<V>>>` | Compile-time guaranteed null safety; automatic recursive RAII deallocation without `free()`. |
| Bitfields: `iskey:1, isnull:1, iscompr:1` | `enum NodeKind<V> { Leaf { value }, Internal { edges } }` | Impossible to represent invalid states; discriminated by compiler-checked algebraic types. |
| Flexible array: `raxEdge*` with `realloc()` | `Vec<ChildEdge<V>>` | Bounds-checked dynamic growth without buffer overflow risk or memory fragmentation. |
| Status codes: `1 (ok), 0 (oom), NULL (not found)` | `Result<Option<V>, RadixError>` | Explicit domain error modeling using `thiserror`. |
| Pointer casts: `(void*)data` | Generic type parameter `V` | Type-safe value storage with compile-time monomorphization and zero runtime casting overhead. |

### Memory Representation Comparison

```mermaid
flowchart LR
    subgraph CLayout["Redis Rax Node (C Memory Layout)"]
        direction TB
        CHeader["Header: flags_and_size (u32)<br/>[iskey:1][isnull:1][iscompr:1][size:29]"]
        CData["data[] (Flexible Array Member)"]
        CPtrs["Pointers: void* value, raxNode* children[]"]
        CHeader --- CData --- CPtrs
    end

    subgraph RustLayout["Lifted SafeRadixTree (Safe Rust Layout)"]
        direction TB
        RNode["struct RadixNode&lt;V&gt; {<br/>  prefix: Vec&lt;u8&gt;,<br/>  kind: NodeKind&lt;V&gt;<br/>}"]
        REnum["enum NodeKind&lt;V&gt; {<br/>  Leaf { value: V },<br/>  Internal { edges: Vec&lt;ChildEdge&lt;V&gt;&gt; }<br/>}"]
        RBox["struct ChildEdge&lt;V&gt; {<br/>  byte_key: u8,<br/>  node: Box&lt;RadixNode&lt;V&gt;&gt;<br/>}"]
        RNode --> REnum
        REnum --> RBox
    end
```

---

## Verification & Quality Architecture

Migrating mission-critical infrastructure demands formal proof that the new implementation introduces no regressions, memory leaks, or subtle semantic shifts.

### 1. Miri Formal Verification (Stacked Borrows)
Running `cargo +nightly miri test` executes the test suite inside an interpreter that tracks the provenance and allocation boundaries of every single byte:
- Proves the absolute absence of **Undefined Behavior (UB)**.
- Validates that the tree traversal satisfies Rust's strict **Stacked Borrows** aliasing model.
- Verifies that recursive node deallocations leave zero memory leaks.

### 2. Differential Fuzzing (C ASan vs Safe Rust)

A continuous fuzzing harness (`fuzz/fuzz_targets/diff_c_vs_rust.rs`) submits randomized operation streams simultaneously to both implementations:

```mermaid
sequenceDiagram
    autonumber
    participant Fuzzer as libFuzzer Engine
    participant Generator as Arbitrary FuzzOp Stream
    participant CRax as Redis C Rax (AddressSanitizer)
    participant RustTree as Lifted SafeRadixTree (Safe Rust)

    Fuzzer->>Generator: Generate mutated bytecode
    Generator-->>Fuzzer: Vec<FuzzOp> [Insert, Get, Remove]

    loop For each FuzzOp in sequence
        Fuzzer->>CRax: Apply Op (raxInsert / raxFind / raxRemove)
        CRax-->>Fuzzer: Return c_status, c_val
        Fuzzer->>RustTree: Apply Op (insert / get / remove)
        RustTree-->>Fuzzer: Return rust_status, rust_val

        Note over Fuzzer: Differential Assertion
        Fuzzer->>Fuzzer: assert_eq!(c_val, rust_val)
        Fuzzer->>Fuzzer: assert_eq!(c_status, rust_status)
    end
```

Any discrepancy in prefix splitting, compressed node handling, or memory access triggers an immediate reproducible crash artifact.

---

## Repository Structure

```text
safe-radix-kv/
├── Dockerfile                  # Clang, Bear, C2Rust, Rust Nightly, Miri, Cargo-Fuzz
├── c_reference/
│   ├── Makefile                # Build with AddressSanitizer and UndefinedBehaviorSanitizer
│   ├── rax.h                   # Official Redis Rax header
│   ├── rax.c                   # Official Redis Rax implementation
│   ├── rax_malloc.h            # Memory allocator abstraction
│   └── main_test.c             # C integration test suite
├── rust_port/
│   ├── Cargo.toml              # Rust library configuration
│   └── src/
│       ├── c2rust_raw.rs       # Unsafe transpilation baseline (historical reference)
│       └── lib.rs              # Lifted 100% Safe Rust Radix Tree (#![forbid(unsafe_code)])
└── fuzz/
    ├── Cargo.toml              # cargo-fuzz & libfuzzer-sys configuration
    └── fuzz_targets/
        └── diff_c_vs_rust.rs   # Differential fuzzing harness (C vs Safe Rust)
```

---

## Getting Started & Verification

### Prerequisites
- Clang / GCC with AddressSanitizer support
- Rust Nightly (`rustup default nightly && rustup component add miri`)
- `cargo-fuzz` (`cargo install cargo-fuzz`)

### Building and Running Safe Rust Tests
```bash
cd rust_port
cargo test
```

### Running Miri Undefined Behavior Audit
```bash
cd rust_port
cargo +nightly miri test
```

### Running Differential Fuzzing
```bash
cd fuzz
cargo fuzz run diff_c_vs_rust -- -max_total_time=300
```
