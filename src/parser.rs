use std::sync::LazyLock;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum BlockToken {
    Paragraph(Vec<InlineToken>),
    Header{level: u8, content: Vec<InlineToken>},
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
    Link{caption: String, url: String},
}

enum BlockState {
    IsInParagraph,
    IsInCode,
}

static RE_HEADER: LazyLock<Regex> = LazyLock::new(||Regex::new(r"(^#{1,6})(\s+)(.*)").unwrap());
static RE_PARAGRAPH: LazyLock<Regex> = LazyLock::new(||Regex::new(r"^\s?$").unwrap());
static RE_CODE: LazyLock<Regex> = LazyLock::new(||Regex::new(r"^```\s?$").unwrap());
static RE_LINE: LazyLock<Regex> = LazyLock::new(||Regex::new(r"^-+\s?$").unwrap());
static RE_UL: LazyLock<Regex> = LazyLock::new(||Regex::new(r"^[-*]\s(.*)").unwrap());
static RE_ASTERIX: LazyLock<Regex> = LazyLock::new(||Regex::new(r"(\*+)([^\s*](?:[^*]*?[^\s*])?)(\*+)").unwrap());
static RE_LINK_NAMED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[(.+)]\((.+)\)").unwrap());
static RE_LINK_UNNAMED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<(.+)>").unwrap());




pub struct Parser {
    pub tokens: Vec<BlockToken>,
    state: BlockState,
}

impl Parser {
    pub fn default() -> Self {
        Self {
            tokens: vec![],
            state: BlockState::IsInParagraph,
        }
    }

    pub fn parse(&mut self, input: &str) -> Vec<BlockToken> {
        let lines = input.split("\n").collect::<Vec<&str>>();
        let mut temp_block_content: String = String::new();
        for line in lines {
            self.get_block_token(line, &mut temp_block_content);
        }
        self.tokens.clone()
    }
    
    fn get_block_token(&mut self, line: &str, temp_block_content: &mut String) {
        // ------------ Header ------------------
        let mut header_content: &str = "";
        let mut header_level: u8 = 0;
        for (_, [header_tag, _, content]) in RE_HEADER.captures_iter(line).map(|c| c.extract()) {
            header_content = content;
            header_level = header_tag.len() as u8;
        }
        if !header_content.is_empty() {
            self.tokens.push(BlockToken::Header{level: header_level, content: self.process_inline(header_content)});
            return;
        }

        // ----------- Code Block ------------------
        if RE_CODE.captures_iter(line).count() > 0 {
            match self.state {
                BlockState::IsInCode => {
                    self.tokens.push(BlockToken::Code(temp_block_content.clone()));
                    temp_block_content.clear();
                    self.state = BlockState::IsInParagraph;
                },
                _ => {
                    self.flush_paragraph(temp_block_content);
                    self.state = BlockState::IsInCode;
                }
            }
            return;
        }
        if matches!(self.state, BlockState::IsInCode) {
            push_with_nl(temp_block_content, line);
            return;
        }

        // ---------- Divider line ---------------
        if RE_LINE.is_match(line) {
            self.tokens.push(BlockToken::Line);
            return;
        }

        // ----------- Unordered List -------------
        if let Some(caps) = RE_UL.captures(line) {
            self.tokens.push(BlockToken::UnorderedListItem(self.process_inline(caps.get(1).unwrap().as_str())));
            return;
        }

        // ----------- Paragraph ------------------
        if RE_PARAGRAPH.captures_iter(line).count() > 0 {
            self.flush_paragraph(temp_block_content);
            return;
        }
        if matches!(self.state, BlockState::IsInParagraph) {
            push_with_nl(temp_block_content, line);
            return;
        }
    }

    fn flush_paragraph(&mut self, temp_content: &mut String) {
        if !temp_content.is_empty() {
            self.tokens.push(BlockToken::Paragraph(self.process_inline(temp_content)));
        }
        temp_content.clear();
    }

    fn process_inline(&self, text: &str) -> Vec<InlineToken> {
        let mut inline_tokens: Vec<InlineToken> = Vec::new();
        let mut collected_tokens: Vec<(usize, usize, InlineToken)> = vec![]; // index start, index end, token
        // ----- Bold and italics ---------
        for capture in RE_ASTERIX.captures_iter(text) {
            let re_match = capture.get(2).unwrap();
            let index_start = re_match.start();
            let index_end = re_match.end();
            let (_, [opening_astx, content, closing_astx]) = capture.extract();
            let astx_num = opening_astx.len().min(closing_astx.len());
            if astx_num == 1 {
                let token = InlineToken::Italics(content.to_string());
                collected_tokens.push((index_start-astx_num, index_end+astx_num, token));
            } else if astx_num == 2 || astx_num > 3 {
                let token = InlineToken::Bold(content.to_string());
                collected_tokens.push((index_start-astx_num, index_end+astx_num, token));
            } else if astx_num == 3 {
                let token = InlineToken::BoldItalics(content.to_string());
                collected_tokens.push((index_start-astx_num, index_end+astx_num, token));
            }
        };
        // ------ Named Link ---------
        for capture in RE_LINK_NAMED.captures_iter(text) {
            let re_match = capture.get(1).unwrap();
            let index_start = re_match.start()-1;
            let index_end = re_match.end();
            let (_, [caption, url]) = capture.extract();
            collected_tokens.push((index_start, index_end, InlineToken::Link{caption: caption.to_string(), url: url.to_string()}));
        }
        // ----- Unnamed Link -------
        for capture in RE_LINK_UNNAMED.captures_iter(text) {
            let re_match = capture.get(1).unwrap();
            let index_start = re_match.start()-1;
            let index_end = re_match.end();
            let (_, [url]) = capture.extract();
            collected_tokens.push((index_start, index_end, InlineToken::Link{caption: url.to_string(), url: url.to_string()}));
        }
        let mut last_unformatted_index: usize = 0;
        if collected_tokens.is_empty() {
            inline_tokens.push(InlineToken::Unformatted(String::from(text)));
            return inline_tokens;
        }
        collected_tokens.sort_by(|a, b| {a.0.cmp(&b.0)});
        for token in collected_tokens {
            let unformatted = text[last_unformatted_index..token.0].to_string();
            if !unformatted.is_empty() {
                inline_tokens.push(InlineToken::Unformatted(unformatted));
            }
            inline_tokens.push(token.2);
            last_unformatted_index = token.1 + 1;
        }
        inline_tokens
    }
}

fn push_with_nl(orig_text: &mut String, new_line: &str) {
    if !orig_text.is_empty() {
        orig_text.push_str("\n");
    }
    orig_text.push_str(new_line);
}