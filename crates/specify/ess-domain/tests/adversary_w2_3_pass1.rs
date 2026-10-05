//! Adversary pass 1 on W2-3 (beyond10x/ess#448 and #426b).
//!
//! #448's decision: "Every unparsable predicate in a file is reported with its declaration path,
//! line and code, and the file's other semantic refusals are still reported", and a declaration
//! whose predicate failed "does not cascade into misleading secondary refusals". The diagnostics
//! reference now says a predicate that does not parse does not stop the file. These cases drive
//! the assembly against those sentences at positions and neighbours the unit's own cases do not
//! reach.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;

const EVERY: &str =
    include_str!("../../../edge/ess-cli/tests/fixtures/parse-refusals/every-position.yaml");
const SET_EFFECTS: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");

/// Every refusal of `text`, as `(code, location, message)`; empty when it validates. A file the
/// reader stops is a failure of the case, with the reader's sentence.
fn refusals(text: &str) -> Vec<(ValidationCode, String, String)> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| {
        panic!("the reader stopped the file, so nothing beside it is reported: {error}")
    });
    match Specification::assemble([(Source::new("probe.yaml"), raw)]) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .as_slice()
            .iter()
            .map(|error| (error.code, error.location.clone(), error.message.clone()))
            .collect(),
    }
}

