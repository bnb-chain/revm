pub mod eof;
pub mod legacy;

use eof::EofDecodeError;
pub use eof::{Eof, EOF_MAGIC, EOF_MAGIC_BYTES, EOF_MAGIC_HASH};
pub use legacy::{JumpTable, LegacyAnalyzedBytecode};
use std::sync::Arc;

use crate::{keccak256, Bytes, B256, KECCAK_EMPTY};

/// State of the [`Bytecode`] analysis.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bytecode {
    /// No analysis has been performed.
    LegacyRaw(Bytes),
    /// The bytecode has been analyzed for valid jump destinations.
    LegacyAnalyzed(LegacyAnalyzedBytecode),
    /// Ethereum Object Format
    Eof(Arc<Eof>),
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        // Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
        Self::new()
    }
}

impl Bytecode {
    // Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::default())
    }

    /// Return jump table if bytecode is analyzed
    #[inline]
    pub fn legacy_jump_table(&self) -> Option<&JumpTable> {
        match &self {
            Self::LegacyAnalyzed(analyzed) => Some(analyzed.jump_table()),
            _ => None,
        }
    }

    /// Calculate hash of the bytecode.
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(self.original_byte_slice())
        }
    }

    /// Return reference to the EOF if bytecode is EOF.
    #[inline]
    pub const fn eof(&self) -> Option<&Arc<Eof>> {
        match self {
            Self::Eof(eof) => Some(eof),
            _ => None,
        }
    }

    /// Return true if bytecode is EOF.
    #[inline]
    pub const fn is_eof(&self) -> bool {
        matches!(self, Self::Eof(_))
    }

    /// Creates a new legacy [`Bytecode`].
    #[inline]
    pub fn new_legacy(raw: Bytes) -> Self {
        Self::LegacyRaw(raw)
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// # Panics
    ///
    /// Panics if bytecode is EOF and has incorrect format.
    #[inline]
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self::new_raw_checked(bytecode).expect("Expect correct EOF bytecode")
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// Returns an error on incorrect EOF format.
    #[inline]
    pub fn new_raw_checked(bytecode: Bytes) -> Result<Self, EofDecodeError> {
        if bytecode.starts_with(&EOF_MAGIC_BYTES) {
            Ok(Self::Eof(Arc::new(Eof::decode(bytecode)?)))
        } else {
            Ok(Self::LegacyRaw(bytecode))
        }
    }

    /// Create new checked bytecode.
    ///
    /// # Safety
    ///
    /// Bytecode needs to end with STOP (0x00) opcode as checked bytecode assumes
    /// that it is safe to iterate over bytecode without checking lengths.
    pub unsafe fn new_analyzed(
        bytecode: Bytes,
        original_len: usize,
        jump_table: JumpTable,
    ) -> Self {
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::new(
            bytecode,
            original_len,
            jump_table,
        ))
    }

    /// Returns a reference to the bytecode.
    ///
    /// In case of EOF this will be the first code section.
    #[inline]
    pub fn bytecode(&self) -> &Bytes {
        match self {
            Self::LegacyRaw(bytes) => bytes,
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            Self::Eof(eof) => eof
                .body
                .code(0)
                .expect("Valid EOF has at least one code section"),
        }
    }

    /// Returns false if bytecode can't be executed in Interpreter.
    pub fn is_execution_ready(&self) -> bool {
        !matches!(self, Self::LegacyRaw(_))
    }

    /// Returns bytes
    #[inline]
    pub fn bytes(&self) -> Bytes {
        match self {
            Self::LegacyRaw(bytes) => bytes.clone(),
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode().clone(),
            Self::Eof(eof) => eof.raw().clone(),
        }
    }

    /// Returns bytes slice
    #[inline]
    pub fn bytes_slice(&self) -> &[u8] {
        match self {
            Self::LegacyRaw(bytes) => bytes,
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            Self::Eof(eof) => eof.raw(),
        }
    }

    /// Returns a reference to the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self {
            Self::LegacyRaw(bytes) => bytes.clone(),
            Self::LegacyAnalyzed(analyzed) => analyzed.original_bytes(),
            Self::Eof(eof) => eof.raw().clone(),
        }
    }

    /// Returns the original bytecode as a byte slice.
    #[inline]
    pub fn original_byte_slice(&self) -> &[u8] {
        match self {
            Self::LegacyRaw(bytes) => bytes,
            Self::LegacyAnalyzed(analyzed) => analyzed.original_byte_slice(),
            Self::Eof(eof) => eof.raw(),
        }
    }

    /// Returns the length of the raw bytes.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::LegacyRaw(bytes) => bytes.len(),
            Self::LegacyAnalyzed(analyzed) => analyzed.original_len(),
            Self::Eof(eof) => eof.size(),
        }
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn eof_arc_clone() {
        let eof = Arc::new(Eof::default());
        let bytecode = Bytecode::Eof(Arc::clone(&eof));

        // Cloning the Bytecode should not clone the underlying Eof
        let cloned_bytecode = bytecode.clone();
        if let Bytecode::Eof(original_arc) = bytecode {
            if let Bytecode::Eof(cloned_arc) = cloned_bytecode {
                assert!(Arc::ptr_eq(&original_arc, &cloned_arc));
            } else {
                panic!("Cloned bytecode is not Eof");
            }
        } else {
            panic!("Original bytecode is not Eof");
        }
    }
}
