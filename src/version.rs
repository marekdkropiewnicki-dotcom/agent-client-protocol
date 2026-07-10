use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::Serialize;

/// Protocol version identifier.
///
/// This version is only bumped for breaking changes.
/// Non-breaking changes should be introduced via capabilities.
#[derive(
    Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord, From, Display,
)]
pub struct ProtocolVersion(u16);

impl ProtocolVersion {
    /// Version `0` of the protocol.
    ///
    /// This was a pre-release version that shouldn't be used in production.
    /// It is used as a fallback for any request whose version cannot be parsed
    /// as a valid version, and should likely be treated as unsupported.
    pub const V0: Self = Self(0);
    /// Version `1` of the protocol.
    ///
    /// <https://agentclientprotocol.com/protocol/overview>
    pub const V1: Self = Self(1);
    /// Version `2` of the protocol.
    ///
    /// This is an unstable draft used for protocol iteration. It is only
    /// available when the `unstable_protocol_v2` feature is enabled and is
    /// **not** advertised by [`ProtocolVersion::LATEST`] yet — callers must
    /// opt into V2 explicitly.
    #[cfg(feature = "unstable_protocol_v2")]
    pub const V2: Self = Self(2);
    /// The latest stable supported version of the protocol.
    ///
    /// Currently this is version `1`. Enabling the `unstable_protocol_v2`
    /// feature exposes `ProtocolVersion::V2` but does **not** change the
    /// value of `LATEST` — v2 will only become the latest once it stabilizes.
    pub const LATEST: Self = Self::V1;

    #[cfg(test)]
    #[must_use]
    pub const fn new(version: u16) -> Self {
        Self(version)
    }
}

use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ProtocolVersionVisitor;

        impl Visitor<'_> for ProtocolVersionVisitor {
            type Value = ProtocolVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a protocol version number or string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match u16::try_from(value) {
                    Ok(value) => Ok(ProtocolVersion(value)),
                    Err(_) => Err(E::custom(format!("protocol version {value} is too large"))),
                }
            }

            fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion::V0)
            }

            fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Old versions used strings, we consider all of those version 0
                Ok(ProtocolVersion::V0)
            }
        }

        deserializer.deserialize_any(ProtocolVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_u64() {
        let json = "1";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(1));
    }

    #[test]
    fn test_deserialize_string() {
        let json = "\"1.0.0\"";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_large_number() {
        let json = "100000";
        let result: Result<ProtocolVersion, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_zero() {
        let json = "0";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(0));
    }

    #[test]
    fn test_deserialize_max_u16() {
        let json = "65535";
        let version: ProtocolVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version, ProtocolVersion::new(65535));
    }

    /// Negative numbers are not valid u16 versions and must be rejected
    /// rather than silently coerced or accepted as a string.
    #[test]
    fn test_deserialize_negative_number_rejected() {
        let result: Result<ProtocolVersion, _> = serde_json::from_str("-1");
        assert!(
            result.is_err(),
            "negative protocol version must be rejected"
        );
    }

    /// Floats are not valid versions; a regression that accepted `1.0` would
    /// silently round-trip through `visit_u64` after truncation.
    #[test]
    fn test_deserialize_float_rejected() {
        let result: Result<ProtocolVersion, _> = serde_json::from_str("1.5");
        assert!(
            result.is_err(),
            "fractional protocol version must be rejected"
        );
    }

    /// Invariant: `LATEST` is `V1` until v2 stabilizes. Bumping `LATEST`
    /// without a coordinated release is a breaking change for every
    /// consumer that caches the value, so make it visible in tests.
    #[test]
    fn latest_is_v1() {
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::V1);
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::new(1));
    }

    /// Pin the `Serialize` contract — protocol versions go on the wire as a
    /// JSON integer, never a string. The asymmetric Deserialize impl
    /// (which folds strings down to V0) makes the Serialize side easy to
    /// regress accidentally.
    #[test]
    fn serializes_as_integer() {
        let v = serde_json::to_value(ProtocolVersion::V1).unwrap();
        assert!(v.is_number(), "expected integer, got {v}");
        assert_eq!(v, serde_json::json!(1));

        let v = serde_json::to_value(ProtocolVersion::V0).unwrap();
        assert_eq!(v, serde_json::json!(0));

        // Round trip number → ProtocolVersion → number.
        let v: ProtocolVersion = serde_json::from_value(serde_json::json!(7)).unwrap();
        assert_eq!(serde_json::to_value(v).unwrap(), serde_json::json!(7));
    }

    /// Versions are monotonically ordered by their integer value; the
    /// negotiation logic in higher-level crates relies on this so we pin
    /// it here.
    #[test]
    fn ordering_is_numeric() {
        assert!(ProtocolVersion::V0 < ProtocolVersion::V1);
        assert!(ProtocolVersion::new(1) < ProtocolVersion::new(2));
        assert_eq!(ProtocolVersion::V1, ProtocolVersion::new(1));

        // Min/max selection works as expected.
        let max = std::cmp::max(ProtocolVersion::V1, ProtocolVersion::V0);
        assert_eq!(max, ProtocolVersion::V1);
    }

    #[test]
    fn display_renders_inner_number() {
        assert_eq!(ProtocolVersion::V0.to_string(), "0");
        assert_eq!(ProtocolVersion::V1.to_string(), "1");
        assert_eq!(ProtocolVersion::new(42).to_string(), "42");
    }

    /// `From<u16>` is part of the public API thanks to `derive_more::From`
    /// — verify it still constructs the same value `new` produces.
    #[test]
    fn from_u16_constructs_same_value() {
        let v: ProtocolVersion = 5u16.into();
        assert_eq!(v, ProtocolVersion::new(5));
    }
}
