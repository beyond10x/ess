//! The Rust and Go emitters order generated selection by the precedence plan
//! (`story:generated-behaviour-reads-selection-plan`, `docs/design/selection-plan.md`).
//!
//! Each emitter writes one `if` block per branch it selects by, in the order of the command's
//! `ess_compiler::ir::PrecedencePlan`. With the held-state and accepting phases exchanged through
//! the classification's scoped phase-order override (`with_phase_order`, the one test seam every
//! consumer of the plan uses), the order of a held-state guard and an accepting guard in the
//! generated behaviour changes with it, for Rust and for Go.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

/// One command with a held-state refusal declared before an accepting `when:` branch, a default
/// and an `unknown_instance:` answer: every branch one the Rust and the Go emitters generate.
const MODEL: &str = "format: ess/18
system: swap
version: v1
domain: swap.desk
types:
  - {name: swap.desk.TicketId, kind: newtype, of: String}
entities:
  - name: swap.desk.Ticket
    identity: {name: ticket_id, type: swap.desk.TicketId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: swap.desk.TicketNoted
    fields: [{name: ticket_id, type: swap.desk.TicketId}]
errors:
  - name: swap.desk.Refused
actors:
  - {name: swap.desk.Clerk, may: [swap.desk.NoteTicket, swap.desk.CloseTicket]}
commands:
  - name: swap.desk.NoteTicket
    input:
      - {name: ticket_id, type: swap.desk.TicketId}
      - {name: count, type: Integer}
      - {name: text, type: String}
    outcomes:
      - name: already-closed
        when_subject_state: Closed
        error: swap.desk.Refused
      - name: rushed
        when: count > 0
        updates: swap.desk.Ticket
        instance: ticket_id
        sets: {note: input.text}
        emits: [swap.desk.TicketNoted]
        payload:
          swap.desk.TicketNoted: {ticket_id: input.ticket_id}
      - name: noted
        updates: swap.desk.Ticket
        instance: ticket_id
        sets: {note: input.text}
        emits: [swap.desk.TicketNoted]
        payload:
          swap.desk.TicketNoted: {ticket_id: input.ticket_id}
      - {name: no-such-ticket, unknown_instance: true, error: swap.desk.Refused}
  - name: swap.desk.CloseTicket
    input: [{name: ticket_id, type: swap.desk.TicketId}]
    outcomes:
      - name: closed
        moves: swap.desk.Ticket.close
        instance: ticket_id
        emits: [swap.desk.TicketNoted]
        payload:
          swap.desk.TicketNoted: {ticket_id: input.ticket_id}
      - {name: not-open, wrong_state: true, error: swap.desk.Refused}
";

/// The comment each emitter writes above the held-state branch's `if` block.
const HELD_STATE: &str = "// `already-closed`: selected by the addressed row.";
/// The comment each emitter writes above the accepting branch's `if` block.
const ACCEPTING: &str = "// `rushed`: an accepting branch, in declaration order.";

/// The precedence order with the held-state and accepting phases exchanged.
const EXCHANGED: [Phase; 8] = [
    Phase::InputAbsent,
    Phase::RelatedRow,
    Phase::InputRefusal,
    Phase::Existence,
    Phase::Accepting,
    Phase::PresentRelated,
    Phase::HeldState,
    Phase::Default,
];

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let specification = Specification::assemble([(Source::new("swap.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|errors| panic!("the model compiles: {errors:?}"))
}

/// The generated behaviour artifact of `target`, synthesized in the current phase order.
fn behaviour(ir: &EssIr, target: Target, file: &str) -> String {
    let synthesis = synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("{target:?} emits the model: {failure:?}"));
    let Some((_, artifact)) = synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with(file))
    else {
        panic!("{target:?} writes a `{file}`")
    };
    artifact.contents.clone()
}

/// Where the held-state guard and the accepting guard are written, in that order.
fn positions(behaviour: &str) -> (usize, usize) {
    let at = |comment: &str| {
        behaviour
            .find(comment)
            .unwrap_or_else(|| panic!("the behaviour writes `{comment}`:\n{behaviour}"))
    };
    (at(HELD_STATE), at(ACCEPTING))
}

fn exchanged_phases_reorder_the_guards(target: Target, file: &str) {
    let ir = ir();
    let (held, accepting) = positions(&behaviour(&ir, target, file));
    assert!(
        held < accepting,
        "{target:?}: in the precedence order the held-state guard is written before the accepting one"
    );
    let (held, accepting) =
        with_phase_order(EXCHANGED, || positions(&behaviour(&ir, target, file)));
    assert!(
        accepting < held,
        "{target:?}: with the held-state and accepting phases exchanged, the accepting guard is \
         written before the held-state one"
    );
}

#[test]
fn the_rust_emitter_orders_guards_by_the_precedence_plan() {
    exchanged_phases_reorder_the_guards(Target::Rust, "behaviour.rs");
}

#[test]
fn the_go_emitter_orders_guards_by_the_precedence_plan() {
    exchanged_phases_reorder_the_guards(Target::Go, "behaviour.go");
}
