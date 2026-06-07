use dioxus::prelude::*;
use sp_markdown::{ChatMarkdown, Markdown};

#[derive(Props, Clone, PartialEq)]
struct WrapProps {
    text: String,
}

#[component]
fn TrustedWrap(props: WrapProps) -> Element {
    rsx! { Markdown { text: props.text } }
}

#[component]
fn ChatWrap(props: WrapProps) -> Element {
    rsx! { ChatMarkdown { text: props.text } }
}

fn render_trusted(text: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(
        TrustedWrap,
        WrapProps {
            text: text.to_string(),
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn render_chat(text: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ChatWrap,
        WrapProps {
            text: text.to_string(),
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[test]
fn trusted_renders_heading() {
    let html = render_trusted("## Section");
    assert!(html.contains("<h2"), "expected <h2 in: {html}");
    assert!(html.contains("Section"));
}

#[test]
fn trusted_renders_link_with_safe_attrs() {
    let html = render_trusted("[docs](https://example.com)");
    assert!(html.contains("<a "), "expected <a in: {html}");
    assert!(html.contains("href=\"https://example.com\""));
    assert!(html.contains("target=\"_blank\""));
    assert!(html.contains("rel=\"noopener noreferrer\""));
    assert!(html.contains("docs"));
}

#[test]
fn trusted_renders_inline_code() {
    let html = render_trusted("run `foo`");
    assert!(html.contains("<code"), "expected <code in: {html}");
    assert!(html.contains("foo"));
}

#[test]
fn trusted_renders_fenced_code_block() {
    let html = render_trusted("```js\nx = 1\n```");
    assert!(html.contains("<pre"), "expected <pre in: {html}");
    assert!(html.contains("<code"));
    assert!(html.contains("x = 1"));
}

#[test]
fn trusted_renders_unordered_list() {
    let html = render_trusted("- one\n- two");
    assert!(html.contains("<ul"));
    assert!(html.contains("<li"));
    assert!(html.contains("one"));
    assert!(html.contains("two"));
}

#[test]
fn trusted_renders_blockquote() {
    let html = render_trusted("> hello");
    assert!(html.contains("<blockquote"), "expected blockquote in: {html}");
    assert!(html.contains("hello"));
}

#[test]
fn trusted_renders_bold_and_italic() {
    let html = render_trusted("**bold** and *italic*");
    assert!(html.contains("<strong"));
    assert!(html.contains("bold"));
    assert!(html.contains("<em"));
    assert!(html.contains("italic"));
}

#[test]
fn chat_strips_link_tag_keeps_text_drops_href() {
    let html = render_chat("[click here](https://evil.com)");
    assert!(!html.contains("<a "), "<a> tag must not appear in chat mode: {html}");
    assert!(!html.contains("evil.com"), "href must not leak: {html}");
    assert!(!html.contains("href"), "no href attr at all: {html}");
    assert!(html.contains("click here"));
}

#[test]
fn chat_strips_inline_code_tag() {
    let html = render_chat("see `foo` here");
    assert!(!html.contains("<code"), "<code> tag must not appear: {html}");
    assert!(html.contains("foo"));
    assert!(html.contains("see "));
    assert!(html.contains(" here"));
}

#[test]
fn chat_strips_fenced_code_block() {
    let html = render_chat("```js\nconsole.log('x')\n```");
    assert!(!html.contains("<pre"), "<pre> must not appear: {html}");
    assert!(!html.contains("<code"), "<code> must not appear: {html}");
    assert!(html.contains("console.log"));
}

#[test]
fn chat_flattens_heading() {
    let html = render_chat("## Section");
    assert!(!html.contains("<h1"));
    assert!(!html.contains("<h2"));
    assert!(!html.contains("<h3"));
    assert!(html.contains("Section"));
}

#[test]
fn chat_unwraps_blockquote() {
    let html = render_chat("> quoted");
    assert!(!html.contains("<blockquote"));
    assert!(html.contains("quoted"));
}

#[test]
fn chat_keeps_emphasis_and_lists() {
    let html = render_chat("**bold** and *italic*\n\n- one\n- two");
    assert!(html.contains("<strong"));
    assert!(html.contains("<em"));
    assert!(html.contains("<ul"));
    assert!(html.contains("<li"));
}

#[test]
fn chat_user_message_renders_italic_too() {
    // No more asymmetric italic suppression — both user and AI messages should render emphasis identically.
    let html = render_chat("user wrote *foo*");
    assert!(html.contains("<em"));
    assert!(html.contains("foo"));
}

#[test]
fn chat_renders_nested_list_as_nested_ul() {
    let html = render_chat("- top\n  - sub a\n  - sub b");
    let first_ul = html.find("<ul").expect("expected outer ul");
    let nested_ul = html[first_ul + 3..]
        .find("<ul")
        .expect("expected nested ul inside the outer one");
    assert!(nested_ul > 0);
    assert!(html.contains("sub a"));
    assert!(html.contains("sub b"));
    assert!(html.contains("top"));
}

#[test]
fn chat_renders_indented_bullets_as_a_list() {
    let html = render_chat("**Heading**\n  - one\n  - two");
    assert!(html.contains("<strong"));
    assert!(html.contains("<ul"), "indented bullets must be a list: {html}");
    assert!(html.contains("<li"));
    assert!(html.contains("one"));
    assert!(html.contains("two"));
    let li_count = html.matches("<li").count();
    assert_eq!(li_count, 2, "exactly two list items: {html}");
}

#[test]
fn chat_real_world_ai_narrative() {
    let src = "**Speak with Sarah the Blacksmith**\n\
        \x20\x20- Introduce yourself.\n\
        \x20\x20- Ask about armor.\n\n\
        **Follow up on the Library Clues**\n\
        \x20\x20- Visit the library.";
    let html = render_chat(src);
    assert_eq!(html.matches("<ul").count(), 2, "two separate lists: {html}");
    assert_eq!(html.matches("<strong").count(), 2, "two bold headings: {html}");
    assert!(!html.contains("&lt;"), "should not contain escaped tags: {html}");
}

#[test]
fn chat_javascript_link_dropped_text_kept() {
    let html = render_chat("[hi](javascript:alert(1))");
    assert!(!html.contains("<a "));
    assert!(!html.contains("javascript"));
    assert!(html.contains("hi"));
}
