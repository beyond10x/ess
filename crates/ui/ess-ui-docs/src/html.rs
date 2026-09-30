//! The reference as one self-contained HTML page: inline style, an inline filter for the
//! sidebar, examples coloured here rather than by a script.

use std::fmt::Write as _;

use crate::doc::{Block, Inline, FILTER_ID};
use crate::yaml::{self, Kind};

const STYLE: &str = r#"
:root {
  --bg: #fbfbfd; --panel: #ffffff; --side: #f3f4f8; --text: #1d2130; --muted: #5d6478;
  --line: #e2e5ee; --accent: #3b5bdb; --accent-soft: #edf1ff; --code-bg: #f4f5f9;
  --k: #1f5fbf; --s: #2b7a3d; --n: #a4500f; --l: #8b3fb8; --p: #7a8194;
  color-scheme: light dark;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #12141b; --panel: #181b24; --side: #151821; --text: #e4e7f0; --muted: #9aa1b5;
    --line: #2a2f3d; --accent: #8ea6ff; --accent-soft: #232a45; --code-bg: #1f2330;
    --k: #8ab4ff; --s: #9ad48f; --n: #f0a86a; --l: #d39cf5; --p: #8a91a6;
  }
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0; background: var(--bg); color: var(--text);
  font: 15px/1.6 system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  display: grid; grid-template-columns: 280px minmax(0, 1fr);
}
nav.sidebar {
  position: sticky; top: 0; height: 100vh; overflow-y: auto; padding: 20px 16px;
  background: var(--side); border-right: 1px solid var(--line); font-size: 14px;
}
nav .brand { font-weight: 700; font-size: 17px; margin: 0 0 12px 4px; }
nav input {
  width: 100%; padding: 7px 10px; margin-bottom: 14px; border-radius: 8px;
  border: 1px solid var(--line); background: var(--panel); color: var(--text); font: inherit;
}
nav ul { list-style: none; margin: 0; padding: 0; }
nav li.chapter > a { display: block; font-weight: 600; margin-top: 12px; }
nav li.item > a { display: block; padding: 1px 0 1px 12px; color: var(--muted); }
nav a { text-decoration: none; color: var(--text); border-radius: 6px; }
nav a:hover { color: var(--accent); }
main { padding: 32px 48px 80px; max-width: 1080px; }
h1 { font-size: 30px; margin: 0 0 8px; letter-spacing: -0.01em; }
h2 {
  font-size: 23px; margin: 48px 0 8px; padding-bottom: 6px; border-bottom: 2px solid var(--line);
}
h3 { font-size: 19px; margin: 0 0 6px; }
h3 .kind { font-size: 12px; font-weight: 500; color: var(--muted); margin-left: 8px; }
h4 {
  font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted);
  margin: 18px 0 6px;
}
section.construct, section.section {
  background: var(--panel); border: 1px solid var(--line); border-radius: 12px;
  padding: 20px 24px; margin: 20px 0;
}
section.construct > p:first-of-type { font-size: 16px; font-weight: 500; }
a { color: var(--accent); }
code {
  font: 13px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  background: var(--code-bg); padding: 1px 5px; border-radius: 5px;
}
table { border-collapse: collapse; width: 100%; font-size: 14px; margin: 4px 0 8px; }
th {
  text-align: left; font-weight: 600; color: var(--muted); font-size: 12px;
  text-transform: uppercase; letter-spacing: 0.04em; border-bottom: 1px solid var(--line);
  padding: 6px 8px;
}
td { vertical-align: top; padding: 6px 8px; border-bottom: 1px solid var(--line); }
tr:last-child td { border-bottom: none; }
table.fields td:first-child { white-space: nowrap; }
pre.example {
  background: var(--code-bg); border-radius: 10px; padding: 14px 16px; overflow-x: auto;
  margin: 4px 0 8px; border: 1px solid var(--line);
}
pre.example code { background: none; padding: 0; }
.k { color: var(--k); } .s { color: var(--s); } .n { color: var(--n); }
.l { color: var(--l); } .p { color: var(--p); }
@media (max-width: 860px) {
  body { display: block; }
  nav.sidebar { position: static; height: auto; }
  main { padding: 20px; }
}
"#;

const SCRIPT: &str = r"
const filter = document.getElementById('FILTER_ID');
filter.addEventListener('input', () => {
  const q = filter.value.trim().toLowerCase();
  for (const chapter of document.querySelectorAll('nav li.chapter')) {
    let shown = 0;
    for (const item of chapter.querySelectorAll('li.item')) {
      const hit = !q || item.textContent.toLowerCase().includes(q);
      item.style.display = hit ? '' : 'none';
      if (hit) shown += 1;
    }
    const title = chapter.querySelector('a').textContent.toLowerCase();
    chapter.style.display = !q || shown || title.includes(q) ? '' : 'none';
  }
});
";

