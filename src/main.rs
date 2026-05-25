use std::cmp::PartialEq;
use std::fs;
use regex::Regex;

#[derive(Debug)]
enum BlockToken {
    Paragraph(Vec<InlineToken>),
    Header{level: u8, content: Vec<InlineToken>},
    Code(String),
}

enum BlockState {
    IsInParagraph,
    IsInCode
}

#[derive(Debug)]
enum InlineToken {
    Unformatted(String),
    Bold(String),
    Italics(String),
    Code(String),
}


fn main() {
    let file_content = fs::read_to_string("test_file.md")
        .expect("File does not exist");

    let mut block_tokens: Vec<BlockToken> = Vec::new();
    let lines = file_content.split("\n").collect::<Vec<&str>>();
    let mut temp_content: String = String::new();
    let mut block_state = BlockState::IsInParagraph;
    for line in lines {
        // ------------ Header ------------------
        let mut header_content: &str = "";
        let mut header_level: u8 = 0;
        let re_header = Regex::new(r"(#^{1,6})(\s+)(.*)").unwrap();
        for (_, [header_tag, _, header_match]) in re_header.captures_iter(line).map(|c| c.extract()) {
            header_content = header_match;
            header_level = header_tag.len() as u8;
        }
        if !header_content.is_empty() {
            block_tokens.push(BlockToken::Header{level: header_level, content: process_inline(header_content)});
            continue;
        }

        // ----------- Code Block ------------------
        let re_code = Regex::new(r"^```\s?$").unwrap();
        if re_code.captures_iter(line).count() > 0 {
            match block_state {
                BlockState::IsInCode => {
                    block_tokens.push(BlockToken::Code(temp_content.clone()));
                    temp_content.clear();
                    block_state = BlockState::IsInParagraph;
                },
                BlockState::IsInParagraph => {
                    flush_paragraph(&mut temp_content, &mut block_tokens);
                    block_state = BlockState::IsInCode;
                }
            }
            continue;
        }
        if matches!(block_state, BlockState::IsInCode) {
            push_with_nl(&mut temp_content, line);
            continue;
        }

        // ----------- Paragraph ------------------
        let re_paragraph = Regex::new(r"^\s?$").unwrap();
        if re_paragraph.captures_iter(line).count() > 0 {
            if !temp_content.is_empty() {
                block_tokens.push(BlockToken::Paragraph(process_inline(&temp_content)));
                temp_content.clear();
            }
            continue;
        }
        if matches!(block_state, BlockState::IsInParagraph) {
            if !temp_content.is_empty() {
                temp_content.push('\n');
            }
            temp_content.push_str(line);
            continue;
        }
    }
    flush_paragraph(&mut temp_content, &mut block_tokens);
    for token in block_tokens {
        println!("{:?}", token);
    }
}

fn push_with_nl(orig_text: &mut String, new_line: &str) {
    if !orig_text.is_empty() {
        orig_text.push_str("\n");
    }
    orig_text.push_str(new_line);
}

fn flush_paragraph(temp_content: &mut String, block_tokens: &mut Vec<BlockToken>) {
    if !temp_content.is_empty() {
        block_tokens.push(BlockToken::Paragraph(process_inline(temp_content)));
    }
    temp_content.clear();
}

fn process_inline(inline_string: &str) -> Vec<InlineToken> {
    // todo
    let mut inline_tokens: Vec<InlineToken> = Vec::new();
    inline_tokens.push(InlineToken::Unformatted(inline_string.to_string()));
    inline_tokens
}


