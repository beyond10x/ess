//! Two errors of one wire name whose fields differ, answered on one status with `{error: error}`
//! (`docs/design/http-presentation-binding.md`, "Status per outcome", `ESS-HTTP-025`).
//!
//! One rule decides whether the wire tells the two bodies apart: one error always writes a field,
//! by wire name and with a type that is not `Optional`, that the other never writes. The status
//! check and the `alike` check both ask it. A pair it decides binds without `alike:` and is refused
//! with it, naming the field; a pair whose fields differ and that it does not decide is refused in
//! both forms with one fact; a pair with identical fields keeps both of today's answers.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const NOTES: &str = include_str!("fixtures/notes.yaml");

/// What both refusals say of a pair whose fields differ and no always-written field decides.
const UNDECIDED: &str =
    "their fields differ and no field one error always writes is one the other never writes, so a \
     body one of them answers can be one the other answers";

/// The forms that validate as written, as both refusals of such a pair name them.
const FORMS: &str = "one error with the optional field, reported by both outcomes, under \
                     `alike:`; or, with the `alike:` group dropped, a field one error always \
                     writes that the other never writes, distinct error wire names, or `envelope`";

/// `notes.yaml` with `negative` reporting an error of wire name `refused` with the fields `first`
/// and `blank` one of the same wire name with the fields `second`, each a list of `{name, type}`
/// flow entries.
fn model(first: &[&str], second: &[&str]) -> EssIr {
    model_with("", first, second)
}

/// [`model`] with `types`, `types:` entries one per line, declared beside `NoteId`.
fn model_with(types: &str, first: &[&str], second: &[&str]) -> EssIr {
    let fields = |fields: &[&str]| {
        if fields.is_empty() {
            String::new()
        } else {
            let list: Vec<String> = fields
                .iter()
                .map(|field| format!("      - {field}\n"))
                .collect();
            format!("    fields:\n{}", list.concat())
        }
    };
    let text = NOTES
        .replace(
            "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
            &format!("  - {{name: demo.notes.NoteId, kind: newtype, of: String}}\n{types}"),
        )
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            &format!(
                "  - name: demo.notes.Refused\n    naming: {{wire: refused}}\n{}  - name: demo.notes.Declined\n    naming: {{wire: refused}}\n{}",
                fields(first),
                fields(second)
            ),
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    assert_ne!(text, NOTES);
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("notes.yaml", &text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/derived/commands/{}", handle.name().local()),
            });
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

/// The route of `CreateNote` answering `negative` and `blank` on `409` with `{error: error}`, and
/// the other errors under `errors`, with `alike` as written.
fn bound(ir: &EssIr, alike: bool) -> Result<HttpIr, HttpDiagnostics> {
    let group = if alike {
        "    alike: [[negative, blank]]\n"
    } else {
        ""
    };
    bound_keys(
        ir,
        &format!("    outcomes:\n      negative: {{status: 409, body: {{error: error}}}}\n      blank: {{status: 409, body: {{error: error}}}}\n      too-large: {{status: 422, body: none}}\n{group}"),
    )
}

/// The route of `CreateNote` with `keys` beside its path.
fn bound_keys(ir: &EssIr, keys: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n{keys}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    compile(&spec, ir, &derived(ir))
}

