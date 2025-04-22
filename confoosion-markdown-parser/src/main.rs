use core::panic;
use std::path::Path;

use confoosion_markdown_parser::error::ParseError;
use confoosion_markdown_parser::putback::PutBackChars;
use confoosion_markdown_parser::template::TemplateMap;
use confoosion_markdown_parser::{markdown_charbuff_to_html, markdown_file_to_html};

fn main() {
    let mut templates: TemplateMap = TemplateMap::new();
    templates.insert("double".to_string(), Box::new(template_double));
    let parsed = match markdown_file_to_html("examples/stars.md", &mut templates) {
        Ok(x) => x,
        Err(e) => panic!("{e}"),
    };
    println!("{}", parsed.html);
}

fn template_double<P: AsRef<Path>>(
    args: Vec<String>,
    templates: &TemplateMap,
    dir: P,
) -> Result<
    (
        confoosion_markdown_parser::ParsedHTML,
        confoosion_markdown_parser::ExitMode,
    ),
    ParseError,
> {
    if args.len() == 1 {
        let mut chars: PutBackChars = args.first().unwrap().chars().into();
        let (mut parsed, exit) = markdown_charbuff_to_html(&mut chars, templates, dir)?;
        parsed.html.push_str(parsed.html.clone().as_str());
        Ok((parsed, exit))
    } else {
        Err(ParseError::empty(
            format!(
                "{{{{double}}}} only accepts one argument, not {}",
                args.len()
            )
            .as_str(),
        ))
    }
}
