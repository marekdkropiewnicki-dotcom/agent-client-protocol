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

    /// Negotiation logic depends on `<` / `>` over `ProtocolVersion`. A regression
    /// in the derived `Ord` (e.g. someone making it `derive(Ord)` over a tuple
    /// struct in a way that flips comparison) would silently route clients to
    /// the wrong protocol surface.
    #[test]
    fn ordering_is_numeric_and_total() {
        assert!(ProtocolVersion::V0 < ProtocolVersion::V1);
        assert!(ProtocolVersion::V1 > ProtocolVersion::V0);
        assert_eq!(
            ProtocolVersion::V1.cmp(&ProtocolVersion::V1),
            std::cmp::Ordering::Equal
        );

        // A higher numeric version compares strictly greater than V1.
        let future = ProtocolVersion::new(99);
        assert!(future > ProtocolVersion::V1);
    }

    /// Wire format for the integer version must serialize as a bare number,
    /// not a struct. Several clients pin against this shape directly.
    #[test]
    fn serializes_as_bare_number() {
        assert_eq!(
            serde_json::to_value(ProtocolVersion::V1).unwrap(),
            serde_json::json!(1)
        );
        assert_eq!(serde_json::to_string(&ProtocolVersion::V0).unwrap(), "0");
    }

    /// Display should render the bare integer too — used in CLI banners
    /// and log messages where the surrounding text already provides
    /// "ACP version" framing.
    #[test]
    fn display_renders_bare_integer() {
        assert_eq!(ProtocolVersion::V0.to_string(), "0");
        assert_eq!(ProtocolVersion::V1.to_string(), "1");
        assert_eq!(ProtocolVersion::new(42).to_string(), "42");
    }

    /// Negative integers must not be silently coerced to a valid version —
    /// they should fail deserialization so that the agent is aware that
    /// the peer sent garbage.
    #[test]
    fn negative_numbers_fail_to_deserialize() {
        let result: Result<ProtocolVersion, _> = serde_json::from_str("-1");
        assert!(result.is_err(), "negative versions must be rejected");
    }

    /// Floating-point versions are also garbage — both legitimate floats
    /// (`1.5`) and integer-valued floats (`1.0`) must not produce a valid
    /// `ProtocolVersion`. Without this guard, `serde_json` would happily
    /// give us back `ProtocolVersion(1)` for `1.0`, masking a peer that
    /// fundamentally doesn't speak the protocol.
    #[test]
    fn non_integer_numbers_fail_to_deserialize() {
        assert!(serde_json::from_str::<ProtocolVersion>("1.5").is_err());
        // serde_json currently flows non-integer u64 values through
        // visit_f64; assert that they don't sneak through.
        assert!(serde_json::from_str::<ProtocolVersion>("1.0").is_err());
    }

    /// Stability invariant: `LATEST` always points at the latest *stable*
    /// version. This must hold whether or not the v2 draft feature is
    /// enabled — that's documented contract for crate consumers who pin
    /// against `LATEST` for negotiation.
    #[test]
    fn latest_is_v1_regardless_of_feature_flags() {
        assert_eq!(ProtocolVersion::LATEST, ProtocolVersion::V1);
    }

    /// `From<u16>` is the canonical way SDKs construct versions from raw
    /// negotiated bytes. Make sure the conversion is identity.
    #[test]
    fn from_u16_is_identity() {
        let v: ProtocolVersion = 7u16.into();
        assert_eq!(v, ProtocolVersion::new(7));
    }
}
