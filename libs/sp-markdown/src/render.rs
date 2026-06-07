use dioxus::prelude::*;

use crate::ast::{Block, Inline};
use crate::classes;
use crate::parser::parse;

#[derive(Clone, Copy)]
struct RenderConfig {
    allow_links: bool,
    allow_inline_code: bool,
    allow_code_blocks: bool,
    allow_headings: bool,
    allow_blockquotes: bool,
}

impl RenderConfig {
    const TRUSTED: Self = Self {
        allow_links: true,
        allow_inline_code: true,
        allow_code_blocks: true,
        allow_headings: true,
        allow_blockquotes: true,
    };
    const CHAT: Self = Self {
        allow_links: false,
        allow_inline_code: false,
        allow_code_blocks: false,
        allow_headings: false,
        allow_blockquotes: false,
    };
}

/// Trusted-mode markdown renderer. Use only for repo-controlled markdown
/// (e.g. bundled privacy/terms documents). Renders the full supported
/// feature set: headings, links, inline + fenced code, lists, blockquotes,
/// emphasis. Do **not** use this for model-generated or user-supplied content
/// — use [`ChatMarkdown`] instead.
#[component]
pub fn Markdown(text: String, class: Option<String>) -> Element {
    render(&text, RenderConfig::TRUSTED, class.as_deref())
}

/// Restricted-mode markdown renderer for untrusted content (AI chat
/// messages, user-supplied text). Renders only visual emphasis: bold,
/// italic, soft line breaks, and lists. Links, inline code, fenced code
/// blocks, headings, and blockquotes are stripped of their styled
/// affordance — their visible text passes through as plain inline content.
/// Link `href` values never reach the DOM.
#[component]
pub fn ChatMarkdown(text: String, class: Option<String>) -> Element {
    render(&text, RenderConfig::CHAT, class.as_deref())
}

fn render(text: &str, cfg: RenderConfig, class: Option<&str>) -> Element {
    let blocks = parse(text);
    let class = class.unwrap_or("").to_string();
    rsx! {
        div { class: "{class}",
            {render_blocks(&blocks, cfg).into_iter().enumerate().map(|(i, el)| rsx! { Fragment { key: "{i}", {el} } })}
        }
    }
}

fn render_blocks(blocks: &[Block], cfg: RenderConfig) -> Vec<Element> {
    let mut out: Vec<Element> = Vec::new();
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => {
                out.push(rsx! { p { class: classes::P, {render_inlines(inlines, cfg)} } });
            }
            Block::Heading { level, content } => {
                if !cfg.allow_headings {
                    out.push(rsx! { p { class: classes::P, {render_inlines(content, cfg)} } });
                } else {
                    let cls = match level {
                        1 => classes::H1,
                        2 => classes::H2,
                        _ => classes::H3,
                    };
                    let inner = render_inlines(content, cfg);
                    out.push(match level {
                        1 => rsx! { h1 { class: cls, {inner} } },
                        2 => rsx! { h2 { class: cls, {inner} } },
                        _ => rsx! { h3 { class: cls, {inner} } },
                    });
                }
            }
            Block::CodeBlock { code, .. } => {
                if !cfg.allow_code_blocks {
                    out.push(rsx! { p { class: classes::P, "{code}" } });
                } else {
                    out.push(rsx! {
                        pre { class: classes::PRE,
                            code { class: classes::CODE_BLOCK, "{code}" }
                        }
                    });
                }
            }
            Block::List { ordered, items } => {
                let cls = if *ordered { classes::OL } else { classes::UL };
                let rendered_items: Vec<Element> = items
                    .iter()
                    .enumerate()
                    .map(|(i, item_blocks)| {
                        let inner = render_blocks(item_blocks, cfg);
                        rsx! { li { key: "{i}", class: classes::LI, {inner.into_iter()} } }
                    })
                    .collect();
                if *ordered {
                    out.push(rsx! { ol { class: cls, {rendered_items.into_iter()} } });
                } else {
                    out.push(rsx! { ul { class: cls, {rendered_items.into_iter()} } });
                }
            }
            Block::Blockquote(inner) => {
                if !cfg.allow_blockquotes {
                    out.extend(render_blocks(inner, cfg));
                } else {
                    let inner_els = render_blocks(inner, cfg);
                    out.push(rsx! {
                        blockquote { class: classes::BLOCKQUOTE, {inner_els.into_iter()} }
                    });
                }
            }
        }
    }
    out
}

fn render_inlines(inlines: &[Inline], cfg: RenderConfig) -> Element {
    rsx! {
        {inlines.iter().enumerate().map(|(i, inline)| render_inline(inline, cfg, i))}
    }
}

fn render_inline(inline: &Inline, cfg: RenderConfig, key: usize) -> Element {
    match inline {
        Inline::Text(s) => rsx! { Fragment { key: "{key}", "{s}" } },
        Inline::Bold(children) => rsx! {
            strong { key: "{key}", {render_inlines(children, cfg)} }
        },
        Inline::Italic(children) => rsx! {
            em { key: "{key}", {render_inlines(children, cfg)} }
        },
        Inline::Code(s) => {
            if cfg.allow_inline_code {
                rsx! { code { key: "{key}", class: classes::CODE_INLINE, "{s}" } }
            } else {
                rsx! { Fragment { key: "{key}", "{s}" } }
            }
        }
        Inline::Link { text, href } => {
            if cfg.allow_links {
                rsx! {
                    a {
                        key: "{key}",
                        class: classes::A,
                        href: "{href}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        {render_inlines(text, cfg)}
                    }
                }
            } else {
                rsx! { Fragment { key: "{key}", {render_inlines(text, cfg)} } }
            }
        }
        Inline::SoftBreak => rsx! { br { key: "{key}" } },
    }
}
