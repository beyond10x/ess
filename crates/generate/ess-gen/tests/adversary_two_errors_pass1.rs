//! Adversarial cases for the `OpenAPI` projection of two outcomes answering one status with
//! `{error: error}` over two errors of one wire name (`ESS-HTTP-025`,
//! `docs/design/http-presentation-binding.md`, "Status per outcome").
//!
//! Without a discriminator, the response's `oneOf` is only a description of the answer when no
//! body fits two of its members (`OpenAPI` 3.1 reads `oneOf` as JSON Schema: exactly one member
//! validates). Both members are closed objects, so a body made of the members both require fits
//! both whenever each requires only members the other declares. Each case below either is refused
//! by the binding (`ESS-HTTP-025`), or its document holds no such pair.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::BoundOpenApi;
use serde_json::Value;

const NOTES: &str = include_str!("../../../specify/ess-http/tests/fixtures/notes.yaml");

fn ir_of(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}\n{text}"));
    let mut sources = SourceMap::new();
    sources.insert(name, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The document of `component` for one route, or `None` when the binding is refused with
/// `ESS-HTTP-025` (any other refusal fails the case).
fn document(ir: &EssIr, system: &str, component: &str, route: &str) -> Option<Value> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: {system}\n  version: v1\n  source_digest: sha256:{}\nroutes:\n{route}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let compiled = match ess_http::compile(&spec, ir, &derived(ir)) {
        Ok(compiled) => compiled,
        Err(diagnostics) => {
            assert!(
                diagnostics
                    .0
                    .iter()
                    .all(|found| found.code == "ESS-HTTP-025"),
                "refused for another reason: {diagnostics}"
            );
            return None;
        }
    };
    let binding = Binding::new(compiled).expect("every route has a path");
    let artifacts = run(&BoundOpenApi(binding), ir).expect("generates");
    let yaml: ess_yaml::Value =
        ess_yaml::from_str(&artifacts[&format!("openapi/{component}.yaml")].contents)
            .expect("YAML");
    Some(serde_json::to_value(yaml).expect("JSON"))
}

fn resolved<'a>(document: &'a Value, member: &'a Value) -> &'a Value {
    member["$ref"]
        .as_str()
        .and_then(|reference| reference.strip_prefix("#/components/schemas/"))
        .map_or(member, |key| &document["components"]["schemas"][key])
}

fn names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(|name| name.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Every pair of `oneOf` members on `path`'s `post` response at `status` one body fits.
fn overlapping(document: &Value, path: &str, status: &str) -> Vec<(Value, Value)> {
    let schema = &document["paths"][path]["post"]["responses"][status]["content"]
        ["application/json"]["schema"];
    let members: Vec<&Value> = schema["oneOf"]
        .as_array()
        .unwrap_or_else(|| panic!("a `oneOf` on {status}: {schema}"))
        .iter()
        .map(|member| resolved(document, member))
        .collect();
    let mut out = Vec::new();
    for (index, one) in members.iter().enumerate() {
        for two in &members[index + 1..] {
            let declares = |schema: &Value, name: &str| schema["properties"].get(name).is_some();
            let fits = |required: &Value, other: &Value| {
                names(required).iter().all(|name| declares(other, name))
            };
            if fits(&one["required"], two) && fits(&two["required"], one) {
                out.push(((*one).clone(), (*two).clone()));
            }
        }
    }
    out
}

/// The unknown-instance rule: `Close` moves an input-named door and declares `wrong-state`
/// reporting an error with a field, so a door no record carries is answered by `wrong-state`
/// without that field, and the document requires none of its fields. `blocked` reports an error of
/// the same wire name with no field. `{error: door_conflict}` fits both.
const DOORS: &str = r"format: ess/22
system: doors
version: v1
domain: doors.core
entities:
  - name: doors.core.Door
    identity: {name: door_id, type: String}
    fields:
      - {name: colour, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: doors.core.Installed
    fields: [{name: door_id, type: String}]
  - name: doors.core.Closed
    fields: [{name: door_id, type: String}]
errors:
  - name: doors.core.DoorStateConflict
    naming: {wire: door_conflict}
    fields:
      - {name: state, type: doors.core.Door.State}
  - name: doors.core.DoorBlocked
    naming: {wire: door_conflict}
commands:
  - name: doors.core.Install
    input:
      - {name: door_id, type: String}
    outcomes:
      - name: installed
        creates: doors.core.Door
        instance: door_id
        sets: {colour: white}
        emits: [doors.core.Installed]
        payload:
          doors.core.Installed: {door_id: input.door_id}
  - name: doors.core.Close
    input:
      - {name: door_id, type: String}
      - {name: level, type: Integer}
    outcomes:
      - name: blocked
        when: level > 5
        error: doors.core.DoorBlocked
      - name: closed
        moves: doors.core.Door.close
        instance: door_id
        emits: [doors.core.Closed]
        payload:
          doors.core.Closed: {door_id: input.door_id}
      - name: wrong-state
        wrong_state: true
        error: doors.core.DoorStateConflict
components:
  - component: door-service
    owns: {domains: [doors.core]}
    accepts: {commands: [doors.core.Install, doors.core.Close]}
    publishes: {events: [doors.core.Installed, doors.core.Closed]}
    reached_by: network
";

#[test]
fn the_unknown_instance_answer_of_wrong_state_and_a_fieldless_error_do_not_share_one_one_of() {
    let ir = ir_of("doors.yaml", DOORS);
    let Some(document) = document(
        &ir,
        "doors",
        "door-service",
        "  - command: doors.core.Close\n    method: post\n    path: /api/doors/{door_id}\n    parameters: {door_id: input.door_id}\n    outcomes:\n      blocked: {status: 409, body: {error: error}}\n      wrong-state: {status: 409, body: {error: error}}\n",
    ) else {
        return;
    };
    let overlap = overlapping(&document, "/api/doors/{door_id}", "409");
    assert!(
        overlap.is_empty(),
        "the binding validates and one body fits two members of the `oneOf`: {overlap:#?}"
    );
}

#[test]
fn a_field_of_a_newtype_over_optional_does_not_leave_two_members_one_body_fits() {
    let text = NOTES
        .replace(
            "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
            "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n  - {name: demo.notes.MaybeText, kind: newtype, of: Optional<String>}\n",
        )
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            "  - name: demo.notes.Refused
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
  - name: demo.notes.Declined
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
      - {name: details, type: demo.notes.MaybeText}
",
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    assert_ne!(text, NOTES);
    let ir = ir_of("notes.yaml", &text);
    let Some(document) = document(
        &ir,
        "demo",
        "notes-service",
        "  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n",
    ) else {
        return;
    };
    let overlap = overlapping(&document, "/api/notes", "409");
    assert!(
        overlap.is_empty(),
        "the binding validates and one body fits two members of the `oneOf`: {overlap:#?}"
    );
}

/// The pair the unit admits, `message` against `message` and `details`: no body fits both
/// (a probe of the check above, which must hold).
#[test]
fn the_admitted_pair_leaves_no_body_two_members_fit() {
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
      - {name: message, type: String}
      - {name: details, type: String}
",
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    let ir = ir_of("notes.yaml", &text);
    let document = document(
        &ir,
        "demo",
        "notes-service",
        "  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n",
    )
    .expect("the pair binds");
    assert_eq!(
        overlapping(&document, "/api/notes", "409"),
        Vec::<(Value, Value)>::new()
    );
}
