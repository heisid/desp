mod mdparser;
mod converter;

use crate::mdparser::MdParser;
use std::fs;
use std::io::Error;
use clap::Parser;
use crate::converter::to_html;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, action, default_value_t = true)]
    to_stdout: bool,

    #[arg(short, long, action, default_value_t = true)]
    as_html: bool,

    #[arg(short, long, default_value = "parsed_output.html")]
    output: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let input_file = &args.input;
    let file_content = fs::read_to_string(input_file)
        .expect("File does not exist");

    let mut parser = MdParser::default();
    let output_tokens = parser.parse(&file_content);

    let output: String = if args.as_html {
        to_html(&output_tokens)
    } else {
        format!("{:#?}", output_tokens).to_string()
    };

    if args.to_stdout {
        println!("{}", &output);
    }

    fs::write(args.output, &output)
}