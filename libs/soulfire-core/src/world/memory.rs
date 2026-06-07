//! The adventure memory ladder (`WORLD-9`, `WORLD-10`).
//!
//! Ported from Soulfire-OG `services/adventure_service/memory.rs`. Three stores:
//! recent events (newest-first, capped 20), significant events (`{id,text,weight}`
//! with stable `evt_N` ids, capped 30 via weighted age-decay pruning), and the
//! story summary (a `## Rolling Story` recap plus a `## Recent Turns` list capped
//! at 5). All tolerate legacy/garbled input and are never silently wiped.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Significant-event cap before weighted pruning (`WORLD-9`).
pub const SIGNIFICANT_EVENTS_CAP: usize = 30;
/// Age-decay divisor for significant-event pruning (`WORLD-9`).
pub const SIGNIFICANT_EVENTS_DECAY_RATE: u32 = 5;
/// Cap on the `## Recent Turns` list (`WORLD-9`).
pub const RECENT_TURNS_CAP: usize = 5;
/// Cap on recent-event continuity lines (`WORLD-9`).
pub const RECENT_EVENTS_CAP: usize = 20;

const FALLBACK_PLAYER_ACTION_MAX_WORDS: usize = 200;
const FALLBACK_GM_NARRATIVE_MAX_WORDS: usize = 500;
const RECENT_TURNS_SECTION: &str = "## Recent Turns";
const ROLLING_STORY_SECTION: &str = "## Rolling Story";

fn default_weight() -> u8 {
    3
}

/// A long-lived story memory item with a stable `evt_N` id and 1-5 weight
/// (`WORLD-9`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignificantEvent {
    pub id: String,
    pub text: String,
    #[serde(default = "default_weight")]
    pub weight: u8,
}

/// New significant-event payload from an AI response: a plain string or an object
/// with an explicit weight.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum NewSignificantEvent {
    Plain(String),
    WithWeight {
        text: String,
        #[serde(default = "default_weight")]
        weight: u8,
    },
}

impl NewSignificantEvent {
    pub fn text(&self) -> &str {
        match self {
            NewSignificantEvent::Plain(s) => s,
            NewSignificantEvent::WithWeight { text, .. } => text,
        }
    }
    pub fn weight(&self) -> u8 {
        match self {
            NewSignificantEvent::Plain(_) => default_weight(),
            NewSignificantEvent::WithWeight { weight, .. } => *weight,
        }
    }
}

/// Diff-mode significant-event operations (`WORLD-12`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SignificantEventUpdates {
    #[serde(default)]
    pub add: Vec<NewSignificantEvent>,
    #[serde(default)]
    pub update: HashMap<String, String>,
    #[serde(default)]
    pub remove: Vec<String>,
}

// ----- Recent events (capped 20) -----

/// Parse recent-event memory; a legacy freeform string becomes one event
/// (`WORLD-10`).
pub fn parse_recent_events(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_else(|_| vec![raw.to_string()])
}

pub fn serialize_recent_events(events: &[String]) -> String {
    serde_json::to_string(events).unwrap_or_default()
}

/// Prepend new events and keep the latest [`RECENT_EVENTS_CAP`] (`WORLD-9`).
pub fn merge_recent_events(existing: &[String], new: &[String]) -> Vec<String> {
    let mut merged: Vec<String> = new.to_vec();
    merged.extend_from_slice(existing);
    merged.truncate(RECENT_EVENTS_CAP);
    merged
}

pub fn format_recent_events_for_prompt(events: &[String]) -> String {
    if events.is_empty() {
        return "None yet.".to_string();
    }
    events
        .iter()
        .enumerate()
        .map(|(i, e)| format!("{}. {}", i + 1, e))
        .collect::<Vec<_>>()
        .join("\n")
}

// ----- Story summary (Rolling Story + Recent Turns) -----

