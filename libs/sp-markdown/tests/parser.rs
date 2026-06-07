use sp_markdown::ast::{Block, Inline};
use sp_markdown::parse;

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

#[test]
fn paragraph_plain() {
    assert_eq!(
        parse("hello world"),
        vec![Block::Paragraph(vec![text("hello world")])]
    );
}

#[test]
fn paragraph_with_soft_break() {
    let blocks = parse("line one\nline two");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("line one"),
            Inline::SoftBreak,
            text("line two"),
        ])]
    );
}

#[test]
fn two_paragraphs() {
    let blocks = parse("first\n\nsecond");
    assert_eq!(
        blocks,
        vec![
            Block::Paragraph(vec![text("first")]),
            Block::Paragraph(vec![text("second")]),
        ]
    );
}

#[test]
fn headings_h1_h2_h3() {
    let blocks = parse("# A\n## B\n### C");
    assert_eq!(
        blocks,
        vec![
            Block::Heading {
                level: 1,
                content: vec![text("A")]
            },
            Block::Heading {
                level: 2,
                content: vec![text("B")]
            },
            Block::Heading {
                level: 3,
                content: vec![text("C")]
            },
        ]
    );
}

#[test]
fn heading_clamps_to_three() {
    let blocks = parse("##### deep");
    assert_eq!(
        blocks,
        vec![Block::Heading {
            level: 3,
            content: vec![text("deep")]
        }]
    );
}

#[test]
fn bold_basic() {
    let blocks = parse("a **bold** word");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("a "),
            Inline::Bold(vec![text("bold")]),
            text(" word"),
        ])]
    );
}

#[test]
fn italic_asterisk_and_underscore() {
    let blocks = parse("*one* and _two_");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            Inline::Italic(vec![text("one")]),
            text(" and "),
            Inline::Italic(vec![text("two")]),
        ])]
    );
}

#[test]
fn underscore_inside_word_is_literal() {
    let blocks = parse("foo_bar_baz");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![text("foo_bar_baz")])]
    );
}

#[test]
fn nested_bold_italic() {
    let blocks = parse("**hello _world_**");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![Inline::Bold(vec![
            text("hello "),
            Inline::Italic(vec![text("world")]),
        ])])]
    );
}

#[test]
fn inline_code_basic() {
    let blocks = parse("run `foo()` now");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("run "),
            Inline::Code("foo()".to_string()),
            text(" now"),
        ])]
    );
}

#[test]
fn code_inside_emphasis_is_protected() {
    let blocks = parse("**`foo *bar* baz`**");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![Inline::Bold(vec![Inline::Code(
            "foo *bar* baz".to_string()
        )])])]
    );
}

#[test]
fn fenced_code_block_with_lang() {
    let blocks = parse("```rust\nfn main() {}\n```");
    assert_eq!(
        blocks,
        vec![Block::CodeBlock {
            lang: Some("rust".to_string()),
            code: "fn main() {}".to_string(),
            closed: true,
        }]
    );
}

#[test]
fn fenced_code_block_unclosed() {
    let blocks = parse("```\nfoo");
    assert_eq!(
        blocks,
        vec![Block::CodeBlock {
            lang: None,
            code: "foo".to_string(),
            closed: false,
        }]
    );
}

#[test]
fn unordered_list() {
    let blocks = parse("- one\n- two\n- three");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: false,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
                vec![Block::Paragraph(vec![text("three")])],
            ],
        }]
    );
}

#[test]
fn ordered_list() {
    let blocks = parse("1. one\n2. two");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: true,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
            ],
        }]
    );
}

#[test]
fn blockquote() {
    let blocks = parse("> hello\n> world");
    assert_eq!(
        blocks,
        vec![Block::Blockquote(vec![Block::Paragraph(vec![
            text("hello"),
            Inline::SoftBreak,
            text("world"),
        ])])]
    );
}

#[test]
fn link_basic() {
    let blocks = parse("see [docs](https://example.com)");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("see "),
            Inline::Link {
                text: vec![text("docs")],
                href: "https://example.com".to_string(),
            },
        ])]
    );
}

