pub mod error;
pub mod putback;
pub mod template;

use error::ParseError;
use putback::PutBackChars;
use std::path::{Path, PathBuf};
use template::{read_template_argument, TemplateMap};

#[derive(Debug)]
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
    Heading(u8),
}
#[derive(Debug, Clone, Copy)]
enum ExclusiveModifier {
    Escape,
    Template,
    WikiLink,
    InlineCode,
    CodeBlock,
    Paragraph,
    Link,
    Image,
    EndOfArgument,
    EndOfTemplate,
    EndOfLink,
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
    EndOfLink,
    EndOfFile,
}

pub fn markdown_file_to_html<T>(
    file: T,
    templates: &mut TemplateMap,
) -> Result<ParsedHTML, ParseError>
where
    T: AsRef<Path>,
{
    let contents = match std::fs::read_to_string(&file) {
        Ok(x) => x,
        Err(e) => {
            let msg = match e.into_inner() {
                Some(x) => x.to_string(),
                None => "<No error specified>".to_string(),
            };
            return Err(ParseError::empty(
                format!("Could not read file, error: {}", msg).as_str(),
            ));
        }
    };
    let dir = match file.as_ref().parent() {
        Some(x) => x,
        None => {
            return Err(ParseError::empty(
                format!("File {} has no parent directory", file.as_ref().display()).as_str(),
            ))
        }
    };
    let mut chars: PutBackChars = contents.chars().into();
    chars.putback('\n');
    chars.line_number = 1;
    return match markdown_charbuff_to_html(&mut chars, templates, dir)? {
        (parsed, ExitMode::EndOfFile) => Ok(parsed),
        (_parsed, ExitMode::EndOfArgument) => Err(ParseError::empty("Stray argument separator")),
        (_parsed, ExitMode::EndOfTemplate) => Err(ParseError::empty("Stray template terminator")),
        (_parsed, ExitMode::EndOfLink) => Err(ParseError::empty("Stray wiki-link terminator")),
    };
}

