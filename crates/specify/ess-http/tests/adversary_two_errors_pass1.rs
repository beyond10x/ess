//! Adversarial cases for two errors of one wire name whose fields differ, answered on one status
//! with `{error: error}` (`ESS-HTTP-025`, `docs/design/http-presentation-binding.md`, "Status per
//! outcome").
//!
//! The rule: one error always writes a field, by wire name and with a type that is not
//! `Optional`, that the other never writes. "Always writes" is the premise; the red cases here are
//! fields the rule counts as always written that an answer can leave out.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const NOTES: &str = include_str!("fixtures/notes.yaml");

const ERRORS: &str = "errors:
  - name: demo.notes.Refused
    naming: {wire: refused}
  - name: demo.notes.TooLarge
    naming: {wire: too_large}
";

fn ir_of(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}\n{text}"));
    let mut sources = SourceMap::new();
    sources.insert(name, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// `notes.yaml` with its `errors:` block replaced by `errors`, `types` appended to its types, and
/// `blank` reporting `demo.notes.Declined` when `declined` is set.
fn notes(errors: &str, types: &str, declined: bool) -> EssIr {
    assert_eq!(NOTES.matches(ERRORS).count(), 1, "the fixture moved");
    let mut text = NOTES.replace(ERRORS, errors).replace(
        "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
        &format!("  - {{name: demo.notes.NoteId, kind: newtype, of: String}}\n{types}"),
    );
    if declined {
        text = text.replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    }
    ir_of("notes.yaml", &text)
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

/// One route of `command` at `post <path>`, with `keys` beside it.
fn bound(
    ir: &EssIr,
    system: &str,
    command: &str,
    path: &str,
    keys: &str,
) -> Result<HttpIr, HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: {system}\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: {command}\n    method: post\n    path: {path}\n{keys}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    compile(&spec, ir, &derived(ir))
}

fn notes_bound(ir: &EssIr, keys: &str) -> Result<HttpIr, HttpDiagnostics> {
    bound(ir, "demo", "demo.notes.CreateNote", "/api/notes", keys)
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

const PAIR: &str = "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n";

/// A newtype over `Optional<String>` admits absence (the model says so through the newtype, and
/// every generated writer omits an absent value), so a field of that type is not always written:
/// the answer of `blank` without `details` is the answer of `negative`. The rule counts the field
/// as always written because its type is not literally `Optional`.
#[test]
fn a_field_of_a_newtype_over_optional_is_not_always_written_and_does_not_decide() {
    let ir = notes(
        "errors:
  - name: demo.notes.Refused
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
  - name: demo.notes.Declined
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
      - {name: details, type: demo.notes.MaybeText}
  - name: demo.notes.TooLarge
    naming: {wire: too_large}
",
        "  - {name: demo.notes.MaybeText, kind: newtype, of: Optional<String>}\n",
        true,
    );
    let result = notes_bound(&ir, PAIR);
    assert_eq!(
        located(&result),
        [("ESS-HTTP-025", "routes[0].outcomes.negative".to_owned())],
        "an answer without `details` fits both bodies, yet the pair binds without `alike:`:\n{}",
        worded(&result)
    );
}

/// The unknown-instance rule (`docs/design/typed-literals-and-unknown-instances.md`, section 2;
/// `ess_gen::unknown_instance::unknown_instance_answer`): `Close` moves an input-named door, so a
/// door no record carries is answered with the `wrong_state` outcome, whose error's fields an
/// absent instance cannot supply; that answer writes none of them, and the `OpenAPI` document
/// requires none of them on that response. The field `state` is therefore not always written by
/// `wrong-state`, and `{error: door_conflict}` alone is an answer of `wrong-state` and of
/// `blocked`.
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

const DOORS_PAIR: &str = "    parameters: {door_id: input.door_id}\n    outcomes:\n      blocked: {status: 409, body: {error: error}}\n      wrong-state: {status: 409, body: {error: error}}\n";

#[test]
fn the_unknown_instance_answer_writes_no_field_so_a_wrong_state_field_does_not_decide() {
    let ir = ir_of("doors.yaml", DOORS);
    let result = bound(
        &ir,
        "doors",
        "doors.core.Close",
        "/api/doors/{door_id}",
        DOORS_PAIR,
    );
    assert_eq!(
        located(&result),
        [("ESS-HTTP-025", "routes[0].outcomes.blocked".to_owned())],
        "an unknown door is answered `{{error: door_conflict}}` by `wrong-state`, which `blocked` \
         answers too, yet the pair binds without `alike:`:\n{}",
        worded(&result)
    );
}

// Probes of the rule that are expected to hold.

const MESSAGE: &str = "      - {name: message, type: String}\n";

/// Three errors of one wire name: each pair is told apart by its own field, so the three bind.
#[test]
fn three_errors_each_pair_told_apart_by_its_own_field_bind() {
    let ir = notes(
        &format!(
            "errors:
  - name: demo.notes.Refused
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: x, type: String}}
  - name: demo.notes.Declined
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: y, type: String}}
  - name: demo.notes.TooLarge
    naming: {{wire: refused}}
    fields:
{MESSAGE}"
        ),
        "",
        true,
    );
    let result = notes_bound(
        &ir,
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n      too-large: {status: 409, body: {error: error}}\n",
    );
    assert_eq!(located(&result), [], "{}", worded(&result));
}