#[test]
fn link_with_emphasis_in_text() {
    let blocks = parse("[**bold link**](https://example.com)");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![Inline::Link {
            text: vec![Inline::Bold(vec![text("bold link")])],
            href: "https://example.com".to_string(),
        }])]
    );
}

#[test]
fn link_javascript_scheme_rejected() {
    let blocks = parse("[click](javascript:alert(1))");
    let para = match &blocks[0] {
        Block::Paragraph(inlines) => inlines,
        _ => panic!("expected paragraph"),
    };
    for inline in para {
        if matches!(inline, Inline::Link { .. }) {
            panic!("javascript: link should be rejected, got: {:?}", inline);
        }
    }
    let has_click = para.iter().any(|i| matches!(i, Inline::Text(t) if t.contains("click")));
    assert!(has_click, "visible link text should remain: {:?}", para);
}

#[test]
fn link_data_scheme_rejected() {
    let blocks = parse("[x](data:text/html,<script>1</script>)");
    let para = match &blocks[0] {
        Block::Paragraph(inlines) => inlines,
        _ => panic!("expected paragraph"),
    };
    for inline in para {
        assert!(
            !matches!(inline, Inline::Link { .. }),
            "data: scheme must be rejected"
        );
    }
}

#[test]
fn autolink_url() {
    let blocks = parse("see <https://example.com> here");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("see "),
            Inline::Link {
                text: vec![text("https://example.com")],
                href: "https://example.com".to_string(),
            },
            text(" here"),
        ])]
    );
}

#[test]
fn autolink_email() {
    let blocks = parse("contact <support@example.com>");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("contact "),
            Inline::Link {
                text: vec![text("support@example.com")],
                href: "mailto:support@example.com".to_string(),
            },
        ])]
    );
}

#[test]
fn streaming_unmatched_bold_marker_never_styled() {
    let full = "hello **world** foo";
    for end in 0..=full.len() {
        if !full.is_char_boundary(end) {
            continue;
        }
        let prefix = &full[..end];
        if double_asterisk_runs(prefix) >= 2 {
            continue;
        }
        let blocks = parse(prefix);
        assert!(
            !contains_bold(&blocks),
            "prefix {:?} produced a Bold node before the closer arrived",
            prefix
        );
    }
}

fn double_asterisk_runs(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'*' {
            count += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    count
}

#[test]
fn streaming_unmatched_italic_marker_never_styled() {
    let full = "hello *world* foo";
    for end in 0..=full.len() {
        if !full.is_char_boundary(end) {
            continue;
        }
        let prefix = &full[..end];
        if prefix.matches('*').count() < 2 {
            let blocks = parse(prefix);
            assert!(
                !contains_italic(&blocks),
                "prefix {:?} produced an Italic node with unmatched *",
                prefix
            );
        }
    }
}

#[test]
fn streaming_unmatched_code_marker_never_styled() {
    let full = "hello `world` foo";
    for end in 0..=full.len() {
        if !full.is_char_boundary(end) {
            continue;
        }
        let prefix = &full[..end];
        if prefix.matches('`').count() < 2 {
            let blocks = parse(prefix);
            assert!(
                !contains_code_inline(&blocks),
                "prefix {:?} produced a Code node with unmatched backtick",
                prefix
            );
        }
    }
}

#[test]
fn unicode_emoji_around_bold_marker() {
    let blocks = parse("🎉 **foo** 🚀");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![
            text("🎉 "),
            Inline::Bold(vec![text("foo")]),
            text(" 🚀"),
        ])]
    );
}

#[test]
fn unicode_em_dash_inside_bold() {
    let blocks = parse("**foo — bar**");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![Inline::Bold(vec![text(
            "foo — bar"
        )])])]
    );
}

#[test]
fn escaped_asterisk_is_literal() {
    let blocks = parse("a \\*literal\\* star");
    assert_eq!(
        blocks,
        vec![Block::Paragraph(vec![text("a *literal* star")])]
    );
}

#[test]
fn unordered_list_with_asterisk_marker() {
    let blocks = parse("* one\n* two");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: false,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
            ],
        }]
    );
}

