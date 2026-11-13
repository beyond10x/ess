//! Adversarial cases, second pass: whether each sentence of the two refusals of an undecided pair
//! (`ESS-HTTP-025`) is true for every pair kind that prints it.
//!
//! Both refusals say of the pair that "a body one of them answers can be one the other answers",
//! and the `alike` refusal adds "so the wire does not tell them apart". For a pair whose fields
//! differ only in a type whose JSON values are disjoint, `message` a `String` in one and an
//! `Integer` in the other, no body is an answer of both: `{"error": "refused", "message": "m"}` is
//! only the first's and `{"error": "refused", "message": 1}` only the second's. The story refuses
//! the pair in both forms; the reason the refusals give for it is false there.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const NOTES: &str = include_str!("fixtures/notes.yaml");

/// `notes.yaml` with `negative` reporting `Refused` with `message: String` and `blank` reporting
/// `Declined`, of the same wire name, with `message: Integer`.
fn model() -> EssIr {
    let text = NOTES
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            "  - name: demo.notes.Refused
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
  - name: demo.notes.Declined
    naming: {wire: refused}
    fields:
      - {name: message, type: Integer}
",
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

fn bound(ir: &EssIr, alike: bool) -> Result<HttpIr, HttpDiagnostics> {
    let group = if alike {
        "    alike: [[negative, blank]]\n"
    } else {
        ""
    };
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n    outcomes:\n      negative: {{status: 409, body: {{error: error}}}}\n      blank: {{status: 409, body: {{error: error}}}}\n{group}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    compile(&spec, ir, &derived(ir))
}

const SHARED_BODY: &str = "a body one of them answers can be one the other answers";
const NOT_APART: &str = "the wire does not tell them apart";

#[test]
fn a_type_only_pair_of_disjoint_json_types_is_not_said_to_share_a_body() {
    let ir = model();
    for alike in [false, true] {
        let result = bound(&ir, alike);
        let text = result.as_ref().err().map_or_else(
            || panic!("refused, as the story says (alike: {alike})"),
            ToString::to_string,
        );
        assert!(
            !text.contains(SHARED_BODY) && !text.contains(NOT_APART),
            "`message: String` against `message: Integer`: no body is an answer of both, yet the \
             refusal says one can be (alike: {alike}):\n{text}"
        );
    }
}
