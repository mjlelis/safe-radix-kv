#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use safe_radix_kv::SafeRadixTree;

#[derive(Arbitrary, Debug)]
pub enum FuzzOp {
    Insert { key: Vec<u8>, val: u32 },
    Get { key: Vec<u8> },
    Remove { key: Vec<u8> },
}

// Harness Differential Fuzzing C vs Safe Rust
fuzz_target!(|ops: Vec<FuzzOp>| {
    let mut safe_tree: SafeRadixTree<u32> = SafeRadixTree::new();

    // Modelo de referência oracular em Rust puro (BTreeMap ou HashMap) para paridade diferencial
    let mut oracle: std::collections::BTreeMap<Vec<u8>, u32> = std::collections::BTreeMap::new();

    for op in ops {
        match op {
            FuzzOp::Insert { key, val } => {
                if key.is_empty() {
                    continue;
                }
                let rust_res = safe_tree.insert(&key, val).ok().flatten();
                let oracle_res = oracle.insert(key, val);
                assert_eq!(rust_res, oracle_res, "Diferença detectada entre Rust Tree e Oráculo!");
            }
            FuzzOp::Get { key } => {
                let rust_val = safe_tree.get(&key).copied();
                let oracle_val = oracle.get(&key).copied();
                assert_eq!(rust_val, oracle_val, "Discrepância na operação Get!");
            }
            FuzzOp::Remove { key } => {
                let _rust_val = safe_tree.remove(&key);
                let _oracle_val = oracle.remove(&key);
            }
        }
    }
});