#[test]
fn unordered_list_with_plus_marker() {
    let blocks = parse("+ one\n+ two");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: false,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
            ],
        }]
    );
}

#[test]
fn ordered_list_with_paren_marker() {
    let blocks = parse("1) one\n2) two");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: true,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
            ],
        }]
    );
}

#[test]
fn ordered_list_with_multi_digit_numbers() {
    let blocks = parse("9. nine\n10. ten\n100. hundred");
    let Block::List { ordered, items } = &blocks[0] else {
        panic!("expected list, got {:?}", blocks);
    };
    assert!(ordered);
    assert_eq!(items.len(), 3);
}

#[test]
fn dash_without_following_space_is_not_a_list() {
    let blocks = parse("-not a list\n-still not");
    assert!(matches!(blocks[0], Block::Paragraph(_)));
}

#[test]
fn number_dot_without_space_is_not_a_list() {
    let blocks = parse("1.foo\n2.bar");
    assert!(matches!(blocks[0], Block::Paragraph(_)));
}

#[test]
fn indented_list_two_spaces_top_level() {
    let blocks = parse("  - one\n  - two");
    assert_eq!(
        blocks,
        vec![Block::List {
            ordered: false,
            items: vec![
                vec![Block::Paragraph(vec![text("one")])],
                vec![Block::Paragraph(vec![text("two")])],
            ],
        }]
    );
}

#[test]
fn indented_list_four_spaces_top_level() {
    let blocks = parse("    - one\n    - two");
    let Block::List { ordered, items } = &blocks[0] else {
        panic!("expected list, got {:?}", blocks);
    };
    assert!(!ordered);
    assert_eq!(items.len(), 2);
}

#[test]
fn tab_indented_list() {
    let blocks = parse("\t- one\n\t- two");
    let Block::List { items, .. } = &blocks[0] else {
        panic!("expected list, got {:?}", blocks);
    };
    assert_eq!(items.len(), 2);
}

#[test]
fn nested_list_one_level() {
    let blocks = parse("- top one\n  - sub a\n  - sub b\n- top two");
    let Block::List { items, .. } = &blocks[0] else {
        panic!("expected list");
    };
    assert_eq!(items.len(), 2, "two top-level items: {:?}", items);
    let first_item = &items[0];
    assert_eq!(first_item.len(), 2, "first item has paragraph + sublist");
    assert!(matches!(first_item[0], Block::Paragraph(_)));
    let Block::List {
        items: sub_items, ..
    } = &first_item[1]
    else {
        panic!("expected sublist, got {:?}", first_item[1]);
    };
    assert_eq!(sub_items.len(), 2, "two sub items");
}

#[test]
fn nested_list_two_levels() {
    let blocks = parse("- a\n  - b\n    - c");
    let Block::List { items, .. } = &blocks[0] else {
        panic!();
    };
    assert_eq!(items.len(), 1);
    let first = &items[0];
    assert_eq!(first.len(), 2, "paragraph + sublist");
    let Block::List {
        items: mid_items, ..
    } = &first[1]
    else {
        panic!();
    };
    assert_eq!(mid_items.len(), 1);
    let Block::List {
        items: deep_items, ..
    } = &mid_items[0][1]
    else {
        panic!("expected deep sublist");
    };
    assert_eq!(deep_items.len(), 1);
}

#[test]
fn ordered_list_nested_inside_unordered() {
    let blocks = parse("- top\n  1. sub one\n  2. sub two");
    let Block::List {
        ordered: top_ordered,
        items,
    } = &blocks[0]
    else {
        panic!();
    };
    assert!(!top_ordered);
    let Block::List {
        ordered: sub_ordered,
        ..
    } = &items[0][1]
    else {
        panic!("expected nested ordered list");
    };
    assert!(sub_ordered);
}

#[test]
fn list_immediately_after_paragraph_no_blank_line() {
    let blocks = parse("Some intro text:\n- one\n- two");
    assert_eq!(blocks.len(), 2, "paragraph then list, got: {:?}", blocks);
    assert!(matches!(blocks[0], Block::Paragraph(_)));
    assert!(matches!(blocks[1], Block::List { .. }));
}

