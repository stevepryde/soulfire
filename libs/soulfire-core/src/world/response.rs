//! Tolerant parsing of game-master update responses (`WORLD-12`, `WORLD-13`,
//! `WORLD-16`). Strips code fences, rescues top-level memory fields the model may
//! nest inside `patches`, and coerces imperfect significant-event arrays.

use std::str::FromStr;

use serde_json::Value;

use crate::model::world::StoryStatus;

use crate::ai::fence::rescue_json_block;

use super::memory::{SignificantEvent, SignificantEventUpdates, coerce_significant_event_array};
use super::state_patch::{StatePatch, parse_patches_from_value};

/// Top-level memory fields that the model sometimes mis-routes into a patch
/// (`WORLD-12`): lift these out of `patches` to the response top level.
const RESCUE_FIELDS: [&str; 4] = [
    "new_recent_events",
    "significant_event_updates",
    "story_summary",
    "story_status",
];

/// A parsed diff state-update response (`WORLD-12`).
#[derive(Debug, Clone)]
pub struct DiffUpdate {
    pub patches: Vec<StatePatch>,
    pub new_recent_events: Vec<String>,
    pub significant_event_updates: SignificantEventUpdates,
    pub story_summary: Option<String>,
    pub story_status: Option<StoryStatus>,
}

/// A parsed full state-update response (`WORLD-13`).
#[derive(Debug, Clone)]
pub struct FullUpdate {
    /// The complete replacement adventure-state, serialized to a string.
    pub updated_state: String,
    pub recent_events: Vec<String>,
    pub significant_events: Vec<SignificantEvent>,
    pub story_summary: Option<String>,
    pub story_status: Option<StoryStatus>,
}

/// The classified intent of a `/gm` request (`WORLD-16`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GmIntent {
    AnswerOnly,
    AdventureState,
    WorldBlueprint,
    Both,
}

impl GmIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            GmIntent::AnswerOnly => "answer_only",
            GmIntent::AdventureState => "adventure_state",
            GmIntent::WorldBlueprint => "world_blueprint",
            GmIntent::Both => "both",
        }
    }
    pub fn changes_state(self) -> bool {
        matches!(self, GmIntent::AdventureState | GmIntent::Both)
    }
    pub fn changes_blueprint(self) -> bool {
        matches!(self, GmIntent::WorldBlueprint | GmIntent::Both)
    }
}

/// A parsed `/gm` change proposal (`WORLD-16`, `WORLD-17`).
#[derive(Debug, Clone)]
pub struct GmProposalResponse {
    pub response: String,
    pub updated_adventure_state: Option<String>,
    pub updated_world_blueprint: Option<String>,
    pub new_recent_events: Vec<String>,
    pub significant_event_updates: SignificantEventUpdates,
    pub story_summary: Option<String>,
    pub story_status: Option<StoryStatus>,
}

fn parse_value(text: &str) -> anyhow::Result<Value> {
    Ok(serde_json::from_str(rescue_json_block(text))?)
}

fn parse_status(v: &Value) -> Option<StoryStatus> {
    v.get("story_status")
        .and_then(Value::as_str)
        .and_then(|s| StoryStatus::from_str(s).ok())
}

fn parse_string_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// State serialized to a string: a string value passes through; an object is
/// serialized to compact JSON (`WORLD-13`).
fn state_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Parse a diff update, rescuing mis-routed top-level fields (`WORLD-12`).
pub fn parse_diff_update(text: &str) -> anyhow::Result<DiffUpdate> {
    let mut value = parse_value(text)?;
    rescue_misrouted_fields(&mut value);

    let patches = parse_patches_from_value(&value).unwrap_or_default();
    let new_recent_events = parse_string_array(&value, "new_recent_events");
    let significant_event_updates = value
        .get("significant_event_updates")
        .cloned()
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();
    let story_summary = value
        .get("story_summary")
        .and_then(Value::as_str)
        .map(String::from);
    let story_status = parse_status(&value);

    Ok(DiffUpdate {
        patches,
        new_recent_events,
        significant_event_updates,
        story_summary,
        story_status,
    })
}

/// Parse a full update, coercing the significant-events array (`WORLD-13`).
pub fn parse_full_update(text: &str, next_id: u32) -> anyhow::Result<FullUpdate> {
    let value = parse_value(text)?;
    let updated_state = value
        .get("updated_state")
        .map(state_to_string)
        .ok_or_else(|| anyhow::anyhow!("full update missing 'updated_state'"))?;
    let recent_events = parse_string_array(&value, "recent_events");
    let significant_events = value
        .get("significant_events")
        .map(|v| coerce_significant_event_array(v, next_id))
        .unwrap_or_default();
    let story_summary = value
        .get("story_summary")
        .and_then(Value::as_str)
        .map(String::from);
    let story_status = parse_status(&value);

    Ok(FullUpdate {
        updated_state,
        recent_events,
        significant_events,
        story_summary,
        story_status,
    })
}

/// Extract the adventure-state string from an initial-state generation
/// response (`WORLD-3`): the model returns the state object directly, possibly
/// fenced. Returns the compact JSON string (an object passes through serialized).
pub fn rescue_state_string(text: &str) -> String {
    let inner = rescue_json_block(text);
    match serde_json::from_str::<Value>(inner) {
        Ok(v) => state_to_string(&v),
        Err(_) => inner.to_string(),
    }
}

