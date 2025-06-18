pub mod rpc_messages;
pub mod rpc_wrapper;

mod helpers;
mod timestamp;

use std::{fmt::Display, num::ParseIntError};

pub use crate::helpers::{HasSender, IntoSigned};
pub use alloy::primitives::{Address, B256};
pub use timestamp::Timestamp;

pub mod rpc_service;

#[derive(Debug)]
pub enum ParseAmountError {
    NotHex,
    LeadingZeros,
    ParseInt(ParseIntError),
}

impl ToString for ParseAmountError {
    fn to_string(&self) -> String {
        match self {
            ParseAmountError::NotHex => "Amount should start with 0x".to_string(),
            ParseAmountError::LeadingZeros => "Amount should not start with 0x0".to_string(),
            ParseAmountError::ParseInt(parse_int_error) => parse_int_error.to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Default, Debug)]
pub struct Amount {
    subunits: u128,
    decimals: u8,
}

impl Amount {
    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn subunits(&self) -> u128 {
        self.subunits
    }

    pub fn from_tokens(tokens: u128) -> Self {
        Self::from_tokens_with_decimals(tokens, 18)
    }

    pub fn from_subunits(subunits: u128) -> Self {
        Self::from_tokens_with_decimals(subunits, 18)
    }

    pub fn from_subunits_with_decimanls(subunits: u128, decimals: u8) -> Self {
        Self { subunits, decimals }
    }

    pub fn from_tokens_with_decimals(tokens: u128, decimals: u8) -> Self {
        let multiplier = 10_u128.checked_pow(decimals as u32).unwrap();
        let coins = tokens.checked_mul(multiplier).unwrap_or_else(||
            panic!("Speficied amount cannot be represented")
        );
        Self::from_subunits_with_decimanls(coins, decimals)
    }

    pub fn from_hex_str(s: &str) -> Result<Self, ParseAmountError> {
        Self::from_hex_str_with_decimals(s, 18)
    }

    pub fn from_hex_str_with_decimals(s: &str, decimals: u8) -> Result<Self, ParseAmountError> {
        if !s.starts_with("0x") {
            return Err(ParseAmountError::NotHex)
        }
        let s = &s[2..];
        let subunits = 
        if s == "0" {
            0
        } else if s.starts_with('0') {
            return Err(ParseAmountError::LeadingZeros);
        } else {
            u128::from_str_radix(s, 16).map_err(ParseAmountError::ParseInt)?
        };
        Ok(Self { subunits, decimals })
    }

    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.subunits)
    }
}

impl From<Amount> for u128 {
    fn from(value: Amount) -> Self {
        value.subunits
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let multiplier = 10_u128.checked_pow(self.decimals as u32).unwrap();
        let units = self.subunits / multiplier;
        let subunits = self.subunits % multiplier;
        f.write_fmt(format_args!("{}.{:0>width$}", units, subunits, width = self.decimals as usize))
    }
}