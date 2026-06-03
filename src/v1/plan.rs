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

    fn entry(content: &str, status: PlanEntryStatus) -> PlanEntry {
        PlanEntry::new(content, PlanEntryPriority::Medium, status)
    }

    #[test]
    fn plan_round_trip_preserves_entries_and_omits_optional_meta() {
        let plan = Plan::new(vec![
            entry("write tests", PlanEntryStatus::Completed),
            entry("ship it", PlanEntryStatus::InProgress),
        ]);

        let json = serde_json::to_value(&plan).unwrap();
        assert!(
            !json.as_object().unwrap().contains_key("_meta"),
            "meta must be omitted when None to keep the wire format compact"
        );
        let parsed: Plan = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, plan);
    }

    #[test]
    fn plan_priority_and_status_use_snake_case_on_the_wire() {
        let plan = Plan::new(vec![PlanEntry::new(
            "do the thing",
            PlanEntryPriority::High,
            PlanEntryStatus::InProgress,
        )]);
        let json = serde_json::to_value(&plan).unwrap();
        let only = &json["entries"][0];
        assert_eq!(only["priority"], "high");
        assert_eq!(
            only["status"], "in_progress",
            "snake_case is part of the wire contract"
        );
    }

    // ---- Forward-compat: VecSkipError<_, SkipListener> drops bad entries ----

    #[test]
    fn malformed_plan_entries_are_skipped_so_a_single_bad_row_does_not_kill_the_plan() {
        // Three entries: a valid one, one with an unknown status, and another valid one.
        // The unknown-status entry must be skipped while the valid entries survive.
        let payload = json!({
            "entries": [
                {"content": "first", "priority": "low", "status": "pending"},
                {"content": "second", "priority": "medium", "status": "from_the_future"},
                {"content": "third", "priority": "high", "status": "completed"}
            ]
        });

        let plan: Plan = serde_json::from_value(payload).unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].content, "first");
        assert_eq!(plan.entries[1].content, "third");
        assert_eq!(plan.entries[1].priority, PlanEntryPriority::High);
    }

    #[test]
    fn plan_with_non_array_entries_falls_back_to_empty_entries() {
        // DefaultOnError around the outer VecSkipError must default to an empty
        // entries list rather than failing the whole Plan deserialization when
        // the agent sends a wholly malformed payload.
        let payload = json!({
            "entries": {"not": "an array"}
        });
        let plan: Plan = serde_json::from_value(payload).unwrap();
        assert!(plan.entries.is_empty());
    }

    #[test]
    fn plan_entry_unknown_priority_drops_only_that_entry() {
        let payload = json!({
            "entries": [
                {"content": "keep", "priority": "low", "status": "pending"},
                {"content": "drop", "priority": "ULTRA", "status": "pending"}
            ]
        });
        let plan: Plan = serde_json::from_value(payload).unwrap();
        assert_eq!(plan.entries.len(), 1);
        assert_eq!(plan.entries[0].content, "keep");
    }

    #[test]
    fn plan_entry_round_trips_with_meta() {
        let mut meta = Meta::new();
        meta.insert("source".into(), json!("planner-v2"));
        let entry = PlanEntry::new("do work", PlanEntryPriority::Low, PlanEntryStatus::Pending)
            .meta(Some(meta.clone()));

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["_meta"]["source"], "planner-v2");
        let parsed: PlanEntry = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, entry);
    }
}
