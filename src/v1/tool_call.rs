//! Tool calls represent actions that language models request agents to perform.
//!
//! When an LLM determines it needs to interact with external systems—like reading files,
//! running code, or fetching data—it generates tool calls that the agent executes on its behalf.
//!
/// See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
use std::{path::PathBuf, sync::Arc};

use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use crate::{ContentBlock, Error, IntoOption, Meta, SkipListener, TerminalId};

/// Represents a tool call that the language model has requested.
///
/// Tool calls are actions that the agent executes on behalf of the language model,
/// such as reading files, executing code, or fetching data from external sources.
///
/// See protocol docs: [Tool Calls](https://agentclientprotocol.com/protocol/tool-calls)
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCall {
    /// Unique identifier for this tool call within the session.
    pub tool_call_id: ToolCallId,
    /// Human-readable title describing what the tool is doing.
    pub title: String,
    /// The category of tool being invoked.
    /// Helps clients choose appropriate icons and UI treatment.
    #[serde(default, skip_serializing_if = "ToolKind::is_default")]
    pub kind: ToolKind,
    /// Current execution status of the tool call.
    #[serde(default, skip_serializing_if = "ToolCallStatus::is_default")]
    pub status: ToolCallStatus,
    /// Content produced by the tool call.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ToolCallContent>,
    /// File locations affected by this tool call.
    /// Enables "follow-along" features in clients.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<ToolCallLocation>,
    /// Raw input parameters sent to the tool.
    pub raw_input: Option<serde_json::Value>,
    /// Raw output returned by the tool.
    pub raw_output: Option<serde_json::Value>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCall {
    #[must_use]
    pub fn new(tool_call_id: impl Into<ToolCallId>, title: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            title: title.into(),
            kind: ToolKind::default(),
            status: ToolCallStatus::default(),
            content: Vec::default(),
            locations: Vec::default(),
            raw_input: None,
            raw_output: None,
            meta: None,
        }
    }

    /// The category of tool being invoked.
    /// Helps clients choose appropriate icons and UI treatment.
    #[must_use]
    pub fn kind(mut self, kind: ToolKind) -> Self {
        self.kind = kind;
        self
    }

    /// Current execution status of the tool call.
    #[must_use]
    pub fn status(mut self, status: ToolCallStatus) -> Self {
        self.status = status;
        self
    }

    /// Content produced by the tool call.
    #[must_use]
    pub fn content(mut self, content: Vec<ToolCallContent>) -> Self {
        self.content = content;
        self
    }

    /// File locations affected by this tool call.
    /// Enables "follow-along" features in clients.
    #[must_use]
    pub fn locations(mut self, locations: Vec<ToolCallLocation>) -> Self {
        self.locations = locations;
        self
    }

    /// Raw input parameters sent to the tool.
    #[must_use]
    pub fn raw_input(mut self, raw_input: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_input = raw_input.into_option();
        self
    }

    /// Raw output returned by the tool.
    #[must_use]
    pub fn raw_output(mut self, raw_output: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_output = raw_output.into_option();
        self
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

    /// Update an existing tool call with the values in the provided update
    /// fields. Fields with collections of values are overwritten, not extended.
    pub fn update(&mut self, fields: ToolCallUpdateFields) {
        if let Some(title) = fields.title {
            self.title = title;
        }
        if let Some(kind) = fields.kind {
            self.kind = kind;
        }
        if let Some(status) = fields.status {
            self.status = status;
        }
        if let Some(content) = fields.content {
            self.content = content;
        }
        if let Some(locations) = fields.locations {
            self.locations = locations;
        }
        if let Some(raw_input) = fields.raw_input {
            self.raw_input = Some(raw_input);
        }
        if let Some(raw_output) = fields.raw_output {
            self.raw_output = Some(raw_output);
        }
    }
}

/// An update to an existing tool call.
///
/// Used to report progress and results as tools execute. All fields except
/// the tool call ID are optional - only changed fields need to be included.
///
/// See protocol docs: [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallUpdate {
    /// The ID of the tool call being updated.
    pub tool_call_id: ToolCallId,
    /// Fields being updated.
    #[serde(flatten)]
    pub fields: ToolCallUpdateFields,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCallUpdate {
    #[must_use]
    pub fn new(tool_call_id: impl Into<ToolCallId>, fields: ToolCallUpdateFields) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            fields,
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

/// Optional fields that can be updated in a tool call.
///
/// All fields are optional - only include the ones being changed.
/// Collections (content, locations) are overwritten, not extended.
///
/// See protocol docs: [Updating](https://agentclientprotocol.com/protocol/tool-calls#updating)
#[serde_as]
#[skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallUpdateFields {
    /// Update the tool kind.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub kind: Option<ToolKind>,
    /// Update the execution status.
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    pub status: Option<ToolCallStatus>,
    /// Update the human-readable title.
    pub title: Option<String>,
    /// Replace the content collection.
    #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, SkipListener>>>")]
    #[serde(default)]
    pub content: Option<Vec<ToolCallContent>>,
    /// Replace the locations collection.
    #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, SkipListener>>>")]
    #[serde(default)]
    pub locations: Option<Vec<ToolCallLocation>>,
    /// Update the raw input.
    pub raw_input: Option<serde_json::Value>,
    /// Update the raw output.
    pub raw_output: Option<serde_json::Value>,
}

impl ToolCallUpdateFields {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the tool kind.
    #[must_use]
    pub fn kind(mut self, kind: impl IntoOption<ToolKind>) -> Self {
        self.kind = kind.into_option();
        self
    }

    /// Update the execution status.
    #[must_use]
    pub fn status(mut self, status: impl IntoOption<ToolCallStatus>) -> Self {
        self.status = status.into_option();
        self
    }

    /// Update the human-readable title.
    #[must_use]
    pub fn title(mut self, title: impl IntoOption<String>) -> Self {
        self.title = title.into_option();
        self
    }

    /// Replace the content collection.
    #[must_use]
    pub fn content(mut self, content: impl IntoOption<Vec<ToolCallContent>>) -> Self {
        self.content = content.into_option();
        self
    }

    /// Replace the locations collection.
    #[must_use]
    pub fn locations(mut self, locations: impl IntoOption<Vec<ToolCallLocation>>) -> Self {
        self.locations = locations.into_option();
        self
    }

    /// Update the raw input.
    #[must_use]
    pub fn raw_input(mut self, raw_input: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_input = raw_input.into_option();
        self
    }

    /// Update the raw output.
    #[must_use]
    pub fn raw_output(mut self, raw_output: impl IntoOption<serde_json::Value>) -> Self {
        self.raw_output = raw_output.into_option();
        self
    }
}

/// If a given tool call doesn't exist yet, allows for attempting to construct
/// one from a tool call update if possible.
impl TryFrom<ToolCallUpdate> for ToolCall {
    type Error = Error;

    fn try_from(update: ToolCallUpdate) -> Result<Self, Self::Error> {
        let ToolCallUpdate {
            tool_call_id,
            fields:
                ToolCallUpdateFields {
                    kind,
                    status,
                    title,
                    content,
                    locations,
                    raw_input,
                    raw_output,
                },
            meta,
        } = update;

        Ok(Self {
            tool_call_id,
            title: title.ok_or_else(|| {
                Error::invalid_params().data(serde_json::json!("title is required for a tool call"))
            })?,
            kind: kind.unwrap_or_default(),
            status: status.unwrap_or_default(),
            content: content.unwrap_or_default(),
            locations: locations.unwrap_or_default(),
            raw_input,
            raw_output,
            meta,
        })
    }
}

impl From<ToolCall> for ToolCallUpdate {
    fn from(value: ToolCall) -> Self {
        let ToolCall {
            tool_call_id,
            title,
            kind,
            status,
            content,
            locations,
            raw_input,
            raw_output,
            meta,
        } = value;
        Self {
            tool_call_id,
            fields: ToolCallUpdateFields {
                kind: Some(kind),
                status: Some(status),
                title: Some(title),
                content: Some(content),
                locations: Some(locations),
                raw_input,
                raw_output,
            },
            meta,
        }
    }
}

/// Unique identifier for a tool call within a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash, Display, From)]
#[serde(transparent)]
#[from(Arc<str>, String, &'static str)]
#[non_exhaustive]
pub struct ToolCallId(pub Arc<str>);

impl ToolCallId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }
}