fn listed(found: &[(ValidationCode, String, String)]) -> String {
    found
        .iter()
        .map(|(code, location, message)| format!("{code:?} {location}: {message}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the fixture");
    text.replacen(from, to, 1)
}

fn validates(text: &str) {
    let found = refusals(text);
    assert_eq!(found.len(), 0, "the premise validates:\n{}", listed(&found));
}

/// `EVERY` with `view` added as the last view.
fn with_view(text: &str, view: &str) -> String {
    replaced(text, "bindings:\n", &format!("{view}bindings:\n"))
}

const COUNTED: &str = "  - name: probe.item.Counted
    source: probe.item.Item
    consistency: read_your_writes
    group_by: [label]
    fields:
      - {name: label, type: String}
      - {name: total, type: Integer, aggregate: {count: {}, where: label != \"\"}}
";

const BAD_FILTER: (&str, &str) = ("    filter: label != \"\"", "    filter: label ~= \"\"");

/// The fit review lists aggregate `where` among the positions that move ("Every predicate
/// position moves together: … aggregate `where`"), and `diagnostics.md` now says a predicate that
/// does not parse does not stop the file.
#[test]
fn adv_aggregate_where_that_does_not_parse_does_not_stop_the_file() {
    let valid = with_view(EVERY, COUNTED);
    validates(&valid);
    let broken = replaced(
        &replaced(&valid, BAD_FILTER.0, BAD_FILTER.1),
        "where: label != \"\"}}",
        "where: label ~= \"\"}}",
    );
    let found = refusals(&broken);
    let unparsed: Vec<_> = found
        .iter()
        .filter(|(code, _, _)| *code == ValidationCode::UnparsablePredicate)
        .collect();
    assert_eq!(
        unparsed.len(),
        2,
        "the filter and the measure's `where`, each at its view:\n{}",
        listed(&found)
    );
}

/// An outcome's `instances: {where}` is a predicate position like any other.
#[test]
fn adv_instances_where_that_does_not_parse_does_not_stop_the_file() {
    validates(SET_EFFECTS);
    let broken = replaced(
        SET_EFFECTS,
        "instances: {where: team == input.team}",
        "instances: {where: team ~= input.team}",
    );
    let found = refusals(&broken);
    assert!(
        found.iter().any(
            |(code, location, _)| *code == ValidationCode::UnparsablePredicate
                && location.contains("instances")
        ),
        "refused at the outcome's `instances.where`:\n{}",
        listed(&found)
    );
}

/// An outcome's `affects: [{where}]` is a predicate position like any other.
#[test]
fn adv_affects_where_that_does_not_parse_does_not_stop_the_file() {
    validates(SET_EFFECTS);
    let broken = replaced(
        SET_EFFECTS,
        "where: team == subject.team",
        "where: team ~= subject.team",
    );
    let found = refusals(&broken);
    assert!(
        found.iter().any(
            |(code, location, _)| *code == ValidationCode::UnparsablePredicate
                && location.contains("affects")
        ),
        "refused at the outcome's `affects[0].where`:\n{}",
        listed(&found)
    );
}

const READER: &str = "components:
  - component: reader
    reached_by: command_line
    owns:
      domains: [probe.item]
    accepts:
      commands: [probe.item.Create, probe.item.Close]
    cli:
      binary: reader
      commands: [probe.item.Create, probe.item.Close]
      views: [probe.item.Labelled]
";

/// A view withheld because its filter does not parse is still declared: a command tree that
/// reads it is not refused a second time as reading a view no domain projects.
#[test]
fn adv_withheld_view_is_not_refused_again_by_a_command_tree() {
    let valid = format!("{EVERY}{READER}");
    validates(&valid);
    let found = refusals(&replaced(&valid, BAD_FILTER.0, BAD_FILTER.1));
    assert_eq!(
        found.len(),
        1,
        "only the filter's refusal; the tree reading the view is not refused for it:\n{}",
        listed(&found)
    );
}

/// The refusals at `found` whose location is `prefix` or under it.
fn under<'a>(
    found: &'a [(ValidationCode, String, String)],
    prefix: &str,
) -> Vec<&'a (ValidationCode, String, String)> {
    found
        .iter()
        .filter(|(_, location, _)| location.starts_with(prefix))
        .collect()
}

/// Story 448's recorded decision (correction round 1): a declaration with an unparsable predicate
/// is withheld whole, so it yields only the parse refusal, while a refusal in a different
/// declaration of the same file is still reported. Here the declaration is a command whose guard
/// does not parse, and the other is a sibling command refused on its own.
#[test]
fn adv_unparsable_guard_does_not_hide_a_sibling_branchs_refusal() {
    let other = (
        "        when_subject: {predicate: label == \"\"}",
        "        when_subject: {predicate: nosuch == \"\"}",
    );
    let alone = refusals(&replaced(EVERY, other.0, other.1));
    assert_ne!(
        alone.len(),
        0,
        "the premise: the other command is refused on its own"
    );
    assert!(
        alone.iter().all(
            |(code, location, _)| *code != ValidationCode::UnparsablePredicate
                && location.starts_with("command.probe.item.Close")
        ),
        "the premise: the other command is refused on its own, and not as a parse refusal:\n{}",
        listed(&alone)
    );
    let both = replaced(
        &replaced(EVERY, other.0, other.1),
        "        when: input.label == \"\"",
        "        when: input.label ~= \"\"",
    );
    let found = refusals(&both);
    let withheld = under(&found, "command.probe.item.Create");
    assert_eq!(
        withheld.len(),
        1,
        "the command whose guard does not parse yields only that refusal:\n{}",
        listed(&found)
    );
    assert_eq!(withheld[0].0, ValidationCode::UnparsablePredicate);
    assert_eq!(
        withheld[0].1,
        "command.probe.item.Create.outcomes.empty-label.when"
    );
    for (code, location, message) in &alone {
        assert!(
            found
                .iter()
                .any(|(c, l, m)| c == code && l == location && m == message),
            "the other command's refusal `{code:?} {location}` is still reported beside the parse \
             refusal:\n{}",
            listed(&found)
        );
    }
}

/// A view in the same file that is refused on its own.
const OTHERS: &str = "  - name: probe.item.Others
    source: probe.item.Item
    fields:
      - {name: item_id, type: probe.item.ItemId}
      - {name: nosuch, type: String}
";

/// The same within views: a view whose filter does not parse yields only that refusal, even
/// where it would also be refused for a field its source does not have; a different view's own
/// refusal in the same file is still reported.
#[test]
fn adv_unparsable_filter_does_not_hide_the_views_own_refusal() {
    let field = (
        "      - {name: label, type: String}\n    filter:",
        "      - {name: label, type: String}\n      - {name: nosuch, type: String}\n    filter:",
    );
    let other = with_view(EVERY, OTHERS);
    let alone = refusals(&other);
    assert_ne!(
        alone.len(),
        0,
        "the premise: a view field the source does not have is refused"
    );
    assert!(
        alone
            .iter()
            .all(|(_, location, _)| location.starts_with("view.probe.item.Others")),
        "the premise: only the other view is refused:\n{}",
        listed(&alone)
    );
    let found = refusals(&replaced(
        &replaced(&other, field.0, field.1),
        BAD_FILTER.0,
        BAD_FILTER.1,
    ));
    let withheld = under(&found, "view.probe.item.Labelled");
    assert_eq!(
        withheld.len(),
        1,
        "the view whose filter does not parse yields only that refusal:\n{}",
        listed(&found)
    );
    assert_eq!(withheld[0].0, ValidationCode::UnparsablePredicate);
    assert_eq!(withheld[0].1, "view.probe.item.Labelled.filter");
    for (code, location, message) in &alone {
        assert!(
            found
                .iter()
                .any(|(c, l, m)| c == code && l == location && m == message),
            "the other view's refusal `{code:?} {location}` is still reported beside the parse \
             refusal:\n{}",
            listed(&found)
        );
    }
}

/// #426b: `True` written as an enum variant is refused at the type's `variants`. The type is the
/// declaration that failed; every field, input and guard declared with it is not refused again.
#[test]
fn adv_boolean_variant_is_refused_once() {
    let found = refusals(&replaced(
        EVERY,
        "    variants: [plain, fancy]",
        "    variants: [plain, fancy, True]",
    ));
    assert!(
        found
            .iter()
            .any(|(_, location, _)| location == "types.probe.item.Kind.variants"),
        "{}",
        listed(&found)
    );
    assert_eq!(
        found.len(),
        1,
        "only the variant's refusal; nothing declared with the type is refused for it:\n{}",
        listed(&found)
    );
}

/// A newtype's invariant that does not parse is withheld, not the type, in the assembly too.
#[test]
fn adv_newtype_invariant_that_does_not_parse_is_refused_once() {
    let found = refusals(&replaced(
        EVERY,
        "  - {name: probe.item.OwnerId, kind: newtype, of: Uuid}",
        "  - {name: probe.item.OwnerId, kind: newtype, of: Uuid, invariants: [value ~= x]}",
    ));
    assert_eq!(found.len(), 1, "{}", listed(&found));
    assert_eq!(found[0].0, ValidationCode::UnparsablePredicate);
    assert_eq!(found[0].1, "types.probe.item.OwnerId.invariants[0]");
}
