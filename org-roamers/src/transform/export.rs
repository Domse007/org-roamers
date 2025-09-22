use std::cmp::min;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use orgize::rowan::ast::AstNode;
use orgize::{
    export::{Container, Event, HtmlEscape, TraversalContext, Traverser},
    rowan::NodeOrToken,
    SyntaxKind,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct EnvAdvice {
    on: String,
    header: String,
    css_style: String,
    text_styling: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HtmlExportSettings {
    pub env_advices: Vec<EnvAdvice>,
}

impl HtmlExportSettings {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        serde_json::from_str(fs::read_to_string(path)?.as_str()).map_err(Into::into)
    }
}

/// This is needed because if we have the table
///
/// ```org
/// | / | <> |
/// |   | b  |
/// ```
///
/// we only want the second column in the final html output.
/// (while also skipping the first line)
#[derive(Default)]
struct OrgTableHints {
    /// Flag that the current table has a formating row
    has_formating: bool,
    /// Flag that is set at row start to check if a cell is the first in the row.
    next_is_first: bool,
}

pub struct HtmlExport<'a> {
    settings: &'a HtmlExportSettings,
    output: String,
    table_row: TableRow,
    in_descriptive_list: Vec<bool>,
    in_special_block: bool,
    outgoing_id_links: Vec<String>,
    file: String,
    latex_blocks: Vec<String>,
    latex_counter: usize,
    table_hints: OrgTableHints,
}

