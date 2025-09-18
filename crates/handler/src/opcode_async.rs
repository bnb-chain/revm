use crate::opcode_cache::OpCodeCache;
use crate::opcode_optimizer::do_basic_block_opcode_fusion;
use bytecode::Bytecode;
use once_cell::sync::Lazy;
use primitives::{B256, Bytes};
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Clone, Copy)]
enum OptimizeTaskType {
    Generate,
    // Delete,
}

/// static CODE_FUSION_TX: Lazy<Sender<(B256, Bytecode)>> = Lazy::new(|| {
/// let (tx, tx) = mpsc::channel::<(B256, Bytecode)>()})
static CODE_FUSION_TX: Lazy<Sender<(OptimizeTaskType, B256, Bytes)>> = Lazy::new(|| {
    let (tx, rx) = mpsc::channel::<(OptimizeTaskType, B256, Bytes)>();
    thread::Builder::new()
        .name("opcode_fusion_worker".into())
        .spawn(move || {
            while let Ok((typ, hash, code)) = rx.recv() {
                match typ {
                    OptimizeTaskType::Generate => {
                        // Check cache first, only optimize if not cached
                        if let Some(_) = OpCodeCache::get(&hash) {
                            tracing::debug!(
                                target: "revm::superinstructions",
                                "SI already cached, skipping: {}", hash
                            );
                            continue;
                        }

                        // Perform optimization for uncached bytecode
                        match do_basic_block_opcode_fusion(&code) {
                            Ok(fused_vec) => {
                                let fused = Bytecode::new_raw(Bytes::from(fused_vec));

                                tracing::debug!(
                                    target: "revm::superinstructions",
                                    "SI optimized: {}", hash
                                );
                                OpCodeCache::insert(&hash, fused);
                            },
                            Err(_) => {
                                tracing::debug!(
                                    target: "revm::superinstructions",
                                    "SI optimization failed: {}", hash
                                );
                            },
                        }
                    }
                    // OptimizeTaskType::Delete => {
                    //     OpCodeCache::remove(&hash);
                    // }
                }
            }
        })
        .expect("spawn fusion worker");
    tx
});

// Try to fetch from the cache; if it misses, submit the task to the background thread
// asynchronously and return the original code immediately.
pub(crate) fn gen_or_rewrite_optimized_code(hash: &B256, code: Bytecode) -> (Bytecode, bool) {
    // Try to get optimized bytecode from cache first
    if let Some(bytecode) = OpCodeCache::get(hash) {
        tracing::debug!(
            target: "revm::superinstructions",
            "SI cache hit: {}", hash
        );
        (bytecode, true)
    } else {
        // Cache miss: perform synchronous optimization to ensure consistency
        tracing::debug!(
            target: "revm::superinstructions",
            "SI cache miss, performing sync optimization: {}", hash
        );

        match do_basic_block_opcode_fusion(&code.bytes()) {
            Ok(fused_vec) => {
                let fused = Bytecode::new_raw(Bytes::from(fused_vec));
                tracing::debug!(
                    target: "revm::superinstructions",
                    "SI sync optimization successful: {}", hash
                );
                OpCodeCache::insert(hash, fused.clone());
                (fused, true)
            },
            Err(_) => {
                tracing::debug!(
                    target: "revm::superinstructions",
                    "SI sync optimization failed, using original: {}", hash
                );
                (code, false)
            }
        }
    }
}
