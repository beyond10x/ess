//! Adversary pass 1 for beyond10x/ess#229: `state`, the related row's held lifecycle state, in a
//! `when_related` predicate (`ess/20`).
//!
//! Attacks the domain side: `state` combined with stored fields in `all`/`any`/`not`, the `in` form,
//! the format gate on every predicate form, a related row of the command's own entity, a
//! single-state lifecycle, the partition over the related state (overlaps and gaps), and that a
//! `when_subject` `state` predicate in an `ess/20` document keeps meaning what it meant.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");

const NOT_ACCEPTED: &str =
    "        when_related: {via: input.candidate, predicate: state != Accepted}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("release.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn at(text: &str, format: &str) -> String {
    replaced(text, "format: ess/20\n", &format!("format: {format}\n"))
}

/// The release fixture with a stored `channel` on the candidate, set from the proposing input.
fn with_channel() -> String {
    let text = replaced(
        RELEASE,
        "types:\n",
        "types:\n  - {name: demo.release.Channel, kind: enum, variants: [Stable, Beta]}\n",
    );
    // The candidate is the first entity, and `ProposeCandidate` the first command.
    let text = replaced(
        &text,
        "    fields: []\n",
        "    fields:\n      - {name: channel, type: demo.release.Channel}\n",
    );
    let text = replaced(
        &text,
        "    input: []\n",
        "    input:\n      - {name: channel, type: demo.release.Channel}\n",
    );
    replaced(
        &text,
        "        payload: {demo.release.CandidateProposed: {candidate_id: {generated: true}}}\n",
        "        payload: {demo.release.CandidateProposed: {candidate_id: {generated: true}}}\n        sets: {channel: input.channel}\n",
    )
}

fn guarded(text: &str, refusal: &str) -> String {
    replaced(
        text,
        NOT_ACCEPTED,
        &format!("        when_related:\n          via: input.candidate\n          predicate: {refusal}\n"),
    )
}

/// The refusal and the move both guarded on the related row.
fn both(text: &str, refusal: &str, publish: &str) -> String {
    replaced(
        &guarded(text, refusal),
        "      - name: published\n",
        &format!(
            "      - name: published\n        when_related:\n          via: input.candidate\n          predicate: {publish}\n"
        ),
    )
}

#[test]
fn adversary_229_state_combined_with_a_stored_field_validates_under_ess_20() {
    let base = with_channel();
    for refusal in [
        "{all: [state != Accepted, channel == Beta]}",
        "{any: [state != Accepted, channel == Beta]}",
        "{not: {all: [state == Accepted, channel == Stable]}}",
        "{state: {in: [Proposed]}}",
        "{not: {state: {in: [Accepted]}}}",
    ] {
        accepted(&guarded(&base, refusal));
    }
}

#[test]
fn adversary_229_every_predicate_form_reading_state_is_refused_below_ess_20_naming_it() {
    let base = with_channel();
    for refusal in [
        "{all: [state != Accepted, channel == Beta]}",
        "{any: [channel == Beta, state != Accepted]}",
        "{not: state == Accepted}",
        "{state: {in: [Proposed]}}",
    ] {
        for format in ["ess/19", "ess/18"] {
            let errors = refused(&at(&guarded(&base, refusal), format));
            assert!(
                has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/20"),
                "{refusal} at {format}: {errors}"
            );
            assert!(
                !errors.contains(ValidationCode::UnobservableFact),
                "{refusal} at {format}: the header is wrong, not the document: {errors}"
            );
        }
    }
    // A stored-field-only predicate keeps validating under ess/18: the gate is on `state` alone.
    accepted(&at(&guarded(&base, "channel == Beta"), "ess/18"));
}

#[test]
fn adversary_229_a_partition_over_state_and_a_stored_field_is_exhaustive_or_names_the_gap() {
    let base = with_channel();
    // Exactly complementary: refuse unless accepted on the stable channel.
    accepted(&both(
        &base,
        "{any: [state != Accepted, channel == Beta]}",
        "{all: [state == Accepted, channel == Stable]}",
    ));
    // A gap: an accepted beta candidate, and a proposed stable one, are answered by nothing.
    let errors = refused(&both(
        &base,
        "{all: [state == Proposed, channel == Beta]}",
        "{all: [state == Accepted, channel == Stable]}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::NonExhaustiveBranches,
            "state = Accepted"
        ),
        "{errors}"
    );
    assert!(
        has(&errors, ValidationCode::NonExhaustiveBranches, "Beta"),
        "{errors}"
    );
    // An overlap: an accepted beta candidate is answered by both.
    let errors = refused(&both(
        &base,
        "{any: [state != Accepted, channel == Beta]}",
        "state == Accepted",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "state = Accepted"
        ),
        "{errors}"
    );
}

