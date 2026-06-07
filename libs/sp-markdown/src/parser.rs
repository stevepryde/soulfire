use crate::ast::{Block, Inline};

pub fn parse(text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.split('\n').collect();
    parse_blocks(&lines)
}

fn parse_blocks(lines: &[&str]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if let Some(level) = heading_level(trimmed) {
            let content_str = trimmed[level as usize..].trim_start();
            let content_str = content_str.trim_end_matches('#').trim_end();
            out.push(Block::Heading {
                level: level.min(3),
                content: parse_inlines(content_str),
            });
            i += 1;
            continue;
        }

        if let Some(fence) = fence_info(trimmed) {
            let lang = fence.lang.map(|s| s.to_string());
            let mut code = String::new();
            let mut closed = false;
            let mut j = i + 1;
            while j < lines.len() {
                let l = lines[j];
                if is_closing_fence(l.trim_start(), fence.marker_len) {
                    closed = true;
                    j += 1;
                    break;
                }
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(l);
                j += 1;
            }
            out.push(Block::CodeBlock { lang, code, closed });
            i = j;
            continue;
        }

        if line.starts_with("> ") || line == ">" {
            let mut quoted: Vec<&str> = Vec::new();
            while i < lines.len() {
                let l = lines[i];
                if l.starts_with("> ") {
                    quoted.push(&l[2..]);
                } else if l == ">" {
                    quoted.push("");
                } else if l.trim().is_empty() {
                    break;
                } else {
                    break;
                }
                i += 1;
            }
            out.push(Block::Blockquote(parse_blocks(&quoted)));
            continue;
        }

        if let Some(kind) = list_marker(line) {
            let (list_block, j) = parse_list_at(lines, i, kind.indent);
            out.push(list_block);
            i = j;
            continue;
        }

        let mut para = String::new();
        while i < lines.len() {
            let l = lines[i];
            if l.trim().is_empty()
                || heading_level(l.trim_start()).is_some()
                || fence_info(l.trim_start()).is_some()
                || l.starts_with("> ")
                || l == ">"
                || list_marker(l).is_some()
            {
                break;
            }
            if !para.is_empty() {
                para.push('\n');
            }
            para.push_str(l);
            i += 1;
        }
        if !para.is_empty() {
            out.push(Block::Paragraph(parse_inlines(&para)));
        }
    }
    out
}

fn heading_level(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    let mut count = 0u8;
    while count < 6 && (count as usize) < bytes.len() && bytes[count as usize] == b'#' {
        count += 1;
    }
    if count == 0 {
        return None;
    }
    if (count as usize) >= bytes.len() {
        return None;
    }
    if bytes[count as usize] != b' ' {
        return None;
    }
    Some(count)
}

struct Fence<'a> {
    marker_len: usize,
    lang: Option<&'a str>,
}

fn fence_info(s: &str) -> Option<Fence<'_>> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    let ch = bytes[0];
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let mut n = 0;
    while n < bytes.len() && bytes[n] == ch {
        n += 1;
    }
    if n < 3 {
        return None;
    }
    let info = s[n..].trim();
    let lang = if info.is_empty() { None } else { Some(info) };
    Some(Fence {
        marker_len: n,
        lang,
    })
}

fn is_closing_fence(s: &str, marker_len: usize) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < marker_len {
        return false;
    }
    let first = bytes[0];
    if first != b'`' && first != b'~' {
        return false;
    }
    let mut n = 0;
    while n < bytes.len() && bytes[n] == first {
        n += 1;
    }
    if n < marker_len {
        return false;
    }
    s[n..].trim().is_empty()
}

struct ListKind {
    ordered: bool,
    indent: usize,
    consumed: usize,
}

fn list_marker(line: &str) -> Option<ListKind> {
    let bytes = line.as_bytes();
    let mut indent = 0;
    while indent < bytes.len() && (bytes[indent] == b' ' || bytes[indent] == b'\t') {
        indent += 1;
    }
    if indent + 1 >= bytes.len() {
        return None;
    }
    let first = bytes[indent];
    if (first == b'-' || first == b'*' || first == b'+') && bytes[indent + 1] == b' ' {
        return Some(ListKind {
            ordered: false,
            indent,
            consumed: indent + 2,
        });
    }
    let mut i = indent;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == indent || i + 1 >= bytes.len() {
        return None;
    }
    if (bytes[i] == b'.' || bytes[i] == b')') && bytes[i + 1] == b' ' {
        return Some(ListKind {
            ordered: true,
            indent,
            consumed: i + 2,
        });
    }
    None
}

