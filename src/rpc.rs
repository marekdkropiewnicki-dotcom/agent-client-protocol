use std::sync::Arc;

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// JSON RPC Request Id
///
/// An identifier established by the Client that MUST contain a String, Number, or NULL value if included. If it is not included it is assumed to be a notification. The value SHOULD normally not be Null \[1\] and Numbers SHOULD NOT contain fractional parts \[2\]
///
/// The Server MUST reply with the same value in the Response object if included. This member is used to correlate the context between the two objects.
///
/// \[1\] The use of Null as a value for the id member in a Request object is discouraged, because this specification uses a value of Null for Responses with an unknown id. Also, because JSON-RPC 1.0 uses an id value of Null for Notifications this could cause confusion in handling.
///
/// \[2\] Fractional parts may be problematic, since many decimal fractions cannot be represented exactly as binary fractions.
#[derive(
    Debug,
    PartialEq,
    Clone,
    Hash,
    Eq,
    Deserialize,
    Serialize,
    PartialOrd,
    Ord,
    Display,
    JsonSchema,
    From,
)]
#[serde(untagged)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[from(String, i64)]
pub enum RequestId {
    #[display("null")]
    Null,
    Number(i64),
    Str(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Request<Params> {
    pub id: RequestId,
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_enums,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[serde(untagged)]
#[schemars(rename = "{Result}", extend("x-docs-ignore" = true))]
pub enum Response<Result, Error> {
    Result { id: RequestId, result: Result },
    Error { id: RequestId, error: Error },
}

impl<R, E> Response<R, E> {
    #[must_use]
    pub fn new(id: impl Into<RequestId>, result: std::result::Result<R, E>) -> Self {
        match result {
            Ok(result) => Self::Result {
                id: id.into(),
                result,
            },
            Err(error) => Self::Error {
                id: id.into(),
                error,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[allow(
    clippy::exhaustive_structs,
    reason = "This comes from the JSON-RPC specification itself"
)]
#[schemars(rename = "{Params}", extend("x-docs-ignore" = true))]
#[skip_serializing_none]
pub struct Notification<Params> {
    pub method: Arc<str>,
    pub params: Option<Params>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

/// A message (request, response, or notification) with `"jsonrpc": "2.0"` specified as
/// [required by JSON-RPC 2.0 Specification][1].
///
/// [1]: https://www.jsonrpc.org/specification#compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(inline)]
pub struct JsonRpcMessage<M> {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    message: M,
}

impl<M> JsonRpcMessage<M> {
    /// Wraps the provided message into a versioned [`JsonRpcMessage`].
    #[must_use]
    pub fn wrap(message: M) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            message,
        }
    }

    /// Unwraps the contained message.
    #[must_use]
    pub fn into_inner(self) -> M {
        self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AgentNotification, CancelNotification, ClientNotification, ContentBlock, ContentChunk,
        SessionId, SessionNotification, SessionUpdate, TextContent,
    };
    use serde_json::{Number, Value, json};

    #[test]
    fn id_deserialization() {
        let id = serde_json::from_value::<RequestId>(Value::Null).unwrap();
        assert_eq!(id, RequestId::Null);

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_u128(1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(1));

        let id = serde_json::from_value::<RequestId>(Value::Number(Number::from_i128(-1).unwrap()))
            .unwrap();
        assert_eq!(id, RequestId::Number(-1));

        let id = serde_json::from_value::<RequestId>(Value::String("id".to_owned())).unwrap();
        assert_eq!(id, RequestId::Str("id".to_owned()));
    }

    #[test]
    fn id_serialization() {
        let id = serde_json::to_value(RequestId::Null).unwrap();
        assert_eq!(id, Value::Null);

        let id = serde_json::to_value(RequestId::Number(1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_u128(1).unwrap()));

        let id = serde_json::to_value(RequestId::Number(-1)).unwrap();
        assert_eq!(id, Value::Number(Number::from_i128(-1).unwrap()));

        let id = serde_json::to_value(RequestId::Str("id".to_owned())).unwrap();
        assert_eq!(id, Value::String("id".to_owned()));
    }

    #[test]
    fn id_display() {
        let id = RequestId::Null;
        assert_eq!(id.to_string(), "null");

        let id = RequestId::Number(1);
        assert_eq!(id.to_string(), "1");

        let id = RequestId::Number(-1);
        assert_eq!(id.to_string(), "-1");

        let id = RequestId::Str("id".to_owned());
        assert_eq!(id.to_string(), "id");
    }

    #[test]
    fn id_from_string_and_i64() {
        // The `#[from(String, i64)]` derive on `RequestId` powers `.into()`
        // calls throughout the crate (and downstream SDKs). If either impl
        // is dropped, callers silently lose ergonomic conversions and may
        // pick up an unexpected variant via type inference.
        let from_string: RequestId = String::from("abc").into();
        assert_eq!(from_string, RequestId::Str("abc".to_owned()));

        let from_i64: RequestId = 42_i64.into();
        assert_eq!(from_i64, RequestId::Number(42));

        // Negative ids are valid per the JSON-RPC spec.
        let negative: RequestId = (-1_i64).into();
        assert_eq!(negative, RequestId::Number(-1));
    }

    #[test]
    fn response_new_maps_ok_to_result_variant() {
        // `Response::new` is the canonical way SDK glue code turns a
        // `Result<R, E>` back into a wire response. Inverting the mapping
        // (e.g. sending `Ok` payloads under the `error` key) would silently
        // turn successful handler returns into errors for every peer.
        let response: Response<i32, &'static str> = Response::new(1, Ok(7));

        match response {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(1));
                assert_eq!(result, 7);
            }
            Response::Error { .. } => panic!("Ok must map to Response::Result"),
        }
    }

    #[test]
    fn response_new_maps_err_to_error_variant() {
        let response: Response<i32, &'static str> =
            Response::new(String::from("req"), Err("boom"));

        match response {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("req".to_owned()));
                assert_eq!(error, "boom");
            }
            Response::Result { .. } => panic!("Err must map to Response::Error"),
        }
    }

    #[test]
    fn response_wire_shape_uses_result_or_error_key() {
        // The `Response` enum is `#[serde(untagged)]`. The result variant
        // must emit `{id, result}` and the error variant `{id, error}` so
        // peers (which are not Rust-aware) can route by key. Any accidental
        // rename of these field names is an unrecoverable wire break.
        let ok: Response<i32, String> = Response::new(1, Ok(42));
        assert_eq!(
            serde_json::to_value(&ok).unwrap(),
            json!({ "id": 1, "result": 42 })
        );

        let err: Response<i32, String> = Response::new(1, Err("nope".to_owned()));
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            json!({ "id": 1, "error": "nope" })
        );
    }

    #[test]
    fn notification_with_missing_params_deserializes_to_none() {
        // Notifications without a `"params"` key should round-trip cleanly to
        // `params: None` so peers that omit the field (instead of sending
        // `"params": null`) are still accepted.
        let parsed: Notification<i32> = serde_json::from_value(json!({
            "method": "ping",
        }))
        .unwrap();

        assert_eq!(&*parsed.method, "ping");
        assert_eq!(parsed.params, None);
    }

    #[test]
    fn json_rpc_message_requires_version_2_0() {
        // The `JsonRpcMessage` envelope is the spec-mandated wrapper. The
        // `JsonRpcVersion` enum only has a `V2` variant tagged `"2.0"`, so
        // any other version (or a missing `jsonrpc` field) must fail to
        // deserialize. Without this guard a peer could ship 1.0 messages
        // and we would silently accept them.
        let valid: serde_json::Value = json!({
            "jsonrpc": "2.0",
            "method": "ping",
        });
        let parsed: JsonRpcMessage<Notification<i32>> = serde_json::from_value(valid).unwrap();
        assert_eq!(&*parsed.into_inner().method, "ping");

        let wrong_version = json!({
            "jsonrpc": "1.0",
            "method": "ping",
        });
        assert!(
            serde_json::from_value::<JsonRpcMessage<Notification<i32>>>(wrong_version).is_err(),
            "expected JsonRpcMessage to reject jsonrpc != 2.0"
        );

        let missing_version = json!({
            "method": "ping",
        });
        assert!(
            serde_json::from_value::<JsonRpcMessage<Notification<i32>>>(missing_version).is_err(),
            "expected JsonRpcMessage to require the jsonrpc field"
        );
    }

    #[test]
    fn json_rpc_message_serializes_with_jsonrpc_2_0_and_flattens_inner() {
        // `wrap` must always emit `"jsonrpc": "2.0"` and the inner message
        // must be flattened next to it (not nested under a `"message"` key).
        let envelope = JsonRpcMessage::wrap(Notification::<i32> {
            method: "ping".into(),
            params: Some(7),
        });

        let serialized: Value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "ping",
                "params": 7,
            })
        );
    }

    #[test]
    fn json_rpc_message_wrap_and_into_inner_round_trip() {
        let notif = Notification::<i32> {
            method: "ping".into(),
            params: Some(7),
        };
        let original_method = notif.method.clone();
        let unwrapped = JsonRpcMessage::wrap(notif).into_inner();
        assert_eq!(unwrapped.method, original_method);
        assert_eq!(unwrapped.params, Some(7));
    }

    #[test]
    fn notification_wire_format() {
        // Test client -> agent notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("test-123".into()),
                meta: None,
            })),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "cancel",
                "params": {
                    "sessionId": "test-123"
                },
            })
        );

        // Test agent -> client notification wire format
        let outgoing_msg = JsonRpcMessage::wrap(Notification {
            method: "sessionUpdate".into(),
            params: Some(AgentNotification::SessionNotification(
                SessionNotification {
                    session_id: SessionId("test-456".into()),
                    update: SessionUpdate::AgentMessageChunk(ContentChunk {
                        content: ContentBlock::Text(TextContent {
                            annotations: None,
                            text: "Hello".to_string(),
                            meta: None,
                        }),
                        #[cfg(feature = "unstable_message_id")]
                        message_id: None,
                        meta: None,
                    }),
                    meta: None,
                },
            )),
        });

        let serialized: Value = serde_json::to_value(&outgoing_msg).unwrap();
        assert_eq!(
            serialized,
            json!({
                "jsonrpc": "2.0",
                "method": "sessionUpdate",
                "params": {
                    "sessionId": "test-456",
                    "update": {
                        "sessionUpdate": "agent_message_chunk",
                        "content": {
                            "type": "text",
                            "text": "Hello"
                        }
                    }
                }
            })
        );
    }
}