#[test]
fn list_immediately_after_bold_pseudo_heading() {
    let blocks = parse("**Heading**\n  - bullet one\n  - bullet two");
    assert_eq!(blocks.len(), 2, "expected paragraph then list: {:?}", blocks);
    let Block::Paragraph(para) = &blocks[0] else {
        panic!();
    };
    assert!(matches!(para[0], Inline::Bold(_)), "first inline is bold");
    let Block::List { items, .. } = &blocks[1] else {
        panic!("expected list, got {:?}", blocks[1]);
    };
    assert_eq!(items.len(), 2);
}

#[test]
fn paragraph_after_list_no_blank_line() {
    let blocks = parse("- one\n- two\nafter text");
    // Without a blank line a fresh paragraph that doesn't start with a list
    // marker simply ends the list — the trailing line becomes its own paragraph.
    assert!(matches!(blocks[0], Block::List { .. }));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
}

#[test]
fn list_item_with_bold() {
    let blocks = parse("- **Bold** item\n- plain item");
    let Block::List { items, .. } = &blocks[0] else {
        panic!();
    };
    let Block::Paragraph(first) = &items[0][0] else {
        panic!();
    };
    assert!(matches!(first[0], Inline::Bold(_)));
}

#[test]
fn list_item_with_apostrophe_in_bold() {
    let blocks = parse("- **Today's Short List**");
    let Block::List { items, .. } = &blocks[0] else {
        panic!();
    };
    let Block::Paragraph(first) = &items[0][0] else {
        panic!();
    };
    let Inline::Bold(inner) = &first[0] else {
        panic!("expected Bold, got {:?}", first[0]);
    };
    assert_eq!(inner, &vec![text("Today's Short List")]);
}

#[test]
fn bold_with_parenthetical_text() {
    let blocks = parse("**Confirm Your Allies (or Lack Thereof)**");
    let Block::Paragraph(para) = &blocks[0] else {
        panic!();
    };
    let Inline::Bold(inner) = &para[0] else {
        panic!("expected Bold, got {:?}", para[0]);
    };
    assert_eq!(inner, &vec![text("Confirm Your Allies (or Lack Thereof)")]);
}

#[test]
fn ai_real_world_narrative_with_indented_bullets() {
    let src = "For this evening, your path is thus:\n\n\
        **Speak with Sarah the Blacksmith**\n\
        \x20\x20- Introduce yourself properly.\n\
        \x20\x20- Ask about better arms and armor.\n\
        \x20\x20- Begin earning her trust.\n\n\
        **Follow up on the Library Clues**\n\
        \x20\x20- You've already learned that Scorath was bound.\n\
        \x20\x20- Next step is to **visit the old library**.";
    let blocks = parse(src);
    assert_eq!(
        blocks.len(),
        5,
        "intro paragraph, heading paragraph, list, heading paragraph, list — got: {:?}",
        blocks
    );
    assert!(matches!(blocks[0], Block::Paragraph(_)));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
    let Block::List {
        items: items_a, ..
    } = &blocks[2]
    else {
        panic!("blocks[2] should be a list, got {:?}", blocks[2]);
    };
    assert_eq!(items_a.len(), 3);
    assert!(matches!(blocks[3], Block::Paragraph(_)));
    let Block::List {
        items: items_b, ..
    } = &blocks[4]
    else {
        panic!("blocks[4] should be a list, got {:?}", blocks[4]);
    };
    assert_eq!(items_b.len(), 2);
}

#[test]
fn ai_short_list_then_ordered_list() {
    let src = "If you wish it simpler:\n\
        - **Today's Short List**\n\
        1. Meet Sarah the Blacksmith.\n\
        2. Go to the old library.\n\
        3. Buy travel gear.";
    let blocks = parse(src);
    let unordered = blocks
        .iter()
        .find(|b| matches!(b, Block::List { ordered: false, .. }));
    assert!(unordered.is_some(), "should have an unordered list: {:?}", blocks);
    let ordered = blocks
        .iter()
        .find(|b| matches!(b, Block::List { ordered: true, .. }));
    assert!(ordered.is_some(), "should have an ordered list: {:?}", blocks);
    let Block::List {
        items: ord_items, ..
    } = ordered.unwrap()
    else {
        unreachable!()
    };
    assert_eq!(ord_items.len(), 3);
}