fn parse_list_at(lines: &[&str], start: usize, base_indent: usize) -> (Block, usize) {
    let first = list_marker(lines[start]).expect("caller verified marker present");
    let ordered = first.ordered;
    let mut items: Vec<Vec<Block>> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            // Loose-list rule: blank lines do not end the list if the next
            // non-blank line is another list marker at this indent (or a
            // nested marker at greater indent). They only end it if the
            // next content is unrelated.
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j >= lines.len() {
                break;
            }
            match list_marker(lines[j]) {
                Some(next) if next.indent >= base_indent => {
                    i = j;
                    continue;
                }
                _ => break,
            }
        }
        let info = match list_marker(line) {
            Some(m) => m,
            None => break,
        };
        if info.indent < base_indent {
            break;
        }
        if info.indent > base_indent {
            let (sublist, j) = parse_list_at(lines, i, info.indent);
            if let Some(last_item) = items.last_mut() {
                last_item.push(sublist);
            } else {
                items.push(vec![sublist]);
            }
            i = j;
            continue;
        }
        if info.ordered != ordered {
            break;
        }
        let item_text = &line[info.consumed..];
        items.push(vec![Block::Paragraph(parse_inlines(item_text))]);
        i += 1;
    }
    (Block::List { ordered, items }, i)
}

pub(crate) fn parse_inlines(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut i = 0;

    while i < text.len() {
        let rest = &text[i..];

        if rest.starts_with("\\\n") {
            flush(&mut buf, &mut out);
            out.push(Inline::SoftBreak);
            i += 2;
            continue;
        }

        if rest.starts_with('\\') && rest.len() >= 2 {
            let next = rest.as_bytes()[1];
            if matches!(
                next,
                b'\\' | b'*' | b'_' | b'`' | b'[' | b']' | b'(' | b')' | b'<' | b'>' | b'#'
            ) {
                buf.push(next as char);
                i += 2;
                continue;
            }
        }

        if rest.starts_with("**") {
            if let Some(close) = find_close(&text[i + 2..], "**") {
                flush(&mut buf, &mut out);
                let inner = &text[i + 2..i + 2 + close];
                out.push(Inline::Bold(parse_inlines(inner)));
                i = i + 2 + close + 2;
                continue;
            }
            buf.push_str("**");
            i += 2;
            continue;
        }

        let first_byte = rest.as_bytes()[0];
        if first_byte == b'*' || first_byte == b'_' {
            let marker_char = first_byte as char;
            if can_open_emphasis(text, i, marker_char) {
                if let Some(close) = find_close_emphasis(text, i + 1, marker_char) {
                    flush(&mut buf, &mut out);
                    let inner = &text[i + 1..close];
                    out.push(Inline::Italic(parse_inlines(inner)));
                    i = close + 1;
                    continue;
                }
            }
            buf.push(marker_char);
            i += 1;
            continue;
        }

        if first_byte == b'`' {
            if let Some(close_rel) = text[i + 1..].find('`') {
                flush(&mut buf, &mut out);
                let inner = &text[i + 1..i + 1 + close_rel];
                out.push(Inline::Code(inner.to_string()));
                i = i + 1 + close_rel + 1;
                continue;
            }
            buf.push('`');
            i += 1;
            continue;
        }

        if first_byte == b'[' {
            if let Some((text_end_rel, href_start_rel, href_end_rel)) = parse_link_brackets(rest) {
                let link_text_raw = &rest[1..text_end_rel];
                let link_href = &rest[href_start_rel..href_end_rel];
                flush(&mut buf, &mut out);
                if is_safe_href(link_href) {
                    out.push(Inline::Link {
                        text: parse_inlines(link_text_raw),
                        href: link_href.to_string(),
                    });
                } else {
                    let inner = parse_inlines(link_text_raw);
                    out.extend(inner);
                }
                i += href_end_rel + 1;
                continue;
            }
            buf.push('[');
            i += 1;
            continue;
        }

        if first_byte == b'<' {
            if let Some(close_rel) = text[i + 1..].find('>') {
                let inner = &text[i + 1..i + 1 + close_rel];
                if let Some(href) = autolink_href(inner) {
                    flush(&mut buf, &mut out);
                    if is_safe_href(&href) {
                        out.push(Inline::Link {
                            text: vec![Inline::Text(inner.to_string())],
                            href,
                        });
                    } else {
                        out.push(Inline::Text(inner.to_string()));
                    }
                    i = i + 1 + close_rel + 1;
                    continue;
                }
            }
            buf.push('<');
            i += 1;
            continue;
        }

        if first_byte == b'\n' {
            flush(&mut buf, &mut out);
            out.push(Inline::SoftBreak);
            i += 1;
            continue;
        }

        let ch = rest.chars().next().unwrap();
        buf.push(ch);
        i += ch.len_utf8();
    }
    flush(&mut buf, &mut out);
    out
}

