//! `section_row`: `section.<name>.row` is the first row of a section that reads, as
//! `header.title_from` reads it. A section that reads nothing holds none, and a read whose params
//! read its own section's row, directly or through other sections, never settles.

use std::collections::{BTreeMap, BTreeSet};

use ess_ui::{Document, NodePath};

use crate::rules::cycles;
use crate::walk::composite_reads;
use crate::Sink;

/// One `section.<name>.row[.<field>]` path written in an expression.
pub(crate) struct RowPath<'t> {
    pub section: &'t str,
    /// The first segment of the field read from the row, when one is.
    pub field: Option<&'t str>,
}

fn is_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

/// Every `section.<name>.row` path in an expression, where `section` stands alone (not the tail
/// of a longer name or path).
pub(crate) fn row_paths(text: &str) -> Vec<RowPath<'_>> {
    const PREFIX: &str = "section.";
    let mut found = Vec::new();
    for (start, _) in text.match_indices(PREFIX) {
        let alone = text[..start]
            .chars()
            .next_back()
            .is_none_or(|before| !is_name_char(before) && before != '.');
        if !alone {
            continue;
        }
        let rest = &text[start + PREFIX.len()..];
        let name_end = rest.find(|c: char| !is_name_char(c)).unwrap_or(rest.len());
        let (name, after) = rest.split_at(name_end);
        let Some(after) = after.strip_prefix(".row") else {
            continue;
        };
        if after.chars().next().is_some_and(is_name_char) {
            continue;
        }
        let field = after.strip_prefix('.').and_then(|field| {
            let end = field
                .find(|c: char| !is_name_char(c))
                .unwrap_or(field.len());
            (end > 0).then(|| &field[..end])
        });
        if !name.is_empty() {
            found.push(RowPath {
                section: name,
                field,
            });
        }
    }
    found
}

/// The sections of each page whose read params read a section's own row, or a row that reads
/// back, reported at the read of each section on a cycle.
pub(crate) fn run(document: &Document, sink: &mut Sink) {
    for (page_name, page) in &document.pages {
        let reading: BTreeSet<&str> = page
            .sections
            .iter()
            .filter(|section| section.body.reads().is_some())
            .map(|section| section.name.as_str())
            .collect();
        let mut waits: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for section in &page.sections {
            let Some(reads) = section.body.reads() else {
                continue;
            };
            for expr in reads.params.values() {
                for path in row_paths(&expr.0) {
                    if let Some(target) = reading.get(path.section) {
                        waits
                            .entry(section.name.as_str())
                            .or_default()
                            .insert(target);
                    }
                }
            }
        }
        let names: Vec<&str> = reading.iter().copied().collect();
        for (section, cycle) in cycles(&names, &waits) {
            let Some(found) = page.sections.iter().find(|found| found.name == section) else {
                continue;
            };
            let key = match &found.body {
                ess_ui::Body::Composite(composite) => {
                    composite_reads(composite).map_or("reads", |(key, _)| key)
                }
                _ => "reads",
            };
            let at = NodePath::root()
                .child("pages")
                .child(page_name)
                .child("sections")
                .child(section)
                .child(key);
            let message = if cycle.iter().all(|step| *step == section) {
                format!(
                    "section `{section}` reads its own row in the params of its read, which \
                     would wait on itself"
                )
            } else {
                format!(
                    "section `{section}` waits on its own row through the params of the reads of \
                     sections that read each other's row ({}); no read of them could start",
                    cycle
                        .iter()
                        .map(|step| format!("`{step}`"))
                        .collect::<Vec<_>>()
                        .join(" → ")
                )
            };
            sink.push("section_row", &at, message);
        }
    }
}