/// Compose the story-summary blob with a `## Rolling Story` recap and a
/// `## Recent Turns` list capped at [`RECENT_TURNS_CAP`] (`WORLD-9`).
pub fn compose_story_summary_with_recent_turns(
    story_summary: &str,
    previous_story_summary: &str,
    recent_turns: &[String],
    action: &str,
    narrative: &str,
) -> String {
    let rolling_story = strip_recent_turns_section(story_summary);
    let mut turns = if recent_turns.is_empty() {
        vec![fallback_recent_turn(action, narrative)]
    } else {
        recent_turns
            .iter()
            .map(|turn| turn.trim().to_string())
            .filter(|turn| !turn.is_empty())
            .collect()
    };

    for turn in parse_recent_turns_from_story_summary(previous_story_summary) {
        if !turns.iter().any(|existing| existing == &turn) {
            turns.push(turn);
        }
        if turns.len() >= RECENT_TURNS_CAP {
            break;
        }
    }
    turns.truncate(RECENT_TURNS_CAP);

    let mut sections = Vec::new();
    let rolling_story = rolling_story.trim();
    if !rolling_story.is_empty() {
        sections.push(format!("{ROLLING_STORY_SECTION}\n{rolling_story}"));
    }
    if !turns.is_empty() {
        sections.push(format!(
            "{RECENT_TURNS_SECTION}\n{}",
            turns
                .iter()
                .enumerate()
                .map(|(index, turn)| format!("{}. {}", index + 1, turn))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    sections.join("\n\n")
}

pub fn story_summary_has_recent_turns(summary: &str) -> bool {
    summary.contains(RECENT_TURNS_SECTION)
}

fn strip_recent_turns_section(summary: &str) -> String {
    let before_recent_turns = summary
        .split_once(RECENT_TURNS_SECTION)
        .map(|(before, _)| before)
        .unwrap_or(summary)
        .trim();
    before_recent_turns
        .strip_prefix(ROLLING_STORY_SECTION)
        .unwrap_or(before_recent_turns)
        .trim()
        .to_string()
}

fn parse_recent_turns_from_story_summary(summary: &str) -> Vec<String> {
    let Some((_, turns_section)) = summary.split_once(RECENT_TURNS_SECTION) else {
        return Vec::new();
    };
    turns_section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            Some(strip_turn_marker(trimmed).to_string())
        })
        .filter(|turn| !turn.is_empty())
        .collect()
}

fn strip_turn_marker(line: &str) -> &str {
    let trimmed = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .unwrap_or(line);
    let Some((prefix, rest)) = trimmed.split_once(". ") else {
        return trimmed;
    };
    if prefix.chars().all(|ch| ch.is_ascii_digit()) {
        rest
    } else {
        trimmed
    }
}

fn fallback_recent_turn(action: &str, narrative: &str) -> String {
    format!(
        "Player: {}; GM: {}",
        compact_for_recent_turn(action, FALLBACK_PLAYER_ACTION_MAX_WORDS),
        compact_for_recent_turn(narrative, FALLBACK_GM_NARRATIVE_MAX_WORDS)
    )
}

fn compact_for_recent_turn(text: &str, max_words: usize) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        return words.join(" ");
    }
    if let Some(sentence_limited) = compact_to_sentence_boundary(text, max_words) {
        return sentence_limited;
    }
    let mut truncated = words
        .into_iter()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ");
    truncated.push_str("...");
    truncated
}

fn compact_to_sentence_boundary(text: &str, max_words: usize) -> Option<String> {
    let mut result = Vec::new();
    let mut word_count = 0;
    for sentence in text.split_inclusive(['.', '!', '?']) {
        let sentence = sentence.trim();
        if sentence.is_empty() {
            continue;
        }
        let sentence_word_count = sentence.split_whitespace().count();
        if sentence_word_count > max_words || word_count + sentence_word_count > max_words {
            break;
        }
        result.push(sentence);
        word_count += sentence_word_count;
    }
    if result.is_empty() {
        None
    } else {
        let mut compact = result.join(" ");
        compact.push_str("...");
        Some(compact)
    }
}

// ----- Significant events (capped 30, weighted decay) -----

/// Parse significant-event memory; legacy freeform becomes one `evt_1`
/// (`WORLD-10`).
pub fn parse_significant_events(raw: &str) -> Vec<SignificantEvent> {
    if raw.is_empty() {
        return Vec::new();
    }
    serde_json::from_str::<Vec<SignificantEvent>>(raw).unwrap_or_else(|_| {
        vec![SignificantEvent {
            id: "evt_1".to_string(),
            text: raw.to_string(),
            weight: default_weight(),
        }]
    })
}

