mod parser;

use crate::parser::Parser;
use std::fs;


fn main() {
    let file_content = fs::read_to_string("test_file.md")
        .expect("File does not exist");

    let mut parser = Parser::default();
    let output_tokens = parser.parse(&file_content);
    println!("{:#?}", output_tokens);
}