fn flush(buf: &mut String, out: &mut Vec<Inline>) {
    if !buf.is_empty() {
        out.push(Inline::Text(std::mem::take(buf)));
    }
}

fn find_close(s: &str, marker: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let m = marker.as_bytes();
    let mut i = 0;
    while i + m.len() <= bytes.len() {
        if bytes[i] == b'`' {
            if let Some(end) = s[i + 1..].find('`') {
                i = i + 1 + end + 1;
                continue;
            }
            return None;
        }
        if &bytes[i..i + m.len()] == m {
            return Some(i);
        }
        let ch_len = s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        i += ch_len;
    }
    None
}

fn can_open_emphasis(text: &str, pos: usize, marker: char) -> bool {
    let prev = if pos == 0 {
        None
    } else {
        text[..pos].chars().next_back()
    };
    let next_pos = pos + marker.len_utf8();
    let next = if next_pos >= text.len() {
        None
    } else {
        text[next_pos..].chars().next()
    };
    let next_ok = matches!(next, Some(c) if !c.is_whitespace());
    if !next_ok {
        return false;
    }
    if marker == '_' {
        let prev_alnum = matches!(prev, Some(c) if c.is_alphanumeric());
        if prev_alnum {
            return false;
        }
    }
    true
}

fn find_close_emphasis(text: &str, search_from: usize, marker: char) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = search_from;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(end) = text[i + 1..].find('`') {
                i = i + 1 + end + 1;
                continue;
            }
            return None;
        }
        if bytes[i] as char == marker {
            if marker == '*' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                i += 2;
                continue;
            }
            let prev = text[..i].chars().next_back();
            let prev_ws = matches!(prev, Some(c) if c.is_whitespace());
            if !prev_ws {
                if marker == '_' {
                    let next_pos = i + 1;
                    let next_alnum = if next_pos < text.len() {
                        text[next_pos..]
                            .chars()
                            .next()
                            .map(|c| c.is_alphanumeric())
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    if next_alnum {
                        i += 1;
                        continue;
                    }
                }
                return Some(i);
            }
        }
        let ch_len = text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        i += ch_len;
    }
    None
}

fn parse_link_brackets(s: &str) -> Option<(usize, usize, usize)> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return None;
    }
    let mut i = 1;
    let mut depth = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'[' => {
                depth += 1;
                i += 1;
            }
            b']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                i += 1;
            }
            _ => {
                let ch_len = s[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                i += ch_len;
            }
        }
    }
    if depth != 0 {
        return None;
    }
    let text_end = i;
    if i + 1 >= bytes.len() || bytes[i + 1] != b'(' {
        return None;
    }
    let href_start = i + 2;
    let mut j = href_start;
    while j < bytes.len() && bytes[j] != b')' {
        j += 1;
    }
    if j >= bytes.len() {
        return None;
    }
    Some((text_end, href_start, j))
}

fn autolink_href(inner: &str) -> Option<String> {
    if inner.is_empty() {
        return None;
    }
    if inner.contains(' ') || inner.contains('\n') {
        return None;
    }
    if inner.starts_with("http://") || inner.starts_with("https://") || inner.starts_with("mailto:")
    {
        return Some(inner.to_string());
    }
    if let Some(at) = inner.find('@') {
        if at > 0 && at < inner.len() - 1 && !inner[at + 1..].contains('@') {
            return Some(format!("mailto:{}", inner));
        }
    }
    None
}

pub(crate) fn is_safe_href(href: &str) -> bool {
    let h = href.trim();
    let lower = h.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || h.starts_with('/')
        || h.starts_with('#')
}
