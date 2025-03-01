use confoosion_markdown_parser::{markdown_to_html, ParsedHTML};

fn main() {
    let html = markdown_to_html("examples/stars.md");
    println!("{}", html.html);
}
