//! Tolerant JSON parsing for model output (`AI-5`).
//!
//! Providers sometimes wrap JSON in Markdown code fences or surround it with
//! prose. These helpers strip ```` ```json ````/```` ``` ```` fences and rescue a
//! top-level object/array before parsing.

use serde::de::DeserializeOwned;

/// Strip a surrounding Markdown code fence and whitespace from `s`, returning the
/// inner text. Handles ```` ```json ````, ```` ``` ````, and untagged fences.
pub fn strip_json_fence(s: &str) -> &str {
    let trimmed = s.trim();
    let Some(after_open) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    // Drop the rest of the opening-fence line (e.g. `json`).
    let after_lang = match after_open.find('\n') {
        Some(nl) => &after_open[nl + 1..],
        None => after_open,
    };
    // Drop the closing fence.
    let inner = match after_lang.rfind("```") {
        Some(idx) => &after_lang[..idx],
        None => after_lang,
    };
    inner.trim()
}

/// Rescue the outermost JSON object or array from `s` when it is surrounded by
/// prose: returns the substring from the first `{`/`[` to its matching close.
/// Falls back to the fence-stripped string when no balanced block is found.
pub fn rescue_json_block(s: &str) -> &str {
    let stripped = strip_json_fence(s);
    let bytes = stripped.as_bytes();
    let Some(start) = bytes.iter().position(|&b| b == b'{' || b == b'[') else {
        return stripped;
    };
    let open = bytes[start];
    let close = if open == b'{' { b'}' } else { b']' };
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            x if x == open => depth += 1,
            x if x == close => {
                depth -= 1;
                if depth == 0 {
                    return &stripped[start..=i];
                }
            }
            _ => {}
        }
    }
    stripped
}

/// Parse JSON from possibly-fenced, possibly-prose-wrapped model output (`AI-5`).
pub fn parse_lenient<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    let candidate = rescue_json_block(s);
    serde_json::from_str(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn strips_json_tagged_fence() {
        let input = "```json\n{\"a\": 1}\n```";
        assert_eq!(strip_json_fence(input), "{\"a\": 1}");
    }

    #[test]
    fn strips_untagged_fence() {
        let input = "```\n[1,2,3]\n```";
        assert_eq!(strip_json_fence(input), "[1,2,3]");
    }

    #[test]
    fn passes_through_bare_json() {
        assert_eq!(strip_json_fence("  {\"x\":true}  "), "{\"x\":true}");
    }

    #[test]
    fn rescues_object_from_prose() {
        let input = "Sure! Here is the state:\n{\"hp\": 10, \"name\": \"Lyra\"}\nHope that helps.";
        let v: Value = parse_lenient(input).unwrap();
        assert_eq!(v["hp"], 10);
        assert_eq!(v["name"], "Lyra");
    }

    #[test]
    fn parses_fenced_json_into_struct() {
        let input = "```json\n{\"hp\": 5}\n```";
        let v: Value = parse_lenient(input).unwrap();
        assert_eq!(v["hp"], 5);
    }

    #[test]
    fn rescue_respects_strings_with_braces() {
        let input = "prefix {\"text\": \"a } inside string\"} suffix";
        let v: Value = parse_lenient(input).unwrap();
        assert_eq!(v["text"], "a } inside string");
    }
}
