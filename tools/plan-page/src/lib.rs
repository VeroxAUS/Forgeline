// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Turns Forgeline plan markdown into one self-contained, dark-themed HTML page.
//!
//! The styling lives in `style.css` and is baked into the binary, so every plan page
//! comes out the same. Raw HTML in the markdown is escaped, never passed through.

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

const STYLE: &str = include_str!("style.css");

fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if (c == ' ' || c == '-') && !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn raw(s: &str) -> Event<'static> {
    Event::Html(CowStr::from(s.to_string()))
}

/// Renders markdown to the page body: an HTML fragment (headings with ids,
/// tables wrapped for scrolling, checkbox task lists).
pub fn render_body(md: &str) -> String {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH;
    let events: Vec<Event> = Parser::new_ext(md, opts).collect();

    let mut out: Vec<Event> = Vec::with_capacity(events.len() + 16);
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading {
                level,
                id: _,
                classes,
                attrs,
            }) => {
                // Gather the heading text to build its id.
                let mut text = String::new();
                let mut j = i + 1;
                while j < events.len() && !matches!(events[j], Event::End(TagEnd::Heading(_))) {
                    if let Event::Text(t) | Event::Code(t) = &events[j] {
                        text.push_str(t);
                    }
                    j += 1;
                }
                let base = slug(&text);
                let n = seen.entry(base.clone()).or_insert(0);
                let id = if *n == 0 { base } else { format!("{base}-{n}") };
                *n += 1;
                out.push(Event::Start(Tag::Heading {
                    level: *level,
                    id: Some(CowStr::from(id)),
                    classes: classes.clone(),
                    attrs: attrs.clone(),
                }));
            }
            // Raw HTML is shown as text (escaped), so `<hours>` stays visible.
            Event::Html(s) | Event::InlineHtml(s) => out.push(Event::Text(s.clone())),
            Event::Start(Tag::Table(a)) => {
                out.push(raw("<div class=\"tw\">"));
                out.push(Event::Start(Tag::Table(a.clone())));
            }
            Event::End(TagEnd::Table) => {
                out.push(Event::End(TagEnd::Table));
                out.push(raw("</div>\n"));
            }
            Event::TaskListMarker(done) => out.push(raw(if *done {
                "<span class=\"cb on\" role=\"img\" aria-label=\"done\"></span>"
            } else {
                "<span class=\"cb\" role=\"img\" aria-label=\"not done\"></span>"
            })),
            other => out.push(other.clone()),
        }
        i += 1;
    }

    let mut body = String::new();
    html::push_html(&mut body, out.into_iter());
    body
}

fn title_of(md: &str) -> Option<String> {
    md.lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(|t| t.trim().to_string())
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Builds the full page. With `standalone` false the output is the fragment the
/// Artifact tool expects (it adds its own document skeleton); with it true the
/// output is a complete HTML document that opens correctly in a browser.
pub fn render_page(md: &str, title: Option<&str>, standalone: bool) -> String {
    let title = title
        .map(str::to_string)
        .or_else(|| title_of(md))
        .unwrap_or_else(|| "Forgeline plan".to_string());
    let head = format!(
        "<title>{}</title>\n<meta name=\"claude:viewport-fit-sides\" content=\"cover\">\n<style>\n{}</style>\n",
        escape(&title),
        STYLE
    );
    let main = format!("<main>{}</main>\n", render_body(md));
    if standalone {
        format!(
            "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width,initial-scale=1,viewport-fit=cover\">\n{head}</head>\n<body>\n{main}</body></html>\n"
        )
    } else {
        format!("{head}{main}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_after_bold_label_is_a_list() {
        let html = render_body("**Run order:**\n\n- one\n- two\n");
        assert!(html.contains("<ul>"), "{html}");
        assert_eq!(html.matches("<li>").count(), 2);
    }

    #[test]
    fn tables_are_wrapped_for_scrolling() {
        let html = render_body("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<div class=\"tw\"><table>"), "{html}");
        assert!(html.contains("</table>\n</div>"), "{html}");
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn raw_html_and_angle_brackets_are_escaped() {
        let html = render_body("Elapsed: `<hours>` and <b>x</b> and <hours>\n");
        assert!(html.contains("&lt;hours&gt;"), "{html}");
        assert!(!html.contains("<b>"), "{html}");
    }

    #[test]
    fn task_lists_get_checkboxes() {
        let html = render_body("- [ ] open\n- [x] done\n");
        assert!(html.contains("<span class=\"cb\""), "{html}");
        assert!(html.contains("<span class=\"cb on\""), "{html}");
    }

    #[test]
    fn headings_get_unique_ids() {
        let html = render_body("## 3.1 Architecture\n\n## Same\n\n## Same\n");
        assert!(html.contains("id=\"31-architecture\""), "{html}");
        assert!(html.contains("id=\"same\""));
        assert!(html.contains("id=\"same-1\""));
    }

    #[test]
    fn page_uses_h1_as_title_and_fragment_has_no_doctype() {
        let page = render_page("# Phase 1: Spikes\n\nhi\n", None, false);
        assert!(page.starts_with("<title>Phase 1: Spikes</title>"));
        assert!(!page.contains("<!doctype"));
        let full = render_page("# T\n", None, true);
        assert!(full.starts_with("<!doctype html>"));
    }

    #[test]
    fn page_makes_no_network_requests() {
        let page = render_page("# T\n", None, true);
        assert!(!page.contains("http://") && !page.contains("<script"));
        assert!(!page.contains("<link"));
    }
}