impl IntoOption<ToolCallId> for &str {
    fn into_option(self) -> Option<ToolCallId> {
        Some(ToolCallId::new(self))
    }
}

/// Categories of tools that can be invoked.
///
/// Tool kinds help clients choose appropriate icons and optimize how they
/// display tool execution progress.
///
/// See protocol docs: [Creating](https://agentclientprotocol.com/protocol/tool-calls#creating)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolKind {
    /// Reading files or data.
    Read,
    /// Modifying files or content.
    Edit,
    /// Removing files or data.
    Delete,
    /// Moving or renaming files.
    Move,
    /// Searching for information.
    Search,
    /// Running commands or code.
    Execute,
    /// Internal reasoning or planning.
    Think,
    /// Retrieving external data.
    Fetch,
    /// Switching the current session mode.
    SwitchMode,
    /// Other tool types (default).
    #[default]
    #[serde(other)]
    Other,
}

impl ToolKind {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "Required by serde")]
    fn is_default(&self) -> bool {
        matches!(self, ToolKind::Other)
    }
}

/// Execution status of a tool call.
///
/// Tool calls progress through different statuses during their lifecycle.
///
/// See protocol docs: [Status](https://agentclientprotocol.com/protocol/tool-calls#status)
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolCallStatus {
    /// The tool call hasn't started running yet because the input is either
    /// streaming or we're awaiting approval.
    #[default]
    Pending,
    /// The tool call is currently running.
    InProgress,
    /// The tool call completed successfully.
    Completed,
    /// The tool call failed with an error.
    Failed,
}