/// Parse the `/gm` classification, defaulting to answer-only (`WORLD-16`).
pub fn parse_gm_intent(text: &str) -> GmIntent {
    let intent = parse_value(text)
        .ok()
        .and_then(|v| v.get("intent").and_then(Value::as_str).map(String::from))
        .unwrap_or_default();
    match intent.as_str() {
        "adventure_state" => GmIntent::AdventureState,
        "world_blueprint" => GmIntent::WorldBlueprint,
        "both" => GmIntent::Both,
        _ => GmIntent::AnswerOnly,
    }
}

/// Parse the `/gm` answer-only response text (`WORLD-16`).
pub fn parse_gm_answer(text: &str) -> String {
    parse_value(text)
        .ok()
        .and_then(|v| v.get("response").and_then(Value::as_str).map(String::from))
        .unwrap_or_else(|| text.trim().to_string())
}

/// Parse a `/gm` change proposal (`WORLD-16`, `WORLD-17`).
pub fn parse_gm_proposal(text: &str) -> anyhow::Result<GmProposalResponse> {
    let value = parse_value(text)?;
    let response = value
        .get("response")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let updated_adventure_state = value
        .get("updated_adventure_state")
        .filter(|v| !v.is_null())
        .map(state_to_string);
    let updated_world_blueprint = value
        .get("updated_world_blueprint")
        .and_then(Value::as_str)
        .map(String::from);
    let significant_event_updates = value
        .get("significant_event_updates")
        .cloned()
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();

    Ok(GmProposalResponse {
        response,
        updated_adventure_state,
        updated_world_blueprint,
        new_recent_events: parse_string_array(&value, "new_recent_events"),
        significant_event_updates,
        story_summary: value
            .get("story_summary")
            .and_then(Value::as_str)
            .map(String::from),
        story_status: parse_status(&value),
    })
}

/// Move any rescue fields nested inside `patches` up to the top level (`WORLD-12`).
fn rescue_misrouted_fields(value: &mut Value) {
    let Some(patches) = value.get("patches").and_then(Value::as_array).cloned() else {
        return;
    };
    let mut rescued: Vec<(String, Value)> = Vec::new();
    let mut kept: Vec<Value> = Vec::new();
    for patch in patches {
        let path = patch.get("path").and_then(Value::as_str).unwrap_or("");
        if RESCUE_FIELDS.contains(&path) {
            if let Some(v) = patch.get("value").cloned() {
                rescued.push((path.to_string(), v));
            }
        } else {
            kept.push(patch);
        }
    }
    if rescued.is_empty() {
        return;
    }
    if let Some(obj) = value.as_object_mut() {
        obj.insert("patches".to_string(), Value::Array(kept));
        for (k, v) in rescued {
            obj.entry(k).or_insert(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_diff_update() {
        // AC-AI-b / WORLD-12: a fenced JSON diff parses.
        let text = r###"```json
{
  "patches": [{"path": "current_situation.time", "value": "evening"}],
  "new_recent_events": ["Left the tavern at dusk"],
  "significant_event_updates": {"add": [{"text": "Met the captain", "weight": 4}]},
  "story_summary": "## Rolling Story\nThings happened.",
  "story_status": "ongoing"
}
```"###;
        let diff = parse_diff_update(text).unwrap();
        assert_eq!(diff.patches.len(), 1);
        assert_eq!(diff.new_recent_events, vec!["Left the tavern at dusk"]);
        assert_eq!(diff.significant_event_updates.add.len(), 1);
        assert_eq!(diff.story_status, Some(StoryStatus::Ongoing));
    }

    #[test]
    fn rescues_misrouted_memory_fields_from_patches() {
        // WORLD-12: model nests story_summary inside patches; we lift it out.
        let text = r###"{
            "patches": [
                {"path": "player.hp", "value": 9},
                {"path": "story_summary", "value": "## Rolling Story\nrescued"}
            ]
        }"###;
        let diff = parse_diff_update(text).unwrap();
        assert_eq!(diff.patches.len(), 1); // story_summary patch removed
        assert_eq!(diff.patches[0].path, "player.hp");
        assert_eq!(
            diff.story_summary.as_deref(),
            Some("## Rolling Story\nrescued")
        );
    }

    #[test]
    fn parses_full_update_with_object_or_string_state() {
        let obj = r#"{"updated_state": {"player": {"hp": 10}}, "story_status": "success"}"#;
        let full = parse_full_update(obj, 1).unwrap();
        assert!(full.updated_state.contains("\"hp\""));
        assert_eq!(full.story_status, Some(StoryStatus::Success));

        let strv = r#"{"updated_state": "{\"player\":{\"hp\":3}}"}"#;
        let full2 = parse_full_update(strv, 1).unwrap();
        assert_eq!(full2.updated_state, "{\"player\":{\"hp\":3}}");
    }

    #[test]
    fn gm_intent_parses_and_defaults_to_answer_only() {
        assert_eq!(parse_gm_intent(r#"{"intent":"both"}"#), GmIntent::Both);
        assert_eq!(parse_gm_intent("garbage"), GmIntent::AnswerOnly);
        assert!(GmIntent::Both.changes_state());
        assert!(GmIntent::Both.changes_blueprint());
    }

    #[test]
    fn gm_proposal_parses_optional_targets() {
        let text = r#"{
            "response": "Skipped to morning.",
            "updated_adventure_state": {"current_situation": {"time": "morning", "day": 2}}
        }"#;
        let p = parse_gm_proposal(text).unwrap();
        assert_eq!(p.response, "Skipped to morning.");
        assert!(p.updated_adventure_state.is_some());
        assert!(p.updated_world_blueprint.is_none());
    }
}