pub(crate) fn render(blocks: &[Block], title: &str) -> String {
    let mut out = String::new();
    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(out, "<title>{}</title>", escape(title));
    let _ = writeln!(out, "<style>{STYLE}</style>\n</head>\n<body>");
    sidebar(blocks, title, &mut out);
    out.push_str("<main>\n");
    for block in blocks {
        write_block(block, &mut out);
    }
    out.push_str("</main>\n");
    let _ = writeln!(
        out,
        "<script>{}</script>\n</body>\n</html>",
        SCRIPT.replace("FILTER_ID", FILTER_ID)
    );
    out
}

/// Chapters and their sections, read off the blocks: a region opened at level 2 is a chapter,
/// one at level 3 an entry under it.
fn sidebar(blocks: &[Block], title: &str, out: &mut String) {
    let _ = writeln!(
        out,
        "<nav class=\"sidebar\">\n<div class=\"brand\">{}</div>",
        escape(title)
    );
    let _ = writeln!(
        out,
        "<input id=\"{FILTER_ID}\" type=\"search\" placeholder=\"Filter constructs\" aria-label=\"Filter constructs\">\n<ul>"
    );
    let mut open_chapter = false;
    let mut pending: Option<&str> = None;
    for block in blocks {
        match block {
            Block::Open { id, .. } => pending = Some(id),
            Block::Heading { level, text } => {
                let Some(id) = pending.take() else { continue };
                match level {
                    2 => {
                        if open_chapter {
                            out.push_str("</ul></li>\n");
                        }
                        let _ = writeln!(
                            out,
                            "<li class=\"chapter\"><a href=\"#{id}\">{}</a><ul>",
                            escape(text)
                        );
                        open_chapter = true;
                    }
                    3 => {
                        let _ = writeln!(
                            out,
                            "<li class=\"item\"><a href=\"#{id}\">{}</a></li>",
                            escape(text)
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    if open_chapter {
        out.push_str("</ul></li>\n");
    }
    out.push_str("</ul>\n</nav>\n");
}

fn write_block(block: &Block, out: &mut String) {
    match block {
        Block::Open { class, id } => {
            let _ = writeln!(out, "<section class=\"{class}\" id=\"{id}\">");
        }
        Block::Close => out.push_str("</section>\n"),
        Block::Heading { level, text } => {
            let _ = writeln!(out, "<h{level}>{}</h{level}>", escape(text));
        }
        Block::Label(text) => {
            let _ = writeln!(out, "<h4>{}</h4>", escape(text));
        }
        Block::Para(inlines) => {
            out.push_str("<p>");
            write_inlines(inlines, out);
            out.push_str("</p>\n");
        }
        Block::Table { class, head, rows } => {
            let _ = write!(out, "<table class=\"{class}\">\n<thead><tr>");
            for cell in head {
                let _ = write!(out, "<th>{}</th>", escape(cell));
            }
            out.push_str("</tr></thead>\n<tbody>\n");
            for row in rows {
                out.push_str("<tr>");
                for cell in row {
                    out.push_str("<td>");
                    write_inlines(cell, out);
                    out.push_str("</td>");
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody>\n</table>\n");
        }
        Block::Yaml(value) => {
            out.push_str("<pre class=\"example\"><code>");
            for line in yaml::block(value) {
                for tok in line {
                    let class = match tok.kind {
                        Kind::Key => "k",
                        Kind::Str => "s",
                        Kind::Num => "n",
                        Kind::Lit => "l",
                        Kind::Punct => "p",
                        Kind::Space => {
                            out.push_str(&tok.text);
                            continue;
                        }
                    };
                    let _ = write!(out, "<span class=\"{class}\">{}</span>", escape(&tok.text));
                }
                out.push('\n');
            }
            out.push_str("</code></pre>\n");
        }
    }
}

fn write_inlines(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => out.push_str(&escape(text)),
            Inline::Code(code) => {
                let _ = write!(out, "<code>{}</code>", escape(code));
            }
            Inline::Link { text, anchor } => {
                let _ = write!(out, "<a href=\"#{anchor}\">{}</a>", escape(text));
            }
            Inline::Prose(prose) => {
                for (i, part) in prose.split('`').enumerate() {
                    if i % 2 == 1 {
                        let _ = write!(out, "<code>{}</code>", escape(part));
                    } else {
                        out.push_str(&escape(part));
                    }
                }
            }
        }
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}
