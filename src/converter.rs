use crate::mdparser::{BlockToken, InlineToken};

pub fn to_html(block_tokens: &Vec<BlockToken>) -> String {
    let mut html = String::new();
    let mut is_in_ul = false;
    for block_token in block_tokens {
        match block_token {
            BlockToken::Paragraph(content) => {
                if is_in_ul { html.push_str("</ul>\n"); }
                is_in_ul = false;

                html.push_str("<p>");
                html.push_str(&inline_to_html(&content));
                html.push_str("</p>\n");
            }
            BlockToken::Header{level, content} => {
                if is_in_ul { html.push_str("</ul>\n"); }
                is_in_ul = false;

                html.push_str(format!("<h{}>", level).as_str());
                html.push_str(&inline_to_html(&content));
                html.push_str(format!("</h{}>\n", level).as_str());
            }
            BlockToken::Code(content) => {
                if is_in_ul { html.push_str("</ul>\n"); }
                is_in_ul = false;

                html.push_str("<pre><code>");
                html.push_str(&content);
                html.push_str("</code></pre>");
            }
            BlockToken::Line => {
                if is_in_ul { html.push_str("</ul>\n"); }
                is_in_ul = false;

                html.push_str("<hr>");
            }
            BlockToken::UnorderedListItem(content) => {
                if !is_in_ul {
                    html.push_str("<ul>\n");
                }
                html.push_str("<li>");
                html.push_str(&inline_to_html(&content));
                html.push_str("</li>\n");
                is_in_ul = true;
            }
        }
    };
    html
}

fn inline_to_html(inline_tokens: &Vec<InlineToken>) -> String {
    let mut html = String::new();
    for inline_token in inline_tokens {
        let mut start_tag = String::new();
        let mut end_tag = String::new();
        let mut content = String::new();
        match inline_token {
            InlineToken::Bold(inner) => {
                start_tag = "<strong>".to_string();
                end_tag = "</strong>".to_string();
                content = inner.to_string();
            },
            InlineToken::Italics(inner) => {
                start_tag = "<i>".to_string();
                end_tag = "</i>".to_string();
                content = inner.to_string();
            }
            InlineToken::BoldItalics(inner) => {
                start_tag = "<strong><i>".to_string();
                end_tag = "</i></strong>".to_string();
                content = inner.to_string();
            }
            InlineToken::Code(inner) => {
                start_tag = "<code>".to_string();
                end_tag = "</code>".to_string();
                content = inner.to_string();
            }
            InlineToken::Link { caption, url } => {
                start_tag = format!("<a href=\"{}\">", url);
                end_tag = "</a>".to_string();
                content = caption.to_string();
            }
            InlineToken::Image { caption, url } => {
                start_tag = format!("<img src=\"{}\">", url);
                end_tag = "</img>".to_string();
                content = caption.to_string();
            }
            InlineToken::Unformatted(unformatted) => { content = unformatted.to_string() }
        };
        html.push_str(&start_tag);
        html.push_str(&content);
        html.push_str(&end_tag);
    };
    html
}