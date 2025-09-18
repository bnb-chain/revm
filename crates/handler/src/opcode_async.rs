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
                        if let Some(_) = OpCodeCache::get(&hash) {
                            continue;
                        }
                        match do_basic_block_opcode_fusion(&code) {
                            Ok(fused_vec) => {
                                let fused = Bytecode::new_raw(Bytes::from(fused_vec));
                                
                                // 只记录关键信息：成功优化并缓存
                                tracing::debug!(
                                    target: "revm::superinstructions",
                                    "SI optimized: {}", hash
                                );
                                OpCodeCache::insert(&hash, fused);
                            },
                            Err(_) => {},
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
    // 尝试从缓存中获取优化的字节码
    if let Some(bytecode) = OpCodeCache::get(hash) {
        // 这是关键路径，记录缓存命中信息
        tracing::debug!(
            target: "revm::superinstructions",
            "SI cache hit: {}", hash
        );
        (bytecode, true)
    } else {
        // 异步提交字节码优化任务
        let _ = CODE_FUSION_TX.send((OptimizeTaskType::Generate, hash.clone(), code.bytes()));
        (code, false)
    }
}
