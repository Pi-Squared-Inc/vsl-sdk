pub mod rpc_messages;
pub mod rpc_wrapper;

mod helpers;
mod timestamp;

use std::{fmt::Display, ops::Mul};

pub use crate::helpers::{HasSender, IntoSigned};
pub use alloy::primitives::{Address, B256, wrap_fixed_bytes};
use alloy_rlp::{RlpDecodable, RlpEncodable};
use derive_more::derive;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use timestamp::Timestamp;

#[repr(transparent)]
#[derive(
    Clone,
    Copy,
    derive::From,
    derive::Into,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive::Add,
    derive::Sub,
    Debug,
    derive::Display,
    derive::FromStr,
    derive::LowerHex,
    derive::UpperHex,
    Serialize,
    Deserialize,
    RlpEncodable,
    RlpDecodable,
    JsonSchema,
)]
pub struct Amount(u128);

const ONE_TOKEN: u128 = 1_000_000_000_000_000_000;
impl Amount {
    pub const ZERO: Amount = Amount(0);
    pub fn from_attos(attos: u128) -> Self {
        Amount(attos)
    }
    pub fn from_tokens(tokens: u128) -> Self {
        let converted = tokens
            .checked_mul(ONE_TOKEN)
            .expect("overflow converting into amount from token count");
        Amount(converted)
    }

    pub const fn checked_sub(self, rhs: Amount) -> Option<Amount> {
        if let Some(v) = self.0.checked_sub(rhs.0) {
            Some(Amount(v))
        } else {
            None
        }
    }
    pub const fn checked_add(self, rhs: Amount) -> Option<Amount> {
        if let Some(v) = self.0.checked_add(rhs.0) {
            Some(Amount(v))
        } else {
            None
        }
    }
}
impl Mul for Amount {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Amount(self.0 * rhs.0)
    }
}
impl Mul<u128> for Amount {
    type Output = Self;

    fn mul(self, rhs: u128) -> Self::Output {
        Amount(self.0 * rhs)
    }
}

wrap_fixed_bytes! {
    // suppress default derive of Display
    extra_derives: [],
    /// Assest Id is a 256-bit hash.
    pub struct AssetId<32>;
}
impl Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

pub mod rpc_service;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::AssetId;

    #[test]
    fn test_asset_printing() {
        let asset = AssetId::from_slice(&[0xfeu8; 32]);
        assert_eq!(
            asset.to_string(),
            "fefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe"
        );
    }

    #[test]
    fn test_asset_parsing() {
        AssetId::from_str("fefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe")
            .expect("unprefixed hex works");
        AssetId::from_str("0xfefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe")
            .expect("prefixed hex works");
        AssetId::from_str("fefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe")
            .expect_err("too short should be rejected");
        AssetId::from_str("fefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe")
            .expect_err("too long should be rejected");
    }
}