pub fn serialize_significant_events(events: &[SignificantEvent]) -> String {
    serde_json::to_string(events).unwrap_or_default()
}

/// Compute the next `evt_N` id from the highest existing numeric id.
pub fn next_id_from_events(events: &[SignificantEvent]) -> u32 {
    events
        .iter()
        .filter_map(event_number)
        .max()
        .map(|max_id| max_id + 1)
        .unwrap_or(1)
}

/// Apply diff-mode memory ops (remove, then text-update, then add), returning the
/// new list and next id. New weights clamp to 1-5; updates preserve weight.
pub fn apply_significant_event_updates(
    existing: &[SignificantEvent],
    updates: &SignificantEventUpdates,
    mut next_id: u32,
) -> (Vec<SignificantEvent>, u32) {
    let remove_set: HashSet<&str> = updates.remove.iter().map(String::as_str).collect();
    let mut result: Vec<SignificantEvent> = existing
        .iter()
        .filter(|event| !remove_set.contains(event.id.as_str()))
        .map(|event| match updates.update.get(&event.id) {
            Some(new_text) => SignificantEvent {
                id: event.id.clone(),
                text: new_text.clone(),
                weight: event.weight,
            },
            None => event.clone(),
        })
        .collect();
    for new_event in &updates.add {
        result.push(SignificantEvent {
            id: format!("evt_{next_id}"),
            text: new_event.text().to_string(),
            weight: new_event.weight().clamp(1, 5),
        });
        next_id += 1;
    }
    (result, next_id)
}

/// Keep significant-event memory within the cap, pruning by `weight - age_decay`
/// (oldest low-weight first), preserving the order of those retained (`WORLD-9`).
pub fn prune_significant_events(events: &mut Vec<SignificantEvent>, next_id: u32) {
    if events.len() <= SIGNIFICANT_EVENTS_CAP {
        return;
    }
    let mut prune_order: Vec<(usize, i32, u32)> = events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            (
                index,
                effective_priority(event, next_id),
                event_number(event).unwrap_or(0),
            )
        })
        .collect();
    prune_order.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));
    let remove_count = events.len() - SIGNIFICANT_EVENTS_CAP;
    let remove_indices: HashSet<usize> = prune_order
        .iter()
        .take(remove_count)
        .map(|(index, _, _)| *index)
        .collect();
    let mut index = 0;
    events.retain(|_| {
        let keep = !remove_indices.contains(&index);
        index += 1;
        keep
    });
}

