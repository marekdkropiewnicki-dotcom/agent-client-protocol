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

    /// `wrap` then `into_inner` must hand back a structurally identical
    /// inner message. This is the contract every transport in the wild
    /// relies on to peel the JSON-RPC envelope off before dispatching.
    #[test]
    fn wrap_then_into_inner_round_trips() {
        let inner = Notification::<String> {
            method: "ping".into(),
            params: Some("hello".to_string()),
        };
        let wrapped = JsonRpcMessage::wrap(inner);
        let unwrapped = wrapped.into_inner();
        assert_eq!(&*unwrapped.method, "ping");
        assert_eq!(unwrapped.params.as_deref(), Some("hello"));
    }

    /// JSON-RPC 2.0 mandates the literal string `"2.0"` for the
    /// `jsonrpc` field. Anything else (including the older `"1.0"`,
    /// numbers, or omission entirely) must be rejected — peers that
    /// send the wrong value are not speaking the same protocol.
    #[test]
    fn rejects_messages_with_wrong_jsonrpc_version() {
        // Missing entirely.
        let result: Result<JsonRpcMessage<Notification<String>>, _> =
            serde_json::from_value(json!({
                "method": "ping"
            }));
        assert!(result.is_err(), "missing jsonrpc must fail");

        // Wrong literal.
        let result: Result<JsonRpcMessage<Notification<String>>, _> =
            serde_json::from_value(json!({
                "jsonrpc": "1.0",
                "method": "ping"
            }));
        assert!(result.is_err(), "jsonrpc=1.0 must be rejected");

        // Numeric — many naive implementations send 2.0 as a float.
        let result: Result<JsonRpcMessage<Notification<String>>, _> =
            serde_json::from_value(json!({
                "jsonrpc": 2.0,
                "method": "ping"
            }));
        assert!(result.is_err(), "numeric jsonrpc must be rejected");
    }

    /// `JsonRpcMessage::wrap` must produce JSON whose top-level shape is
    /// `{"jsonrpc": "2.0", ...flattened-message...}`. Inner fields are
    /// flattened, not nested under a key, because that's how peers parse
    /// the envelope. The `jsonrpc` literal must always be `"2.0"`.
    #[test]
    fn wrap_flattens_inner_message_into_envelope() {
        let params = ClientNotification::CancelNotification(CancelNotification {
            session_id: SessionId("s1".into()),
            meta: None,
        });
        let inner = Notification {
            method: "cancel".into(),
            params: Some(params),
        };
        let wrapped = JsonRpcMessage::wrap(inner);
        let serialized = serde_json::to_value(&wrapped).unwrap();
        // jsonrpc is at the top level alongside the flattened message
        // fields; nothing nests the original Notification under a key.
        assert_eq!(serialized.get("jsonrpc"), Some(&json!("2.0")));
        assert_eq!(serialized.get("method"), Some(&json!("cancel")));
        assert!(
            serialized.get("params").is_some(),
            "params must be flattened into the envelope, not nested"
        );
    }

    /// `Response::new` is the central place where successful results
    /// turn into wire `result` payloads and `Err`s turn into `error`
    /// payloads. Both branches must carry the request id through
    /// faithfully so the peer can correlate the response.
    #[test]
    fn response_new_correlates_id_for_both_ok_and_err() {
        let ok: Response<i32, String> = Response::new(42i64, Ok::<_, String>(7));
        match ok {
            Response::Result { id, result } => {
                assert_eq!(id, RequestId::Number(42));
                assert_eq!(result, 7);
            }
            Response::Error { .. } => panic!("Ok must produce Result variant"),
        }

        let err: Response<i32, String> =
            Response::new("req-1".to_string(), Err::<i32, _>("nope".to_string()));
        match err {
            Response::Error { id, error } => {
                assert_eq!(id, RequestId::Str("req-1".to_string()));
                assert_eq!(error, "nope");
            }
            Response::Result { .. } => panic!("Err must produce Error variant"),
        }
    }

    /// `RequestId::From<i64>` and `From<String>` are the documented
    /// ergonomic constructors. Lock them in so we don't accidentally
    /// remove them when tweaking the derive surface.
    #[test]
    fn request_id_from_conversions_pick_the_right_variant() {
        let n: RequestId = 7i64.into();
        assert_eq!(n, RequestId::Number(7));

        let s: RequestId = "abc".to_string().into();
        assert_eq!(s, RequestId::Str("abc".to_string()));
    }

    /// `Notification<Params>` must not emit an `id` field — the presence
    /// of `id` is what distinguishes a request from a notification per
    /// the JSON-RPC spec. Without this guard, accidentally adding an `id`
    /// field would silently change every notification into a
    /// request-shaped message.
    #[test]
    fn notification_has_no_id_field() {
        let note = Notification {
            method: "cancel".into(),
            params: Some(ClientNotification::CancelNotification(CancelNotification {
                session_id: SessionId("s".into()),
                meta: None,
            })),
        };
        let json = serde_json::to_value(&note).unwrap();
        assert!(json.get("id").is_none(), "notifications must not carry id");
        assert_eq!(json.get("method"), Some(&json!("cancel")));
    }

    /// Round-trip a request through JSON: the deserialized value must
    /// match what we sent. Catches regressions where someone changes the
    /// envelope shape and breaks deserialization. Uses `String` params
    /// to avoid the `()` unit-type quirk in serde.
    #[test]
    fn request_round_trips_through_json() {
        let req = Request::<String> {
            id: RequestId::Number(7),
            method: "ping".into(),
            params: Some("payload".to_string()),
        };
        let json = serde_json::to_value(&req).unwrap();
        let back: Request<String> = serde_json::from_value(json).unwrap();
        assert_eq!(back.id, req.id);
        assert_eq!(back.method, req.method);
        assert_eq!(back.params, req.params);
    }
}
