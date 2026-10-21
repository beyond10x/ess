//! A selector's `where:` says what an absent `Optional` field of the candidate row compares as
//! (`ess/30`, beyond10x/ess#627): `coalesce(enabled, true) == true` selects the rows whose
//! `enabled` is true or absent.
//!
//! The sites are the `where:` of a row-set guard (`when_related: {entity, where, …}`), of a
//! filtered read (`{related: {entity, where, field}}`) and of an ordered head (`{related: {entity,
//! where, order_by, first}}`); the form is `coalesce(<field of the candidate>, <literal>) <op>
//! <literal>`. Below `ess/30` the call keeps its `unsupported_construct` byte for byte, its hint
//! naming `ess/30`: the compact spelling at `ess/29`, the structured `{compare: {left: {expr: …}}}`
//! at `ess/27` and `ess/28`, where the compact one keeps its refusal naming `ess/29`. Every other
//! call in a selector and a `forall:` body keep their refusal, with no hint.

use ess_primitives::error::{ValidationCode, ValidationError};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/coalesce-selector.yaml");

/// The refusal every format below `ess/30` keeps, byte for byte.
const REFUSAL: &str = "`coalesce(enabled, true)` computes a value from `enabled`, and a function \
                       of a fact is admitted only in a command outcome's guards (`when:`, \
                       `when_subject:`, `when_related:`); compare stored values here, or move the \
                       rule into the command's guard";

const SELECTOR: &str = "- coalesce(enabled, true) == true";

fn spec(body: &str) -> Result<ess_domain::Specification, Vec<ValidationError>> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).unwrap_or_else(|e| panic!("{e}"));
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("picking.yaml"), raw)])
        .map_err(|errors| errors.as_slice().to_vec())
}

