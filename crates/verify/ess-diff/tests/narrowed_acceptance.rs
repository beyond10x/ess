//! A change that narrows what an existing caller may send is breaking for callers
//! (<https://github.com/beyond10x/ess/issues/514>).
//!
//! Three revisions of `demo.tok.Present` from the report: a refusal added, a required input added,
//! and a refusal's guard widened so it now refuses an input it let through. Each was `unknown` for
//! callers and readers, so `--fail-on breaking` passed it. From `ess-diff/17` the classification
//! records which of the three it is (`narrows`) beside the answer, and a reader re-derives the
//! answer from it. An added accepting outcome, an added `Optional` input and a guard change the
//! satisfiability check cannot decide keep their earlier answer, format and bytes.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, Compatibility, EssDelta, FailOn, Gate, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const V1: &str = include_str!("fixtures/narrowed/v1.yaml");
const ADDED_REFUSAL: &str = include_str!("fixtures/narrowed/added-refusal.yaml");
const REQUIRED_INPUT: &str = include_str!("fixtures/narrowed/required-input.yaml");
const WIDENED_GUARD: &str = include_str!("fixtures/narrowed/widened-guard.yaml");

const REFUSAL: &str = "command/demo.tok.Present/outcome-added/admin-scope";
const INPUT: &str = "command/demo.tok.Present/input-added/audience";
const GUARD: &str = "command/demo.tok.Present/outcome-condition-changed/empty-scope";

use Compatibility::{Breaking as B, Compatible as C, Unknown as U};

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

/// `text` with `from` replaced by `to`, which must occur in it.
fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` in\n{text}");
    text.replacen(from, to, 1)
}

fn answers(delta: &EssDelta, id: &str) -> [Compatibility; 3] {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    [
        compatibility.callers(),
        compatibility.readers(),
        compatibility.history(),
    ]
}

/// The written change with this id, as the document carries it.
fn written(delta: &EssDelta, id: &str) -> Value {
    let document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` in {document}"))
        .clone()
}

