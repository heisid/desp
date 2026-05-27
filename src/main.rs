mod mdparser;
mod converter;

use crate::mdparser::MdParser;
use std::fs;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, action, default_value_t = true)]
    to_stdout: bool,

    #[arg(short, long, default_value = "parsed_output.html")]
    output: String,
}

fn main() {
    let args = Args::parse();
    let file_content = fs::read_to_string(args.input)
        .expect("File does not exist");

    let mut parser = MdParser::default();
    let output_tokens = parser.parse(&file_content);

    if args.to_stdout {
        println!("{:#?}", output_tokens);
    }


}