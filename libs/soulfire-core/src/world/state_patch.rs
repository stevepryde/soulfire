//! The adventure-state patch system and validator (`WORLD-12`, `WORLD-14`).
//!
//! Ported from Soulfire-OG `services/state_patch.rs`, dropping the multiplayer
//! orchestrator leftovers (`orchestrator_narrative`, `still_deciding_user_ids`,
//! `PROD-11`). A diff update is a list of dot-notation patches applied
//! sequentially to the parsed state; the validator rejects malformed paths,
//! out-of-range array indices, and a non-object root, causing the diff to abort
//! with no partial commit (the caller then falls back to a full reconciliation).

use serde_json::Value;

/// The operation a patch performs (`WORLD-12`).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchOp {
    /// Replace the value at the path (default). Setting null removes the key.
    #[default]
    Set,
    /// Append `value` to the array at the path; an array value spreads.
    Append,
    /// Remove array elements matching `value` (objects match partially,
    /// primitives match exactly).
    Remove,
}

/// A single dot-notation patch (`WORLD-12`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatePatch {
    pub path: String,
    #[serde(default)]
    pub op: PatchOp,
    pub value: Value,
}

/// The outcome of applying a patch set (`WORLD-12`, `WORLD-14`).
#[derive(Debug)]
pub enum PatchResult {
    /// All patches applied and the result validated.
    Success(Value),
    /// The validator rejected the result.
    VerificationFailed(Vec<String>),
    /// A patch path/index was malformed or out of bounds.
    InvalidPath(String),
}

impl PatchResult {
    /// Whether the patch set succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, PatchResult::Success(_))
    }
}

/// Apply patches sequentially to a state object, then validate. The first failure
/// aborts with no partial commit (`WORLD-12`).
pub fn apply_patches(state: &Value, patches: &[StatePatch]) -> PatchResult {
    let mut new_state = state.clone();

    for patch in patches {
        let result = match patch.op {
            PatchOp::Set => set_path(&mut new_state, &patch.path, patch.value.clone()),
            PatchOp::Append => append_path(&mut new_state, &patch.path, &patch.value),
            PatchOp::Remove => remove_path(&mut new_state, &patch.path, &patch.value),
        };
        if let Err(e) = result {
            return PatchResult::InvalidPath(e);
        }
    }

    let errors = verify_state(&new_state);
    if !errors.is_empty() {
        return PatchResult::VerificationFailed(errors);
    }
    PatchResult::Success(new_state)
}

/// Set a value at a dot-delimited path. Setting `null` removes the key. Supports
/// object keys and numeric array indices.
fn set_path(root: &mut Value, path: &str, value: Value) -> Result<(), String> {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        return Err("Empty path".to_string());
    }

    let mut current = root;
    for (i, segment) in segments.iter().enumerate() {
        if i == segments.len() - 1 {
            match current {
                Value::Object(map) => {
                    // Setting null removes the key (WORLD-12).
                    if value.is_null() {
                        map.remove(*segment);
                    } else {
                        map.insert(segment.to_string(), value);
                    }
                    return Ok(());
                }
                Value::Array(arr) => {
                    if let Ok(idx) = segment.parse::<usize>() {
                        if idx < arr.len() {
                            arr[idx] = value;
                            return Ok(());
                        }
                        return Err(format!(
                            "Array index {} out of bounds (len {}) at path '{}'",
                            idx,
                            arr.len(),
                            path
                        ));
                    }
                    return Err(format!(
                        "Cannot use non-numeric key '{}' on array at path '{}'",
                        segment, path
                    ));
                }
                _ => {
                    return Err(format!(
                        "Cannot set key '{}' on non-object at path '{}'",
                        segment, path
                    ));
                }
            }
        } else {
            current = descend(current, segment, path)?;
        }
    }

    Err(format!("Failed to set path '{}'", path))
}

/// Navigate one level deeper, creating missing object keys.
fn descend<'a>(current: &'a mut Value, segment: &str, path: &str) -> Result<&'a mut Value, String> {
    match current {
        Value::Object(map) => Ok(map
            .entry(segment.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()))),
        Value::Array(arr) => {
            if let Ok(idx) = segment.parse::<usize>() {
                if idx < arr.len() {
                    Ok(&mut arr[idx])
                } else {
                    Err(format!(
                        "Array index {} out of bounds (len {}) at path '{}'",
                        idx,
                        arr.len(),
                        path
                    ))
                }
            } else {
                Err(format!(
                    "Cannot use non-numeric key '{}' on array at path '{}'",
                    segment, path
                ))
            }
        }
        _ => Err(format!(
            "Cannot traverse into primitive at segment '{}' in path '{}'",
            segment, path
        )),
    }
}

