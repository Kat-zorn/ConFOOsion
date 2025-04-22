use std::{cell::RefCell, collections::HashMap, path::PathBuf};

use crate::{error::ParseError, putback::PutBackChars, ExitMode, ParsedHTML};

pub type Template =
    dyn Fn(Vec<String>, &TemplateMap, PathBuf) -> Result<(ParsedHTML, ExitMode), ParseError>;
pub struct TemplateMap {
    pub map: HashMap<String, Box<Template>>,
    recursion_depth: RefCell<u16>,
}

impl TemplateMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            recursion_depth: RefCell::new(0),
        }
    }
    pub fn insert(&mut self, name: String, function: Box<Template>) -> bool {
        self.map.insert(name, function).is_none()
    }
    pub fn call(
        &self,
        name: String,
        args: Vec<String>,
        dir: PathBuf,
    ) -> Result<(ParsedHTML, ExitMode), ParseError> {
        if *self.recursion_depth.borrow() >= Self::max_recursion_depth() {
            return Err(ParseError::empty(
                "Maximum template recursion depth exceeded.",
            ));
        }
        *self.recursion_depth.borrow_mut() += 1;
        let out = match self.map.get(&name) {
            Some(callback) => callback(args, self, dir),
            None => Err(ParseError::empty(
                format!("Template {{{{{}}}}} not found", name).as_str(),
            )),
        };
        *self.recursion_depth.borrow_mut() -= 1;
        out
    }
    pub fn max_recursion_depth() -> u16 {
        128
    }
}

impl Default for TemplateMap {
    fn default() -> Self {
        Self::new()
    }
}

pub fn read_template_argument(chars: &mut PutBackChars) -> (String, ExitMode) {
    enum NestKind {
        WikiLink,
        Template,
    }
    let mut arg = String::new();
    let mut stack: Vec<NestKind> = Vec::new();

    while let Some(character) = chars.next() {
        if character == '\\' {
            arg.push(character);
            arg.push(
                chars
                    .next()
                    .expect("Stray escape character and EndOfFile inside template or wiki-link."),
            );
            continue;
        }
        let _ = match stack.last() {
            Some(NestKind::Template) => {
                if character == '}' {
                    if let Some(character) = chars.next() {
                        if character == '}' {
                            arg.push_str("}}");
                            stack.pop().unwrap();
                            continue;
                        } else {
                            chars.putback(character);
                            chars.putback('}');
                        }
                    } else {
                        chars.putback('}');
                    }
                } else {
                }
            }
            Some(NestKind::WikiLink) => {
                if character == ']' {
                    if let Some(character) = chars.next() {
                        if character == ']' {
                            arg.push_str("]]");
                            stack.pop().unwrap();
                            continue;
                        } else {
                            chars.putback(character);
                            chars.putback(']');
                        }
                    } else {
                        chars.putback(']');
                    }
                } else {
                }
            }
            None => match character {
                '|' => return (arg, ExitMode::EndOfArgument),
                ']' => {
                    assert_eq!(
                        chars.next(),
                        Some(']'),
                        "Lone “]” inside link or template at {}:{}",
                        chars.line_number,
                        chars.column_number
                    );
                    return (arg, ExitMode::EndOfLink);
                }
                '}' => {
                    assert_eq!(
                        chars.next(),
                        Some('}'),
                        "Lone “}}” inside link or template at {}:{}",
                        chars.line_number,
                        chars.column_number
                    );
                    return (arg, ExitMode::EndOfTemplate);
                }
                _ => (),
            },
        };

        match character {
            '{' => match chars.next() {
                Some('{') => {
                    stack.push(NestKind::Template);
                    arg.push('{');
                    arg.push('{');
                }
                other => {
                    arg.push('{');
                    chars.putback_maybe(other)
                }
            },
            '[' => match chars.next() {
                Some('[') => {
                    stack.push(NestKind::WikiLink);
                    arg.push('[');
                    arg.push('[');
                }
                other => {
                    arg.push('[');
                    chars.putback_maybe(other)
                }
            },
            other => arg.push(other),
        }
    }
    (arg, ExitMode::EndOfFile)
}
