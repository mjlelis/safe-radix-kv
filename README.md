# safe-radix-kv

Port and formal verification of Redis's production Radix Tree (**Rax**, Salvatore Sanfilippo) to 100% Safe Rust (`#![forbid(unsafe_code)]`), validated with **Miri** and **Differential Fuzzing**.

[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![Safety](https://img.shields.io/badge/unsafe_code-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)
[![Miri](https://img.shields.io/badge/verified_by-Miri-brightgreen.svg)](https://github.com/rust-lang/miri)

## Overview

Complex pointer-heavy C data structures utilizing manual allocation (`malloc`/`free`), bitfields, and flexible array members are notoriously difficult to port directly to idiomatic Rust. When transpiled through automatic tools like **C2Rust**, the resulting code consists entirely of `unsafe` blocks with unverified pointer aliasing.

`safe-radix-kv` demonstrates the complete end-to-end **lifting** pipeline:

1. **Production C Reference:** The official Redis Rax tree (`rax.c` / `rax.h`), compiled with AddressSanitizer.
2. **C2Rust Intermediate State:** Raw transpilation snapshot (`rust_port/src/c2rust_raw.rs`) showcasing legacy pointer constructs.
3. **Lifted Safe Rust Architecture:** Full rewrite into safe Rust (`rust_port/src/lib.rs`):
   - Nullable pointers (`*mut raxNode`) $\rightarrow$ `Option<Box<RadixNode<V>>>`
   - Bitfield discriminants $\rightarrow$ Algebraic enums (`NodeKind<V>`)
   - Reallocated edge arrays $\rightarrow$ `Vec<ChildEdge<V>>`
   - Integer status codes $\rightarrow$ `Result<Option<V>, RadixError>` with `thiserror`
4. **Miri Verification:** Audited under Rust's Miri virtual machine for Stacked Borrows / Tree Borrows compliance.
5. **Differential Fuzzing:** Continuous libFuzzer target validating operational equivalence between the C implementation and Safe Rust.

## Repository Architecture

```text
safe-radix-kv/
├── Dockerfile                  # Clang, Bear, C2Rust, Rust Nightly + Miri
├── c_reference/
│   ├── Makefile                # Clang build with ASan
│   ├── rax.h                   # Redis Rax header
│   ├── rax.c                   # Redis Rax implementation
│   ├── rax_malloc.h            # Memory allocator wrapper
│   └── main_test.c             # C test harness
├── rust_port/
│   ├── Cargo.toml
│   └── src/
│       ├── c2rust_raw.rs       # Unsafe transpilation baseline
│       └── lib.rs              # 100% Safe Rust lifted library (#![forbid(unsafe_code)])
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        └── diff_c_vs_rust.rs   # Differential fuzzing harness
```

## Quality Gates & Verification

### Safe Rust Unit Tests
```bash
cd rust_port
cargo test
```

### Miri Undefined Behavior Audit
```bash
cd rust_port
cargo +nightly miri test
```

### Differential Fuzzing (C vs Safe Rust)
```bash
cd fuzz
cargo fuzz run diff_c_vs_rust
```
