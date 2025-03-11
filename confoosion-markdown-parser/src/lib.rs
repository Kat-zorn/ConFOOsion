mod error;
mod putback;

use error::ParseError;
use putback::PutBackChars;
use std::path::Path;

pub struct ParsedHTML {
    pub html: String,
    pub links_to: Vec<String>,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum TextModifier {
    Bold,
    Italics,
    Strikethrough,
    Underline,
    Quote,
}
#[derive(Debug, Clone, Copy)]
enum ExclusiveModifier {
    Escape,
    Template,
    WikiLink,
    InlineCode,
    CodeBlock,
    Paragraph,
    Heading(u8),
    Link,
    Image,
    EndOfArgument,
    EndOfTemplate,
}
#[derive(Debug, Clone, Copy)]
enum Delimiter {
    TextModifier(TextModifier),
    ExclusiveModifier(ExclusiveModifier),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitMode {
    EndOfArgument,
    EndOfTemplate,
    EndOfFile,
}

pub fn markdown_file_to_html<T>(file: T) -> Result<ParsedHTML, ParseError>
where
    T: AsRef<Path>,
{
    let contents = std::fs::read_to_string(file).unwrap();
    let mut chars: PutBackChars = contents.chars().into();
    chars.putback('\n');
    chars.line_number = 1;
    return match markdown_charbuff_to_html(&mut chars)? {
        (parsed, ExitMode::EndOfFile) => Ok(parsed),
        (_parsed, ExitMode::EndOfArgument) => Err(ParseError::empty("Stray argument separator")),
        (_parsed, ExitMode::EndOfTemplate) => Err(ParseError::empty("Stray template terminator")),
    };
}

pub fn markdown_charbuff_to_html(
    chars: &mut PutBackChars,
) -> Result<(ParsedHTML, ExitMode), ParseError> {
    let html = String::new();
    let links_to = Vec::new();
    let parents = Vec::new();
    let mut parsed_html = ParsedHTML {
        html,
        links_to,
        parents,
    };

    let mut modifier_stack = Vec::new();

    // parsed_html.html.push_str("<p>");
    while let Some(character) = chars.next() {
        chars.putback(character);
        if let Some(&open_delimiter) = modifier_stack.last() {
            if has_close_delimiter(chars, open_delimiter) {
                let _ = modifier_stack.pop().unwrap();
                parsed_html.html.push_str(open_delimiter.close());
                continue;
            }
        }
        if let Some(delimiter) = find_open_delimiter(chars) {
            match delimiter {
                Delimiter::TextModifier(text_modifier) => {
                    modifier_stack.push(text_modifier);
                    parsed_html.html.push_str(text_modifier.open());
                }
                Delimiter::ExclusiveModifier(ExclusiveModifier::EndOfArgument) => {
                    return Ok((parsed_html, ExitMode::EndOfArgument));
                }
                Delimiter::ExclusiveModifier(ExclusiveModifier::EndOfTemplate) => {
                    return Ok((parsed_html, ExitMode::EndOfTemplate));
                }
                Delimiter::ExclusiveModifier(exclusive_modifier) => {
                    exclusive_modifier.to_html(chars, &mut parsed_html);
                }
            }
        } else {
            parsed_html.html.push(chars.next().unwrap());
        }
    }
    if modifier_stack.is_empty() {
        Ok((parsed_html, ExitMode::EndOfFile))
    } else {
        Err(ParseError::from_str(
            &chars,
            "Unclosed modifiers left on the stack",
        ))
    }
}

fn has_close_delimiter(chars: &mut PutBackChars, delimiter: TextModifier) -> bool {
    match delimiter {
        TextModifier::Bold => match chars.next() {
            Some('*') => match chars.next() {
                Some('*') => true,
                other => {
                    chars.putback_maybe(other);
                    false
                }
            },
            other => {
                chars.putback_maybe(other);
                false
            }
        },
        TextModifier::Italics => match chars.next() {
            Some('*') => true,
            other => {
                chars.putback_maybe(other);
                false
            }
        },
        TextModifier::Strikethrough => match chars.next() {
            Some('~') => match chars.next() {
                Some('~') => true,
                other => {
                    chars.putback_maybe(other);
                    false
                }
            },
            other => {
                chars.putback_maybe(other);
                false
            }
        },
        TextModifier::Underline => match chars.next() {
            Some('_') => match chars.next() {
                Some('_') => true,
                other => {
                    chars.putback_maybe(other);
                    false
                }
            },
            other => {
                chars.putback_maybe(other);
                false
            }
        },
        TextModifier::Quote => match chars.next() {
            Some('\n') => match chars.next() {
                Some('>') => match chars.next() {
                    Some(' ') => {
                        chars.putback('\n');
                        false
                    }
                    other => {
                        chars.putback_maybe(other);
                        chars.putback('>');
                        chars.putback('\n');
                        true
                    }
                },
                other => {
                    chars.putback_maybe(other);
                    chars.putback('\n');
                    true
                }
            },
            other => {
                chars.putback_maybe(other);
                false
            }
        },
    }
}

fn find_open_delimiter(chars: &mut PutBackChars) -> Option<Delimiter> {
    match chars.next()? {
        '*' => match chars.next() {
            Some('*') => Some(Delimiter::TextModifier(TextModifier::Bold)),
            other => {
                chars.putback_maybe(other);
                Some(Delimiter::TextModifier(TextModifier::Italics))
            }
        },
        '~' => match chars.next() {
            Some('~') => Some(Delimiter::TextModifier(TextModifier::Strikethrough)),
            other => {
                chars.putback_maybe(other);
                chars.putback('~');
                None
            }
        },
        '_' => match chars.next() {
            Some('_') => Some(Delimiter::TextModifier(TextModifier::Underline)),
            other => {
                chars.putback_maybe(other);
                chars.putback('_');
                None
            }
        },
        '\n' => match chars.next() {
            Some('\n') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Paragraph)),
            Some('`') => match chars.next() {
                Some('`') => match chars.next() {
                    Some('`') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::CodeBlock)),
                    other => {
                        chars.putback_maybe(other);
                        chars.putback('`');
                        chars.putback('`');
                        chars.putback('\n');
                        None
                    }
                },
                other => {
                    chars.putback_maybe(other);
                    chars.putback('`');
                    chars.putback('\n');
                    None
                }
            },
            Some('>') => match chars.next() {
                Some(' ') => Some(Delimiter::TextModifier(TextModifier::Quote)),
                other => {
                    chars.putback_maybe(other);
                    chars.putback('>');
                    chars.putback('\n');
                    None
                }
            },
            Some('#') => {
                let mut header_level: u8 = 1;
                loop {
                    match chars.next() {
                        Some('#') => header_level += 1,
                        other => {
                            chars.putback_maybe(other);
                            break;
                        }
                    }
                }
                Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Heading(
                    header_level,
                )))
            }
            other => {
                chars.putback_maybe(other);
                chars.putback('\n');
                None
            }
        },
        '\\' => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Escape)),
        '!' => match chars.next() {
            Some('[') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Image)),
            other => {
                chars.putback_maybe(other);
                chars.putback('!');
                None
            }
        },
        '`' => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::InlineCode)),
        '[' => match chars.next() {
            Some('[') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::WikiLink)),
            other => {
                chars.putback_maybe(other);
                Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Link))
            }
        },
        '{' => match chars.next() {
            Some('{') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Template)),
            other => {
                chars.putback_maybe(other);
                chars.putback('{');
                None
            }
        },
        '}' => match chars.next() {
            Some('}') => Some(Delimiter::ExclusiveModifier(
                ExclusiveModifier::EndOfTemplate,
            )),
            other => {
                chars.putback_maybe(other);
                chars.putback('}');
                None
            }
        },
        '|' => Some(Delimiter::ExclusiveModifier(
            ExclusiveModifier::EndOfArgument,
        )),
        other => {
            chars.putback(other);
            None
        }
    }
}

