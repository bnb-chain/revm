pub use alloy_eip7702::{
    Authorization, RecoveredAuthority, RecoveredAuthorization, SignedAuthorization,
};
pub use alloy_primitives::{Parity, Signature};

use std::{boxed::Box, vec::Vec};

/// Authorization list for EIP-7702 transaction type.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AuthorizationList {
    Signed(Vec<SignedAuthorization>),
    Recovered(Vec<RecoveredAuthorization>),
}

impl From<Vec<SignedAuthorization>> for AuthorizationList {
    fn from(signed: Vec<SignedAuthorization>) -> Self {
        Self::Signed(signed)
    }
}

impl From<Vec<RecoveredAuthorization>> for AuthorizationList {
    fn from(recovered: Vec<RecoveredAuthorization>) -> Self {
        Self::Recovered(recovered)
    }
}

impl AuthorizationList {
    /// Returns length of the authorization list.
    pub fn len(&self) -> usize {
        match self {
            Self::Signed(signed) => signed.len(),
            Self::Recovered(recovered) => recovered.len(),
        }
    }

    /// Return empty authorization list.
    pub fn empty() -> Self {
        Self::Recovered(Vec::new())
    }

    /// Returns true if the authorization list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns iterator of recovered Authorizations.
    pub fn recovered_iter<'a>(&'a self) -> Box<dyn Iterator<Item = RecoveredAuthorization> + 'a> {
        match self {
            Self::Signed(signed) => Box::new(signed.iter().map(|signed| signed.clone().into())),
            Self::Recovered(recovered) => Box::new(recovered.clone().into_iter()),
        }
    }

    /// Returns recovered authorizations list.
    pub fn into_recovered(self) -> Self {
        let Self::Signed(signed) = self else {
            return self;
        };
        Self::Recovered(signed.into_iter().map(|signed| signed.into()).collect())
    }
}
