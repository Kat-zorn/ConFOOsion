# ConFOOsion

> Personal wiki software

## License

See the LICENSE file. In summary (non-binding)

- Bugs can happen. If ConFOOsion deletes your files, it's not my problem.
- Use ConFOOsion for taking notes, not running DOOM or breaking the law.
- If you fork, make it open-source, or at least available to me.
- If you fork or use this as a dependency, give credit.
- If you contribute, you give me the rights to your code, but you get your name on the list.
- I will distribute the source code together with the binaries.

## Modules

### Markdown parser

A Rust-based Markdown parser custom-built for the purposes of ConFOOsion.
This means that it supports ConFOOsion's _wiki-links_ and _templates_.

Status: Not even started.

### Core module

The Rust-based core that tracks relations, deals with renaming, splitting, and merging of articles and sections, and generally does all the writing to disk.

Status: Not even started.

### Frontend

Allows you to view and edit the Markdown files in both text and HTML, navigate them using the links, and interact with the core module in a graphical way.

Status: Not even started.

## Vision

ConFOOsion does not have a rigid hierarchy (no folders). As a stretch goal, ConFOOsions might implement user/plugin-defined templates and visualizations.