/// Append value(s) to the array at a path; creates the array if missing.
fn append_path(root: &mut Value, path: &str, value: &Value) -> Result<(), String> {
    let (parent, last_segment) = navigate_to_parent(root, path)?;

    let last_key = last_segment.to_string();
    let target = match parent {
        Value::Object(map) => map
            .entry(last_key)
            .or_insert_with(|| Value::Array(Vec::new())),
        Value::Array(arr) => {
            if let Ok(idx) = last_segment.parse::<usize>() {
                if idx < arr.len() {
                    &mut arr[idx]
                } else {
                    return Err(format!(
                        "Array index {} out of bounds (len {}) at path '{}'",
                        idx,
                        arr.len(),
                        path
                    ));
                }
            } else {
                return Err(format!(
                    "Cannot use non-numeric key '{}' on array at path '{}'",
                    last_segment, path
                ));
            }
        }
        _ => {
            return Err(format!(
                "Cannot append at path '{}': parent is not an object or array",
                path
            ));
        }
    };

    match target {
        Value::Array(arr) => {
            if let Some(items) = value.as_array() {
                arr.extend(items.iter().cloned());
            } else {
                arr.push(value.clone());
            }
            Ok(())
        }
        _ => Err(format!("Cannot append to non-array at path '{}'", path)),
    }
}

/// Remove array elements matching `value` at a path; missing target is a no-op.
fn remove_path(root: &mut Value, path: &str, value: &Value) -> Result<(), String> {
    let (parent, last_segment) = navigate_to_parent(root, path)?;

    let target = match parent {
        Value::Object(map) => match map.get_mut(last_segment as &str) {
            Some(v) => v,
            None => return Ok(()),
        },
        Value::Array(arr) => {
            if let Ok(idx) = last_segment.parse::<usize>() {
                if idx < arr.len() {
                    &mut arr[idx]
                } else {
                    return Ok(());
                }
            } else {
                return Err(format!(
                    "Cannot use non-numeric key '{}' on array at path '{}'",
                    last_segment, path
                ));
            }
        }
        _ => {
            return Err(format!(
                "Cannot remove at path '{}': parent is not an object or array",
                path
            ));
        }
    };

    match target {
        Value::Array(arr) => {
            arr.retain(|elem| !matches_value(elem, value));
            Ok(())
        }
        _ => Err(format!("Cannot remove from non-array at path '{}'", path)),
    }
}

/// Navigate to the parent of the final path segment, creating objects as needed.
fn navigate_to_parent<'a, 'p>(
    root: &'a mut Value,
    path: &'p str,
) -> Result<(&'a mut Value, &'p str), String> {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        return Err("Empty path".to_string());
    }
    let (parents, last) = segments.split_at(segments.len() - 1);
    let mut current = root;
    for segment in parents {
        current = descend(current, segment, path)?;
    }
    Ok((current, last[0]))
}

/// Match `elem` against a removal `pattern`: objects match partially (all
/// pattern keys equal), primitives match exactly.
fn matches_value(elem: &Value, pattern: &Value) -> bool {
    if let (Some(elem_obj), Some(pattern_obj)) = (elem.as_object(), pattern.as_object()) {
        pattern_obj.iter().all(|(k, v)| elem_obj.get(k) == Some(v))
    } else {
        elem == pattern
    }
}

/// Validate a world state (`WORLD-14`). A non-object root is rejected; stronger
/// invariants (conservation, etc.) can be added here later.
fn verify_state(state: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if !state.is_object() {
        errors.push("World state must be a JSON object".to_string());
    }
    errors
}

/// Parse patches from a pre-parsed JSON value (`WORLD-12`). Accepts either a
/// `{"patches": [...]}` wrapper or a raw array. A `remove` op may carry `match`
/// instead of `value`.
pub fn parse_patches_from_value(parsed: &Value) -> anyhow::Result<Vec<StatePatch>> {
    let patches_array = if let Some(arr) = parsed.as_array() {
        arr
    } else {
        parsed
            .get("patches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'patches' array in state-update response"))?
    };

    let mut patches = Vec::new();
    for patch_val in patches_array {
        let path = patch_val
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' in patch"))?
            .to_string();
        let op = parse_op(patch_val);
        let value = patch_val
            .get("value")
            .cloned()
            .or_else(|| patch_val.get("match").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' in patch for path '{}'", path))?;
        patches.push(StatePatch { path, op, value });
    }
    Ok(patches)
}

