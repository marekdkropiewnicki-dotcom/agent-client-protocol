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

    #[test]
    fn test_serialize_to_integer() {
        // The wire format MUST be a JSON number, not a string. This is
        // load-bearing because peers using the old wire format (strings)
        // are intentionally bucketed into V0 on the read path; if we
        // start emitting strings here, every peer would silently downgrade.
        assert_eq!(serde_json::to_string(&ProtocolVersion::V1).unwrap(), "1");
        assert_eq!(serde_json::to_string(&ProtocolVersion::V0).unwrap(), "0");
    }

    #[test]
    fn test_latest_is_v1() {
        // LATEST must stay anchored to V1 even when V2 is feature-gated.
        // Bumping LATEST is a breaking protocol change; this test makes
        // sure that bump can't happen accidentally.
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::V1);
    }

    #[test]
    fn test_ordering() {
        // Ordering is used in capability negotiation; v0 must be the
        // lowest sentinel so that peers with a strict "must be >= v1"
        // check correctly reject v0 fallback responses.
        assert!(ProtocolVersion::V0 < ProtocolVersion::V1);
        assert!(ProtocolVersion::V1 > ProtocolVersion::V0);
        assert_eq!(ProtocolVersion::V1, ProtocolVersion::new(1));
    }

    #[test]
    fn test_display_formats_as_number() {
        assert_eq!(ProtocolVersion::V1.to_string(), "1");
        assert_eq!(ProtocolVersion::V0.to_string(), "0");
    }

    #[test]
    fn test_deserialize_negative_number_errors() {
        // Negative numbers must NOT silently fall through to V0 (which is
        // reserved for string-based old-version peers). They are a hard
        // protocol error.
        let result: Result<ProtocolVersion, _> = serde_json::from_str("-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_string_with_arbitrary_content_becomes_v0() {
        // The string-fallback path is documented as "any string becomes
        // V0". Verify a few non-numeric / non-semver inputs.
        for input in ["\"\"", "\"abc\"", "\"v0.1.0\"", "\"1\""] {
            let version: ProtocolVersion = serde_json::from_str(input).unwrap();
            assert_eq!(
                version,
                ProtocolVersion::V0,
                "input {input} should be coerced to V0"
            );
        }
    }

    #[cfg(feature = "unstable_protocol_v2")]
    #[test]
    fn test_v2_constant_exists_and_orders_above_v1() {
        // V2 is feature-gated and explicitly not the LATEST. This test
        // pins both invariants so a contributor cannot accidentally
        // promote V2 to LATEST in the future without removing this test.
        assert_eq!(ProtocolVersion::V2, ProtocolVersion::new(2));
        assert!(ProtocolVersion::V2 > ProtocolVersion::V1);
        assert_ne!(ProtocolVersion::LATEST, ProtocolVersion::V2);
    }
}