pub fn markdown_charbuff_to_html<P: AsRef<Path>>(
    chars: &mut PutBackChars,
    templates: &TemplateMap,
    directory: P,
) -> Result<(ParsedHTML, ExitMode), ParseError> {
    let mut parsed_html = ParsedHTML {
        html: String::new(),
        links_to: Vec::new(),
        parents: Vec::new(),
    };

    let mut modifier_stack = Vec::new();

    parsed_html.html.push_str("<p>");
    while let Some(character) = chars.next() {
        chars.putback(character);
        if let Some(&open_delimiter) = modifier_stack.last() {
            if has_close_delimiter(chars, open_delimiter) {
                let _ = modifier_stack.pop().unwrap();
                parsed_html.html.push_str(open_delimiter.close().as_str());
                continue;
            }
        }
        if let Some(delimiter) = find_open_delimiter(chars) {
            match delimiter {
                Delimiter::TextModifier(text_modifier) => {
                    modifier_stack.push(text_modifier);
                    parsed_html.html.push_str(text_modifier.open().as_str());
                }
                Delimiter::ExclusiveModifier(ExclusiveModifier::EndOfArgument) => {
                    return Ok((parsed_html, ExitMode::EndOfArgument));
                }
                Delimiter::ExclusiveModifier(ExclusiveModifier::EndOfTemplate) => {
                    return Ok((parsed_html, ExitMode::EndOfTemplate));
                }
                Delimiter::ExclusiveModifier(exclusive_modifier) => {
                    match exclusive_modifier.to_html(
                        chars,
                        &mut parsed_html,
                        &templates,
                        directory.as_ref().to_path_buf(),
                    ) {
                        Some(e) => return Err(e),
                        None => (),
                    }
                }
            }
        } else {
            parsed_html.html.push(chars.next().unwrap());
        }
    }
    parsed_html.html.push_str("</p>");
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
            Some('*') => match chars.next() {
                Some('*') => {
                    chars.putback('*');
                    chars.putback('*');
                    false
                }
                other => {
                    chars.putback_maybe(other);
                    true
                }
            },
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
        TextModifier::Heading(_) => match chars.next() {
            Some('\n') => true,
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
            Some('\n') => {
                chars.putback('\n');
                Some(Delimiter::ExclusiveModifier(ExclusiveModifier::Paragraph))
            }
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
                Some(Delimiter::TextModifier(TextModifier::Heading(header_level)))
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
        ']' => match chars.next() {
            Some(']') => Some(Delimiter::ExclusiveModifier(ExclusiveModifier::EndOfLink)),
            other => {
                chars.putback_maybe(other);
                chars.putback(']');
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
    pub(crate) fn close(self) -> String {
        match self {
            TextModifier::Bold => "</b>".to_string(),
            TextModifier::Italics => "</i>".to_string(),
            TextModifier::Strikethrough => "</del>".to_string(),
            TextModifier::Underline => "</u>".to_string(),
            TextModifier::Quote => "</blockquote>".to_string(),
            TextModifier::Heading(level) => format!("</h{level}>\n<p>"),
        }
    }
    pub(crate) fn open(self) -> String {
        match self {
            TextModifier::Bold => "<b>".to_string(),
            TextModifier::Italics => "<i>".to_string(),
            TextModifier::Strikethrough => "<del>".to_string(),
            TextModifier::Underline => "<u>".to_string(),
            TextModifier::Quote => "<blockquote>".to_string(),
            TextModifier::Heading(level) => format!("</p>\n<h{level}>"),
        }
    }
}

impl ExclusiveModifier {
    pub(crate) fn to_html(
        self,
        chars: &mut PutBackChars,
        parsed: &mut ParsedHTML,
        templates: &TemplateMap,
        directory: PathBuf,
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
                let (name, name_exit) = read_template_argument(chars);
                let mut args = Vec::new();
                // let _ = read_template_argument(chars);
                if name_exit == ExitMode::EndOfArgument {
                    loop {
                        match read_template_argument(chars) {
                            (result, reason) => {
                                args.push(result);
                                match reason {
                                    ExitMode::EndOfArgument => continue,
                                    ExitMode::EndOfTemplate => break,
                                    ExitMode::EndOfFile => {
                                        panic!("End of file inside template argument")
                                    }
                                    ExitMode::EndOfLink => panic!(
                                        "Stray wiki-link terminator inside template argument"
                                    ),
                                }
                            }
                        }
                    }
                }
                let mut result: ParsedHTML = match templates.call(name.clone(), args, directory) {
                    Ok((result, ExitMode::EndOfFile)) => result,
                    Ok(_) => return Some(ParseError::from_str(chars, "If you ever get this error, please send a bug report. I'm very curious how you can get this")),
                    Err(e) => return Some(ParseError::from_string(chars, format!("Error occurred while parsing template {name}:\n{}", e.comment))),
                };
                parsed.html.push_str(&result.html);
                parsed.links_to.append(&mut result.links_to);
                parsed.parents.append(&mut result.parents);
                None
            }
            ExclusiveModifier::WikiLink => {
                let (name, reason) = read_template_argument(chars);
                let full_name = format!("{name}.md");
                let absolute_path = directory.join(&full_name);

                let display_name = if reason == ExitMode::EndOfArgument {
                    let (out, reason) = read_template_argument(chars);
                    match reason {
                        ExitMode::EndOfArgument => {
                            return Some(ParseError::from_str(
                                chars,
                                "Cannot supply more than two arguments to a wikilink.",
                            ))
                        }
                        ExitMode::EndOfTemplate => {
                            return Some(ParseError::from_str(
                                chars,
                                "Cannot close template inside wikilink.",
                            ))
                        }
                        ExitMode::EndOfLink => out,
                        ExitMode::EndOfFile => {
                            return Some(ParseError::from_str(chars, "Unclosed wikilink."))
                        }
                    }
                } else {
                    match read_title(&absolute_path) {
                        Some(title) => title,
                        None => full_name.clone(),
                    }
                };
                let path_name = match absolute_path.to_str() {
                    Some(x) => x,
                    None => return Some(ParseError::from_str(chars, "Path could not be resolved")),
                };
                parsed
                    .html
                    .push_str(format!("<a href={path_name}>{display_name}</a>").as_str());
                None
            }
            ExclusiveModifier::InlineCode => {
                parsed.html.push_str("<code>");
                while let Some(character) = chars.next() {
                    match character {
                        '\\' => {
                            let next = chars.next();
                            match next {
                                None => {
                                    return Some(ParseError::from_str(
                                        chars,
                                        "File may not end with escape character.",
                                    ))
                                }
                                Some('`') => parsed.html.push('`'),
                                Some(other) => {
                                    parsed.html.push('\\');
                                    parsed.html.push(other);
                                }
                            }
                        }
                        '`' => break,
                        other => parsed.html.push(other),
                    }
                }
                parsed.html.push_str("</code>");
                None
            }
            ExclusiveModifier::CodeBlock => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        None => {
                            return Some(ParseError::from_str(
                                chars,
                                "Cannot end file inside codeblock",
                            ))
                        }
                        Some('\n') => break,
                        Some(other) => name.push(other),
                    }
                }
                parsed
                    .html
                    .push_str(format!("<pre><code class={name}>").as_str());
                while let Some(character) = chars.next() {
                    match character {
                        '\n' => match chars.next() {
                            None => {
                                return Some(ParseError::from_str(
                                    chars,
                                    "Cannot end file inside codeblock",
                                ))
                            }
                            Some('`') => match chars.next() {
                                None => {
                                    return Some(ParseError::from_str(
                                        chars,
                                        "Cannot end file inside codeblock",
                                    ))
                                }
                                Some('`') => match chars.next() {
                                    None => {
                                        return Some(ParseError::from_str(
                                            chars,
                                            "Cannot end file inside codeblock",
                                        ))
                                    }
                                    Some('`') => match chars.next() {
                                        None => {
                                            return Some(ParseError::from_str(
                                                chars,
                                                "Cannot end file inside codeblock",
                                            ))
                                        }
                                        Some('\n') => break,
                                        Some(other) => {
                                            parsed.html.push_str("\n```");
                                            parsed.html.push(other);
                                        }
                                    },
                                    Some(other) => {
                                        parsed.html.push_str("\n``");
                                        parsed.html.push(other);
                                    }
                                },
                                Some(other) => {
                                    parsed.html.push_str("\n`");
                                    parsed.html.push(other);
                                }
                            },
                            Some(other) => {
                                parsed.html.push('\n');
                                parsed.html.push(other);
                            }
                        },
                        other => parsed.html.push(other),
                    }
                }
                parsed.html.push_str("</code></pre>");
                None
            }
            ExclusiveModifier::Paragraph => {
                parsed.html.push_str("</pre>\n<p>");
                None
            }
            ExclusiveModifier::Link => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(character) => name.push(character),
                        None => return Some(ParseError::from_str(chars, "Unterminated link name")),
                    };
                }
                match chars.next() {
                    Some('(') => (),
                    other => {
                        chars.putback_maybe(other);
                        return None;
                    }
                }
                let mut url = String::new();
                loop {
                    match chars.next() {
                        Some(')') => break,
                        Some(character) => url.push(character),
                        None => return Some(ParseError::from_str(chars, "Unterminated link body")),
                    }
                }
                parsed
                    .html
                    .push_str(format!("<a href={url}>{name}</a>\n").as_str());
                None
            }
            ExclusiveModifier::Image => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(character) => name.push(character),
                        None => return Some(ParseError::from_str(chars, "Unterminated link name")),
                    };
                }
                match chars.next() {
                    Some('(') => (),
                    other => {
                        chars.putback_maybe(other);
                        return None;
                    }
                }
                let mut url = String::new();
                loop {
                    match chars.next() {
                        Some(')') => break,
                        Some(character) => url.push(character),
                        None => return Some(ParseError::from_str(chars, "Unterminated link body")),
                    }
                }
                parsed
                    .html
                    .push_str(format!("<img src=\"{url}\" alt=\"{name}\"/>\n").as_str());
                None
            }
            ExclusiveModifier::EndOfArgument => panic!("Unreachable state"),
            ExclusiveModifier::EndOfTemplate => panic!("Unreachable state"),
            ExclusiveModifier::EndOfLink => panic!("Unreachable state"),
        }
    }
}

fn read_title<T>(file: T) -> Option<String>
where
    T: AsRef<Path>,
{
    let contents = match std::fs::read_to_string(&file) {
        Ok(x) => x,
        Err(_) => {
            return None;
        }
    };
    let mut chars: PutBackChars = contents.chars().into();
    if chars.next() != Some('#') {
        return None;
    }
    let mut out_unparsed = String::new();
    loop {
        match chars.next() {
            None => return None,
            Some('#') => return None,
            Some('\\') => {
                out_unparsed.push('\\');
                out_unparsed.push(chars.next()?);
            }
            Some('\n') => break,
            Some(other) => out_unparsed.push(other),
        }
    }
    let mut out_chars: PutBackChars = out_unparsed.chars().into();
    let result =
        markdown_charbuff_to_html(&mut out_chars, &TemplateMap::new(), file.as_ref().parent()?);
    match result {
        Err(_) => None,
        Ok(x) => Some(x.0.html),
    }
}
