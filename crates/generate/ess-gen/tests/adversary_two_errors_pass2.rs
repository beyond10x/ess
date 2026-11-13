//! Adversarial cases, second pass, for the field rule of `ESS-HTTP-025` against the
//! unknown-instance rule (`docs/design/unknown-instance-seams.md`).
//!
//! `ess-http` cannot call [`unknown_instance_answer`], so it takes every `wrong_state` outcome of a
//! command that moves, updates or deletes an instance its input names as the unknown-instance
//! answer. That replica drops three of the real function's conditions; one of them, a declared
//! `unknown_instance:` branch (ess/15), is the answer for an unknown instance in place of
//! `wrong_state`, which then always writes its error's fields. The story's rule admits such a
//! pair; the replica refuses it, and its refusal says the `wrong_state` outcome answers an unknown
//! instance without its fields, which that command never does.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::http::derived;
use ess_gen::unknown_instance::unknown_instance_answer;

fn ir_of(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}\n{text}"));
    let mut sources = SourceMap::new();
    sources.insert(name, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// `Close` moves an input-named door. `wrong-state` reports `DoorStateConflict` with `state`;
/// `blocked` reports `DoorBlocked` of the same wire name with no field. With `MISSING`, `Close`
/// also declares `missing`, its `unknown_instance:` branch.
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
  - name: doors.core.DoorMissing
    naming: {wire: door_missing}
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
MISSING      - name: blocked
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

const MISSING: &str = "      - name: missing\n        unknown_instance: true\n        error: doors.core.DoorMissing\n";

fn doors(missing: bool) -> EssIr {
    ir_of(
        "doors.yaml",
        &DOORS.replace("MISSING", if missing { MISSING } else { "" }),
    )
}

/// What the real function answers for `Close`.
fn answer(ir: &EssIr) -> Option<String> {
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "doors.core.Close")
        .expect("the fixture declares `Close`");
    unknown_instance_answer(ir, command).map(|outcome| outcome.name.to_string())
}

/// `blocked` and `wrong-state` on one `409` with `{error: error}`.
fn bound(ir: &EssIr) -> Result<ess_http::HttpIr, ess_http::HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: doors\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: doors.core.Close\n    method: post\n    path: /api/doors/{{door_id}}\n    parameters: {{door_id: input.door_id}}\n    outcomes:\n      blocked: {{status: 409, body: {{error: error}}}}\n      wrong-state: {{status: 409, body: {{error: error}}}}\n",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    ess_http::compile(&spec, ir, &derived(ir))
}

/// With a declared `unknown_instance:` branch, an unknown door is answered by `missing`, so
/// `wrong-state` answers only a door a record carries and always writes `state`, which `blocked`
/// never writes: the story's rule admits the pair.
#[test]
fn a_declared_unknown_instance_branch_leaves_the_wrong_state_field_deciding() {
    let ir = doors(true);
    assert_eq!(
        answer(&ir),
        None,
        "the real function: `missing` answers an unknown door"
    );
    let result = bound(&ir);
    assert!(
        result.is_ok(),
        "`wrong-state` always writes `state` and `blocked` never does, yet:\n{}",
        result
            .err()
            .map(|found| found.to_string())
            .unwrap_or_default()
    );
}

/// The control: without `missing`, the real function names `wrong-state`, and the pair is
/// refused.
#[test]
fn without_the_branch_the_real_function_names_wrong_state_and_the_pair_is_refused() {
    let ir = doors(false);
    assert_eq!(answer(&ir).as_deref(), Some("wrong-state"));
    let result = bound(&ir);
    let codes: Vec<&str> = result
        .as_ref()
        .err()
        .map(|found| found.0.iter().map(|diagnostic| diagnostic.code).collect())
        .unwrap_or_default();
    assert_eq!(codes, ["ESS-HTTP-025"]);
}
