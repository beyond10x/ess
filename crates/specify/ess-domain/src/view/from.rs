//! `from:` on a view field (beyond10x/ess#675): a guessed spelling of a read through a reference,
//! refused at its line with the spelling that reads it.
//!
//! An adopter wrote `{name: cover, type: Optional<String>, from: book.cover}` for a view field
//! reading the cover of the book its source row names, was told ``unknown field `from` `` for the
//! whole file, and concluded the read does not exist. It does: `field: {related: {via: book_id,
//! field: cover}}` ([`super::navigation`], `ess/28`). A value computed from the source row's own
//! fields is `expr:` (`ess/27`). `from:` is admitted as a second spelling of neither.
//!
//! [`RawSpecFile::parse`](crate::spec::RawSpecFile::parse) takes the key out of each view field
//! ([`Written::take`]), as it takes `undeclared_fields:` and `shorthand:` out of a declaration, so
//! the rest of the file is read. Assembly withholds the view whole, as it withholds a declaration
//! that does not convert, so nothing the field would have read is refused a second time. Once
//! every file is assembled, [`refusals`] refuses each `from:` as
//! [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) at `view.<view>.fields[<i>].from`:
//!
//! | written | hint |
//! |---|---|
//! | `<x>.<field>`, `<x>` a reference field of the source or a `references` relation carried by one | the rewrite, `field: {related: {via: <reference field>, field: <field>}}` |
//! | anything else | both spellings: `field: {related: …}` (`ess/28`) and `expr:` (`ess/27`) |
//!
//! Below `ess/28` the rewrite also names `format: ess/28`, the format that reads it. Everywhere
//! else — a field of an entity, an event or a command, or a view itself — `from:` stays an unknown
//! key that stops the file.
//!
//! # Where the refusal is cited
//!
//! The compiler's `Locator` cites a refusal at its trailing key only where that key occurs once in
//! every file, and every lifecycle transition writes `from:` (adversary pass 1 on this story). So
//! the refusal carries its own [`SyntaxSpan`]: [`Written::take`] reads the line and column of each
//! `from:` key out of the text, inside the item of the view that wrote it ([`positions`]). Where
//! that reading does not find exactly the keys the document holds for the view — a flow-style
//! `views:`, a `from:` spelled inside a string — the refusal carries no span and the `Locator`
//! cites it as before.

use std::collections::BTreeMap;

use ess_primitives::error::{
    ConstructKind, ConstructRef, SyntaxSpan, ValidationCode, ValidationError, ValidationErrors,
};

use super::navigation::{self, ViewReference};
use super::{join, RawViewSpec, WrittenSource};
use crate::entity::{EntitySpec, RelationKind};
use crate::name::QualifiedName;
use crate::system::{FormatVersion, Source};
use crate::Specification;

/// The key it is written under.
pub(crate) const KEY: &str = "from";

/// One view field's `from:`, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Taken {
    /// The field's `name:`, where it wrote one.
    field: Option<String>,
    /// What `from:` holds, where it is text.
    written: Option<String>,
    /// The 1-based line and column of the `from:` key, where [`positions`] found it.
    at: Option<(usize, usize)>,
}

/// The `from:` each view field of one document wrote, by view position and field position.
///
/// Never part of the document, its schema or its bytes: read by [`Self::take`] beside the typed
/// fields, and spent by assembly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Written {
    declared: BTreeMap<usize, BTreeMap<usize, Taken>>,
}

impl Written {
    /// Takes `from:` out of every field of every view of `document`, read from `text`, that writes
    /// it.
    ///
    /// Any value is taken, text or not: each is refused at its field once the specification is
    /// assembled, never as the document is read.
    pub(crate) fn take(document: &mut ess_yaml::Value, text: &str) -> Self {
        let mut written = Self::default();
        let Some(views) = document
            .get_mut("views")
            .and_then(ess_yaml::Value::as_sequence_mut)
        else {
            return written;
        };
        for (view, item) in views.iter_mut().enumerate() {
            let Some(fields) = item
                .get_mut("fields")
                .and_then(ess_yaml::Value::as_sequence_mut)
            else {
                continue;
            };
            for (index, field) in fields.iter_mut().enumerate() {
                let Some(mapping) = field.as_mapping_mut() else {
                    continue;
                };
                let Some(value) = mapping.shift_remove(KEY) else {
                    continue;
                };
                let taken = Taken {
                    field: mapping
                        .get("name")
                        .and_then(ess_yaml::Value::as_str)
                        .map(str::to_owned),
                    written: value.as_str().map(|text| text.trim().to_owned()),
                    at: None,
                };
                written
                    .declared
                    .entry(view)
                    .or_default()
                    .insert(index, taken);
            }
        }
        let mut found = positions(text);
        for (view, fields) in &mut written.declared {
            // The keys of one view in document order are its fields' in field order; anything but
            // one key per field taken is a reading this cannot trust.
            let Some(keys) = found.remove(view).filter(|keys| keys.len() == fields.len()) else {
                continue;
            };
            for (taken, at) in fields.values_mut().zip(keys) {
                taken.at = Some(at);
            }
        }
        written
    }

