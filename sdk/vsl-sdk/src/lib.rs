pub mod rpc_messages;
pub mod rpc_wrapper;

mod helpers;
mod timestamp;

use std::{fmt::Display, num::ParseIntError, ops::Mul};

pub use crate::helpers::{HasSender, IntoSigned};
pub use alloy::primitives::{Address, B256, wrap_fixed_bytes};
use derive_more::derive;
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
)]
pub struct Amount(u128);

const ONE_VSL_TOKEN: u128 = 1_000_000_000_000_000_000;

#[derive(Debug)]
pub enum ParseAmountError {
    NotHex,
    LeadingZeros,
    ParseInt(ParseIntError),
    NotDecimal,
    TooManyDecimals,
}

impl ToString for ParseAmountError {
    fn to_string(&self) -> String {
        match self {
            ParseAmountError::NotHex => "hex Amount should start with 0x".to_string(),
            ParseAmountError::LeadingZeros => {
                "non-zero hex Amount should not start with 0x0".to_string()
            }
            ParseAmountError::ParseInt(parse_int_error) => parse_int_error.to_string(),
            ParseAmountError::NotDecimal => {
                "decimal Amount should be of the form <units>.<subunits>".to_string()
            }
            ParseAmountError::TooManyDecimals => {
                "decimal Amount had more decimals than supported".to_string()
            }
        }
    }
}

impl Amount {
    pub const ZERO: Amount = Amount(0);
    pub fn from_subunits(subunits: u128) -> Self {
        Amount(subunits)
    }

    pub fn from_vsl_tokens(tokens: u128) -> Self {
        let converted = tokens
            .checked_mul(ONE_VSL_TOKEN)
            .expect("overflow converting into amount from token count");
        Amount(converted)
    }

    pub fn from_tokens(tokens: u128, decimals: u8) -> Self {
        let one_token = 10u128.checked_pow(decimals as u32).expect("overflow");
        let converted = tokens
            .checked_mul(one_token)
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

    pub fn to_hex_str(&self) -> String {
        format!("{:#x}", self)
    }

    pub fn from_hex_str(s: &str) -> Result<Self, ParseAmountError> {
        if !s.starts_with("0x") {
            return Err(ParseAmountError::NotHex);
        }
        let s = &s[2..];
        if s == "0" {
            return Ok(Self::ZERO);
        }
        if s.starts_with('0') {
            return Err(ParseAmountError::LeadingZeros);
        } else {
            let subunits =
                u128::from_str_radix(s, 16).map_err(ParseAmountError::ParseInt)?;
            return Ok(Self::from_subunits(subunits));
        }
    }

    pub fn to_str_with_decimals(&self, mut decimals: u8) -> String {
        let one_token = 10u128.checked_pow(decimals as u32).expect("overflow");
        let units = self.0 / one_token;
        let mut subunits = self.0 % one_token;
        if subunits == 0 {
            return units.to_string();
        }
        while subunits % 10 == 0 {
            subunits /= 10;
            decimals -= 1;
        }
        format!("{}.{:0>width$}", units, subunits, width = decimals as usize)
    }

    pub fn from_str_with_decimals(s: &str, decimals: u8) -> Result<Self, ParseAmountError> {
        let mut iter = s.split(".");
        let Some(units) = iter.next() else {
            return Err(ParseAmountError::NotDecimal);
        };
        let subunits = iter.next().unwrap_or("0");
        if iter.next().is_some() {
            return Err(ParseAmountError::NotDecimal);
        }
        let decimals = decimals as usize;
        if subunits.len() > decimals {
            return Err(ParseAmountError::TooManyDecimals);
        }
        let restored = format!("{}{:0<width$}", units, subunits, width = decimals);
        let amount = u128::from_str_radix(&restored, 10).map_err(ParseAmountError::ParseInt)?;
        Ok(Self(amount))
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

    use crate::{Amount, AssetId, ParseAmountError};

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

    #[test]
    fn test_amount_parsing_decimal() {
        assert_eq!(
            Amount::from_str_with_decimals("100", 2).unwrap(),
            Amount::from_tokens(100, 2)
        );
        assert_eq!(
            Amount::from_str_with_decimals("100.1", 2).unwrap(),
            Amount::from_subunits(10010)
        );
        assert_eq!(
            Amount::from_str_with_decimals("100.12", 2).unwrap(),
            Amount::from_subunits(10012)
        );
        assert_eq!(
            Amount::from_str_with_decimals("100.02", 2).unwrap(),
            Amount::from_subunits(10002)
        );
        let ParseAmountError::TooManyDecimals =
            Amount::from_str_with_decimals("100.122", 2).unwrap_err()
        else {
            panic!("Expected too many decimals error");
        };
    }

    #[test]
    fn test_amount_formatting_decimal() {
        assert_eq!(
            Amount::from_subunits(123456).to_str_with_decimals(3),
            "123.456"
        );
        assert_eq!(
            Amount::from_subunits(123450).to_str_with_decimals(3),
            "123.45"
        );
        assert_eq!(
            Amount::from_subunits(123400).to_str_with_decimals(3),
            "123.4"
        );
        assert_eq!(Amount::from_subunits(123000).to_str_with_decimals(3), "123");
        assert_eq!(
            Amount::from_subunits(123045).to_str_with_decimals(3),
            "123.045"
        );
        assert_eq!(
            Amount::from_subunits(123040).to_str_with_decimals(3),
            "123.04"
        );
        assert_eq!(
            Amount::from_subunits(123004).to_str_with_decimals(3),
            "123.004"
        );
    }
}
