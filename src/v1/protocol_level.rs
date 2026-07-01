use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{IntoOption, Meta, RequestId};

/// **UNSTABLE**
///
/// This capability is not part of the spec yet, and may be removed or changed at any point.
///
/// Notification to cancel an ongoing request.
///
/// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/cancellation)
#[cfg(feature = "unstable_cancel_request")]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[schemars(extend("x-side" = "protocol", "x-method" = CANCEL_REQUEST_METHOD_NAME))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CancelRequestNotification {
    /// The ID of the request to cancel.
    pub request_id: RequestId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

#[cfg(feature = "unstable_cancel_request")]
impl CancelRequestNotification {
    #[must_use]
    pub fn new(request_id: impl Into<RequestId>) -> Self {
        Self {
            request_id: request_id.into(),
            meta: None,
        }
    }

    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[must_use]
    pub fn meta(mut self, meta: impl IntoOption<Meta>) -> Self {
        self.meta = meta.into_option();
        self
    }
}

// Method schema

/// Names of all methods that agents handle.
///
/// Provides a centralized definition of method names used in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct GeneralMethodNames {
    #[cfg(feature = "unstable_cancel_request")]
    pub cancel_request: &'static str,
}

/// Constant containing all agent method names.
pub const PROTOCOL_LEVEL_METHOD_NAMES: GeneralMethodNames = GeneralMethodNames {
    #[cfg(feature = "unstable_cancel_request")]
    cancel_request: CANCEL_REQUEST_METHOD_NAME,
};

/// Method name for general cancel notification
pub(crate) const CANCEL_REQUEST_METHOD_NAME: &str = "$/cancel_request";

/// General protocol-level notifications that all sides are expected to
/// implement.
///
/// Notifications whose methods start with '$/' are messages which
/// are protocol implementation dependent and might not be implementable in all
/// clients or agents. For example if the implementation uses a single threaded
/// synchronous programming language then there is little it can do to react to
/// a `$/cancel_request` notification. If an agent or client receives
/// notifications starting with '$/' it is free to ignore the notification.
///
/// Notifications do not expect a response.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
#[non_exhaustive]
pub enum ProtocolLevelNotification {
    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or
    /// changed at any point.
    ///
    /// Cancels an ongoing request.
    ///
    /// This is a notification sent by the side that sent a request to cancel that request.
    ///
    /// Upon receiving this notification, the receiver:
    ///
    /// 1. MUST cancel the corresponding request activity and all nested activities
    /// 2. MAY send any pending notifications.
    /// 3. MUST send one of these responses for the original request:
    ///   - Valid response with appropriate data (partial results or cancellation marker)
    ///   - Error response with code `-32800` (Cancelled)
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/cancellation)
    #[cfg(feature = "unstable_cancel_request")]
    CancelRequestNotification(CancelRequestNotification),
}

impl ProtocolLevelNotification {
    /// Returns the corresponding method name of the notification.
    #[must_use]
    pub fn method(&self) -> &str {
        match self {
            #[cfg(feature = "unstable_cancel_request")]
            Self::CancelRequestNotification(..) => PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
        }
    }
}

#[cfg(all(test, feature = "unstable_cancel_request"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cancel_request_method_name_is_stable() {
        // Pin the wire method name so downstream implementations that dispatch
        // on this string do not silently break if the constant is renamed.
        assert_eq!(CANCEL_REQUEST_METHOD_NAME, "$/cancel_request");
        assert_eq!(
            PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
            CANCEL_REQUEST_METHOD_NAME,
        );
    }

    #[test]
    fn cancel_request_new_defaults_meta_to_none() {
        let notification = CancelRequestNotification::new(RequestId::Number(42));
        assert_eq!(notification.request_id, RequestId::Number(42));
        assert!(notification.meta.is_none());
    }

    #[test]
    fn cancel_request_meta_builder_accepts_option_and_value() {
        let mut meta = serde_json::Map::new();
        meta.insert("progress".to_string(), json!(0.5));

        // IntoOption should accept a bare value and store it as Some.
        let with_value = CancelRequestNotification::new(RequestId::Number(1)).meta(meta.clone());
        assert_eq!(with_value.meta.as_ref(), Some(&meta));

        // IntoOption should accept Some(value).
        let with_some =
            CancelRequestNotification::new(RequestId::Number(1)).meta(Some(meta.clone()));
        assert_eq!(with_some.meta.as_ref(), Some(&meta));

        // IntoOption should accept None to clear the field.
        let cleared = CancelRequestNotification::new(RequestId::Number(1))
            .meta(meta.clone())
            .meta(Option::<Meta>::None);
        assert!(cleared.meta.is_none());
    }

    #[test]
    fn cancel_request_serializes_camel_case_and_skips_none_meta() {
        // The `_meta` field must be renamed and skipped when None (via
        // `skip_serializing_none`); the request id field must be camelCased.
        let notification = CancelRequestNotification::new(RequestId::Number(7));
        let value = serde_json::to_value(&notification).unwrap();
        assert_eq!(value, json!({ "requestId": 7 }));
        assert!(!value.as_object().unwrap().contains_key("_meta"));
        assert!(!value.as_object().unwrap().contains_key("meta"));
    }

    #[test]
    fn cancel_request_serializes_meta_under_underscore_meta() {
        let mut meta = serde_json::Map::new();
        meta.insert("reason".to_string(), json!("user cancelled"));
        let notification = CancelRequestNotification::new(RequestId::Str("abc".into())).meta(meta);
        let value = serde_json::to_value(&notification).unwrap();
        assert_eq!(
            value,
            json!({
                "requestId": "abc",
                "_meta": { "reason": "user cancelled" }
            })
        );
    }

    #[test]
    fn cancel_request_round_trips_all_request_id_variants() {
        for id in [
            RequestId::Number(-1),
            RequestId::Number(0),
            RequestId::Number(i64::MAX),
            RequestId::Str("session-1".into()),
            RequestId::Null,
        ] {
            let notification = CancelRequestNotification::new(id.clone());
            let value = serde_json::to_value(&notification).unwrap();
            let parsed: CancelRequestNotification = serde_json::from_value(value).unwrap();
            assert_eq!(parsed, notification);
            assert_eq!(parsed.request_id, id);
        }
    }

    #[test]
    fn protocol_level_notification_method_matches_constant() {
        let inner = CancelRequestNotification::new(RequestId::Number(1));
        let notification = ProtocolLevelNotification::CancelRequestNotification(inner);
        assert_eq!(notification.method(), "$/cancel_request");
        assert_eq!(
            notification.method(),
            PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
        );
    }

    #[test]
    fn protocol_level_notification_serializes_untagged_as_inner() {
        // Because `ProtocolLevelNotification` is `#[serde(untagged)]`, the enum
        // wrapper must be invisible on the wire — the params body is the inner
        // `CancelRequestNotification` payload verbatim.
        let inner = CancelRequestNotification::new(RequestId::Number(9));
        let notification = ProtocolLevelNotification::CancelRequestNotification(inner.clone());

        let outer = serde_json::to_value(&notification).unwrap();
        let bare = serde_json::to_value(&inner).unwrap();
        assert_eq!(outer, bare);
        assert_eq!(outer, json!({ "requestId": 9 }));
    }
}
