use std::sync::LazyLock;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum BlockToken {
    Paragraph(Vec<InlineToken>),
    Header { level: u8, content: Vec<InlineToken> },
    Code(String),
    Line,
    UnorderedListItem(Vec<InlineToken>),
}

#[derive(Debug, Clone)]
pub enum InlineToken {
    Unformatted(String),
    Bold(String),
    Italics(String),
    BoldItalics(String),
    Code(String),
    Link { caption: String, url: String },
    Image { caption: String, url: String },
}

struct InlineSpan {
    start: usize,
    end: usize,
    token: InlineToken,
}

enum BlockState {
    InParagraph,
    InCode,
}


static RE_HEADER:       LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(^#{1,6})\s+(.*)").unwrap());
static RE_BLANK:        LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*$").unwrap());
static RE_CODE_BLOCK:   LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^```\s*$").unwrap());
static RE_LINE:         LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-{2,}\s*$").unwrap());
static RE_UL:           LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[-*]\s(.*)").unwrap());
static RE_ASTERISK:     LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\*+)([^\s*](?:[^*]*?[^\s*])?)(\*+)").unwrap());
static RE_LINK_NAMED:   LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[(.+)]\((.+)\)").unwrap());
static RE_LINK_UNNAMED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(.+)>").unwrap());
static RE_IMAGE:        LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[(.+)]\((.+)\)").unwrap());



pub struct MdParser {
    tokens: Vec<BlockToken>,
    state: BlockState,
    // Accumulates raw text while inside a paragraph or code block.
    pending: String,
}

impl Default for MdParser {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            state: BlockState::InParagraph,
            pending: String::new(),
        }
    }
}

impl MdParser {
    pub fn parse(&mut self, input: &str) -> Vec<BlockToken> {
        for line in input.lines() {
            self.process_line(line);
        }
        self.flush_paragraph();
        self.tokens.clone()
    }

    fn process_line(&mut self, line: &str) {
        // ----------- Header --------------------
        if let Some(caps) = RE_HEADER.captures(line) {
            let level = caps[1].len() as u8;
            let content = parse_inline(&caps[2]);
            self.tokens.push(BlockToken::Header { level, content });
            return;
        }

        // ------------ Code Block ----------------
        if RE_CODE_BLOCK.is_match(line) {
            match self.state {
                BlockState::InCode => {
                    self.tokens.push(BlockToken::Code(self.pending.clone()));
                    self.pending.clear();
                    self.state = BlockState::InParagraph;
                }
                BlockState::InParagraph => {
                    self.flush_paragraph();
                    self.state = BlockState::InCode;
                }
            }
            return;
        }

        if matches!(self.state, BlockState::InCode) {
            append_line(&mut self.pending, line);
            return;
        }

        // -- Line --
        if RE_LINE.is_match(line) {
            self.tokens.push(BlockToken::Line);
            return;
        }

        // -- Unordered list item --
        if let Some(caps) = RE_UL.captures(line) {
            let content = parse_inline(&caps[1]);
            self.tokens.push(BlockToken::UnorderedListItem(content));
            return;
        }

        // -- Blank line ends a paragraph --
        if RE_BLANK.is_match(line) {
            self.flush_paragraph();
            return;
        }

        append_line(&mut self.pending, line);
    }

    fn flush_paragraph(&mut self) {
        if !self.pending.is_empty() {
            let content = parse_inline(&self.pending);
            self.tokens.push(BlockToken::Paragraph(content));
            self.pending.clear();
        }
    }
}

fn parse_inline(text: &str) -> Vec<InlineToken> {
    let mut spans = collect_inline_spans(text);

    if spans.is_empty() {
        return vec![InlineToken::Unformatted(text.to_string())];
    }

    spans.sort_by_key(|s| s.start);

    let mut tokens: Vec<InlineToken> = Vec::new();
    let mut cursor = 0;

    for span in spans {
        if cursor < span.start {
            let plain = text[cursor..span.start].to_string();
            if !plain.is_empty() {
                tokens.push(InlineToken::Unformatted(plain));
            }
        }
        tokens.push(span.token);
        cursor = span.end + 1;
    }

    if cursor < text.len() {
        let tail = text[cursor..].to_string();
        if !tail.is_empty() {
            tokens.push(InlineToken::Unformatted(tail));
        }
    }

    tokens
}

fn collect_inline_spans(text: &str) -> Vec<InlineSpan> {
    let mut spans: Vec<InlineSpan> = Vec::new();

    // ------- Bold / italics / bold-italics --------------
    for cap in RE_ASTERISK.captures_iter(text) {
        let inner = cap.get(2).unwrap();
        let (_, [opening, content, closing]) = cap.extract();
        let asterisk_count = opening.len().min(closing.len());

        let token = match asterisk_count {
            1 => InlineToken::Italics(content.to_string()),
            3 => InlineToken::BoldItalics(content.to_string()),
            _ => InlineToken::Bold(content.to_string()), // 2 or >3
        };

        spans.push(InlineSpan {
            start: inner.start() - asterisk_count,
            end:   inner.end()   + asterisk_count,
            token,
        });
    }

    // --------- Named link: [caption](url) --------------
    for cap in RE_LINK_NAMED.captures_iter(text) {
        let bracket_start = cap.get(1).unwrap().start() - 1;
        let paren_end     = cap.get(2).unwrap().end();
        let (_, [caption, url]) = cap.extract();
        spans.push(InlineSpan {
            start: bracket_start,
            end:   paren_end,
            token: InlineToken::Link {
                caption: caption.to_string(),
                url:     url.to_string(),
            },
        });
    }

    // ------------ Image ---------------------------------
    for cap in RE_IMAGE.captures_iter(text) {
        let bracket_start = cap.get(1).unwrap().start() - 1;
        let paren_end     = cap.get(2).unwrap().end();
        let (_, [caption, url]) = cap.extract();
        spans.push(InlineSpan {
            start: bracket_start,
            end:   paren_end,
            token: InlineToken::Image {
                caption: caption.to_string(),
                url:     url.to_string(),
            },
        });
    }

    // ------------ Unnamed link: <url> -----------------
    for cap in RE_LINK_UNNAMED.captures_iter(text) {
        let inner = cap.get(1).unwrap();
        let (_, [url]) = cap.extract();
        spans.push(InlineSpan {
            start: inner.start() - 1,
            end:   inner.end(),
            token: InlineToken::Link {
                caption: url.to_string(),
                url:     url.to_string(),
            },
        });
    }

    spans
}

fn append_line(buf: &mut String, line: &str) {
    if !buf.is_empty() {
        buf.push('\n');
    }
    buf.push_str(line);
}