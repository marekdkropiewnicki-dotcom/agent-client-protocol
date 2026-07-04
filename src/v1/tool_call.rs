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
    use crate::{ContentBlock, ErrorCode, TextContent};
    use serde_json::json;

    fn sample_tool_call() -> ToolCall {
        ToolCall::new("call-1", "Read file")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::InProgress)
            .content(vec![ToolCallContent::from(ContentBlock::Text(
                TextContent {
                    annotations: None,
                    text: "before".into(),
                    meta: None,
                },
            ))])
            .locations(vec![ToolCallLocation::new("/tmp/a").line(3u32)])
            .raw_input(json!({ "path": "/tmp/a" }))
            .raw_output(json!({ "bytes": 1 }))
    }

    #[test]
    fn tool_call_new_omits_default_and_optional_fields() {
        let call = ToolCall::new("call-1", "Do work");
        let json = serde_json::to_value(&call).unwrap();
        let object = json.as_object().unwrap();

        assert_eq!(object.get("toolCallId").unwrap(), "call-1");
        assert_eq!(object.get("title").unwrap(), "Do work");
        // Defaults (ToolKind::Other, ToolCallStatus::Pending) and empty
        // collections must be elided so the wire payload stays minimal.
        assert!(
            !object.contains_key("kind"),
            "kind should be omitted when default"
        );
        assert!(
            !object.contains_key("status"),
            "status should be omitted when default"
        );
        assert!(!object.contains_key("content"));
        assert!(!object.contains_key("locations"));
        assert!(!object.contains_key("rawInput"));
        assert!(!object.contains_key("rawOutput"));
        assert!(!object.contains_key("_meta"));
    }

    #[test]
    fn tool_call_non_default_kind_and_status_are_serialized() {
        let call = ToolCall::new("call-1", "Edit")
            .kind(ToolKind::Edit)
            .status(ToolCallStatus::Completed);
        let json = serde_json::to_value(&call).unwrap();
        assert_eq!(json["kind"], "edit");
        assert_eq!(json["status"], "completed");
    }

    #[test]
    fn tool_call_update_leaves_untouched_fields_intact() {
        let mut call = sample_tool_call();
        let before = call.clone();

        // Empty update should be a no-op for every field.
        call.update(ToolCallUpdateFields::default());

        assert_eq!(call.tool_call_id, before.tool_call_id);
        assert_eq!(call.title, before.title);
        assert_eq!(call.kind, before.kind);
        assert_eq!(call.status, before.status);
        assert_eq!(call.content, before.content);
        assert_eq!(call.locations, before.locations);
        assert_eq!(call.raw_input, before.raw_input);
        assert_eq!(call.raw_output, before.raw_output);
        assert_eq!(call.meta, before.meta);
    }

    #[test]
    fn tool_call_update_overwrites_scalar_and_replaces_collections() {
        let mut call = sample_tool_call();

        let replacement_content = vec![ToolCallContent::from(ContentBlock::Text(TextContent {
            annotations: None,
            text: "after".into(),
            meta: None,
        }))];
        let replacement_locations = vec![ToolCallLocation::new("/tmp/b")];

        call.update(
            ToolCallUpdateFields::new()
                .title("Updated title".to_string())
                .kind(ToolKind::Edit)
                .status(ToolCallStatus::Completed)
                .content(replacement_content.clone())
                .locations(replacement_locations.clone())
                .raw_input(json!({ "path": "/tmp/b" }))
                .raw_output(json!({ "bytes": 2 })),
        );

        assert_eq!(call.title, "Updated title");
        assert_eq!(call.kind, ToolKind::Edit);
        assert_eq!(call.status, ToolCallStatus::Completed);
        // Collections are *replaced*, not extended.
        assert_eq!(call.content.len(), 1);
        assert_eq!(call.content, replacement_content);
        assert_eq!(call.locations, replacement_locations);
        assert_eq!(call.raw_input, Some(json!({ "path": "/tmp/b" })));
        assert_eq!(call.raw_output, Some(json!({ "bytes": 2 })));
    }

    #[test]
    fn tool_call_update_cannot_clear_raw_input_once_set() {
        // `ToolCall::update` uses `if let Some(x) = fields.raw_input` semantics,
        // which means callers cannot clear `raw_input`/`raw_output` back to
        // `None` through an update. Lock that behavior in.
        let mut call = ToolCall::new("call-1", "t").raw_input(json!({ "a": 1 }));
        call.update(ToolCallUpdateFields::default());
        assert_eq!(call.raw_input, Some(json!({ "a": 1 })));
    }

    #[test]
    fn tool_call_from_update_requires_title() {
        let update = ToolCallUpdate::new("call-1", ToolCallUpdateFields::default());
        let err = ToolCall::try_from(update).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidParams);
    }

    #[test]
    fn tool_call_from_update_populates_all_fields_and_uses_defaults() {
        // When title is present but other collections/scalars are missing,
        // defaults are filled in and optional fields propagate.
        let update = ToolCallUpdate::new(
            "call-1",
            ToolCallUpdateFields::new().title("hello".to_string()),
        )
        .meta(json!({ "trace": "abc" }).as_object().cloned().unwrap());

        let call = ToolCall::try_from(update).expect("title present -> Ok");
        assert_eq!(call.title, "hello");
        assert_eq!(call.kind, ToolKind::default());
        assert_eq!(call.status, ToolCallStatus::default());
        assert!(call.content.is_empty());
        assert!(call.locations.is_empty());
        assert_eq!(call.raw_input, None);
        assert_eq!(call.raw_output, None);
        assert!(call.meta.is_some());
    }

    #[test]
    fn tool_call_into_update_roundtrip_preserves_fields() {
        let call = sample_tool_call();
        let update: ToolCallUpdate = call.clone().into();

        // Every scalar/collection is preserved and wrapped in Some
        // when converting `ToolCall -> ToolCallUpdate`.
        assert_eq!(update.tool_call_id, call.tool_call_id);
        assert_eq!(update.fields.title.as_deref(), Some(call.title.as_str()));
        assert_eq!(update.fields.kind, Some(call.kind));
        assert_eq!(update.fields.status, Some(call.status));
        assert_eq!(update.fields.content.as_ref(), Some(&call.content));
        assert_eq!(update.fields.locations.as_ref(), Some(&call.locations));
        assert_eq!(update.fields.raw_input, call.raw_input);
        assert_eq!(update.fields.raw_output, call.raw_output);

        // Round-tripping back through `TryFrom` restores an equal ToolCall.
        let restored = ToolCall::try_from(update).unwrap();
        assert_eq!(restored, call);
    }

    #[test]
    fn tool_kind_unknown_deserializes_to_other() {
        // `#[serde(other)]` must catch unknown variants so that new tool
        // kinds added on the wire don't crash existing clients.
        let kind: ToolKind = serde_json::from_value(json!("brand_new_thing")).unwrap();
        assert_eq!(kind, ToolKind::Other);
    }

    #[test]
    fn tool_kind_known_variants_roundtrip() {
        for (kind, wire) in [
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
            assert_eq!(serde_json::to_value(kind).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<ToolKind>(json!(wire)).unwrap(),
                kind,
                "roundtrip failed for {wire}"
            );
        }
    }

    #[test]
    fn tool_call_status_unknown_variant_is_rejected() {
        // ToolCallStatus intentionally has no `#[serde(other)]` fallback,
        // so unrecognized statuses must be surfaced as errors instead of
        // silently mapping to `Pending`.
        let err = serde_json::from_value::<ToolCallStatus>(json!("weird")).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("weird") || msg.contains("variant"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn tool_call_deserialization_skips_malformed_content_entries() {
        // The `content` array uses `DefaultOnError<VecSkipError<_>>` so that a
        // single bad entry doesn't drop the whole tool call update.
        let raw = json!({
            "toolCallId": "id",
            "title": "t",
            "content": [
                { "type": "content", "content": { "type": "text", "text": "keep" } },
                { "type": "not_a_real_variant" },
                { "type": "diff", "path": "/x", "newText": "keep too" }
            ],
            "locations": "not an array"
        });

        let call: ToolCall = serde_json::from_value(raw).unwrap();
        // The malformed middle entry is dropped, the two valid ones survive.
        assert_eq!(call.content.len(), 2);
        // A whole-field type error defaults the field instead of failing.
        assert!(call.locations.is_empty());
    }

    #[test]
    fn tool_call_update_flattens_fields_on_wire() {
        // `ToolCallUpdate` flattens `ToolCallUpdateFields` so that a client
        // sees `{ toolCallId, status, ... }` and not a nested `fields` object.
        let update = ToolCallUpdate::new(
            "call-1",
            ToolCallUpdateFields::new().status(ToolCallStatus::Completed),
        );
        let value = serde_json::to_value(&update).unwrap();
        let object = value.as_object().unwrap();
        assert!(object.contains_key("toolCallId"));
        assert!(object.contains_key("status"));
        assert!(!object.contains_key("fields"));
    }

    #[test]
    fn tool_call_content_from_text_variant_wraps_in_content() {
        let block: ContentBlock = ContentBlock::Text(TextContent {
            annotations: None,
            text: "hi".into(),
            meta: None,
        });
        let content: ToolCallContent = block.into();
        assert!(matches!(content, ToolCallContent::Content(_)));
    }

    #[test]
    fn tool_call_content_from_diff_variant_wraps_in_diff() {
        let diff = Diff::new("/tmp/x", "new");
        let content: ToolCallContent = diff.into();
        assert!(matches!(content, ToolCallContent::Diff(_)));
    }

    #[test]
    fn diff_omits_old_text_when_unset() {
        let diff = Diff::new("/tmp/x", "new");
        let value = serde_json::to_value(&diff).unwrap();
        assert!(!value.as_object().unwrap().contains_key("oldText"));
    }

    #[test]
    fn tool_call_location_line_is_optional_on_wire() {
        let loc = ToolCallLocation::new("/tmp/y");
        let value = serde_json::to_value(&loc).unwrap();
        // `line` is `Option<u32>`; when `None` it should not appear on the wire.
        assert!(!value.as_object().unwrap().contains_key("line"));
    }
}