fn failing(delta: &EssDelta, threshold: FailOn) -> Vec<String> {
    Gate::new(threshold)
        .judge(delta, None)
        .unwrap()
        .failing()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// The issue's three revisions, each with the change it holds and the `narrows` it records.
fn reported() -> [(&'static str, &'static str, &'static str); 3] {
    [
        (ADDED_REFUSAL, REFUSAL, "refusal-added"),
        (REQUIRED_INPUT, INPUT, "required-input"),
        (WIDENED_GUARD, GUARD, "refusal-widened"),
    ]
}

#[test]
fn each_reported_revision_is_breaking_for_callers_and_fails_the_breaking_gate() {
    for (after, id, narrows) in reported() {
        let delta = classified(&ir(V1), &ir(after)).unwrap();
        let json = delta.to_canonical_json();
        assert_eq!(answers(&delta, id), [B, U, C], "{id}\n{json}");
        assert_eq!(
            failing(&delta, FailOn::Breaking),
            vec![id.to_owned()],
            "{json}"
        );
        assert_eq!(
            written(&delta, id)["compatibility"]["narrows"],
            Value::from(narrows),
            "{json}"
        );
        assert_eq!(delta.format.to_string(), "ess-diff/17", "{json}");
        let text = ess_diff::render::text(&delta);
        assert!(text.contains("breaking for callers"), "{text}");
    }
}

#[test]
fn an_added_accepting_outcome_stays_unknown() {
    let after = edited(
        V1,
        "      - name: accepted\n",
        "      - name: guest\n        when: scope == \"guest\"\n        creates: demo.tok.Token\n        \
         instance: token_id\n        sets: {scope: input.scope}\n        emits: [demo.tok.Presented]\n        \
         payload:\n          demo.tok.Presented: {token_id: input.token_id}\n      - name: accepted\n",
    );
    let delta = classified(&ir(V1), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    let id = "command/demo.tok.Present/outcome-added/guest";
    assert_eq!(answers(&delta, id), [U, U, C], "{json}");
    assert!(
        written(&delta, id)["compatibility"]
            .get("narrows")
            .is_none(),
        "{json}"
    );
    assert_eq!(delta.format.to_string(), "ess-diff/14", "{json}");
}

#[test]
fn an_added_optional_input_keeps_its_rating_and_format() {
    let after = edited(
        REQUIRED_INPUT,
        "{name: audience, type: String}",
        "{name: audience, type: Optional<String>}",
    );
    let delta = classified(&ir(V1), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(answers(&delta, INPUT), [U, U, C], "{json}");
    assert!(failing(&delta, FailOn::Breaking).is_empty(), "{json}");
    assert_eq!(delta.format.to_string(), "ess-diff/14", "{json}");
}

#[test]
fn a_refusal_guard_narrowed_back_stays_unknown() {
    let delta = classified(&ir(WIDENED_GUARD), &ir(V1)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(answers(&delta, GUARD), [U, U, C], "{json}");
    assert_eq!(delta.format.to_string(), "ess-diff/14", "{json}");
}

#[test]
fn a_widened_guard_whose_added_inputs_an_earlier_outcome_already_refused_stays_unknown() {
    // `scope == "none"` was refused by `none-scope` before `empty-scope` was reached, and still
    // is: no input moves.
    let earlier = "      - name: none-scope\n        when: scope == \"none\"\n        \
                   error: demo.tok.Rejected\n      - name: empty-scope\n";
    let before = edited(V1, "      - name: empty-scope\n", earlier);
    let after = edited(WIDENED_GUARD, "      - name: empty-scope\n", earlier);
    let delta = classified(&ir(&before), &ir(&after)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(answers(&delta, GUARD), [U, U, C], "{json}");
}

#[test]
fn a_widened_guard_on_an_accepting_outcome_stays_unknown() {
    let accepting = |guard: &str| {
        edited(
            V1,
            "      - name: accepted\n",
            &format!(
                "      - name: guest\n        when: {guard}\n        creates: demo.tok.Token\n        \
                 instance: token_id\n        sets: {{scope: input.scope}}\n        \
                 emits: [demo.tok.Presented]\n        payload:\n          \
                 demo.tok.Presented: {{token_id: input.token_id}}\n      - name: accepted\n"
            ),
        )
    };
    let delta = classified(
        &ir(&accepting("scope == \"guest\"")),
        &ir(&accepting(
            "{any: [scope == \"guest\", scope == \"visitor\"]}",
        )),
    )
    .unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(
        answers(
            &delta,
            "command/demo.tok.Present/outcome-condition-changed/guest"
        ),
        [U, U, C],
        "{json}"
    );
}

#[test]
fn a_guard_change_the_satisfiability_check_cannot_decide_stays_unknown() {
    // An ordering is not decided: its satisfying values lie between any finite representatives.
    let levelled = |guard: &str| {
        edited(
            &edited(
                V1,
                "      - {name: scope, type: String}\n    outcomes:",
                "      - {name: scope, type: String}\n      - {name: level, type: Integer}\n    outcomes:",
            ),
            "when: scope == \"\"",
            &format!("when: {guard}"),
        )
    };
    let delta = classified(&ir(&levelled("level > 3")), &ir(&levelled("level > 2"))).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(answers(&delta, GUARD), [U, U, C], "{json}");
}

#[test]
fn the_narrowing_is_written_as_ess_diff_17_and_read_back() {
    for (after, id, _) in reported() {
        let delta = classified(&ir(V1), &ir(after)).unwrap();
        let json = delta.to_canonical_json();
        let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "{id} reads back");
        assert!(
            delta
                .to_canonical_json_for("ess-diff/16".parse().unwrap())
                .is_err(),
            "ess-diff/16 cannot carry `narrows` ({id})"
        );
    }
}

/// Edits the written delta of `V1` → `after` and reads the result back.
fn reread(after: &str, edit: impl FnOnce(&mut Value)) -> Result<EssDelta, String> {
    let delta = classified(&ir(V1), &ir(after)).unwrap();
    let mut document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    edit(&mut document);
    let raw: RawEssDelta = serde_json::from_value(document).map_err(|error| error.to_string())?;
    EssDelta::try_from(raw).map_err(|errors| errors.to_string())
}

fn change_mut<'a>(document: &'a mut Value, id: &str) -> &'a mut Value {
    document["changes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|change| change["id"] == id)
        .unwrap()
}

#[test]
fn a_reader_refuses_narrows_below_ess_diff_17() {
    for (after, id, _) in reported() {
        let refused = reread(after, |document| {
            document["format"] = Value::from("ess-diff/16");
        })
        .expect_err("an `ess-diff/16` document carries no `narrows`");
        assert!(
            refused.contains("unsupported_format_version"),
            "{id}: {refused}"
        );
    }
}

#[test]
fn a_reader_refuses_an_answer_or_a_narrows_the_change_does_not_derive() {
    // Without `narrows` the change derives unknown, not breaking.
    for (after, id, _) in reported() {
        let refused = reread(after, |document| {
            change_mut(document, id)["compatibility"]
                .as_object_mut()
                .unwrap()
                .remove("narrows");
        })
        .expect_err("unknown is derived without `narrows`");
        assert!(
            refused.contains("conflicting_declaration"),
            "{id}: {refused}"
        );
    }
    // A `narrows` naming another change kind.
    for (after, id, narrows) in reported() {
        let other = if narrows == "refusal-added" {
            "required-input"
        } else {
            "refusal-added"
        };
        let refused = reread(after, |document| {
            change_mut(document, id)["compatibility"]["narrows"] = Value::from(other);
        })
        .expect_err("a narrowing belongs to one change kind");
        assert!(
            refused.contains("conflicting_declaration"),
            "{id}: {refused}"
        );
    }
    // `required-input` on an input whose written type is `Optional`.
    let refused = reread(REQUIRED_INPUT, |document| {
        change_mut(document, INPUT)["change"]["changed"]["type_ref"] =
            Value::from("Optional<String>");
    });
    assert!(
        refused
            .as_ref()
            .is_err_and(|refused| refused.contains("conflicting_declaration")),
        "{refused:?}"
    );
}

#[test]
fn an_ess_diff_16_document_rating_an_added_required_input_unknown_still_reads_back() {
    // What a build before `ess-diff/17` wrote for the same pair: no `narrows`, unknown for callers.
    let delta = classified(&ir(V1), &ir(REQUIRED_INPUT)).unwrap();
    let mut document: Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    document["format"] = Value::from("ess-diff/16");
    let compatibility = change_mut(&mut document, INPUT)["compatibility"]
        .as_object_mut()
        .unwrap();
    compatibility.remove("narrows");
    compatibility.insert("verdict".to_owned(), Value::from("unknown"));
    compatibility.insert("callers".to_owned(), Value::from("unknown"));
    let raw: RawEssDelta = serde_json::from_value(document).unwrap();
    let read = EssDelta::try_from(raw).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(read.format.to_string(), "ess-diff/16");
    assert_eq!(answers(&read, INPUT), [U, U, C]);
}

#[test]
fn the_unclassified_delta_keeps_its_format_and_records_no_narrows() {
    for (after, id, _) in reported() {
        let delta = diff(&ir(V1), &ir(after)).unwrap();
        let json = delta.to_canonical_json();
        assert!(delta.compatibility().is_none(), "{id}: {json}");
        assert!(!json.contains("\"narrows\""), "{id}: {json}");
        assert_ne!(delta.format.to_string(), "ess-diff/17", "{id}: {json}");
    }
}