fn refused(body: &str) -> Vec<ValidationError> {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn at(format: &str, body: &str) -> String {
    assert!(body.contains("format: ess/30"));
    body.replacen("format: ess/30", &format!("format: {format}"), 1)
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

fn rendered(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(|error| format!("{error:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A command reading the one row its selector selects with a filtered read, refusing where the
/// selector selects none or several.
fn with_filtered_read(selector: &str) -> String {
    let text = edit(
        MODEL,
        "  - {name: demo.picking.Empty,",
        "  - {name: demo.picking.Crowded, summary: More than one enabled element waits., fields: []}\n  - {name: demo.picking.Empty,",
    );
    let text = edit(
        &text,
        "  - name: demo.picking.Finished\n",
        "  - name: demo.picking.Peeked\n    fields:\n      - {name: element_key, type: String}\n  - name: demo.picking.Finished\n",
    );
    let selection = format!(
        "          where:\n            all:\n              - slot_id == input.slot_id\n              {selector}\n"
    );
    edit(
        &text,
        "  - name: demo.picking.Finish\n",
        &format!(
            "  - name: demo.picking.Peek
    input:
      - {{name: slot_id, type: String}}
    outcomes:
      - name: none
        when_related:
          entity: demo.picking.Member
{selection}          count: {{eq: 0}}
        error: demo.picking.Empty
      - name: crowded
        when_related:
          entity: demo.picking.Member
{selection}          count: {{gt: 1}}
        error: demo.picking.Crowded
      - name: peeked
        emits: [demo.picking.Peeked]
        payload:
          demo.picking.Peeked:
            element_key:
              related:
                entity: demo.picking.Member
                where:
                  all:
                    - slot_id == input.slot_id
                    {selector}
                field: element_key
  - name: demo.picking.Finish\n"
        ),
    )
}

#[test]
fn a_coalesced_selector_validates_from_ess_30_at_every_selector_site() {
    // A row-set guard's `where:` and an ordered head's.
    spec(MODEL).unwrap_or_else(|errors| panic!("{}", rendered(&errors)));
    // A filtered read's, beside the row-set guards that answer none and several rows.
    let filtered = with_filtered_read(SELECTOR);
    spec(&filtered).unwrap_or_else(|errors| panic!("{}\n{filtered}", rendered(&errors)));
    // Every operator, and a default of the field's type on either side of its value.
    for written in [
        "- coalesce(enabled, false) != false",
        "- coalesce(element_score, 0) >= 10",
        "- coalesce(element_key, \"none\") == \"k\"",
    ] {
        spec(&MODEL.replace(SELECTOR, written))
            .unwrap_or_else(|errors| panic!("{written}: {}", rendered(&errors)));
    }
}

/// The structured spelling of [`SELECTOR`], which `ess/27` and `ess/28` read (beyond10x/ess#620).
const STRUCTURED: &str =
    "- {compare: {left: {expr: \"coalesce(enabled, true)\"}, op: eq, right: true}}";

/// Every error is the unchanged refusal, its hint naming `ess/30`.
fn assert_refused_with_hint(format: &str, errors: &[ValidationError], count: usize) {
    assert_eq!(errors.len(), count, "{format}: {}", rendered(errors));
    for error in errors {
        assert_eq!(
            error.code,
            ValidationCode::UnsupportedConstruct,
            "{error:?}"
        );
        assert_eq!(error.message, REFUSAL, "{format}: the refusal is unchanged");
        let hint = error.hint.as_deref().unwrap_or_default();
        assert!(
            hint.contains("format: ess/30") && hint.contains("absent"),
            "{format}: the hint names ess/30: {error:?}"
        );
    }
}

#[test]
fn below_ess_30_a_coalesced_selector_keeps_its_refusal_its_hint_naming_ess_30() {
    // `ess/29` reads the compact spelling; `ess/27` and `ess/28` the structured one.
    for (format, written) in [
        ("ess/29", SELECTOR),
        ("ess/28", STRUCTURED),
        ("ess/27", STRUCTURED),
    ] {
        let model = at(format, &MODEL.replace(SELECTOR, written));
        assert_refused_with_hint(format, &refused(&model), 4);
        let filtered = at(
            format,
            &with_filtered_read(SELECTOR).replace(SELECTOR, written),
        );
        assert_refused_with_hint(format, &refused(&filtered), 7);
    }
}

/// Below `ess/29` the compact spelling keeps its own refusal, naming `ess/29` and the structured
/// form, with no hint.
#[test]
fn below_ess_29_the_compact_spelling_keeps_its_refusal_naming_ess_29() {
    let errors = refused(&at("ess/28", MODEL));
    assert_eq!(errors.len(), 4, "{}", rendered(&errors));
    for error in &errors {
        assert_eq!(
            error.code,
            ValidationCode::UnsupportedFormatVersion,
            "{error:?}"
        );
        assert!(
            error.message.contains("ess/29") && error.message.contains("{compare: {left: {expr:"),
            "{error:?}"
        );
        assert_eq!(error.hint, None, "{error:?}");
    }
}

/// Text that is no `coalesce(<field of the candidate>, <literal>)` compared with a literal keeps the
/// refusal at `ess/30`, with no hint: another function, a default read from a fact, a fact on the
/// right, a field of the input or of the subject.
#[test]
fn any_other_call_in_a_selector_keeps_its_refusal_without_a_hint() {
    for (written, message) in [
        (
            "- sha256(element_key) == \"k\"",
            "`sha256(element_key)` computes a value from `element_key`",
        ),
        (
            "- coalesce(element_key, \"none\") == input.slot_id",
            "`coalesce(element_key, \"none\")` computes a value from `element_key`",
        ),
        (
            "- coalesce(input.slot_id, \"a\") == \"a\"",
            "`coalesce(input.slot_id, \"a\")` computes a value from `input.slot_id`",
        ),
    ] {
        let errors = refused(&MODEL.replace(SELECTOR, written));
        assert!(
            errors
                .iter()
                .any(|error| error.message.starts_with(message)),
            "{written}: {}",
            rendered(&errors)
        );
        for error in errors
            .iter()
            .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
        {
            assert_eq!(error.hint, None, "{written}: {error:?}");
        }
    }
}

/// A `forall:` body is no selector: it keeps its refusal, with no hint.
#[test]
fn a_forall_body_keeps_its_refusal() {
    let text = edit(
        MODEL,
        "  - name: demo.picking.Finish\n",
        "  - name: demo.picking.Check
    input:
      - {name: slot_id, type: String}
    outcomes:
      - name: all-enabled
        when_related:
          entity: demo.picking.Member
          where: slot_id == input.slot_id
          forall: coalesce(enabled, true) == true
        emits: [demo.picking.Checked]
        payload:
          demo.picking.Checked: {slot_id: input.slot_id}
      - name: not-all
        error: demo.picking.Empty
  - name: demo.picking.Finish\n",
    );
    let text = edit(
        &text,
        "  - name: demo.picking.Finished\n",
        "  - name: demo.picking.Checked\n    fields:\n      - {name: slot_id, type: String}\n  - name: demo.picking.Finished\n",
    );
    assert_one_refusal_without_hint(&refused(&text));
}

/// A set effect's filter is no selector either: it keeps its refusal, with no hint.
#[test]
fn a_set_effect_filter_keeps_its_refusal() {
    let text = edit(
        MODEL,
        "  - name: demo.picking.Finish\n",
        "  - name: demo.picking.Reset
    input:
      - {name: slot_id, type: String}
    outcomes:
      - name: reset
        updates: demo.picking.Member
        instances:
          where:
            all:
              - slot_id == input.slot_id
              - coalesce(enabled, true) == true
        sets: {enabled: false}
        emits: [demo.picking.Cleared]
        payload:
          demo.picking.Cleared: {slot_id: input.slot_id}
  - name: demo.picking.Finish\n",
    );
    let text = edit(
        &text,
        "  - name: demo.picking.Finished\n",
        "  - name: demo.picking.Cleared\n    fields:\n      - {name: slot_id, type: String}\n  - name: demo.picking.Finished\n",
    );
    assert_one_refusal_without_hint(&refused(&text));
}

fn assert_one_refusal_without_hint(errors: &[ValidationError]) {
    let refusals: Vec<&ValidationError> = errors
        .iter()
        .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
        .collect();
    assert_eq!(refusals.len(), 1, "{}", rendered(errors));
    assert_eq!(refusals[0].message, REFUSAL);
    assert_eq!(refusals[0].hint, None);
}