    /// The view at `index` of `views:`, `raw`, written in `source`, withheld for the `from:` its
    /// fields wrote, or `None` where none wrote one.
    pub(crate) fn withheld(
        &self,
        index: usize,
        raw: &RawViewSpec,
        source: &Source,
    ) -> Option<Withheld> {
        let fields = self.declared.get(&index)?;
        Some(Withheld {
            view: raw.name.clone(),
            label: source.as_str().to_owned(),
            source: match &raw.source {
                WrittenSource::One(source) => Some(source.clone()),
                WrittenSource::Several(_) => None,
            },
            fields: fields.clone(),
        })
    }
}

/// The 1-based line and column of every `from:` key in the text of each item of the top-level
/// block sequence `views:`, by item position, in document order.
///
/// Not a YAML reader: it reads the indentation of a block sequence and the shape of a key, and
/// [`Written::take`] trusts it only where it finds exactly the keys the parsed document holds. A
/// flow-style `views: [...]` gives nothing.
fn positions(text: &str) -> BTreeMap<usize, Vec<(usize, usize)>> {
    let mut found: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    let lines: Vec<&str> = text.lines().collect();
    let Some(start) = lines.iter().position(|line| {
        line.strip_prefix("views:").is_some_and(|rest| {
            let rest = rest.trim();
            rest.is_empty() || rest.starts_with('#')
        })
    }) else {
        return found;
    };
    let mut item_indent = None;
    let mut view: Option<usize> = None;
    for (number, line) in lines.iter().enumerate().skip(start + 1) {
        let body = line.trim_start_matches(' ');
        if body.is_empty() || body.starts_with('#') {
            continue;
        }
        let indent = line.len() - body.len();
        let dash = body == "-" || body.starts_with("- ");
        let item = *item_indent.get_or_insert(indent);
        if indent < item || (indent == item && !dash) {
            break;
        }
        if indent == item {
            view = Some(view.map_or(0, |view| view + 1));
        }
        let Some(view) = view else {
            break;
        };
        for column in keys(line) {
            found
                .entry(view)
                .or_default()
                .push((number + 1, column + 1));
        }
    }
    found
}

/// The 0-based byte offset of each `from:` written as a key on `line`: at the start of its content
/// or after a space, `{` or `,`, and followed by a space or the end of the line, before any comment.
fn keys(line: &str) -> Vec<usize> {
    let content = line.find(" #").map_or(line, |comment| &line[..comment]);
    let needle = format!("{KEY}:");
    content
        .match_indices(&needle)
        .map(|(offset, _)| offset)
        .filter(|offset| {
            let before = content[..*offset].chars().next_back();
            let after = content[offset + needle.len()..].chars().next();
            matches!(before, None | Some(' ' | '{' | ',' | '\t'))
                && matches!(after, None | Some(' ' | '\t'))
        })
        .collect()
}

/// A view withheld from the specification because a field of it wrote `from:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Withheld {
    view: QualifiedName,
    /// The label of the file that wrote it.
    label: String,
    /// Its one source, where it names one.
    source: Option<QualifiedName>,
    fields: BTreeMap<usize, Taken>,
}

/// The refusal of every `from:` the `withheld` views wrote, each at its field, against the
/// assembled `spec`: its format and its entities' references.
pub(crate) fn refusals(spec: &Specification, withheld: &[Withheld]) -> ValidationErrors {
    let below_ess_28 = spec.system().format.major() < FormatVersion::V28.major();
    let mut errors = ValidationErrors::new();
    for view in withheld {
        let entity = view
            .source
            .as_ref()
            .and_then(|source| spec.entities().get(source));
        let references = view
            .source
            .as_ref()
            .map(|source| navigation::source_references(spec, source))
            .unwrap_or_default();
        for (index, taken) in &view.fields {
            errors.push(refusal(
                view,
                *index,
                taken,
                (entity, &references),
                below_ess_28,
            ));
        }
    }
    errors
}