pub fn format_significant_events_for_prompt(events: &[SignificantEvent]) -> String {
    if events.is_empty() {
        return "None yet.".to_string();
    }
    events
        .iter()
        .map(|event| format!("[{}] {}", event.id, event.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Coerce tolerant AI output (object array / string array / single string) into
/// significant-event records, for the full-replacement fallback (`WORLD-10`).
pub fn coerce_significant_event_array(
    value: &serde_json::Value,
    id_start: u32,
) -> Vec<SignificantEvent> {
    if let Ok(events) = serde_json::from_value::<Vec<SignificantEvent>>(value.clone()) {
        return events;
    }
    if let serde_json::Value::Array(items) = value {
        let strings: Vec<String> = items
            .iter()
            .filter_map(|item| match item {
                serde_json::Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        if !strings.is_empty() {
            return strings
                .into_iter()
                .enumerate()
                .map(|(index, text)| SignificantEvent {
                    id: format!("evt_{}", id_start + index as u32),
                    text,
                    weight: default_weight(),
                })
                .collect();
        }
    }
    if let serde_json::Value::String(text) = value {
        return vec![SignificantEvent {
            id: format!("evt_{id_start}"),
            text: text.clone(),
            weight: default_weight(),
        }];
    }
    Vec::new()
}

fn effective_priority(event: &SignificantEvent, next_id: u32) -> i32 {
    let event_number = event_number(event).unwrap_or(0);
    let age = next_id.saturating_sub(event_number);
    let decay = (age / SIGNIFICANT_EVENTS_DECAY_RATE) as i32;
    (event.weight as i32 - decay).max(0)
}

fn event_number(event: &SignificantEvent) -> Option<u32> {
    event
        .id
        .strip_prefix("evt_")
        .and_then(|number| number.parse::<u32>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recent_events_parse_merge_and_cap() {
        assert_eq!(parse_recent_events(r#"["a","b"]"#), vec!["a", "b"]);
        // Legacy freeform string becomes one event (WORLD-10).
        assert_eq!(parse_recent_events("legacy"), vec!["legacy"]);
        assert!(parse_recent_events("").is_empty());
        // Newest-first, capped at 20.
        let existing: Vec<String> = (0..20).map(|i| format!("old{i}")).collect();
        let merged = merge_recent_events(&existing, &["new".to_string()]);
        assert_eq!(merged.len(), RECENT_EVENTS_CAP);
        assert_eq!(merged[0], "new");
    }

    #[test]
    fn significant_events_legacy_and_next_id() {
        assert_eq!(parse_significant_events("freeform")[0].id, "evt_1");
        let events = vec![
            SignificantEvent {
                id: "evt_1".into(),
                text: "a".into(),
                weight: 3,
            },
            SignificantEvent {
                id: "evt_7".into(),
                text: "b".into(),
                weight: 3,
            },
        ];
        assert_eq!(next_id_from_events(&events), 8);
    }

    #[test]
    fn diff_updates_remove_update_add_with_clamp() {
        let existing = vec![
            SignificantEvent {
                id: "evt_1".into(),
                text: "keep".into(),
                weight: 4,
            },
            SignificantEvent {
                id: "evt_2".into(),
                text: "old".into(),
                weight: 2,
            },
        ];
        let mut update = HashMap::new();
        update.insert("evt_2".to_string(), "revised".to_string());
        let updates = SignificantEventUpdates {
            add: vec![NewSignificantEvent::WithWeight {
                text: "fresh".into(),
                weight: 9,
            }],
            update,
            remove: vec![],
        };
        let (result, next) = apply_significant_event_updates(&existing, &updates, 3);
        assert_eq!(result[1].text, "revised");
        assert_eq!(result[1].weight, 2); // weight preserved on update
        assert_eq!(result[2].id, "evt_3");
        assert_eq!(result[2].weight, 5); // clamped to 1-5
        assert_eq!(next, 4);
    }

    #[test]
    fn prune_keeps_cap_and_drops_old_low_weight_first() {
        // 31 events; weight-1 oldest should be pruned to reach the cap of 30.
        let mut events: Vec<SignificantEvent> = (1..=31)
            .map(|i| SignificantEvent {
                id: format!("evt_{i}"),
                text: format!("e{i}"),
                weight: if i == 1 { 1 } else { 5 },
            })
            .collect();
        prune_significant_events(&mut events, 32);
        assert_eq!(events.len(), SIGNIFICANT_EVENTS_CAP);
        assert!(!events.iter().any(|e| e.id == "evt_1"));
    }

    #[test]
    fn coerce_tolerates_strings_and_objects() {
        let objs = json!([{"id": "evt_5", "text": "x", "weight": 2}]);
        assert_eq!(coerce_significant_event_array(&objs, 1)[0].id, "evt_5");
        let strs = json!(["a", "b"]);
        let out = coerce_significant_event_array(&strs, 10);
        assert_eq!(out[0].id, "evt_10");
        assert_eq!(out[1].id, "evt_11");
        let single = json!("solo");
        assert_eq!(coerce_significant_event_array(&single, 3)[0].id, "evt_3");
    }

    #[test]
    fn story_summary_composes_rolling_and_recent_turns() {
        let composed = compose_story_summary_with_recent_turns(
            "## Rolling Story\nThe hero set out.",
            "",
            &["Player went north; GM described a cave".to_string()],
            "go north",
            "A cave looms.",
        );
        assert!(composed.contains("## Rolling Story"));
        assert!(composed.contains("## Recent Turns"));
        assert!(story_summary_has_recent_turns(&composed));
    }
}
