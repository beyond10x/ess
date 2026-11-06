//! Adversary pass 1 on story:an-each-write-may-take-repeated-identities-in-list-order (ess/35,
//! beyond10x/ess#748, wave-2026-11-06-ess U3).
//!
//! The pass found that where one entity is written by more than one `each:` entry over one list
//! (the ess/27 shape: one entry per verdict), the interpreter walks the `affects:` entries in the
//! order written and, inside each entry, the elements its `where:` selects, so for a repeated
//! identity the two orders differ and a later element was applied before an earlier one.
//!
//! story:an-each-write-may-take-repeated-identities-in-list-order, fix round: list order holds
//! inside one entry only, so `repeated: last_wins` is refused, fail closed, as
//! `conflicting_declaration` at the entry's `each.repeated` where another `each:` entry of the
//! same outcome writes the same entity. Both cases below assert that refusal.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const STORY: &str = "story:an-each-write-may-take-repeated-identities-in-list-order";

/// What assembling `text` refuses; the model must be refused.
fn refusals(text: &str) -> ValidationErrors {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    match Specification::assemble([(Source::new("model.yaml"), raw)]) {
        Ok(_) => panic!(
            "{STORY}: `repeated: last_wins` beside another `each:` entry over one entity is \
             refused, fail closed:\n{text}"
        ),
        Err(errors) => errors,
    }
}

/// Every entry in `entries` of `branch` is refused at its `each.repeated` as
/// `conflicting_declaration`, naming the key, and nothing else is refused.
fn assert_refused_at(errors: &ValidationErrors, branch: &str, entries: &[usize]) {
    for entry in entries {
        let at = format!("{branch}.affects[{entry}].each.repeated");
        assert!(
            errors.as_slice().iter().any(|error| {
                error.code == ValidationCode::ConflictingDeclaration
                    && error.location.ends_with(&at)
                    && error.message.contains("repeated: last_wins")
            }),
            "{STORY}: `repeated: last_wins` is refused at `{at}` as conflicting_declaration, \
             since list order holds inside one entry only:\n{errors}"
        );
    }
    assert_eq!(
        errors.len(),
        entries.len(),
        "{STORY}: one refusal per repeated entry, and no other:\n{errors}"
    );
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture holds:\n{from}");
    text.replacen(from, to, 1)
}

// ---- three moving entries over one list (the ess/27 `each-moves.yaml` fixture) -------------------

mod moves {
    use super::*;

    const EACH: &str = include_str!("fixtures/each-moves.yaml");
    const DUPLICATED: &str = "      - name: duplicated\n        when: {not: {distinct: {in: answers, as: a, by: a.proposal_id}}}\n        error: perrow.batch.Duplicate\n";

    /// The fixture at ess/35, its `distinct:` refusal dropped and `repeated: last_wins` on each of
    /// its three entries.
    fn repeated() -> String {
        let mut text = edit(EACH, "format: ess/27", "format: ess/35");
        text = edit(&text, DUPLICATED, "");
        for verdict in ["applied", "refused", "invalid"] {
            text = edit(
                &text,
                &format!("where: a.verdict == {verdict}}}"),
                &format!("where: a.verdict == {verdict}, repeated: last_wins}}"),
            );
        }
        text
    }

    /// A list naming one proposal twice, `refused` first and `applied` second, would end `Applied`
    /// walking the entries and `Refused` in list order. The three entries over `Proposal` are
    /// refused instead (story:an-each-write-may-take-repeated-identities-in-list-order).
    #[test]
    fn a_repeated_identity_across_two_moving_entries_is_refused() {
        let errors = refusals(&repeated());
        assert_refused_at(&errors, "Apply.outcomes.applied", &[0, 1, 2]);
    }
}

// ---- two updating entries over one list ----------------------------------------------------------

mod upserts {
    use super::*;

    const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
    const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
    const ENTRY: &str = "          - entity: demo.feed.SeenDocument\n            each: {in: input.applied, as: doc}\n            instance: doc.document_id\n            sets: {source_id: input.source_id, content_hash: doc.content_hash, revision: doc.revision}\n";

    /// `set-each.yaml` at ess/35 with a `kind` member on the element and its one entry split in
    /// two, one per kind, both `repeated: last_wins` and writing the same fields.
    fn repeated() -> String {
        let mut text = edit(EACH, "format: ess/23", "format: ess/35");
        text = edit(&text, DISTINCT, "");
        text = edit(
            &text,
            "  - name: demo.feed.AppliedDocument\n",
            "  - {name: demo.feed.Kind, kind: enum, variants: [early, late]}\n  - name: demo.feed.AppliedDocument\n",
        );
        text = edit(
            &text,
            "      - {name: revision, type: Integer}\nentities:",
            "      - {name: revision, type: Integer}\n      - {name: kind, type: demo.feed.Kind}\nentities:",
        );
        let entry = |kind: &str| {
            ENTRY.replace(
                "each: {in: input.applied, as: doc}",
                &format!("each: {{in: input.applied, as: doc, where: doc.kind == {kind}, repeated: last_wins}}"),
            )
        };
        edit(
            &text,
            ENTRY,
            &format!("{}{}", entry("early"), entry("late")),
        )
    }

    /// A list naming one document twice, a `late` element first and an `early` one last, would
    /// keep the first element's values walking the entries and the last's in list order. Both
    /// entries over `SeenDocument` are refused instead
    /// (story:an-each-write-may-take-repeated-identities-in-list-order).
    #[test]
    fn a_repeated_identity_across_two_updating_entries_is_refused() {
        let errors = refusals(&repeated());
        assert_refused_at(&errors, "RunSource.outcomes.ran", &[0, 1]);
    }
}