impl TextModifier {
    pub(crate) fn close(self) -> &'static str {
        match self {
            TextModifier::Bold => "</b>",
            TextModifier::Italics => "</i>",
            TextModifier::Strikethrough => "</del>",
            TextModifier::Underline => "</u>",
            TextModifier::Quote => "</blockquote>",
        }
    }
    pub(crate) fn open(self) -> &'static str {
        match self {
            TextModifier::Bold => "<b>",
            TextModifier::Italics => "<i>",
            TextModifier::Strikethrough => "<del>",
            TextModifier::Underline => "<u>",
            TextModifier::Quote => "<blockquote>",
        }
    }
}

impl ExclusiveModifier {
    pub(crate) fn to_html(
        self,
        chars: &mut PutBackChars,
        parsed: &mut ParsedHTML,
    ) -> Option<ParseError> {
        match self {
            ExclusiveModifier::Escape => {
                if let Some(character) = chars.next() {
                    parsed.html.push(character);
                    None
                } else {
                    Some(ParseError::from_str(
                        chars,
                        "File may not end with an escape character",
                    ))
                }
            }
            ExclusiveModifier::Template => {
                let mut name = String::new();
                let name_exit;
                loop {
                    match chars.next() {
                        Some('|') => {
                            name_exit = ExitMode::EndOfArgument;
                            break;
                        }
                        Some('}') => {
                            if chars.next() != Some('}') {
                                return Some(ParseError::from_str(
                                    chars,
                                    "Incorrectly closed template",
                                ));
                            }
                            name_exit = ExitMode::EndOfTemplate;
                            break;
                        }
                        Some(character) => {
                            name.push(character);
                        }
                        None => {
                            return Some(ParseError::from_str(
                                chars,
                                "Unexpected file ending inside template",
                            ))
                        }
                    };
                }
                let mut args = Vec::new();
                if name_exit == ExitMode::EndOfArgument {
                    loop {
                        match markdown_charbuff_to_html(chars) {
                            Ok((result, reason)) => {
                                args.push(result);
                                match reason {
                                    ExitMode::EndOfArgument => continue,
                                    ExitMode::EndOfTemplate => break,
                                    ExitMode::EndOfFile => {
                                        return Some(ParseError::from_str(
                                            chars,
                                            "Unexpected file ending inside template",
                                        ))
                                    }
                                }
                            }
                            Err(e) => return Some(e),
                        }
                    }
                }
                let mut result: ParsedHTML = match parse_template(name.clone(), args) {
                    Ok((result, ExitMode::EndOfFile)) => result,
                    Ok(_) => return Some(ParseError::from_str(chars, "If you ever get this error, please send a bug report. I'm very curious how you can get this")),
                    Err(e) => return Some(ParseError::from_string(chars, format!("Error occurred while parsing template {name}:\n{}", e.comment))),
                };
                parsed.html.push_str(&result.html);
                parsed.links_to.append(&mut result.links_to);
                parsed.parents.append(&mut result.parents);
                None
            }
            ExclusiveModifier::WikiLink => todo!(),
            ExclusiveModifier::InlineCode => todo!(),
            ExclusiveModifier::CodeBlock => todo!(),
            ExclusiveModifier::Paragraph => {
                parsed.html.push_str("<br>\n"); // TODO: do these properly
                None
            }
            ExclusiveModifier::Heading(level) => {
                let mut raw_header = String::new();
                loop {
                    match chars.next() {
                        Some('\n') => {
                            chars.putback('\n');
                            break;
                        }
                        Some(other) => raw_header.push(other),
                        None => break,
                    }
                }
                let mut parsed_header =
                    match markdown_charbuff_to_html(&mut raw_header.chars().into()) {
                        Ok((parsed, ExitMode::EndOfFile)) => parsed,
                        Ok((_parsed, _other)) => {
                            return Some(ParseError::from_str(
                                chars,
                                "Stray argument or template ending in heading.",
                            ))
                        }
                        Err(e) => return Some(e),
                    };
                parsed
                    .html
                    .push_str(format!("<h{level}>{}</h{level}>", parsed_header.html).as_str());
                parsed.links_to.append(&mut parsed_header.links_to);
                parsed.parents.append(&mut parsed_header.parents);
                None
            }
            ExclusiveModifier::Link => todo!(),
            ExclusiveModifier::Image => todo!(),
            ExclusiveModifier::EndOfArgument => todo!(),
            ExclusiveModifier::EndOfTemplate => todo!(),
        }
    }
}

fn parse_template(
    name: String,
    args: Vec<ParsedHTML>,
) -> Result<(ParsedHTML, ExitMode), ParseError> {
    todo!()
}