/// The refusal of the `from:` the field at `index` of `view` wrote.
fn refusal(
    view: &Withheld,
    index: usize,
    taken: &Taken,
    (entity, references): (Option<&EntitySpec>, &[ViewReference]),
    below_ess_28: bool,
) -> ValidationError {
    let name = &view.view;
    let field = taken.field.as_deref().map_or_else(
        || format!("a field of `{name}`"),
        |field| format!("`{field}` of `{name}`"),
    );
    let written = taken
        .written
        .as_deref()
        .map_or_else(|| format!("`{KEY}:`"), |text| format!("`{KEY}: {text}`"));
    let at = ConstructRef::new(ConstructKind::View, name.to_string())
        .key("fields")
        .index(index)
        .key(KEY);
    let refused = |message: String| match taken.at {
        Some((line, column)) => ValidationError::at_span(
            at.clone(),
            SyntaxSpan {
                source: view.label.clone(),
                line,
                column,
            },
            ValidationCode::UnsupportedConstruct,
            message,
        ),
        None => ValidationError::at(at.clone(), ValidationCode::UnsupportedConstruct, message),
    };
    let resolved = taken
        .written
        .as_deref()
        .and_then(|text| rewrite(text, entity, references));
    if let Some((via, read)) = resolved {
        let format = if below_ess_28 {
            "declare `format: ess/28`, which reads it, and "
        } else {
            ""
        };
        return refused(format!(
            "{field} writes {written}, which is not a key of a view field; `{read}` of the row \
             `{via}` names is read as `field: {{related: {{via, field}}}}`"
        ))
        .with_hint(format!(
            "{format}write `field: {{related: {{via: {via}, field: {read}}}}}` in place of \
             {written}"
        ));
    }
    let listed = match (&view.source, entity) {
        (Some(source), Some(_)) => format!(
            "; the references of `{source}`: {}",
            join(references.iter().map(|reference| reference.via.clone()))
        ),
        _ => String::new(),
    };
    refused(format!(
        "{field} writes {written}, which is not a key of a view field"
    ))
    .with_hint(format!(
        "read a field of the row a reference of the source names as `field: {{related: {{via: \
         <reference field>, field: <field>}}}}` (ess/28), or compute a value from the source \
         row's own fields as `expr:` (ess/27){listed}"
    ))
}

/// `(via, field)` for `<x>.<field>` where `<x>` is a reference field of the source or a
/// `references` relation of it carried by one; `None` for anything else.
fn rewrite(
    written: &str,
    entity: Option<&EntitySpec>,
    references: &[ViewReference],
) -> Option<(String, String)> {
    let (root, read) = written.split_once('.')?;
    if root.is_empty() || read.is_empty() || read.contains('.') {
        return None;
    }
    let is_reference = |field: &str| references.iter().any(|reference| reference.via == field);
    if is_reference(root) {
        return Some((root.to_owned(), read.to_owned()));
    }
    let relation = entity?
        .relations
        .iter()
        .find(|relation| relation.name == root && relation.kind == RelationKind::References)?;
    is_reference(&relation.via).then(|| (relation.via.clone(), read.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::positions;

    #[test]
    fn each_view_item_answers_its_own_from_keys_in_document_order() {
        let text = "\
entities:
  - name: a.b.Thing
    lifecycle:
      transitions:
        - {name: go, from: [Open], to: Shut}
views:
  # a comment between items
  - name: a.b.First
    fields:
      - {name: x, type: String}
  - name: a.b.Second
    fields:
      - {name: y, type: String, from: thing.y}
      - name: z
        type: String
        from: thing.z  # from: inside a comment is no key
      - {name: w, type: String, title: 'copied from:here'}
commands: []
";
        let found = positions(text);
        assert_eq!(found.get(&0), None, "{found:?}");
        assert_eq!(found.get(&1), Some(&vec![(13, 33), (16, 9)]), "{found:?}");
        assert_eq!(found.len(), 1, "nothing after `views:` ends: {found:?}");
    }

    #[test]
    fn a_flow_sequence_of_views_gives_nothing() {
        assert!(positions("views: [{name: a.b.V, fields: [{name: x, from: y.x}]}]\n").is_empty());
    }
}
