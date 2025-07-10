use once_cell::sync::Lazy;
use parking_lot::RwLock;
use lru::LruCache;
use primitives::B256;
use bytecode::Bytecode;
use std::num::NonZeroUsize;

const MAX_CACHE_SIZE: usize = 1024 * 128;

/// global opcode cache (LRU, cap 1024)
static OPCODE_CACHE: Lazy<RwLock<LruCache<B256, Bytecode>>> = Lazy::new(|| {
    const CAP: usize = MAX_CACHE_SIZE;
    RwLock::new(LruCache::new(NonZeroUsize::new(CAP).expect("non-zero")))
});

/// simple interface for outer use
pub(crate) struct OpCodeCache;

impl OpCodeCache {
    /// fetch bytecode by code_hash, return `OpCodeCacheError::NotFound` if not exist
    pub(crate) fn get(key: &B256) -> Option<Bytecode> {
        let mut guard = OPCODE_CACHE.write();
        guard
            .get(key)
            .cloned()
    }

    /// insert to update
    pub(crate) fn insert(key: &B256, value: Bytecode) {
        let mut guard = OPCODE_CACHE.write();
        guard.put(*key, value);
    }

    // /// delete
    // pub(crate) fn remove(key: &B256) {
    //     let mut guard = OPCODE_CACHE.write();
    //     guard.pop(key);
    // }
}
