//! A value type's invariants on a type nothing reaches: no command, event, error, view, entity,
//! actor, binding or conversion carries it, so no scenario sends or reads a value of it. Like an
//! unreached `Binary64` position (`Note::UnreachedBinary64`), it refuses nothing and is named in a
//! note, so a reader sees what the run does not exercise. A type something reaches keeps its
//! `ESS-SYNTH-013` refusal, in both of its shapes.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Note, Refusal, RefusalCause, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// `Port` declares its bounds; `Aperture` holds one and `Policy` a list of them. `{types}` adds
/// declarations, `{input}` command input members, `{entities}` entities and `{views}` views. With
/// every slot empty nothing names `Port`, `Aperture` or `Policy`.
const NET: &str = r"format: ess/34
system: demo
version: v1
domain: demo.net
types:
  - {name: demo.net.GateId, kind: newtype, of: Uuid}
  - {name: demo.net.Port, kind: newtype, of: Integer, invariants: ['value >= 1', 'value <= 65535']}
  - name: demo.net.Aperture
    kind: struct
    fields:
      - {name: host, type: String}
      - {name: port, type: demo.net.Port}
  - name: demo.net.Policy
    kind: struct
    fields:
      - {name: apertures, type: 'List<demo.net.Aperture>'}
{types}entities:
  - name: demo.net.Gate
    identity: {name: gate_id, type: demo.net.GateId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
{entities}events:
  - name: demo.net.GateOpened
    fields: [{name: gate_id, type: demo.net.GateId}]
commands:
  - name: demo.net.OpenGate
    input:
      - {name: label, type: String}
{input}    outcomes:
      - name: opened
        creates: demo.net.Gate
        instance: gate_id
        sets: {label: input.label}
        emits: [demo.net.GateOpened]
        payload: {demo.net.GateOpened: {gate_id: {generated: true}}}
views:
  - name: demo.net.Gates
    source: demo.net.Gate
    consistency: read_your_writes
    fields:
      - {name: gate_id, type: demo.net.GateId}
      - {name: label, type: String}
{views}";

#[derive(Default)]
struct Slots<'a> {
    types: &'a str,
    input: &'a str,
    entities: &'a str,
    views: &'a str,
}

fn model(slots: &Slots<'_>) -> String {
    NET.replace("{types}", slots.types)
        .replace("{input}", slots.input)
        .replace("{entities}", slots.entities)
        .replace("{views}", slots.views)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("net.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("net.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(slots: &Slots<'_>) -> Synthesis {
    synthesize(&ir(&model(slots)))
}

fn listed(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.code()))
        .chain(synthesis.notes.iter().map(|note| format!("note: {note}")))
        .collect()
}

/// The `ESS-SYNTH-013` refusals about `Port`.
fn port_refusals(synthesis: &Synthesis) -> Vec<&Refusal> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            matches!(
                &refusal.cause,
                RefusalCause::ValueInvariantUnwitnessed { value, .. }
                    if value.to_string() == "demo.net.Port"
            )
        })
        .collect()
}

/// The notes naming `Port`'s invariants as unexercised.
fn port_notes(synthesis: &Synthesis) -> Vec<&Note> {
    synthesis
        .notes
        .iter()
        .filter(|note| {
            matches!(note, Note::UnreachedValueInvariant { value, .. }
                if value.to_string() == "demo.net.Port")
        })
        .collect()
}

fn assert_noted(synthesis: &Synthesis) {
    let all = listed(synthesis);
    assert!(synthesis.refusals.is_empty(), "{all:#?}");
    let notes = port_notes(synthesis);
    assert_eq!(notes.len(), 1, "{all:#?}");
    let Note::UnreachedValueInvariant { invariants, .. } = notes[0] else {
        unreachable!("filtered above");
    };
    assert_eq!(invariants, &["value >= 1", "value <= 65535"], "{all:#?}");
    let text = notes[0].to_string();
    for named in ["demo.net.Port", "value >= 1", "value <= 65535"] {
        assert!(text.contains(named), "the note names {named}: {text}");
    }
}

#[test]
fn an_invariant_on_a_type_nothing_reaches_is_a_note() {
    let result = synthesis(&Slots::default());
    assert_noted(&result);
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == "demo.net.OpenGate/outcome/opened"),
        "the rest of the model synthesizes: {:#?}",
        listed(&result)
    );
}

#[test]
fn an_invariant_reached_only_inside_an_optional_member_of_an_unreached_struct_is_a_note() {
    let result = synthesis(&Slots {
        types: "  - name: demo.net.Envelope\n    kind: struct\n    fields:\n      - {name: name, type: String}\n      - {name: policy, type: 'Optional<demo.net.Policy>'}\n",
        ..Slots::default()
    });
    assert_noted(&result);
}

#[test]
fn an_invariant_on_a_type_a_command_input_reaches_still_refuses() {
    let result = synthesis(&Slots {
        input: "      - {name: port, type: demo.net.Port}\n",
        ..Slots::default()
    });
    let all = listed(&result);
    let refused = port_refusals(&result);
    assert_eq!(refused.len(), 1, "{all:#?}");
    assert!(
        matches!(
            refused[0].cause,
            RefusalCause::ValueInvariantUnwitnessed { at: None, .. }
        ),
        "{all:#?}"
    );
    assert!(port_notes(&result).is_empty(), "{all:#?}");
}

#[test]
fn an_invariant_on_a_type_a_view_publishes_unarranged_still_refuses() {
    // A relay holds a port and a view publishes it, and no outcome creates a relay: no row the
    // view shows can be arranged.
    let result = synthesis(&Slots {
        entities: "  - name: demo.net.Relay\n    identity: {name: relay_id, type: demo.net.GateId}\n    fields:\n      - {name: port, type: demo.net.Port}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n",
        views: "  - name: demo.net.Relays\n    source: demo.net.Relay\n    consistency: read_your_writes\n    fields:\n      - {name: relay_id, type: demo.net.GateId}\n      - {name: port, type: demo.net.Port}\n",
        ..Slots::default()
    });
    let all = listed(&result);
    let refused = port_refusals(&result);
    assert!(!refused.is_empty(), "{all:#?}");
    assert!(
        refused.iter().all(|refusal| matches!(
            refusal.cause,
            RefusalCause::ValueInvariantUnwitnessed { at: Some(_), .. }
        )),
        "{all:#?}"
    );
    assert!(port_notes(&result).is_empty(), "{all:#?}");
}