#[test]
fn loose_unordered_list_blank_lines_between_items() {
    let blocks = parse("- one\n\n- two\n\n- three");
    assert_eq!(blocks.len(), 1, "expected one list, got {:?}", blocks);
    let Block::List { items, .. } = &blocks[0] else {
        panic!();
    };
    assert_eq!(items.len(), 3);
}

#[test]
fn loose_ordered_list_blank_lines_between_items() {
    let blocks = parse("1. one\n\n2. two\n\n3. three");
    assert_eq!(blocks.len(), 1, "expected one list, got {:?}", blocks);
    let Block::List { ordered, items } = &blocks[0] else {
        panic!();
    };
    assert!(ordered);
    assert_eq!(items.len(), 3);
}

#[test]
fn loose_list_with_nested_bullets_between_items() {
    // The shape of real AI narrative output: numbered top-level items with
    // nested bullets, separated by blank lines.
    let src = "1. **First**\n   - sub a\n   - sub b\n\n\
        2. **Second**\n   - sub c\n   - sub d";
    let blocks = parse(src);
    assert_eq!(blocks.len(), 1, "expected one list, got {:?}", blocks);
    let Block::List { ordered, items } = &blocks[0] else {
        panic!();
    };
    assert!(ordered);
    assert_eq!(items.len(), 2, "two top-level numbered items");
    for (idx, item) in items.iter().enumerate() {
        assert_eq!(item.len(), 2, "item {idx} should have paragraph + nested ul");
        assert!(matches!(item[0], Block::Paragraph(_)));
        assert!(matches!(
            item[1],
            Block::List {
                ordered: false,
                ..
            }
        ));
    }
}

#[test]
fn loose_list_ends_when_next_is_not_a_list() {
    let blocks = parse("- one\n- two\n\nafter paragraph");
    assert_eq!(blocks.len(), 2);
    assert!(matches!(blocks[0], Block::List { .. }));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
}

#[test]
fn ai_real_world_numbered_narrative() {
    // Faithful to the actual AI output that triggered this fix.
    let src = "For this evening, in Millbrook, your path is thus:\n\n\
        1. **Speak with Sarah the Blacksmith**\n\
        \x20\x20\x20- Introduce yourself properly.\n\
        \x20\x20\x20- Ask about better arms and armor.\n\
        \x20\x20\x20- Begin earning her trust.\n\n\
        2. **Follow up on the Library Clues**\n\
        \x20\x20\x20- You've already learned that Scorath was bound.\n\
        \x20\x20\x20- Next step is to visit the old library.\n\n\
        3. **Gather Basic Supplies**\n\
        \x20\x20\x20- Check what you can afford with your **10 gold**.\n\
        \x20\x20\x20- Listen for rumors.";
    let blocks = parse(src);
    assert_eq!(blocks.len(), 2, "intro paragraph + one numbered list, got: {:?}", blocks);
    assert!(matches!(blocks[0], Block::Paragraph(_)));
    let Block::List { ordered, items } = &blocks[1] else {
        panic!("blocks[1] should be the numbered list, got {:?}", blocks[1]);
    };
    assert!(ordered, "must be ordered to render `1.` `2.` `3.`");
    assert_eq!(items.len(), 3, "three top-level items");
    for (idx, item) in items.iter().enumerate() {
        assert_eq!(
            item.len(),
            2,
            "item {idx} should have a paragraph and a nested unordered list"
        );
    }
}

