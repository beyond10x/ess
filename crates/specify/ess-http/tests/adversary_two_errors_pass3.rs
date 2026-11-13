//! Adversarial cases, third pass: pairs whose fields the refusal of an undecided pair
//! (`ESS-HTTP-025`, `Fields::Undecided` in `ess-http`) says can share a body, though no body is an
//! answer of both.
//!
//! The refusal's fact is "so a body one of them answers can be one the other answers", and under
//! `alike:` "so the wire does not tell them apart". `json_kind` keeps it wherever the two types
//! have one JSON type, or either has none (`Json`, a union) or is `Optional`. One JSON type is not
//! one value, and neither a union nor an absent value holds a string or a number:
//!
//! - two enums with disjoint variants are both strings, and no string is a variant of both;
//! - a union is written as its tagged object (or `null` for a unit variant), never as a string;
//! - `Optional<String>` against `Integer`: the first writes no field or a string, the second
//!   always a number.
//!
//! Each pair is refused in both forms, as the story says; the reason given is false for each.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const NOTES: &str = include_str!("fixtures/notes.yaml");

const TYPES: &str = "  - {name: demo.notes.Colour, kind: enum, variants: [red, blue]}
  - {name: demo.notes.Shape, kind: enum, variants: [round, square]}
  - name: demo.notes.Box
    kind: struct
    fields:
      - {name: size, type: Integer}
  - name: demo.notes.Pick
    kind: union
    tag: kind
    variants:
      boxed: demo.notes.Box
";

/// `notes.yaml` with `negative` reporting `Refused` with `first` and `blank` reporting `Declined`,
/// of the same wire name, with `second`, each one flow field entry.
fn model(first: &str, second: &str) -> EssIr {
    let text = NOTES
        .replace(
            "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
            &format!("  - {{name: demo.notes.NoteId, kind: newtype, of: String}}\n{TYPES}"),
        )
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            &format!(
                "  - name: demo.notes.Refused\n    naming: {{wire: refused}}\n    fields:\n      - {first}\n  - name: demo.notes.Declined\n    naming: {{wire: refused}}\n    fields:\n      - {second}\n"
            ),
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}\n{text}"));
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

/// Both forms are refused (the story), and neither says a body fits both.
fn refused_without_a_shared_body(first: &str, second: &str) {
    let ir = model(first, second);
    for alike in [false, true] {
        let result = bound(&ir, alike);
        let text = result.as_ref().err().map_or_else(
            || panic!("`{first}` against `{second}`: refused (alike: {alike})"),
            ToString::to_string,
        );
        assert!(
            !text.contains(SHARED_BODY) && !text.contains(NOT_APART),
            "`{first}` against `{second}`: no body is an answer of both, yet the refusal says one \
             can be (alike: {alike}):\n{text}"
        );
    }
}

#[test]
fn two_enums_with_disjoint_variants_share_no_body() {
    refused_without_a_shared_body(
        "{name: kind, type: demo.notes.Colour}",
        "{name: kind, type: demo.notes.Shape}",
    );
}

#[test]
fn a_union_and_a_string_share_no_body() {
    refused_without_a_shared_body(
        "{name: kind, type: demo.notes.Pick}",
        "{name: kind, type: String}",
    );
}

#[test]
fn an_optional_string_and_an_integer_share_no_body() {
    refused_without_a_shared_body(
        "{name: kind, type: Optional<String>}",
        "{name: kind, type: Integer}",
    );
}
