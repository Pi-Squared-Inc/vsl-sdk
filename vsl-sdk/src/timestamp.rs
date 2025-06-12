//! # Timestamp Module
//!
//! This module provides the `Timestamp` struct, a compact representation of time with
//! second and nanosecond precision. It supports natural ordering (`Ord`, `PartialOrd`),
//! formatted display, string parsing, and JSON serialization.
//!
use alloy_rlp::{RlpDecodable, RlpEncodable};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    RlpEncodable,
    RlpDecodable,
)]
/// Records the time elapsed from the [UNIX_EPOCH]
pub struct Timestamp {
    /// Time elapsed from the [UNIX_EPOCH] (in seconds, truncated)
    seconds: u64,
    /// the _remaining fraction_ of a second, expressed in nano-seconds
    nanos: u32,
}

impl Timestamp {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!");
        Timestamp {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }

    pub fn seconds(&self) -> u64 {
        self.seconds
    }

    pub fn nanos(&self) -> u32 {
        self.nanos
    }

    pub fn from_seconds(seconds: u64) -> Self {
        Timestamp { seconds, nanos: 0 }
    }

    /// The timestamp as a formatted string: "seconds.nanos"
    pub fn to_string(&self) -> String {
        format!("{}.{}", self.seconds, self.nanos)
    }

    /// Returns the next timestamp, incrementing the nanos by one.
    /// If the nanos are already at their maximum value, the seconds
    /// are incremented and the nanos are reset to zero.
    pub fn tick(&self) -> Self {
        if self.nanos == 999_999_999 {
            Timestamp {
                seconds: self.seconds + 1,
                nanos: 0,
            }
        } else {
            Timestamp {
                seconds: self.seconds,
                nanos: self.nanos + 1,
            }
        }
    }
}

use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTimestampError;

impl FromStr for Timestamp {
    type Err = ParseTimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(ParseTimestampError);
        }

        let seconds = u64::from_str(parts[0]).map_err(|_| ParseTimestampError)?;
        let nanos = u32::from_str(parts[1]).map_err(|_| ParseTimestampError)?;

        if nanos >= 1_000_000_000 {
            return Err(ParseTimestampError);
        }

        Ok(Timestamp { seconds, nanos })
    }
}

/// Allow printing the timestamp directly
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.seconds, self.nanos)
    }
}

/// Allow comparing timestamps
impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.seconds.cmp(&other.seconds) {
            Ordering::Equal => self.nanos.cmp(&other.nanos),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_basic_construction() {
        let ts = Timestamp::from_seconds(10);
        assert_eq!(ts.seconds(), 10);
        assert_eq!(ts.nanos(), 0);

        let ts2 = Timestamp {
            seconds: 10,
            nanos: 500,
        };
        assert_eq!(ts2.seconds(), 10);
        assert_eq!(ts2.nanos(), 500);
    }

    #[test]
    fn test_now() {
        let ts = Timestamp::now();
        assert!(ts.seconds() > 0);
        assert!(ts.nanos() < 1_000_000_000);
    }

    #[test]
    fn test_formatting() {
        let ts = Timestamp::from_seconds(1);
        assert_eq!(ts.to_string(), "1.0");

        let ts2 = Timestamp {
            seconds: 12345,
            nanos: 6789,
        };
        assert_eq!(ts2.to_string(), "12345.6789");
    }

    #[test]
    fn test_parsing() {
        let ts = Timestamp::from_str("12345.6789").unwrap();
        assert_eq!(ts.seconds(), 12345);
        assert_eq!(ts.nanos(), 6789);

        assert!("invalid".parse::<Timestamp>().is_err());
        assert!("12345".parse::<Timestamp>().is_err());
        assert!("12345.".parse::<Timestamp>().is_err());
        assert!(".6789".parse::<Timestamp>().is_err());
        assert!("12345.1000000000".parse::<Timestamp>().is_err());
    }

    #[test]
    fn test_ordering() {
        let t1 = Timestamp {
            seconds: 10,
            nanos: 500,
        };
        let t2 = Timestamp {
            seconds: 10,
            nanos: 1000,
        };
        let t3 = Timestamp {
            seconds: 11,
            nanos: 0,
        };

        assert!(t1 < t2);
        assert!(t2 < t3);
        assert!(t1 < t3);
        assert_eq!(
            t2,
            Timestamp {
                seconds: 10,
                nanos: 1000
            }
        );
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_serialization() {
        let ts = Timestamp {
            seconds: 10,
            nanos: 500,
        };
        let serialized = serde_json::to_string(&ts).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&serialized).unwrap();

        assert_eq!(ts, deserialized);
    }

    #[test]
    fn test_extremes() {
        let zero = Timestamp::from_seconds(0);
        let max = Timestamp {
            seconds: u64::MAX,
            nanos: 999_999_999,
        };

        assert_eq!(zero.seconds(), 0);
        assert_eq!(zero.nanos(), 0);
        assert_eq!(max.seconds(), u64::MAX);
        assert_eq!(max.nanos(), 999_999_999);
        assert!(zero < max);
    }

    #[test]
    fn test_display_trait() {
        let ts = Timestamp {
            seconds: 12345,
            nanos: 6789,
        };
        assert_eq!(format!("{}", ts), "12345.6789");
    }

    #[test]
    fn test_next() {
        let ts = Timestamp {
            seconds: 12345,
            nanos: 6789,
        };
        let next_ts = ts.tick();
        assert_eq!(next_ts.seconds(), 12345);
        assert_eq!(next_ts.nanos(), 6790);

        let ts_max_nanos = Timestamp {
            seconds: 12345,
            nanos: 999_999_999,
        };
        let next_ts_max = ts_max_nanos.tick();
        assert_eq!(next_ts_max.seconds(), 12346);
        assert_eq!(next_ts_max.nanos(), 0);
    }
}