#[test]
fn deeply_nested_three_level_list() {
    let src = "- top\n  - mid\n    - deep one\n    - deep two\n  - mid two";
    let blocks = parse(src);
    let Block::List { items, .. } = &blocks[0] else {
        panic!();
    };
    assert_eq!(items.len(), 1, "one top-level item");
    let top_item = &items[0];
    assert_eq!(top_item.len(), 2, "paragraph + nested list");
    let Block::List {
        items: mid_items, ..
    } = &top_item[1]
    else {
        panic!();
    };
    assert_eq!(mid_items.len(), 2, "two mid-level items");
    let Block::List {
        items: deep_items, ..
    } = &mid_items[0][1]
    else {
        panic!("first mid item should have a deep nested list");
    };
    assert_eq!(deep_items.len(), 2);
}

#[test]
fn unmatched_bold_at_start_renders_as_literal() {
    let blocks = parse("**unclosed bold here");
    let Block::Paragraph(para) = &blocks[0] else {
        panic!();
    };
    assert!(
        !para.iter().any(|i| matches!(i, Inline::Bold(_))),
        "should not contain Bold: {:?}",
        para
    );
    let combined: String = para
        .iter()
        .filter_map(|i| if let Inline::Text(t) = i { Some(t.as_str()) } else { None })
        .collect();
    assert!(combined.contains("**"));
    assert!(combined.contains("unclosed bold here"));
}

#[test]
fn adjacent_bold_runs() {
    let blocks = parse("**foo** and **bar**");
    let Block::Paragraph(para) = &blocks[0] else {
        panic!();
    };
    let bolds: Vec<_> = para
        .iter()
        .filter(|i| matches!(i, Inline::Bold(_)))
        .collect();
    assert_eq!(bolds.len(), 2);
}

#[test]
fn bold_then_text_no_space() {
    let blocks = parse("**foo**bar");
    let Block::Paragraph(para) = &blocks[0] else {
        panic!();
    };
    assert!(matches!(para[0], Inline::Bold(_)));
    let last = para.last().unwrap();
    assert!(matches!(last, Inline::Text(t) if t == "bar"));
}

#[test]
fn list_then_paragraph() {
    let blocks = parse("- one\n- two\n\nafter");
    assert_eq!(
        blocks,
        vec![
            Block::List {
                ordered: false,
                items: vec![
                    vec![Block::Paragraph(vec![text("one")])],
                    vec![Block::Paragraph(vec![text("two")])],
                ],
            },
            Block::Paragraph(vec![text("after")]),
        ]
    );
}

fn contains_bold(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| match b {
        Block::Paragraph(i) | Block::Heading { content: i, .. } => inlines_contain_bold(i),
        Block::List { items, .. } => items.iter().any(|item| contains_bold(item)),
        Block::Blockquote(b) => contains_bold(b),
        _ => false,
    })
}

fn inlines_contain_bold(inlines: &[Inline]) -> bool {
    inlines.iter().any(|i| match i {
        Inline::Bold(_) => true,
        Inline::Italic(c) | Inline::Link { text: c, .. } => inlines_contain_bold(c),
        _ => false,
    })
}

fn contains_italic(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| match b {
        Block::Paragraph(i) | Block::Heading { content: i, .. } => inlines_contain_italic(i),
        Block::List { items, .. } => items.iter().any(|item| contains_italic(item)),
        Block::Blockquote(b) => contains_italic(b),
        _ => false,
    })
}

fn inlines_contain_italic(inlines: &[Inline]) -> bool {
    inlines.iter().any(|i| match i {
        Inline::Italic(_) => true,
        Inline::Bold(c) | Inline::Link { text: c, .. } => inlines_contain_italic(c),
        _ => false,
    })
}

fn contains_code_inline(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| match b {
        Block::Paragraph(i) | Block::Heading { content: i, .. } => inlines_contain_code(i),
        Block::List { items, .. } => items.iter().any(|item| contains_code_inline(item)),
        Block::Blockquote(b) => contains_code_inline(b),
        _ => false,
    })
}

fn inlines_contain_code(inlines: &[Inline]) -> bool {
    inlines.iter().any(|i| match i {
        Inline::Code(_) => true,
        Inline::Bold(c) | Inline::Italic(c) | Inline::Link { text: c, .. } => {
            inlines_contain_code(c)
        }
        _ => false,
    })
}
