use regex::Regex;
use std::fs;

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
    BoldItalics(String),
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
        let re_header = Regex::new(r"(^#{1,6})(\s+)(.*)").unwrap();
        for (_, [header_tag, _, content]) in re_header.captures_iter(line).map(|c| c.extract()) {
            header_content = content;
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
            flush_paragraph(&mut temp_content, &mut block_tokens);
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
    let mut inline_tokens: Vec<InlineToken> = Vec::new();
    // Match text wrapped in *...* or **...**, where the content
    // does not start/end with space or contain *
    let re_asterix = Regex::new(r"(\*+)([^\s*](?:[^*]*?[^\s*])?)(\*+)").unwrap();
    let mut collected_tokens: Vec<(usize, usize, InlineToken)> = vec![]; // index start, index end, token
    for capture in re_asterix.captures_iter(inline_string) {
        let re_match = capture.get(1).unwrap();
        let index_start = re_match.start();
        let index_end = re_match.end();
        let (_, [opening_astx, content, closing_astx]) = capture.extract();
        let astx_num = opening_astx.len().min(closing_astx.len());
        if astx_num == 1 {
            let token = InlineToken::Italics(content.to_string());
            collected_tokens.push((index_start, index_end, token));
        } else if astx_num == 2 || astx_num > 3 {
            let token = InlineToken::Bold(content.to_string());
            collected_tokens.push((index_start, index_end, token));
        } else if astx_num == 3 {
            let token = InlineToken::BoldItalics(content.to_string());
            collected_tokens.push((index_start, index_end, token));
        }
    };
    let mut last_unformatted_index: usize = 0;
    if collected_tokens.is_empty() {
        inline_tokens.push(InlineToken::Unformatted(String::from(inline_string)));
        return inline_tokens;
    }
    for token in collected_tokens {
        let unformatted = inline_string[last_unformatted_index..token.0].to_string();
        if !unformatted.is_empty() {
            inline_tokens.push(InlineToken::Unformatted(unformatted));
        }
        inline_tokens.push(token.2);
        last_unformatted_index = token.1 + 1;
    }
    inline_tokens
}


