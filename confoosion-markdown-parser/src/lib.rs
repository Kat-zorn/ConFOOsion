mod putback;

use putback::PutBackChars;
use std::path::Path;

pub struct ParsedHTML {
    pub html: String,
    pub links_to: Vec<String>,
    pub parents: Vec<String>,
}

enum TextModifier {
    Bold,
    Italics,
    Strikethrough,
    Underline,
    Quote,
}

struct ParseState {
    pub is_code: bool,
    pub modifier_stack: Vec<TextModifier>,
}

pub fn markdown_to_html<T>(file: T) -> ParsedHTML
where
    T: AsRef<Path>,
{
    use TextModifier::*;

    let mut html = String::new();
    let mut links_to = Vec::new();
    let mut parents = Vec::new();

    let contents = std::fs::read_to_string(file).unwrap();
    let mut chars: PutBackChars = contents.chars().into();

    let mut state = ParseState {
        is_code: false,
        modifier_stack: Vec::new(),
    };

    while let Some(character) = chars.next() {
        match character {
            '*' => match chars.next() {
                // A BOLD token is found
                Some('*') => match state.modifier_stack.last() {
                    // Close the current BOLD span
                    Some(Bold) => {
                        state.modifier_stack.pop();
                        html.push_str("</b>");
                    }
                    // Open a new BOLD span
                    _else => {
                        state.modifier_stack.push(Bold);
                        html.push_str("<b>");
                    }
                },
                other => {
                    if let Some(character) = other {
                        chars.putback(character);
                    }
                    match state.modifier_stack.last() {
                        Some(Italics) => {
                            state.modifier_stack.pop();
                            html.push_str("</i>");
                        }
                        _else => {
                            state.modifier_stack.push(Italics);
                            html.push_str("<i>");
                        }
                    }
                }
            },
            '\\' => {
                let next = chars.next().expect("A file may not end with an escape character");
                html.push(next);
            },
            '~' => todo!("Strikethough is under development"),
            '\n' => todo!("Paragrapgs are under development"),
            '#' => todo!("Headings are under development"),
            '`' => todo!("Code blocks and inline code is under development"),
            '>' => todo!("Quote blocks are under development"),
            '_' => todo!("Underline is under development"),
            '[' => todo!("Links are under development"),
            '{' => todo!("Templates are under development"),
            '!' => todo!("Images are under development"),
            other => {
                html.push(other);
            }
        }
    }

    ParsedHTML {
        html,
        links_to,
        parents,
    }
}