fn located(result: &Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn worded(result: &Result<HttpIr, HttpDiagnostics>) -> String {
    match result {
        Ok(http) => format!("compiled:\n{}", http.to_canonical_json()),
        Err(diagnostics) => diagnostics.to_string(),
    }
}

const MESSAGE: &str = "{name: message, type: String}";
const DETAILS: &str = "{name: details, type: String}";

#[test]
fn a_pair_one_always_written_field_decides_binds_without_alike() {
    let ir = model(&[MESSAGE], &[MESSAGE, DETAILS]);
    let result = bound(&ir, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
    // Either error may be the one that writes the field.
    let ir = model(&[MESSAGE, DETAILS], &[MESSAGE]);
    let result = bound(&ir, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
    // The field is another one's, not an extra: each writes one the other never writes.
    let ir = model(
        &[MESSAGE, "{name: reason, type: String}"],
        &[MESSAGE, DETAILS],
    );
    let result = bound(&ir, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
    // A field is told by its wire name: `details` under another name is not the field `details`.
    let ir = model(
        &[
            MESSAGE,
            "{name: details, type: String, naming: {wire: why}}",
        ],
        &[MESSAGE, DETAILS],
    );
    let result = bound(&ir, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
}

#[test]
fn with_alike_that_pair_is_refused_naming_the_deciding_field() {
    for (first, second) in [
        (vec![MESSAGE], vec![MESSAGE, DETAILS]),
        (vec![MESSAGE, DETAILS], vec![MESSAGE]),
    ] {
        let ir = model(&first, &second);
        let result = bound(&ir, true);
        assert_eq!(
            located(&result),
            [("ESS-HTTP-025", "routes[0].alike".to_owned())],
            "{}",
            worded(&result)
        );
        let text = worded(&result);
        assert!(
            text.contains("`details`") && text.contains("tells them apart"),
            "the message names the deciding field:\n{text}"
        );
    }
}

/// An optional field only, a type only, an order only: an answer without the field, or with either
/// type, fits both bodies, so neither form holds.
#[test]
fn a_pair_no_always_written_field_decides_is_refused_in_both_forms_with_one_fact() {
    for (first, second) in [
        // An optional field only.
        (
            vec![MESSAGE],
            vec![MESSAGE, "{name: details, type: Optional<String>}"],
        ),
        // The same field optional in one.
        (
            vec![DETAILS],
            vec!["{name: details, type: Optional<String>}"],
        ),
        // A type only.
        (vec![MESSAGE], vec!["{name: message, type: Integer}"]),
        // An order only.
        (vec![MESSAGE, DETAILS], vec![DETAILS, MESSAGE]),
    ] {
        let ir = model(&first, &second);
        let without = bound(&ir, false);
        assert_eq!(
            located(&without),
            [("ESS-HTTP-025", "routes[0].outcomes.negative".to_owned())],
            "{first:?} against {second:?}: {}",
            worded(&without)
        );
        let with = bound(&ir, true);
        assert_eq!(
            located(&with),
            [("ESS-HTTP-025", "routes[0].alike".to_owned())],
            "{first:?} against {second:?}: {}",
            worded(&with)
        );
        for text in [worded(&without), worded(&with)] {
            assert!(
                text.contains(UNDECIDED) && text.contains(FORMS),
                "{first:?} against {second:?}: the refusal states the fact and the forms:\n{text}"
            );
        }
    }
}

#[test]
fn a_pair_with_identical_fields_keeps_its_two_answers() {
    for fields in [vec![], vec![MESSAGE]] {
        let ir = model(&fields, &fields);
        let without = bound(&ir, false);
        assert_eq!(
            located(&without),
            [("ESS-HTTP-025", "routes[0].outcomes.negative".to_owned())],
            "{}",
            worded(&without)
        );
        assert!(
            !worded(&without).contains(UNDECIDED),
            "identical fields are not a difference no field decides:\n{}",
            worded(&without)
        );
        let with = bound(&ir, true);
        assert_eq!(located(&with), [], "{}", worded(&with));
    }
}

#[test]
fn the_status_refusal_names_the_field_rule_among_the_forms_that_tell_outcomes_apart() {
    let ir = model(&[MESSAGE], &[MESSAGE]);
    let text = worded(&bound(&ir, false));
    assert!(
        text.contains(
            "by `{<member>: error}` on one member with distinct error wire names or with one \
             error always writing a field the other never writes"
        ),
        "{text}"
    );
}

const MAYBE: &str = "  - {name: demo.notes.MaybeText, kind: newtype, of: Optional<String>}\n";
const DEEPER: &str = "  - {name: demo.notes.Deeper, kind: newtype, of: demo.notes.MaybeText}\n  - {name: demo.notes.Deepest, kind: newtype, of: demo.notes.Deeper}\n";

/// A newtype admits absence when what it wraps does, through any number of newtypes, and a field
/// of such a type is not always written: it decides nothing. A newtype over a `String` is always
/// written, as before. The model has no other wrapping type (`kind` is `newtype`, `struct`, `enum`
/// or `union`), so there is no alias to follow.
#[test]
fn a_field_through_newtypes_over_optional_decides_nothing_and_one_over_a_string_decides() {
    let types = format!("{MAYBE}{DEEPER}");
    for field in [
        "{name: details, type: demo.notes.MaybeText}",
        "{name: details, type: demo.notes.Deeper}",
        "{name: details, type: demo.notes.Deepest}",
        "{name: details, type: Optional<demo.notes.Deepest>}",
    ] {
        let ir = model_with(&types, &[MESSAGE], &[MESSAGE, field]);
        let without = bound(&ir, false);
        assert_eq!(
            located(&without),
            [("ESS-HTTP-025", "routes[0].outcomes.negative".to_owned())],
            "{field}: {}",
            worded(&without)
        );
        let text = worded(&without);
        assert!(text.contains(UNDECIDED), "{field}: {text}");
    }
    let ir = model_with(
        &types,
        &[MESSAGE],
        &[MESSAGE, "{name: details, type: demo.notes.NoteId}"],
    );
    let result = bound(&ir, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
}

/// Each form the two refusals of an undecided pair name validates when applied as they say.
#[test]
fn each_form_the_refusals_name_validates_when_applied() {
    let optional = "{name: details, type: Optional<String>}";
    let undecided = model(&[MESSAGE], &[MESSAGE, optional]);
    for text in [
        worded(&bound(&undecided, false)),
        worded(&bound(&undecided, true)),
    ] {
        assert!(text.contains(FORMS), "{text}");
    }
    // One error with the optional field, reported by both outcomes, under `alike:`.
    let one_error = model(&[MESSAGE, optional], &[MESSAGE, optional]);
    let result = bound(&one_error, true);
    assert_eq!(located(&result), [], "{}", worded(&result));
    // With the group dropped: a field one error always writes that the other never writes.
    let always = model(&[MESSAGE], &[MESSAGE, DETAILS]);
    let result = bound(&always, false);
    assert_eq!(located(&result), [], "{}", worded(&result));
    // ... which `alike:` refuses, so the refusal says the group is dropped.
    assert_eq!(
        located(&bound(&always, true)),
        [("ESS-HTTP-025", "routes[0].alike".to_owned())]
    );
    // With the group dropped: distinct error wire names (`Refused` and `TooLarge`).
    let result = bound_keys(
        &undecided,
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      too-large: {status: 409, body: {error: error}}\n",
    );
    assert_eq!(located(&result), [], "{}", worded(&result));
    // With the group dropped: `envelope`.
    let result = bound_keys(
        &undecided,
        "    outcomes:\n      negative: {status: 409, body: envelope}\n      blank: {status: 409, body: envelope}\n",
    );
    assert_eq!(located(&result), [], "{}", worded(&result));
}