impl ToolCallStatus {
    #[expect(clippy::trivially_copy_pass_by_ref, reason = "Required by serde")]
    fn is_default(&self) -> bool {
        matches!(self, ToolCallStatus::Pending)
    }
}

/// Content produced by a tool call.
///
/// Tool calls can produce different types of content including
/// standard content blocks (text, images) or file diffs.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(extend("discriminator" = {"propertyName": "type"}))]
#[non_exhaustive]
pub enum ToolCallContent {
    /// Standard content block (text, images, resources).
    Content(Content),
    /// File modification shown as a diff.
    Diff(Diff),
    /// Embed a terminal created with `terminal/create` by its id.
    ///
    /// The terminal must be added before calling `terminal/release`.
    ///
    /// See protocol docs: [Terminal](https://agentclientprotocol.com/protocol/terminals)
    Terminal(Terminal),
}

impl<T: Into<ContentBlock>> From<T> for ToolCallContent {
    fn from(content: T) -> Self {
        ToolCallContent::Content(Content::new(content))
    }
}

impl From<Diff> for ToolCallContent {
    fn from(diff: Diff) -> Self {
        ToolCallContent::Diff(diff)
    }
}

/// Standard content block (text, images, resources).
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Content {
    /// The actual content block.
    pub content: ContentBlock,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Content {
    #[must_use]
    pub fn new(content: impl Into<ContentBlock>) -> Self {
        Self {
            content: content.into(),
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

/// Embed a terminal created with `terminal/create` by its id.
///
/// The terminal must be added before calling `terminal/release`.
///
/// See protocol docs: [Terminal](https://agentclientprotocol.com/protocol/terminals)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Terminal {
    pub terminal_id: TerminalId,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Terminal {
    #[must_use]
    pub fn new(terminal_id: impl Into<TerminalId>) -> Self {
        Self {
            terminal_id: terminal_id.into(),
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

/// A diff representing file modifications.
///
/// Shows changes to files in a format suitable for display in the client UI.
///
/// See protocol docs: [Content](https://agentclientprotocol.com/protocol/tool-calls#content)
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Diff {
    /// The file path being modified.
    pub path: PathBuf,
    /// The original content (None for new files).
    pub old_text: Option<String>,
    /// The new content after modification.
    pub new_text: String,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Diff {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, new_text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            old_text: None,
            new_text: new_text.into(),
            meta: None,
        }
    }

    /// The original content (None for new files).
    #[must_use]
    pub fn old_text(mut self, old_text: impl IntoOption<String>) -> Self {
        self.old_text = old_text.into_option();
        self
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

/// A file location being accessed or modified by a tool.
///
/// Enables clients to implement "follow-along" features that track
/// which files the agent is working with in real-time.
///
/// See protocol docs: [Following the Agent](https://agentclientprotocol.com/protocol/tool-calls#following-the-agent)
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ToolCallLocation {
    /// The file path being accessed or modified.
    pub path: PathBuf,
    /// Optional line number within the file.
    #[serde(default)]
    pub line: Option<u32>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl ToolCallLocation {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            line: None,
            meta: None,
        }
    }

    /// Optional line number within the file.
    #[must_use]
    pub fn line(mut self, line: impl IntoOption<u32>) -> Self {
        self.line = line.into_option();
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorCode;
    use serde_json::json;

    // ---- ToolCallUpdate -> ToolCall conversion ----

    #[test]
    fn try_from_update_with_title_succeeds() {
        let update = ToolCallUpdate::new(
            "call-1",
            ToolCallUpdateFields::new()
                .title("Read foo.rs")
                .kind(ToolKind::Read)
                .status(ToolCallStatus::InProgress),
        );

        let call: ToolCall = update.try_into().expect("update with title should convert");
        assert_eq!(call.tool_call_id, ToolCallId::new("call-1"));
        assert_eq!(call.title, "Read foo.rs");
        assert_eq!(call.kind, ToolKind::Read);
        assert_eq!(call.status, ToolCallStatus::InProgress);
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
        assert!(call.raw_input.is_none());
        assert!(call.raw_output.is_none());
    }

    #[test]
    fn try_from_update_missing_title_returns_invalid_params() {
        let update = ToolCallUpdate::new(
            "call-2",
            ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
        );

        let err: Error = ToolCall::try_from(update).expect_err("missing title must error");
        assert_eq!(err.code, ErrorCode::InvalidParams);
        // The error data must mention the offending field so callers can surface a useful message.
        let data = err.data.expect("error must include data");
        assert!(
            data.to_string().contains("title"),
            "expected data to mention title, got {data}"
        );
    }

    #[test]
    fn try_from_update_uses_defaults_for_missing_kind_and_status() {
        let update = ToolCallUpdate::new("call-3", ToolCallUpdateFields::new().title("Something"));

        let call = ToolCall::try_from(update).unwrap();
        assert_eq!(call.kind, ToolKind::default());
        assert_eq!(call.status, ToolCallStatus::default());
    }

    #[test]
    fn tool_call_into_update_round_trips_back() {
        let original = ToolCall::new("call-rt", "Run tests")
            .kind(ToolKind::Execute)
            .status(ToolCallStatus::Completed)
            .raw_input(json!({"cmd": "cargo test"}))
            .raw_output(json!({"exit_code": 0}))
            .content(vec![ToolCallContent::from("done".to_string())])
            .locations(vec![ToolCallLocation::new("/repo/src/lib.rs").line(42)]);

        let update: ToolCallUpdate = original.clone().into();
        // The round-trip should preserve the id and lossless title for ToolCall reconstruction.
        let rebuilt: ToolCall = update.try_into().expect("round-trip must succeed");

        assert_eq!(rebuilt, original);
    }

    // ---- ToolCall::update merge semantics ----

    #[test]
    fn update_only_applies_set_fields() {
        let mut call = ToolCall::new("c1", "original title")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress);

        // Only update title; everything else must be untouched.
        call.update(ToolCallUpdateFields::new().title("new title"));
        assert_eq!(call.title, "new title");
        assert_eq!(call.kind, ToolKind::Read);
        assert_eq!(call.status, ToolCallStatus::InProgress);
    }

    #[test]
    fn update_overwrites_collections_not_extends() {
        let initial_locations = vec![ToolCallLocation::new("/a"), ToolCallLocation::new("/b")];
        let initial_content = vec![ToolCallContent::from("first".to_string())];
        let mut call = ToolCall::new("c2", "title")
            .locations(initial_locations)
            .content(initial_content);

        let new_locations = vec![ToolCallLocation::new("/c")];
        let new_content = vec![ToolCallContent::from("second".to_string())];
        call.update(
            ToolCallUpdateFields::new()
                .locations(new_locations.clone())
                .content(new_content.clone()),
        );

        // Per the spec, collections are replaced (not extended).
        assert_eq!(call.locations, new_locations);
        assert_eq!(call.content, new_content);
    }

    #[test]
    fn update_with_empty_fields_is_noop() {
        let original = ToolCall::new("c3", "stable")
            .kind(ToolKind::Edit)
            .status(ToolCallStatus::Failed)
            .raw_input(json!({"x": 1}));
        let mut call = original.clone();

        call.update(ToolCallUpdateFields::new());

        assert_eq!(call, original);
    }

    // ---- ToolKind / ToolCallStatus serde behavior ----

    #[test]
    fn tool_kind_default_is_skipped_in_serialization() {
        let call = ToolCall::new("c", "t"); // kind defaults to Other, status to Pending
        let value = serde_json::to_value(&call).unwrap();
        let map = value.as_object().unwrap();
        assert!(
            !map.contains_key("kind"),
            "default ToolKind::Other must be omitted from wire format"
        );
        assert!(
            !map.contains_key("status"),
            "default ToolCallStatus::Pending must be omitted from wire format"
        );
    }

    #[test]
    fn tool_kind_unknown_variant_falls_back_to_other() {
        // Forward-compat: an unknown ToolKind in JSON must deserialize to Other rather than failing.
        let call: ToolCall = serde_json::from_value(json!({
            "toolCallId": "c",
            "title": "t",
            "kind": "some_future_kind"
        }))
        .unwrap();
        assert_eq!(call.kind, ToolKind::Other);
    }

    #[test]
    fn tool_kind_known_variants_round_trip() {
        for (kind, expected) in [
            (ToolKind::Read, "read"),
            (ToolKind::Edit, "edit"),
            (ToolKind::Delete, "delete"),
            (ToolKind::Move, "move"),
            (ToolKind::Search, "search"),
            (ToolKind::Execute, "execute"),
            (ToolKind::Think, "think"),
            (ToolKind::Fetch, "fetch"),
            (ToolKind::SwitchMode, "switch_mode"),
            (ToolKind::Other, "other"),
        ] {
            let v = serde_json::to_value(kind).unwrap();
            assert_eq!(v, json!(expected), "serialize {kind:?}");
            let parsed: ToolKind = serde_json::from_value(json!(expected)).unwrap();
            assert_eq!(parsed, kind, "deserialize {expected}");
        }
    }

    #[test]
    fn tool_call_status_known_variants_round_trip() {
        for (status, expected) in [
            (ToolCallStatus::Pending, "pending"),
            (ToolCallStatus::InProgress, "in_progress"),
            (ToolCallStatus::Completed, "completed"),
            (ToolCallStatus::Failed, "failed"),
        ] {
            let v = serde_json::to_value(status).unwrap();
            assert_eq!(v, json!(expected));
            let parsed: ToolCallStatus = serde_json::from_value(json!(expected)).unwrap();
            assert_eq!(parsed, status);
        }
    }

    // ---- Tolerance: ToolCallUpdateFields swallows malformed enum + collection values ----

    #[test]
    fn update_fields_tolerates_unknown_status_in_wire_format() {
        // ToolCallUpdateFields uses DefaultOnError for kind/status: if a peer sends an
        // unknown status string, we must not blow up — the field should fall back to None
        // so the rest of the update still applies.
        let value = json!({
            "toolCallId": "c",
            "title": "still applies",
            "status": "garbage_status"
        });
        let update: ToolCallUpdate = serde_json::from_value(value).unwrap();
        assert_eq!(update.fields.title.as_deref(), Some("still applies"));
        // ToolKind has #[serde(other)], so unknown still parses; ToolCallStatus does not,
        // so DefaultOnError must turn it into None.
        assert_eq!(update.fields.status, None);
    }

    #[test]
    fn update_fields_tolerates_malformed_outer_collections() {
        // `content` and `locations` use DefaultOnError<Option<VecSkipError<...>>> so that
        // wrong outer types collapse to None instead of blowing up the whole update.
        let value = json!({
            "toolCallId": "c",
            "title": "ok",
            "content": "not an array",
            "locations": {"oops": true}
        });
        let update: ToolCallUpdate = serde_json::from_value(value).unwrap();
        assert_eq!(update.fields.content, None);
        assert_eq!(update.fields.locations, None);
        assert_eq!(update.fields.title.as_deref(), Some("ok"));
    }

    #[test]
    fn tool_call_skips_malformed_locations_per_element() {
        // ToolCall uses DefaultOnError<VecSkipError<...>> for locations: bad entries are
        // dropped element-by-element while good ones survive.
        let value = json!({
            "toolCallId": "c",
            "title": "ok",
            "locations": [
                {"path": "/good/path"},
                "this is not a location",
                {"path": "/another/good/path", "line": 7}
            ]
        });
        let call: ToolCall = serde_json::from_value(value).unwrap();
        assert_eq!(call.locations.len(), 2);
        assert_eq!(call.locations[0].path, PathBuf::from("/good/path"));
        assert_eq!(call.locations[1].path, PathBuf::from("/another/good/path"));
        assert_eq!(call.locations[1].line, Some(7));
    }

    // ---- ToolCallContent ----

    #[test]
    fn tool_call_content_diff_serializes_with_type_tag() {
        let diff = Diff::new("/file.rs", "new").old_text("old".to_string());
        let content = ToolCallContent::Diff(diff);
        let value = serde_json::to_value(&content).unwrap();
        assert_eq!(value["type"], "diff");
        assert_eq!(value["path"], "/file.rs");
        assert_eq!(value["oldText"], "old");
        assert_eq!(value["newText"], "new");
    }

    #[test]
    fn diff_for_new_file_omits_old_text_field() {
        // For a brand-new file, old_text is None, and the wire format must omit the key
        // so consumers can distinguish "new file" from "empty original content".
        let diff = Diff::new("/created.rs", "fresh");
        let value = serde_json::to_value(&diff).unwrap();
        let map = value.as_object().unwrap();
        assert!(!map.contains_key("oldText"));
        assert_eq!(map["newText"], "fresh");
    }

    #[test]
    fn tool_call_content_terminal_serializes_with_type_tag() {
        let terminal = Terminal::new("term-7");
        let content = ToolCallContent::Terminal(terminal);
        let value = serde_json::to_value(&content).unwrap();
        assert_eq!(value["type"], "terminal");
        assert_eq!(value["terminalId"], "term-7");
    }

    #[test]
    fn tool_call_content_text_via_from_uses_content_variant() {
        // From<T: Into<ContentBlock>> wraps a string into a ContentBlock::Text and then
        // into ToolCallContent::Content. Verify both layers serialize correctly.
        let content: ToolCallContent = "hello".to_string().into();
        let value = serde_json::to_value(&content).unwrap();
        // The Content variant is tag = "content"; the inner ContentBlock is also tagged with type=text.
        assert_eq!(value["type"], "content");
        assert_eq!(value["content"]["type"], "text");
        assert_eq!(value["content"]["text"], "hello");
    }

    // ---- ToolCallId ----

    #[test]
    fn tool_call_id_display_and_constructors() {
        let id = ToolCallId::new("abc");
        assert_eq!(id.to_string(), "abc");

        let from_str: ToolCallId = "xyz".into();
        assert_eq!(from_str, ToolCallId::new("xyz"));

        let from_string: ToolCallId = String::from("123").into();
        assert_eq!(from_string, ToolCallId::new("123"));
    }

    #[test]
    fn tool_call_id_into_option_for_str_wraps_in_some() {
        let opt: Option<ToolCallId> = "wrapped".into_option();
        assert_eq!(opt, Some(ToolCallId::new("wrapped")));
    }
}
