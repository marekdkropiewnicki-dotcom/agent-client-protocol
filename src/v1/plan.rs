//! Execution plans for complex tasks that require multiple steps.
//!
//! Plans are strategies that agents share with clients through session updates,
//! providing real-time visibility into their thinking and progress.
//!
//! See: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, VecSkipError, serde_as, skip_serializing_none};

use crate::{IntoOption, Meta, SkipListener};

/// An execution plan for accomplishing complex tasks.
///
/// Plans consist of multiple entries representing individual tasks or goals.
/// Agents report plans to clients to provide visibility into their execution strategy.
/// Plans can evolve during execution as the agent discovers new requirements or completes tasks.
///
/// See protocol docs: [Agent Plan](https://agentclientprotocol.com/protocol/agent-plan)
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Plan {
    /// The list of tasks to be accomplished.
    ///
    /// When updating a plan, the agent must send a complete list of all entries
    /// with their current status. The client replaces the entire plan with each update.
    #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, SkipListener>>")]
    pub entries: Vec<PlanEntry>,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl Plan {
    #[must_use]
    pub fn new(entries: Vec<PlanEntry>) -> Self {
        Self {
            entries,
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

/// A single entry in the execution plan.
///
/// Represents a task or goal that the assistant intends to accomplish
/// as part of fulfilling the user's request.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanEntry {
    /// Human-readable description of what this task aims to accomplish.
    pub content: String,
    /// The relative importance of this task.
    /// Used to indicate which tasks are most critical to the overall goal.
    pub priority: PlanEntryPriority,
    /// Current execution status of this task.
    pub status: PlanEntryStatus,
    /// The _meta property is reserved by ACP to allow clients and agents to attach additional
    /// metadata to their interactions. Implementations MUST NOT make assumptions about values at
    /// these keys.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    #[serde(rename = "_meta")]
    pub meta: Option<Meta>,
}

impl PlanEntry {
    #[must_use]
    pub fn new(
        content: impl Into<String>,
        priority: PlanEntryPriority,
        status: PlanEntryStatus,
    ) -> Self {
        Self {
            content: content.into(),
            priority,
            status,
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

/// Priority levels for plan entries.
///
/// Used to indicate the relative importance or urgency of different
/// tasks in the execution plan.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryPriority {
    /// High priority task - critical to the overall goal.
    High,
    /// Medium priority task - important but not critical.
    Medium,
    /// Low priority task - nice to have but not essential.
    Low,
}

/// Status of a plan entry in the execution flow.
///
/// Tracks the lifecycle of each task from planning through completion.
/// See protocol docs: [Plan Entries](https://agentclientprotocol.com/protocol/agent-plan#plan-entries)
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PlanEntryStatus {
    /// The task has not started yet.
    Pending,
    /// The task is currently being worked on.
    InProgress,
    /// The task has been successfully completed.
    Completed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(text: &str, priority: PlanEntryPriority, status: PlanEntryStatus) -> PlanEntry {
        PlanEntry::new(text.to_string(), priority, status)
    }

    #[test]
    fn plan_new_omits_meta_when_none() {
        let plan = Plan::new(vec![]);
        let value = serde_json::to_value(&plan).unwrap();
        // With `#[skip_serializing_none]`, meta should be elided; the
        // entries array is *required* and stays even when empty.
        let object = value.as_object().unwrap();
        assert!(!object.contains_key("_meta"));
        assert!(object.contains_key("entries"));
        assert!(value["entries"].as_array().unwrap().is_empty());
    }

    #[test]
    fn plan_entry_priority_and_status_serialize_as_snake_case() {
        for (priority, wire) in [
            (PlanEntryPriority::High, "high"),
            (PlanEntryPriority::Medium, "medium"),
            (PlanEntryPriority::Low, "low"),
        ] {
            assert_eq!(serde_json::to_value(&priority).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<PlanEntryPriority>(json!(wire)).unwrap(),
                priority
            );
        }

        for (status, wire) in [
            (PlanEntryStatus::Pending, "pending"),
            (PlanEntryStatus::InProgress, "in_progress"),
            (PlanEntryStatus::Completed, "completed"),
        ] {
            assert_eq!(serde_json::to_value(&status).unwrap(), json!(wire));
            assert_eq!(
                serde_json::from_value::<PlanEntryStatus>(json!(wire)).unwrap(),
                status
            );
        }
    }

    #[test]
    fn plan_entry_priority_rejects_unknown_variant() {
        // No `#[serde(other)]` fallback — unknown priorities must fail
        // rather than silently degrade to a known value.
        assert!(serde_json::from_value::<PlanEntryPriority>(json!("critical")).is_err());
        assert!(serde_json::from_value::<PlanEntryStatus>(json!("blocked")).is_err());
    }

    #[test]
    fn plan_deserialization_skips_malformed_entries() {
        // The `entries` array uses `DefaultOnError<VecSkipError<_>>` so that
        // a single bad entry doesn't drop the entire plan update.
        let raw = json!({
            "entries": [
                { "content": "a", "priority": "high", "status": "pending" },
                { "content": "missing status", "priority": "low" },
                { "content": "b", "priority": "medium", "status": "completed" }
            ]
        });

        let plan: Plan = serde_json::from_value(raw).unwrap();
        // The invalid middle entry (missing status) is skipped; the two
        // well-formed entries are preserved in order.
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].content, "a");
        assert_eq!(plan.entries[1].content, "b");
    }

    #[test]
    fn plan_deserialization_defaults_whole_entries_field_on_type_error() {
        // If the whole `entries` field is completely the wrong shape,
        // `DefaultOnError` yields an empty vec rather than propagating.
        let raw = json!({ "entries": "not an array" });
        let plan: Plan = serde_json::from_value(raw).unwrap();
        assert!(plan.entries.is_empty());
    }

    #[test]
    fn plan_roundtrip_preserves_entries_and_meta() {
        let mut meta = serde_json::Map::new();
        meta.insert("trace".into(), json!("id-1"));

        let plan = Plan::new(vec![
            entry("do a", PlanEntryPriority::High, PlanEntryStatus::Pending),
            entry("do b", PlanEntryPriority::Low, PlanEntryStatus::InProgress),
        ])
        .meta(meta.clone());

        let round: Plan = serde_json::from_value(serde_json::to_value(&plan).unwrap()).unwrap();
        assert_eq!(round, plan);
        assert_eq!(round.meta.unwrap(), meta);
    }

    #[test]
    fn plan_entry_new_omits_meta_when_none() {
        let entry = PlanEntry::new(
            "just do it".to_string(),
            PlanEntryPriority::High,
            PlanEntryStatus::Pending,
        );
        let value = serde_json::to_value(&entry).unwrap();
        assert!(!value.as_object().unwrap().contains_key("_meta"));
    }
}