#[test]
fn adversary_229_state_in_versus_equality_partition_the_same() {
    // `state in [Proposed]` refusing and `state == Accepted` moving cover the two states once.
    accepted(&both(
        RELEASE,
        "{state: {in: [Proposed]}}",
        "state == Accepted",
    ));
    // `in` naming both states overlaps the move on `Accepted`.
    let errors = refused(&both(
        RELEASE,
        "{state: {in: [Proposed, Accepted]}}",
        "state == Accepted",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "state = Accepted"
        ),
        "{errors}"
    );
    // A state of another lifecycle inside `in` is refused naming it.
    let errors = refused(&guarded(RELEASE, "{state: {in: [Proposed, Published]}}"));
    assert!(errors.to_string().contains("Published"), "{errors}");
}

#[test]
fn adversary_229_a_single_state_related_lifecycle_is_read() {
    // The candidate holds one state only: `state == Proposed` is always true, `!=` never.
    let single = replaced(
        &replaced(
            RELEASE,
            "      states: [Proposed, Accepted]\n      terminal: [Accepted]\n      transitions:\n        - {name: accept, from: [Proposed], to: Accepted}\n",
            "      states: [Proposed]\n      terminal: [Proposed]\n",
        ),
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n      - name: accepted\n        moves: demo.release.Candidate.accept\n        instance: candidate_id\n        emits: [demo.release.CandidateAccepted]\n        payload: {demo.release.CandidateAccepted: {candidate_id: input.candidate_id}}\n",
        "",
    );
    let single = replaced(&single, "      - demo.release.AcceptCandidate\n", "");
    // Both branches over the one state overlap on it.
    let errors = refused(&both(&single, "state == Proposed", "state == Proposed"));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "state = Proposed"
        ),
        "{errors}"
    );
    // A state the one-state lifecycle does not have is refused naming it.
    let errors = refused(&guarded(&single, "state != Accepted"));
    assert!(errors.to_string().contains("Accepted"), "{errors}");
}

/// A move of a candidate guarded by the state of another candidate: `AcceptCandidate` takes a
/// `parent` that must itself be accepted.
fn own_entity(refusal: &str) -> String {
    replaced(
        RELEASE,
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
        &format!(
            "  - name: demo.release.AcceptCandidate\n    input:\n      - {{name: candidate_id, type: demo.release.CandidateId}}\n      - {{name: parent, type: demo.release.CandidateId}}\n    outcomes:\n      - name: no-parent\n        when_related: {{via: input.parent, exists: false}}\n        error: demo.release.NoCandidate\n      - name: parent-not-accepted\n        when_related: {{via: input.parent, predicate: {refusal}}}\n        error: demo.release.CandidateNotAccepted\n"
        ),
    )
}

#[test]
fn adversary_229_state_of_a_related_row_of_the_commands_own_entity_is_read() {
    let spec = accepted(&own_entity("state != Accepted"));
    let command = &spec.commands()[&"demo.release.AcceptCandidate".parse().unwrap()];
    assert!(command
        .outcomes
        .iter()
        .any(|outcome| outcome.name.as_str() == "parent-not-accepted"));
    // Below ess/20 the related `state` is refused naming the format, even though the subject's
    // own `state` is readable in a `when_subject` predicate from ess/18.
    let errors = refused(&at(&own_entity("state != Accepted"), "ess/19"));
    assert!(
        has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/20"),
        "{errors}"
    );
    // The related row's states are the candidate's: a release state is refused.
    let errors = refused(&own_entity("state != Published"));
    assert!(errors.to_string().contains("Published"), "{errors}");
}

/// `RetractRelease`: refused for a release that is no longer a draft, read through `when_subject`.
fn retracting(format: &str) -> String {
    let text = replaced(
        RELEASE,
        "      states: [Draft, Published]\n      terminal: [Published]\n      transitions:\n        - {name: publish, from: [Draft], to: Published}\n",
        "      states: [Draft, Published, Retracted]\n      terminal: [Published, Retracted]\n      transitions:\n        - {name: publish, from: [Draft], to: Published}\n        - {name: retract, from: [Draft], to: Retracted}\n",
    );
    let text = replaced(
        &text,
        "      - demo.release.PublishRelease\n",
        "      - demo.release.PublishRelease\n      - demo.release.RetractRelease\n",
    );
    let text = replaced(
        &text,
        "views:\n",
        "  - name: demo.release.RetractRelease\n    input:\n      - {name: release_id, type: demo.release.ReleaseId}\n    outcomes:\n      - name: already-published\n        when_subject:\n          predicate: state == Published\n        error: demo.release.CandidateNotAccepted\n      - name: retracted\n        moves: demo.release.Release.retract\n        instance: release_id\n        emits: [demo.release.ReleaseDrafted]\n        payload: {demo.release.ReleaseDrafted: {release_id: input.release_id}}\nviews:\n",
    );
    if format == "ess/20" {
        text
    } else {
        at(&text, format)
    }
}

#[test]
fn adversary_229_a_when_subject_state_predicate_still_validates_at_ess_20_and_18() {
    // At ess/20 with the related `state` beside it.
    accepted(&retracting("ess/20"));
    // Below ess/20 the related branch is taken out, so only `when_subject` reads `state`.
    for format in ["ess/19", "ess/18"] {
        let text = replaced(
            &retracting(format),
            &format!("      - name: not-accepted\n{NOT_ACCEPTED}        error: demo.release.CandidateNotAccepted\n"),
            "",
        );
        accepted(&text);
    }
}