fn parse_op(patch_val: &Value) -> PatchOp {
    patch_val
        .get("op")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "append" => PatchOp::Append,
            "remove" => PatchOp::Remove,
            _ => PatchOp::Set,
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_nested_and_array_index() {
        let state = json!({"location": {"visible_npcs": [
            {"name": "guard", "attitude": "neutral"},
            {"name": "merchant", "attitude": "friendly"}
        ]}});
        let patches = vec![StatePatch {
            path: "location.visible_npcs.0.attitude".into(),
            op: PatchOp::Set,
            value: json!("approving"),
        }];
        match apply_patches(&state, &patches) {
            PatchResult::Success(s) => {
                assert_eq!(s["location"]["visible_npcs"][0]["attitude"], "approving");
                assert_eq!(s["location"]["visible_npcs"][1]["attitude"], "friendly");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn set_null_removes_key() {
        // WORLD-12: setting a value to null removes a key.
        let state = json!({"player": {"buff": "haste", "hp": 10}});
        let patches = vec![StatePatch {
            path: "player.buff".into(),
            op: PatchOp::Set,
            value: Value::Null,
        }];
        match apply_patches(&state, &patches) {
            PatchResult::Success(s) => {
                assert!(s["player"].get("buff").is_none());
                assert_eq!(s["player"]["hp"], 10);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn out_of_bounds_index_is_invalid_path() {
        // WORLD-14: out-of-range array indices are rejected.
        let state = json!({"items": ["sword"]});
        let patches = vec![StatePatch {
            path: "items.5".into(),
            op: PatchOp::Set,
            value: json!("shield"),
        }];
        assert!(matches!(
            apply_patches(&state, &patches),
            PatchResult::InvalidPath(_)
        ));
    }

    #[test]
    fn non_object_root_fails_verification() {
        // WORLD-14: a non-object root is rejected.
        let state = json!(["not", "an", "object"]);
        match apply_patches(&state, &[]) {
            PatchResult::VerificationFailed(errs) => assert!(!errs.is_empty()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn append_spreads_arrays_and_creates_missing() {
        let state = json!({"player": {"name": "Elara"}});
        let patches = vec![StatePatch {
            path: "player.inventory".into(),
            op: PatchOp::Append,
            value: json!([{"name": "key"}, {"name": "map"}]),
        }];
        match apply_patches(&state, &patches) {
            PatchResult::Success(s) => {
                assert_eq!(s["player"]["inventory"].as_array().unwrap().len(), 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn remove_partial_object_match() {
        let state = json!({"inventory": [
            {"name": "healing potion", "type": "potion"},
            {"name": "sword", "type": "weapon"},
            {"name": "mana potion", "type": "potion"}
        ]});
        let patches = vec![StatePatch {
            path: "inventory".into(),
            op: PatchOp::Remove,
            value: json!({"type": "potion"}),
        }];
        match apply_patches(&state, &patches) {
            PatchResult::Success(s) => {
                let inv = s["inventory"].as_array().unwrap();
                assert_eq!(inv.len(), 1);
                assert_eq!(inv[0]["name"], "sword");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn append_to_non_array_fails() {
        let state = json!({"name": "Marcus"});
        let patches = vec![StatePatch {
            path: "name".into(),
            op: PatchOp::Append,
            value: json!("extra"),
        }];
        match apply_patches(&state, &patches) {
            PatchResult::InvalidPath(m) => assert!(m.contains("non-array")),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn first_failure_aborts_with_no_partial_commit() {
        // WORLD-12: no partial commit — a bad patch leaves the original untouched.
        let state = json!({"a": 1, "items": ["sword"]});
        let patches = vec![
            StatePatch {
                path: "a".into(),
                op: PatchOp::Set,
                value: json!(2),
            },
            StatePatch {
                path: "items.5".into(), // out of bounds on the existing array
                op: PatchOp::Set,
                value: json!("x"),
            },
        ];
        assert!(matches!(
            apply_patches(&state, &patches),
            PatchResult::InvalidPath(_)
        ));
        // The input state is never mutated (no partial commit).
        assert_eq!(state["a"], 1);
    }

    #[test]
    fn parse_from_wrapper_and_raw_and_match_alias() {
        let wrapper = json!({"patches": [{"path": "a.b", "value": 1}]});
        assert_eq!(parse_patches_from_value(&wrapper).unwrap().len(), 1);
        let raw = json!([{"path": "x", "value": 2}]);
        assert_eq!(parse_patches_from_value(&raw).unwrap()[0].path, "x");
        let with_match =
            json!({"patches": [{"path": "inv", "op": "remove", "match": {"name": "potion"}}]});
        let p = parse_patches_from_value(&with_match).unwrap();
        assert_eq!(p[0].op, PatchOp::Remove);
        assert_eq!(p[0].value, json!({"name": "potion"}));
    }
}
