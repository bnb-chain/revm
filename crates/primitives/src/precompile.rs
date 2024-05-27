use crate::{Bytes, Env};
use core::fmt;
use dyn_clone::DynClone;
use std::{boxed::Box, string::String, sync::Arc};

/// A precompile operation result.
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<(u64, Bytes), PrecompileError>;

pub type StandardPrecompileFn = fn(&Bytes, u64) -> PrecompileResult;
pub type EnvPrecompileFn = fn(&Bytes, u64, env: &Env) -> PrecompileResult;

/// Stateful precompile trait. It is used to create
/// a arc precompile Precompile::Stateful.
pub trait StatefulPrecompile: Sync + Send {
    fn call(&self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult;
}

/// Mutable stateful precompile trait. It is used to create
/// a boxed precompile in Precompile::StatefulMut.
pub trait StatefulPrecompileMut: DynClone + Send + Sync {
    fn call_mut(&mut self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult;
}

dyn_clone::clone_trait_object!(StatefulPrecompileMut);

/// Arc over stateful precompile.
pub type StatefulPrecompileArc = Arc<dyn StatefulPrecompile>;

/// Box over mutable stateful precompile
pub type StatefulPrecompileBox = Box<dyn StatefulPrecompileMut>;

/// Precompile and its handlers.
#[derive(Clone)]
pub enum Precompile {
    /// Standard simple precompile that takes input and gas limit.
    Standard(StandardPrecompileFn),
    /// Similar to Standard but takes reference to environment.
    Env(EnvPrecompileFn),
    /// Stateful precompile that is Arc over [`StatefulPrecompile`] trait.
    /// It takes a reference to input, gas limit and environment.
    Stateful(StatefulPrecompileArc),
    /// Mutable stateful precompile that is Box over [`StatefulPrecompileMut`] trait.
    /// It takes a reference to input, gas limit and environment.
    StatefulMut(StatefulPrecompileBox),
}

impl From<StandardPrecompileFn> for Precompile {
    fn from(p: StandardPrecompileFn) -> Self {
        Precompile::Standard(p)
    }
}

impl From<EnvPrecompileFn> for Precompile {
    fn from(p: EnvPrecompileFn) -> Self {
        Precompile::Env(p)
    }
}

impl From<StatefulPrecompileArc> for Precompile {
    fn from(p: StatefulPrecompileArc) -> Self {
        Precompile::Stateful(p)
    }
}

impl From<StatefulPrecompileBox> for Precompile {
    fn from(p: StatefulPrecompileBox) -> Self {
        Precompile::StatefulMut(p)
    }
}

impl fmt::Debug for Precompile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precompile::Standard(_) => f.write_str("Standard"),
            Precompile::Env(_) => f.write_str("Env"),
            Precompile::Stateful(_) => f.write_str("Stateful"),
            Precompile::StatefulMut(_) => f.write_str("StatefulMut"),
        }
    }
}

impl Precompile {
    /// Create a new stateful precompile.
    pub fn new_stateful<P: StatefulPrecompile + 'static>(p: P) -> Self {
        Self::Stateful(Arc::new(p))
    }

    /// Create a new mutable stateful precompile.
    pub fn new_stateful_mut<P: StatefulPrecompileMut + 'static>(p: P) -> Self {
        Self::StatefulMut(Box::new(p))
    }

    /// Call the precompile with the given input and gas limit and return the result.
    pub fn call(&mut self, bytes: &Bytes, gas_price: u64, env: &Env) -> PrecompileResult {
        match self {
            Precompile::Standard(p) => p(bytes, gas_price),
            Precompile::Env(p) => p(bytes, gas_price, env),
            Precompile::Stateful(p) => p.call(bytes, gas_price, env),
            Precompile::StatefulMut(p) => p.call_mut(bytes, gas_price, env),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileError {
    /// out of gas is the main error. Others are here just for completeness
    OutOfGas,
    // Blake2 errors
    Blake2WrongLength,
    Blake2WrongFinalIndicatorFlag,
    // Modexp errors
    ModexpExpOverflow,
    ModexpBaseOverflow,
    ModexpModOverflow,
    // Bn128 errors
    Bn128FieldPointNotAMember,
    Bn128AffineGFailedToCreate,
    Bn128PairLength,
    // Blob errors
    /// The input length is not exactly 192 bytes.
    BlobInvalidInputLength,
    /// The commitment does not match the versioned hash.
    BlobMismatchedVersion,
    /// The proof verification failed.
    BlobVerifyKzgProofFailed,
    /// The input length is not matching the expected length.
    BLSInvalidInputLength,
    /// The bls signature is invalid.
    BLSInvalidSignature,
    /// The bls public key is invalid.
    BLSInvalidPublicKey,
    /// The cometbft validation input is invalid.
    CometBftInvalidInput,
    /// The cometbft apply block failed.
    CometBftApplyBlockFailed,
    /// The cometbft consensus state encoding failed.
    CometBftEncodeConsensusStateFailed,
    /// Catch-all variant for other errors.
    Other(String),
}

impl PrecompileError {
    pub fn other(err: impl Into<String>) -> Self {
        Self::Other(err.into())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PrecompileError {}

impl fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::OutOfGas => "out of gas",
            Self::Blake2WrongLength => "wrong input length for blake2",
            Self::Blake2WrongFinalIndicatorFlag => "wrong final indicator flag for blake2",
            Self::ModexpExpOverflow => "modexp exp overflow",
            Self::ModexpBaseOverflow => "modexp base overflow",
            Self::ModexpModOverflow => "modexp mod overflow",
            Self::Bn128FieldPointNotAMember => "field point not a member of bn128 curve",
            Self::Bn128AffineGFailedToCreate => "failed to create affine g point for bn128 curve",
            Self::Bn128PairLength => "bn128 invalid pair length",
            Self::BlobInvalidInputLength => "invalid blob input length",
            Self::BlobMismatchedVersion => "mismatched blob version",
            Self::BlobVerifyKzgProofFailed => "verifying blob kzg proof failed",
            Self::BLSInvalidInputLength => "invalid input length for BLS",
            Self::BLSInvalidSignature => "invalid BLS signature",
            Self::BLSInvalidPublicKey => "invalid BLS public key",
            Self::CometBftInvalidInput => "invalid cometbft light block validation input",
            Self::CometBftApplyBlockFailed => "failed to apply cometbft block",
            Self::CometBftEncodeConsensusStateFailed => "failed to encode cometbft consensus state",
            Self::Other(s) => s,
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stateful_precompile_mut() {
        #[derive(Default, Clone)]
        struct MyPrecompile {}

        impl StatefulPrecompileMut for MyPrecompile {
            fn call_mut(
                &mut self,
                _bytes: &Bytes,
                _gas_price: u64,
                _env: &Env,
            ) -> PrecompileResult {
                PrecompileResult::Err(PrecompileError::OutOfGas)
            }
        }

        let mut p = Precompile::new_stateful_mut(MyPrecompile::default());
        match &mut p {
            Precompile::StatefulMut(p) => {
                let _ = p.call_mut(&Bytes::new(), 0, &Env::default());
            }
            _ => panic!("not a state"),
        }
    }
}
