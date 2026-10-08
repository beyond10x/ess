//! A refusal whose `when:` always holds is refused (beyond10x/ess#489).
//!
//! `when: true` makes a branch unconditional, but a refusal written so had no one step in the
//! precedence order: the model interpreter did not read it among the input refusals, while the
//! Entity Runtime lowering and the Rust and Go targets read it first among them. Beside another
//! refusal that held, the same request was answered differently. An accepting `when: true` branch
//! and a refusal with no `when:` (the default) are unchanged.

const NEEDLE: &str = "is a refusal whose `when:` always holds";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// `Rush` with `branches` before its accepting default.
fn model(branches: &str) -> String {
    format!(
        "format: ess/20
system: demo
version: v1
domain: demo.desk
types:
  - {{name: demo.desk.TicketId, kind: newtype, of: Uuid}}
entities:
  - name: demo.desk.Ticket
    identity: {{name: ticket_id, type: demo.desk.TicketId}}
    fields: []
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.Rush]
commands:
  - name: demo.desk.Rush
    input:
      - {{name: count, type: Integer}}
    outcomes:
{branches}      - name: rushed
        creates: demo.desk.Ticket
        instance: ticket_id
        emits: [demo.desk.Rushed]
        payload:
          demo.desk.Rushed: {{ticket_id: {{generated: true}}}}
events:
  - name: demo.desk.Rushed
    fields: [{{name: ticket_id, type: demo.desk.TicketId}}]
errors:
  - name: demo.desk.Paused
  - name: demo.desk.TooMany
views:
  - name: demo.desk.Tickets
    source: demo.desk.Ticket
    consistency: read_your_writes
    fields:
      - {{name: ticket_id, type: demo.desk.TicketId}}
      - {{name: state, type: demo.desk.Ticket.State}}
"
    )
}

const TOO_MANY: &str = "      - name: too-many
        when: count > 3
        error: demo.desk.TooMany
";

#[test]
fn a_refusal_whose_when_always_holds_is_refused() {
    let paused = "      - name: paused\n        when: \"true\"\n        error: demo.desk.Paused\n";
    let errors = spec(&model(&format!("{paused}{TOO_MANY}"))).unwrap_err();
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.Rush.outcomes.paused: `paused` is a \
             refusal whose `when:` always holds; where it answers beside the command's other \
             refusals is not stated (hint: give the refusal the condition it refuses on, or drop \
             `when:` to declare it as the command's default refusal)"
        ),
        "{errors}"
    );
}

#[test]
fn it_is_refused_alone_and_wherever_it_is_declared() {
    let paused = "      - name: paused\n        when: \"true\"\n        error: demo.desk.Paused\n";
    for branches in [paused.to_owned(), format!("{TOO_MANY}{paused}")] {
        let errors = spec(&model(&branches)).unwrap_err();
        assert!(errors.contains(NEEDLE), "{errors}\n{branches}");
    }
}

#[test]
fn a_guarded_refusal_and_an_accepting_when_true_branch_still_validate() {
    let accepting = "      - name: always\n        when: \"true\"\n        creates: demo.desk.Ticket\n        instance: ticket_id\n        emits: [demo.desk.Rushed]\n        payload:\n          demo.desk.Rushed: {ticket_id: {generated: true}}\n";
    for branches in [TOO_MANY.to_owned(), format!("{TOO_MANY}{accepting}")] {
        let body = model(&branches);
        match spec(&body) {
            Ok(_) => {}
            Err(errors) => assert!(!errors.contains(NEEDLE), "{errors}\n{body}"),
        }
    }
}
