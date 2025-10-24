use std::cmp::min;
use std::fmt::Write;
use std::path::PathBuf;

use crate::config::HtmlExportSettings;
use orgize::rowan::ast::AstNode;
use orgize::{
    export::{Container, Event, HtmlEscape, TraversalContext, Traverser},
    rowan::NodeOrToken,
    SyntaxKind,
};

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
    footnote_open: bool,
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
            footnote_open: false,
        }
    }

    /// Extract label from footnote syntax like "[fn:1]" or "[fn:label]"
    fn extract_footnote_label(raw: &str) -> String {
        if let Some(start) = raw.find("[fn:") {
            let after_prefix = &raw[start + 4..];
            if let Some(end) = after_prefix.find(']') {
                return after_prefix[..end].to_string();
            }
        }
        "unknown".to_string()
    }

    /// Parse org-mode content and extract inner HTML (without wrapper tags)
    fn parse_org_content_to_html(content: &str) -> String {
        use orgize::Org;
        let parsed = Org::parse(content);
        let mut exporter = orgize::export::HtmlExport::default();
        parsed.traverse(&mut exporter);
        let rendered = exporter.finish();

        // Strip wrapper tags: <main><section><p>...</p></section></main>
        let mut html = rendered.trim();
        html = html.strip_prefix("<main>").unwrap_or(html);
        html = html.strip_suffix("</main>").unwrap_or(html);
        html = html.strip_prefix("<section>").unwrap_or(html);
        html = html.strip_suffix("</section>").unwrap_or(html);
        html = html.strip_prefix("<p>").unwrap_or(html);
        html = html.strip_suffix("</p>").unwrap_or(html);
        html.trim().to_string()
    }

    /// Close an open footnote if there is one
    fn close_footnote_if_needed(&mut self) {
        if self.footnote_open {
            self.output += "</div></div>";
            self.footnote_open = false;
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
                if self.settings.respect_noexport && headline.tags().any(|t| t.contains("noexport"))
                {
                    ctx.skip();
                    return;
                }
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
                if !self.in_special_block && !self.footnote_open {
                    self.output += "<p>"
                }
            }
            Event::Leave(Container::Paragraph(_)) => {
                if !self.in_special_block && !self.footnote_open {
                    self.output += "</p>";
                }
            }

            Event::Enter(Container::Section(_)) => self.output += "<section>",
            Event::Leave(Container::Section(_)) => {
                self.close_footnote_if_needed();
                self.output += "</section>";
            }

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

            Event::Enter(Container::FnRef(fnref)) => {
                // Extract label from the raw text like "[fn:1]" or "[fn:label]"
                let raw = fnref.raw();
                let label = raw.trim_start_matches("[fn:").trim_end_matches(']');
                let _ = write!(
                    &mut self.output,
                    "<sup><a id=\"fnr.{}\" class=\"footref\" href=\"#fn.{}\">{}</a></sup>",
                    HtmlEscape(label),
                    HtmlEscape(label),
                    HtmlEscape(label)
                );
                ctx.skip();
            }
            Event::Leave(Container::FnRef(_)) => {}

            Event::Enter(Container::FnDef(fndef)) => {
                // Close any previously open footnote before starting a new one
                self.close_footnote_if_needed();

                // Extract label and content from the raw text
                let raw = fndef.raw();
                let label = Self::extract_footnote_label(&raw);

                // Extract content (first line only - continuation lines come as separate paragraphs)
                let content = if let Some(start) = raw.find(']') {
                    &raw[start + 1..].trim_start()
                } else {
                    ""
                };

                // Write footnote header
                let _ = write!(
                    &mut self.output,
                    "<div class=\"footdef\"><sup><a id=\"fn.{}\" class=\"footnum\" href=\"#fnr.{}\">{}</a></sup> <div class=\"footpara\">",
                    HtmlEscape(&label),
                    HtmlEscape(&label),
                    HtmlEscape(&label)
                );

                // Parse and render the footnote content with inline markup support
                let inner_html = Self::parse_org_content_to_html(content);
                self.output += &inner_html;

                // Mark footnote as open so continuation paragraphs can be included
                self.footnote_open = true;

                // Skip the FnDef's children since we already rendered the first line
                ctx.skip();
            }
            Event::Leave(Container::FnDef(_)) => {
                // Footnote stays open for continuation paragraphs
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

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

    #[test]
    fn test_noexport_single_heading() {
        let org = concat!(
            "* Exported heading\n",
            "This should be exported.\n",
            "\n",
            "* Hidden heading :noexport:\n",
            "This should not be exported.\n",
            "\n",
            "* Another exported heading\n",
            "This should be exported too.\n"
        );
        let exp = concat!(
            "<div>",
            "<h1>Exported heading</h1>",
            "<section><p>This should be exported.\n</p></section>",
            "<h1>Another exported heading</h1>",
            "<section><p>This should be exported too.\n</p></section></div>"
        );
        let mut settings = HtmlExportSettings::default();
        settings.respect_noexport = true;
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_noexport_with_subtree() {
        let org = concat!(
            "* Parent heading :noexport:\n",
            "Parent content.\n",
            "\n",
            "** Child heading\n",
            "Child content should also be excluded.\n",
            "\n",
            "*** Grandchild heading\n",
            "Grandchild content should also be excluded.\n",
            "\n",
            "* Exported heading\n",
            "This should be visible.\n"
        );
        let exp = concat!(
            "<div>",
            "<h1>Exported heading</h1>",
            "<section><p>This should be visible.\n</p></section></div>"
        );
        let mut settings = HtmlExportSettings::default();
        settings.respect_noexport = true;
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_noexport_with_multiple_tags() {
        let org = concat!(
            "* Heading with multiple tags :tag1:noexport:tag2:\n",
            "This should not be exported.\n",
            "\n",
            "* Normal heading :tag1:tag2:\n",
            "This should be exported.\n"
        );
        let exp = concat!(
            "<div>",
            "<h1>Normal heading </h1>",
            "<section><p>This should be exported.\n</p></section></div>"
        );
        let mut settings = HtmlExportSettings::default();
        settings.respect_noexport = true;
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_noexport_disabled() {
        let org = concat!(
            "* Normal heading\n",
            "Exported.\n",
            "\n",
            "* Hidden heading :noexport:\n",
            "This SHOULD be exported when respect_noexport is false.\n"
        );
        let exp = concat!(
            "<div>",
            "<h1>Normal heading</h1>",
            "<section><p>Exported.\n</p></section>",
            "<h1>Hidden heading </h1>",
            "<section><p>This SHOULD be exported when respect<sub>noexport</sub> is false.\n</p></section></div>"
        );
        let mut settings = HtmlExportSettings::default();
        settings.respect_noexport = false;
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_noexport_with_complex_content() {
        let org = concat!(
            "* Visible section\n",
            "Some text.\n",
            "\n",
            "* Secret section :noexport:\n",
            "** Contains subsections\n",
            "- With lists\n",
            "- And items\n",
            "\n",
            "#+BEGIN_SRC python\n",
            "print('hidden code')\n",
            "#+END_SRC\n",
            "\n",
            "| Table | Data |\n",
            "|-------+------|\n",
            "| 1     | 2    |\n",
            "\n",
            "* Back to visible\n",
            "Final content.\n"
        );
        let exp = concat!(
            "<div>",
            "<h1>Visible section</h1>",
            "<section><p>Some text.\n</p></section>",
            "<h1>Back to visible</h1>",
            "<section><p>Final content.\n</p></section></div>"
        );
        let mut settings = HtmlExportSettings::default();
        settings.respect_noexport = true;
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        assert_eq!(handler.finish().0, exp);
    }

    #[test]
    fn test_footnote_export() {
        let org = concat!(
            "* Test Footnotes\n",
            "\n",
            "This is a test[fn:1] with a footnote reference.\n",
            "\n",
            "And another one[fn:second].\n",
            "\n",
            "* Footnotes\n",
            "\n",
            "[fn:1] This is the first footnote definition.\n",
            "\n",
            "[fn:second] This is the second footnote.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;
        println!("Footnote export result:\n{}", result);

        // Check that footnote references are properly formatted with links
        assert!(
            result.contains("<sup><a id=\"fnr.1\" class=\"footref\" href=\"#fn.1\">1</a></sup>")
        );
        assert!(result.contains(
            "<sup><a id=\"fnr.second\" class=\"footref\" href=\"#fn.second\">second</a></sup>"
        ));

        // Check that footnote definitions are properly formatted
        assert!(result.contains("<div class=\"footdef\"><sup><a id=\"fn.1\" class=\"footnum\" href=\"#fnr.1\">1</a></sup>"));
        assert!(result.contains("<div class=\"footdef\"><sup><a id=\"fn.second\" class=\"footnum\" href=\"#fnr.second\">second</a></sup>"));

        // Check that the content is included
        assert!(result.contains("This is the first footnote definition"));
        assert!(result.contains("This is the second footnote"));

        // Check that the label is NOT duplicated in the content
        // Extract just the footpara content
        let footpara1_start =
            result.find("<div class=\"footpara\">").unwrap() + "<div class=\"footpara\">".len();
        let footpara1_end = result[footpara1_start..].find("</div>").unwrap() + footpara1_start;
        let footpara1_content = &result[footpara1_start..footpara1_end];
        println!("Footpara 1 content: '{}'", footpara1_content);

        // The label "1" should NOT appear at the start of the content
        // It should start with "This is the first footnote"
        assert!(!footpara1_content.trim().starts_with("1 This"));
        assert!(footpara1_content.contains("This is the first footnote definition"));
    }

    #[test]
    fn test_footnote_with_inline_markup() {
        let org = concat!(
            "Text with footnote[fn:1].\n",
            "\n",
            "[fn:1] Footnote with *bold* and /italic/ and =code= text.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;

        assert!(result.contains("class=\"footdef\""));

        // Markup should be properly processed
        assert!(result.contains("<b>bold</b>"));
        assert!(result.contains("<i>italic</i>"));
        assert!(result.contains("<code>code</code>"));

        // Literal markup should NOT appear
        assert!(!result.contains("*bold*"));
        assert!(!result.contains("/italic/"));
        assert!(!result.contains("=code="));
    }

    #[test]
    fn test_multiline_footnote_no_indent() {
        let org = concat!(
            "Text with footnote[fn:1].\n",
            "\n",
            "[fn:1] This is the first line of the footnote.\n",
            "This is the second line of the same footnote.\n",
            "And this is the third line.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;

        // Check that the footnote definition exists
        assert!(result.contains("class=\"footdef\""));

        // Verify all lines are included within the footnote
        assert!(result.contains("first line"));
        assert!(result.contains("second line"));
        assert!(result.contains("third line"));

        // Ensure the closing tags are in the right order (content is inside footpara)
        let footdef_start = result.find("<div class=\"footdef\">").unwrap();
        let footdef_end = result[footdef_start..].find("</div></div>").unwrap() + footdef_start;
        let footnote_section = &result[footdef_start..footdef_end];

        assert!(footnote_section.contains("first line"));
        assert!(footnote_section.contains("second line"));
        assert!(footnote_section.contains("third line"));
    }

    #[test]
    fn test_multiline_footnote_with_indent() {
        // In org-mode, continuation lines can be indented
        let org = concat!(
            "Text with footnote[fn:1].\n",
            "\n",
            "[fn:1] This is the first line of the footnote.\n",
            "       This is the second line of the same footnote.\n",
            "       And this is the third line.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;

        // Check that the footnote definition exists
        assert!(result.contains("class=\"footdef\""));

        // Check that all lines are included in the footnote
        assert!(result.contains("first line"));
        assert!(result.contains("second line"));
        assert!(result.contains("third line"));

        // Ensure all content is within the footdef
        let footdef_start = result.find("<div class=\"footdef\">").unwrap();
        let footdef_end = result[footdef_start..].find("</div></div>").unwrap() + footdef_start;
        let footnote_section = &result[footdef_start..footdef_end];

        assert!(footnote_section.contains("first line"));
        assert!(footnote_section.contains("second line"));
        assert!(footnote_section.contains("third line"));
    }

    #[test]
    fn test_multiple_footnotes() {
        let org = concat!(
            "Text with first[fn:1] and second[fn:2] footnote.\n",
            "\n",
            "[fn:1] First footnote content.\n",
            "More content for first footnote.\n",
            "\n",
            "[fn:2] Second footnote content.\n",
            "More content for second footnote.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;

        // Check both footnote references exist
        assert!(result.contains("href=\"#fn.1\""));
        assert!(result.contains("href=\"#fn.2\""));

        // Check both footnote definitions exist
        assert!(result.contains("id=\"fn.1\""));
        assert!(result.contains("id=\"fn.2\""));

        // Verify first footnote contains all its content
        let first_fn_start = result.find("id=\"fn.1\"").unwrap();
        let second_fn_start = result.find("id=\"fn.2\"").unwrap();
        let first_footnote = &result[first_fn_start..second_fn_start];
        assert!(first_footnote.contains("First footnote content"));
        assert!(first_footnote.contains("More content for first footnote"));

        // Verify second footnote contains all its content
        let second_footnote = &result[second_fn_start..];
        assert!(second_footnote.contains("Second footnote content"));
        assert!(second_footnote.contains("More content for second footnote"));
    }

    #[test]
    fn test_footnote_no_paragraph_tags() {
        // Verify that continuation lines in footnotes don't create <p> tags
        let org = concat!(
            "Text with footnote[fn:1].\n",
            "\n",
            "[fn:1] First line.\n",
            "Second line.\n",
            "Third line.\n"
        );
        let settings = HtmlExportSettings::default();
        let mut handler = HtmlExport::new(&settings, "".into());
        Org::parse(org).traverse(&mut handler);
        let result = handler.finish().0;

        // Extract just the footnote definition section
        let footdef_start = result.find("<div class=\"footdef\">").unwrap();
        let footdef_end = result[footdef_start..].find("</div></div>").unwrap() + footdef_start;
        let footnote = &result[footdef_start..=footdef_end];

        // Verify there are NO <p> tags inside the footnote
        assert!(
            !footnote.contains("<p>"),
            "Footnote should not contain <p> tags, but got: {}",
            footnote
        );
        assert!(
            !footnote.contains("</p>"),
            "Footnote should not contain </p> tags, but got: {}",
            footnote
        );

        // Verify the content is still there
        assert!(footnote.contains("First line"));
        assert!(footnote.contains("Second line"));
        assert!(footnote.contains("Third line"));
    }
}
