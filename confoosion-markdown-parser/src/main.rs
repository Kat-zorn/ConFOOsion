use confoosion_markdown_parser::markdown_file_to_html;

fn main() {
    let parsed = markdown_file_to_html("examples/stars.md").unwrap();
    println!("{}", parsed.html);
}