impl<'a> HtmlExport<'a> {
    pub fn new(settings: &'a HtmlExportSettings, file: String) -> Self {
        Self {
            settings,
            output: String::with_capacity(1000),
            table_row: TableRow::default(),
            in_descriptive_list: vec![],
            in_special_block: false,
            outgoing_id_links: vec![],
            file,
            latex_blocks: vec![],
            latex_counter: 0,
            table_hints: OrgTableHints::default(),
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum TableRow {
    #[default]
    HeaderRule,
    Header,
    BodyRule,
    Body,
}

impl HtmlExport<'_> {
    pub fn finish(self) -> (String, Vec<String>, Vec<String>) {
        let mut outgoing = self.outgoing_id_links;
        outgoing.sort();
        outgoing.dedup();
        (self.output, outgoing, self.latex_blocks)
    }
}

impl Traverser for HtmlExport<'_> {
    fn event(&mut self, event: Event, ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Document(document)) => {
                self.output += "<div>";
                if let Some(title) = document.title() {
                    let _ = write!(
                        &mut self.output,
                        r#"<h1 id="org-preview-title">{}</h1>"#,
                        title
                    );
                }
            }
            Event::Leave(Container::Document(_)) => self.output += "</div>",

            Event::Enter(Container::Headline(headline)) => {
                let level = min(headline.level(), 6);
                let _ = write!(&mut self.output, "<h{level}>");
                for elem in headline.title() {
                    self.element(elem, ctx);
                }
                let _ = write!(&mut self.output, "</h{level}>");
            }
            Event::Leave(Container::Headline(_)) => {}

            Event::Enter(Container::SpecialBlock(specialblock)) => {
                let mut iter = specialblock
                    .syntax()
                    .first_child()
                    .unwrap()
                    .children_with_tokens();
                match iter.nth(1).map(|token| token.to_string()) {
                    Some(block_type) => {
                        let advice = self
                            .settings
                            .env_advices
                            .iter()
                            .find(|e| e.on.to_lowercase() == block_type);
                        match advice {
                            Some(advice) => {
                                let _ = write!(
                                    self.output,
                                    "<div class=\"{}\" style=\"{}\">{}<p style=\"{}\">",
                                    advice.on, advice.css_style, advice.header, advice.text_styling
                                );
                            }
                            None => {
                                let _ = write!(self.output, "<div class=\"{}\"><p>", block_type);
                            }
                        }
                    }
                    None => {
                        tracing::warn!("Block type not found.");
                        self.output += "<div><p>";
                    }
                }
                self.in_special_block = true;
            }
            Event::Leave(Container::SpecialBlock(_)) => {
                self.in_special_block = false;
                self.output += "</p></div>";
            }

            Event::Enter(Container::Paragraph(_)) => {
                if !self.in_special_block {
                    self.output += "<p>"
                }
            }
            Event::Leave(Container::Paragraph(_)) => {
                if !self.in_special_block {
                    self.output += "</p>";
                }
            }

            Event::Enter(Container::Section(_)) => self.output += "<section>",
            Event::Leave(Container::Section(_)) => self.output += "</section>",

            Event::Enter(Container::Italic(_)) => self.output += "<i>",
            Event::Leave(Container::Italic(_)) => self.output += "</i>",

            Event::Enter(Container::Bold(_)) => self.output += "<b>",
            Event::Leave(Container::Bold(_)) => self.output += "</b>",

            Event::Enter(Container::Strike(_)) => self.output += "<s>",
            Event::Leave(Container::Strike(_)) => self.output += "</s>",

            Event::Enter(Container::Underline(_)) => self.output += "<u>",
            Event::Leave(Container::Underline(_)) => self.output += "</u>",

            Event::Enter(Container::Verbatim(_)) => self.output += "<code>",
            Event::Leave(Container::Verbatim(_)) => self.output += "</code>",

            Event::Enter(Container::Code(_)) => self.output += "<code>",
            Event::Leave(Container::Code(_)) => self.output += "</code>",

            Event::Enter(Container::SourceBlock(block)) => {
                if let Some(language) = block.language() {
                    let _ = write!(
                        &mut self.output,
                        r#"<pre><code class="language-{}">"#,
                        HtmlEscape(&language)
                    );
                } else {
                    self.output += r#"<pre><code>"#
                }
            }
            Event::Leave(Container::SourceBlock(_)) => self.output += "</code></pre>",

            Event::Enter(Container::QuoteBlock(_)) => self.output += "<blockquote class=\"quote\">",
            Event::Leave(Container::QuoteBlock(_)) => self.output += "</blockquote>",

            Event::Enter(Container::VerseBlock(_)) => self.output += "<p class=\"verse\">",
            Event::Leave(Container::VerseBlock(_)) => self.output += "</p>",

            Event::Enter(Container::ExampleBlock(_)) => self.output += "<pre class=\"example\">",
            Event::Leave(Container::ExampleBlock(_)) => self.output += "</pre>",

            Event::Enter(Container::FixedWidth(_)) => {
                self.output += "<pre class=\"program-output\">"
            }
            Event::Leave(Container::FixedWidth(_)) => self.output += "</pre>",

            Event::Enter(Container::CenterBlock(_)) => self.output += "<div class=\"center\">",
            Event::Leave(Container::CenterBlock(_)) => self.output += "</div>",

            Event::Enter(Container::CommentBlock(_)) => self.output += "<!--",
            Event::Leave(Container::CommentBlock(_)) => self.output += "-->",

            Event::Enter(Container::Comment(_)) => self.output += "<!--",
            Event::Leave(Container::Comment(_)) => self.output += "-->",

            Event::Enter(Container::Subscript(_)) => self.output += "<sub>",
            Event::Leave(Container::Subscript(_)) => self.output += "</sub>",

            Event::Enter(Container::Superscript(_)) => self.output += "<sup>",
            Event::Leave(Container::Superscript(_)) => self.output += "</sup>",

            Event::Enter(Container::List(list)) => {
                self.output += if list.is_ordered() {
                    self.in_descriptive_list.push(false);
                    "<ol>"
                } else if list.is_descriptive() {
                    self.in_descriptive_list.push(true);
                    "<dl>"
                } else {
                    self.in_descriptive_list.push(false);
                    "<ul>"
                };
            }
            Event::Leave(Container::List(list)) => {
                self.output += if list.is_ordered() {
                    "</ol>"
                } else if let Some(true) = self.in_descriptive_list.last() {
                    "</dl>"
                } else {
                    "</ul>"
                };
                self.in_descriptive_list.pop();
            }
            Event::Enter(Container::ListItem(list_item)) => {
                if let Some(&true) = self.in_descriptive_list.last() {
                    self.output += "<dt>";
                    for elem in list_item.tag() {
                        self.element(elem, ctx);
                    }
                    self.output += "</dt><dd>";
                } else {
                    self.output += "<li>";
                }
            }
            Event::Leave(Container::ListItem(_)) => {
                if let Some(&true) = self.in_descriptive_list.last() {
                    self.output += "</dd>";
                } else {
                    self.output += "</li>";
                }
            }

            Event::Enter(Container::OrgTable(table)) => {
                self.output += "<table>";
                self.table_row = if table.has_header() {
                    TableRow::HeaderRule
                } else {
                    TableRow::BodyRule
                }
            }
            Event::Leave(Container::OrgTable(_)) => {
                match self.table_row {
                    TableRow::Body => self.output += "</tbody>",
                    TableRow::Header => self.output += "</thead>",
                    _ => {}
                }
                self.output += "</table>";
            }
            Event::Enter(Container::OrgTableRow(row)) => {
                if let Some(child) = row.syntax().first_child() {
                    if child.text().to_string().trim() == "/" {
                        self.table_hints.has_formating = true;
                        ctx.skip();
                        return;
                    }
                }
                if row.is_rule() {
                    match self.table_row {
                        TableRow::Body => {
                            self.output += "</tbody>";
                            self.table_row = TableRow::BodyRule;
                        }
                        TableRow::Header => {
                            self.output += "</thead>";
                            self.table_row = TableRow::BodyRule;
                        }
                        _ => {}
                    }
                    ctx.skip();
                } else {
                    match self.table_row {
                        TableRow::HeaderRule => {
                            self.table_row = TableRow::Header;
                            self.output += "<thead>";
                        }
                        TableRow::BodyRule => {
                            self.table_row = TableRow::Body;
                            self.output += "<tbody>";
                        }
                        _ => {}
                    }
                    self.output += "<tr>";
                }
                self.table_hints.next_is_first = true;
            }
            Event::Leave(Container::OrgTableRow(row)) => {
                if row.is_rule() {
                    match self.table_row {
                        TableRow::Body => {
                            self.output += "</tbody>";
                            self.table_row = TableRow::BodyRule;
                        }
                        TableRow::Header => {
                            self.output += "</thead>";
                            self.table_row = TableRow::BodyRule;
                        }
                        _ => {}
                    }
                    ctx.skip();
                } else {
                    self.output += "</tr>";
                }
            }
            Event::Enter(Container::OrgTableCell(_)) => {
                if self.table_hints.next_is_first && self.table_hints.has_formating {
                    self.table_hints.next_is_first = false;
                    ctx.skip();
                } else {
                    self.output += "<td>"
                }
            }
            Event::Leave(Container::OrgTableCell(_)) => self.output += "</td>",

            Event::Enter(Container::Link(link)) => {
                let path = link.path();
                let path = path.trim_start_matches("file:");

                if link.path().starts_with("id:") {
                    let id = link.path().trim_start_matches("id:").to_string();
                    let _ = write!(
                        &mut self.output,
                        r#"<a id="{}" class="org-preview-id-link">"#,
                        HtmlEscape(&id),
                    );
                    self.outgoing_id_links.push(id);
                } else {
                    let _ = write!(&mut self.output, r#"<a href="{}">"#, HtmlEscape(&path));
                }

                if link.is_image() {
                    let mut path = PathBuf::from(self.file.clone());
                    path.pop();
                    path.push(link.path().as_ref());
                    let _ = write!(
                        &mut self.output,
                        r#"<img style="width: 80%; margin: auto; display: block;" src="assets?file={}">"#,
                        HtmlEscape(&path.to_str().unwrap())
                    );
                    // return ctx.skip();
                }

                if !link.has_description() {
                    let _ = write!(&mut self.output, "</a>");
                    ctx.skip();
                }
            }
            Event::Leave(Container::Link(_)) => self.output += "</a>",

            Event::Text(text) => {
                let _ = write!(&mut self.output, "{}", HtmlEscape(text));
            }

            Event::LineBreak(_) => self.output += "<br/>",

            Event::Snippet(snippet) => {
                if snippet.backend().eq_ignore_ascii_case("html") {
                    self.output += &snippet.value();
                }
            }

            Event::Rule(_) => self.output += "<hr/>",

            Event::Timestamp(timestamp) => {
                self.output += r#"<span class="timestamp-wrapper"><span class="timestamp">"#;
                for e in timestamp.syntax().children_with_tokens() {
                    match e {
                        NodeOrToken::Token(t) if t.kind() == SyntaxKind::MINUS2 => {
                            self.output += "&#x2013;";
                        }
                        NodeOrToken::Token(t) => {
                            self.output += t.text();
                        }
                        _ => {}
                    }
                }
                self.output += r#"</span></span>"#;
            }

            Event::LatexFragment(latex) => {
                let latex_content = latex.raw().to_string();
                self.latex_blocks.push(latex_content);
                let _ = write!(
                    &mut self.output,
                    r#"<span class="org-latex-placeholder" data-latex-index="{}">[LaTeX Block {}]</span>"#,
                    self.latex_counter, self.latex_counter
                );
                self.latex_counter += 1;
            }
            Event::LatexEnvironment(latex) => {
                let latex_content = latex.raw().to_string();
                self.latex_blocks.push(latex_content);
                let _ = write!(
                    &mut self.output,
                    r#"<div class="org-latex-block-placeholder" data-latex-index="{}">[LaTeX Environment {}]</div>"#,
                    self.latex_counter, self.latex_counter
                );
                self.latex_counter += 1;
            }

            // ignores keyword
            Event::Enter(Container::Keyword(_)) => ctx.skip(),

            Event::Entity(entity) => self.output += entity.html(),

            Event::InlineSrc(src) => {
                let code = src.value();
                let lang = src.language();
                let _ = write!(
                    self.output,
                    "<code class=\"language-{}\">{}</code>",
                    lang, code
                );
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::handler;
    use orgize::Org;

    use super::*;
    #[test]
    fn test_org_table_export_advice_header() {
        let org = concat!(
            "| / | <>    |   |\n",
            "|---+-------+---|\n",
            "|   | hello | 1 |\n",
            "|---+-------+---|\n",
            "|   | world | 2 |\n"
        );
        let exp = concat!(
            "<div><section><table><thead>",
            "<tr><td>hello</td><td>1</td></tr></thead>",
            "<tbody><tr><td>world</td><td>2</td></tr></tbody>",
            "</table></section></div>"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_fixed_width_program_output() {
        let org = concat!(
            "#+BEGIN_SRC python\n",
            "print(\"Hello, world!\")\n",
            "#+END_SRC\n",
            "\n",
            ": Hello, world!\n",
            "\n",
            "Regular text.\n",
            "\n",
            ": More output\n",
            ": Another line\n"
        );
        let exp = concat!(
            "<div><section>",
            "<pre><code class=\"language-python\">print(&quot;Hello, world!&quot;)\n</code></pre>",
            "<pre class=\"program-output\">Hello, world!\n</pre>",
            "<p>Regular text.\n</p>",
            "<pre class=\"program-output\">More output\nAnother line\n</pre>",
            "</section></div>"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }
    #[test]
    fn test_org_table_export_empty_cells() {
        let org = concat!(
            "|-------+---|\n",
            "|       | 1 |\n",
            "|-------+---|\n",
            "| world |   |\n"
        );
        let exp = concat!(
            "<div><section><table><thead>",
            "<tr><td></td><td>1</td></tr></thead>",
            "<tbody><tr><td>world</td><td></td></tr></tbody>",
            "</table></section></div>"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }
    #[test]
    fn test_org_table_export_format_args() {
        let org = concat!("| / | <> |\n", "|   |  a |\n",);
        let exp = concat!(
            "<div><section><table><tbody>",
            "<tr><td>a</td></tr>",
            "</tbody></table></section></div>"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }
}
