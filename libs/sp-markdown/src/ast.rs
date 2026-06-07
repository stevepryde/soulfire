#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
        closed: bool,
    },
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
    },
    Blockquote(Vec<Block>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Code(String),
    Link { text: Vec<Inline>, href: String },
    SoftBreak,
}