/// Three errors where two pairs are decided and the third is not: refused, at the undecided pair.
#[test]
fn three_errors_one_pair_undecided_is_refused() {
    let ir = notes(
        &format!(
            "errors:
  - name: demo.notes.Refused
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: x, type: String}}
  - name: demo.notes.Declined
    naming: {{wire: refused}}
    fields:
{MESSAGE}
  - name: demo.notes.TooLarge
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: z, type: Optional<String>}}
"
        ),
        "",
        true,
    );
    let result = notes_bound(
        &ir,
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n      too-large: {status: 409, body: {error: error}}\n",
    );
    assert_eq!(
        located(&result),
        [("ESS-HTTP-025", "routes[0].outcomes.blank".to_owned())],
        "{}",
        worded(&result)
    );
}

/// The group written in the other order is refused alike, naming the field.
#[test]
fn alike_in_the_other_order_is_refused_naming_the_field() {
    let ir = notes(
        &format!(
            "errors:
  - name: demo.notes.Refused
    naming: {{wire: refused}}
    fields:
{MESSAGE}  - name: demo.notes.Declined
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: details, type: String}}
  - name: demo.notes.TooLarge
    naming: {{wire: too_large}}
"
        ),
        "",
        true,
    );
    let result = notes_bound(&ir, &format!("{PAIR}    alike: [[blank, negative]]\n"));
    assert_eq!(
        located(&result),
        [("ESS-HTTP-025", "routes[0].alike".to_owned())],
        "{}",
        worded(&result)
    );
    assert!(worded(&result).contains("`details`"), "{}", worded(&result));
}

/// One outcome listed and one covered by `errors`, on one status: the field decides there too.
#[test]
fn a_listed_and_an_errors_covered_outcome_are_told_apart_by_the_field() {
    let ir = notes(
        &format!(
            "errors:
  - name: demo.notes.Refused
    naming: {{wire: refused}}
    fields:
{MESSAGE}  - name: demo.notes.Declined
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: details, type: String}}
  - name: demo.notes.TooLarge
    naming: {{wire: too_large}}
"
        ),
        "",
        true,
    );
    let result = notes_bound(
        &ir,
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n    errors: {status: 409, body: {error: error}}\n",
    );
    assert_eq!(located(&result), [], "{}", worded(&result));
}

/// The first form a refusal of an undecided pair names: one error with the optional field,
/// reported by both outcomes, under `alike:`, validates when applied.
#[test]
fn the_first_named_form_validates_when_applied() {
    let ir = notes(
        &format!(
            "errors:
  - name: demo.notes.Refused
    naming: {{wire: refused}}
    fields:
{MESSAGE}      - {{name: details, type: Optional<String>}}
  - name: demo.notes.TooLarge
    naming: {{wire: too_large}}
"
        ),
        "",
        false,
    );
    let result = notes_bound(&ir, &format!("{PAIR}    alike: [[negative, blank]]\n"));
    assert_eq!(located(&result), [], "{}", worded(&result));
}
