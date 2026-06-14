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
    fn request_round_trips_with_none_params() {
        // `params` is `Option<Params>`. Deserializing a request that omits
        // `params` entirely must succeed and produce `None`, which is how
        // notification-style requests arrive on the wire.
        let parsed: Request<ClientNotification> = serde_json::from_value(json!({
            "id": 7,
            "method": "cancel",
        }))
        .unwrap();
        assert_eq!(parsed.id, RequestId::Number(7));
        assert_eq!(&*parsed.method, "cancel");
        assert!(parsed.params.is_none());

        // Re-serializing the same value must keep the id and method intact.
        let reserialized = serde_json::to_value(&parsed).unwrap();
        assert_eq!(reserialized["id"], json!(7));
        assert_eq!(reserialized["method"], json!("cancel"));
    }

    #[test]
    fn notification_round_trips_with_none_params() {
        // Same contract as `Request`: omitted `params` deserializes to `None`.
        let parsed: Notification<ClientNotification> = serde_json::from_value(json!({
            "method": "cancel",
        }))
        .unwrap();
        assert_eq!(&*parsed.method, "cancel");
        assert!(parsed.params.is_none());

        let reserialized = serde_json::to_value(&parsed).unwrap();
        assert_eq!(reserialized["method"], json!("cancel"));
    }

    #[test]
    fn response_new_constructs_result_for_ok() {
        let response: Response<i32, String> = Response::new(RequestId::Number(1), Ok(42));
        match response {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(1));
                assert_eq!(result, 42);
            }
            Response::Error { .. } => panic!("expected Result variant"),
        }
    }

    #[test]
    fn response_new_constructs_error_for_err() {
        let response: Response<i32, String> =
            Response::new(RequestId::Str("abc".into()), Err("boom".to_owned()));
        match response {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("abc".into()));
                assert_eq!(error, "boom");
            }
            Response::Result { .. } => panic!("expected Error variant"),
        }
    }

    #[test]
    fn response_untagged_deserialization_picks_result_or_error() {
        // A response with a `result` field should deserialize into `Result`.
        let success: Response<serde_json::Value, serde_json::Value> = serde_json::from_value(json!({
            "id": 1,
            "result": {"ok": true},
        }))
        .unwrap();
        match success {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(1));
                assert_eq!(result, json!({"ok": true}));
            }
            Response::Error { .. } => panic!("expected Result variant"),
        }

        // A response with an `error` field should deserialize into `Error`.
        let failure: Response<serde_json::Value, serde_json::Value> = serde_json::from_value(json!({
            "id": "req-2",
            "error": {"code": -32601, "message": "Method not found"},
        }))
        .unwrap();
        match failure {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("req-2".into()));
                assert_eq!(error, json!({"code": -32601, "message": "Method not found"}));
            }
            Response::Result { .. } => panic!("expected Error variant"),
        }
    }

    #[test]
    fn jsonrpc_message_round_trips_through_json() {
        // Build a JsonRpcMessage by parsing the canonical wire form so the
        // test does not depend on whether `params` is emitted as omitted vs.
        // explicit `null` (both are valid per JSON-RPC 2.0).
        let parsed: JsonRpcMessage<Notification<ClientNotification>> =
            serde_json::from_value(json!({
                "jsonrpc": "2.0",
                "method": "ping",
            }))
            .unwrap();
        let inner = parsed.into_inner();
        assert_eq!(&*inner.method, "ping");
        assert!(inner.params.is_none());

        // Round-trip: serializing a wrapped notification must keep the
        // jsonrpc + method fields exactly as produced.
        let outgoing = JsonRpcMessage::wrap(Notification::<ClientNotification> {
            method: "ping".into(),
            params: None,
        });
        let serialized = serde_json::to_value(&outgoing).unwrap();
        assert_eq!(serialized["jsonrpc"], json!("2.0"));
        assert_eq!(serialized["method"], json!("ping"));
    }

    #[test]
    fn jsonrpc_message_rejects_wrong_or_missing_version() {
        // Missing the required `jsonrpc` field.
        let missing: Result<JsonRpcMessage<Notification<ClientNotification>>, _> =
            serde_json::from_value(json!({
                "method": "ping",
            }));
        assert!(
            missing.is_err(),
            "JsonRpcMessage must require a jsonrpc field"
        );

        // Wrong version string.
        let wrong: Result<JsonRpcMessage<Notification<ClientNotification>>, _> =
            serde_json::from_value(json!({
                "jsonrpc": "1.0",
                "method": "ping",
            }));
        assert!(
            wrong.is_err(),
            "JsonRpcMessage must reject any jsonrpc value other than 2.0"
        );
    }

    #[test]
    fn request_id_orders_variants_consistently() {
        // The derived `Ord` for the untagged enum follows declaration order:
        // Null < Number < Str. This is relied upon when ids are used as map
        // keys or sorted for stable output.
        assert!(RequestId::Null < RequestId::Number(0));
        assert!(RequestId::Number(i64::MAX) < RequestId::Str(String::new()));

        // Within the Number variant, ordering follows the numeric value.
        assert!(RequestId::Number(-5) < RequestId::Number(5));

        // Within the Str variant, ordering follows lexicographic order.
        assert!(RequestId::Str("a".into()) < RequestId::Str("b".into()));
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
