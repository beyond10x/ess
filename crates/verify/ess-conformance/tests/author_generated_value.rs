//! An authored exact value on a field the model marks `{generated: true}` (beyond10x/ess#544).
//!
//! The implementation mints a generated field, and synthesis checks only its presence and type. An
//! authored act that pins an exact value on one — in an event's payload, an error's fields, or a
//! view row over a stored field a `sets:` entry generates — passes only when the implementation
//! happens to mint that value, and fails a conforming one. `author` refuses it
//! (`ESS-AUTHOR-042`), naming the field. A field the outcome fills from the input or a literal
//! keeps its exact value, and a generated field left out of the expectation is accepted.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source as SpecSource;
use ess_domain::Specification;

const CODE: &str = "ESS-AUTHOR-042";

/// A selection that mints `connect_seq` on its event and on the row, a re-numbering that sets it
/// from the input, an optional `ticket` minted where the caller sends none, a struct whose `label`
/// leaf is minted, and a refusal whose error carries a minted `retry_token`.
const RELAY: &str = r"
format: ess/19
system: relay
version: v1
domain: relay.desk
types:
  - name: relay.desk.Ref
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: label, type: String}
entities:
  - name: relay.desk.Entry
    identity: {name: entry_id, type: Uuid}
    fields:
      - {name: agent_id, type: String}
      - {name: connect_seq, type: Integer}
    lifecycle: {initial: Waiting, states: [Waiting], terminal: [Waiting]}
events:
  - name: relay.desk.EntryAgentSelected
    fields:
      - {name: entry_id, type: Uuid}
      - {name: agent_id, type: String}
      - {name: connect_seq, type: Integer}
  - name: relay.desk.EntryRenumbered
    fields:
      - {name: entry_id, type: Uuid}
      - {name: connect_seq, type: Integer}
  - name: relay.desk.Ticketed
    fields:
      - {name: ticket, type: String}
      - {name: ref, type: relay.desk.Ref}
errors:
  - name: relay.desk.Busy
    fields:
      - {name: retry_token, type: String}
      - {name: agent_id, type: String}
views:
  - name: relay.desk.Entries
    source: relay.desk.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: Uuid}
      - {name: agent_id, type: String}
      - {name: connect_seq, type: Integer}
commands:
  - name: relay.desk.SelectAgentForEntry
    input:
      - {name: agent_id, type: String}
    outcomes:
      - name: busy
        when: agent_id == busy
        error: relay.desk.Busy
        payload:
          relay.desk.Busy: {retry_token: {generated: true}, agent_id: input.agent_id}
      - name: selected
        creates: relay.desk.Entry
        instance: entry_id
        sets: {agent_id: input.agent_id, connect_seq: {generated: true}}
        emits: [relay.desk.EntryAgentSelected]
        payload:
          relay.desk.EntryAgentSelected:
            entry_id: {generated: true}
            agent_id: input.agent_id
            connect_seq: {generated: true}
  - name: relay.desk.Renumber
    input:
      - {name: entry_id, type: Uuid}
      - {name: connect_seq, type: Integer}
    outcomes:
      - name: renumbered
        updates: relay.desk.Entry
        instance: entry_id
        sets: {connect_seq: input.connect_seq}
        emits: [relay.desk.EntryRenumbered]
        payload:
          relay.desk.EntryRenumbered: {entry_id: input.entry_id, connect_seq: input.connect_seq}
  - name: relay.desk.IssueTicket
    input:
      - {name: ticket, type: Optional<String>}
      - {name: ref_id, type: String}
    outcomes:
      - name: issued
        emits: [relay.desk.Ticketed]
        payload:
          relay.desk.Ticketed:
            ticket: {input: ticket, else: {generated: true}}
            ref:
              id: input.ref_id
              label: {generated: true}
";

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// A `relay.desk` document with `arrange`, a timeline of `acts` and the `assert` block.
fn document(arrange: &str, acts: &str, assert: &str) -> String {
    format!(
        "type: ess-scenario/2\ndomain: relay.desk\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n{arrange}timeline:\n{acts}{assert}"
    )
}

/// One act at second `at` sending `command` the `input` flow mapping, with `rest` under it.
fn act(at: u32, command: &str, input: &str, rest: &str) -> String {
    format!(
        "  - at: 2026-01-05T09:00:{at:02}Z\n    command: relay.desk.{command}\n    \
         input: {input}\n{rest}"
    )
}

/// The selection act, claiming `EntryAgentSelected` with the `payload` flow mapping.
fn selected(payload: &str) -> String {
    act(
        0,
        "SelectAgentForEntry",
        "{agent_id: a1}",
        &format!(
            "    outcome: selected\n    events:\n      - event: relay.desk.EntryAgentSelected\n        \
             payload: {payload}\n"
        ),
    )
}

/// Every refusal of `text`, as `(code, text)`.
fn refusals(ir: &EssIr, text: &str) -> Vec<(String, String)> {
    authoring(ir, text)
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect()
}

/// The one refusal of `text`, which is `ESS-AUTHOR-042`; its text.
fn pinned(ir: &EssIr, text: &str) -> String {
    let compiled = authoring(ir, text);
    let refused = refusals(ir, text);
    assert!(
        compiled.scenarios.is_empty(),
        "a refused document produced a scenario: {refused:?}"
    );
    assert_eq!(refused.len(), 1, "exactly one refusal: {refused:?}");
    assert_eq!(refused[0].0, CODE, "{refused:?}");
    refused[0].1.clone()
}

fn accepted(ir: &EssIr, text: &str) {
    let compiled = authoring(ir, text);
    assert!(
        compiled.is_complete(),
        "accepted: {:?}",
        compiled
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(compiled.scenarios.len(), 1);
}

fn names(text: &str, named: &[&str]) {
    for name in named {
        assert!(text.contains(name), "names {name}: {text}");
    }
}

#[test]
fn an_exact_value_on_a_generated_payload_field_is_refused() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document("", &selected("{agent_id: a1, connect_seq: 7}"), ""),
    );
    names(
        &text,
        &[
            "`connect_seq`",
            "`relay.desk.EntryAgentSelected`",
            "`relay.desk.SelectAgentForEntry/selected`",
            "help:",
            "{$observed:",
        ],
    );
    assert!(
        !text.contains("`agent_id`"),
        "names only the pinned field: {text}"
    );
}

#[test]
fn the_refusal_reads_in_full() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document("", &selected("{agent_id: a1, connect_seq: 7}"), ""),
    );
    assert_eq!(
        text,
        "refusal[ESS-AUTHOR-042]: `relay.desk/authored/a-scenario` in scenario.yaml\n  \
         the payload of `relay.desk.EntryAgentSelected` pins `connect_seq` to 7, a value \
         `relay.desk.SelectAgentForEntry/selected` marks `{generated: true}`\n  \
         help: the implementation mints a `{generated: true}` field, so an exact value passes \
         only where it happens to mint that one: drop the field from the expectation, and the \
         suite still checks its presence and type, or state its relation to another value with \
         a reference, `{$observed: {event: …, field: …}}` in a view row or `capture:` on the \
         event"
    );
}

#[test]
fn an_exact_value_on_a_generated_stored_field_is_refused() {
    let ir = fixture(RELAY);
    for claim in ["contains", "excludes"] {
        let text = pinned(
            &ir,
            &document(
                "",
                &selected("{agent_id: a1}"),
                &format!(
                    "assert:\n  - view: relay.desk.Entries\n    {claim}: \
                     {{agent_id: a1, connect_seq: 7}}\n"
                ),
            ),
        );
        names(
            &text,
            &[
                "`connect_seq`",
                "a row of `relay.desk.Entries`",
                "`relay.desk.SelectAgentForEntry/selected`",
            ],
        );
    }
}

#[test]
fn a_generated_field_left_out_of_the_expectation_is_accepted() {
    let ir = fixture(RELAY);
    accepted(
        &ir,
        &document(
            "",
            &selected("{agent_id: a1}"),
            "assert:\n  - view: relay.desk.Entries\n    contains: {agent_id: a1}\n",
        ),
    );
}

#[test]
fn a_field_set_from_input_keeps_its_exact_value() {
    let ir = fixture(RELAY);
    // `agent_id` is the input's on the event and on the row; `connect_seq` is the input's once
    // `Renumber` has written it, on every row of the entity this scenario wrote.
    accepted(
        &ir,
        &document(
            "arrange:\n  - {instance: entry, entity: relay.desk.Entry}\n",
            &format!(
                "{}{}",
                act(
                    0,
                    "SelectAgentForEntry",
                    "{agent_id: a1}",
                    "    outcome: selected\n    events:\n      - event: \
                     relay.desk.EntryAgentSelected\n        payload: {agent_id: a1}\n    \
                     capture: {instance: entry, event: relay.desk.EntryAgentSelected, field: \
                     entry_id}\n",
                ),
                act(
                    1,
                    "Renumber",
                    "{entry_id: {$instance: entry}, connect_seq: 7}",
                    "    outcome: renumbered\n    events:\n      - event: \
                     relay.desk.EntryRenumbered\n        payload: {connect_seq: 7}\n",
                ),
            ),
            "assert:\n  - view: relay.desk.Entries\n    contains: {agent_id: a1, connect_seq: 7}\n",
        ),
    );
}

#[test]
fn a_reference_on_a_generated_field_is_accepted() {
    let ir = fixture(RELAY);
    accepted(
        &ir,
        &document(
            "arrange:\n  - {instance: entry, entity: relay.desk.Entry}\n",
            &act(
                0,
                "SelectAgentForEntry",
                "{agent_id: a1}",
                "    outcome: selected\n    events:\n      - event: \
                 relay.desk.EntryAgentSelected\n        payload: {agent_id: a1}\n    \
                 capture: {instance: entry, event: relay.desk.EntryAgentSelected, field: \
                 entry_id}\n",
            ),
            "assert:\n  - view: relay.desk.Entries\n    contains:\n      entry_id: \
             {$instance: entry}\n      connect_seq: {$observed: {event: \
             relay.desk.EntryAgentSelected, field: connect_seq}}\n",
        ),
    );
}

#[test]
fn an_arranged_row_that_states_the_field_keeps_its_exact_value() {
    let ir = fixture(RELAY);
    // The row `setup:` establishes holds the value the file states, so a view claim of it is not
    // a claim about what the implementation minted.
    accepted(
        &ir,
        &document(
            "arrange:\n  - instance: held\n    entity: relay.desk.Entry\n    setup:\n      \
             identity: 3f1d5b7e-0000-4000-8000-000000000001\n      \
             fields: {agent_id: a0, connect_seq: 7}\n      state: Waiting\n",
            &selected("{agent_id: a1}"),
            "assert:\n  - view: relay.desk.Entries\n    contains: {agent_id: a0, connect_seq: 7}\n",
        ),
    );
}

#[test]
fn an_act_naming_no_outcome_is_refused_where_every_branch_publishing_the_event_mints_it() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document(
            "",
            &act(
                0,
                "SelectAgentForEntry",
                "{agent_id: a1}",
                "    events:\n      - event: relay.desk.EntryAgentSelected\n        \
                 payload: {connect_seq: 7}\n",
            ),
            "",
        ),
    );
    names(
        &text,
        &["`connect_seq`", "`relay.desk.SelectAgentForEntry/selected`"],
    );
}

#[test]
fn an_exact_value_on_a_generated_error_field_is_refused() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document(
            "",
            &act(
                0,
                "SelectAgentForEntry",
                "{agent_id: busy}",
                "    outcome: busy\n    error:\n      name: relay.desk.Busy\n      \
                 fields: {agent_id: busy, retry_token: t-1}\n",
            ),
            "",
        ),
    );
    names(
        &text,
        &[
            "the fields of `relay.desk.Busy`",
            "`retry_token`",
            "`relay.desk.SelectAgentForEntry/busy`",
        ],
    );
}

#[test]
fn an_exact_value_on_a_generated_struct_leaf_is_refused() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document(
            "",
            &act(
                0,
                "IssueTicket",
                "{ticket: T-1, ref_id: r1}",
                "    outcome: issued\n    events:\n      - event: relay.desk.Ticketed\n        \
                 payload: {ticket: T-1, ref: {id: r1, label: L}}\n",
            ),
            "",
        ),
    );
    names(&text, &["`ref.label`", "`relay.desk.IssueTicket/issued`"]);
}

#[test]
fn an_optional_input_left_out_is_minted_and_its_exact_value_refused() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document(
            "",
            &act(
                0,
                "IssueTicket",
                "{ref_id: r1}",
                "    outcome: issued\n    events:\n      - event: relay.desk.Ticketed\n        \
                 payload: {ticket: T-1}\n",
            ),
            "",
        ),
    );
    names(&text, &["`ticket`", "`relay.desk.IssueTicket/issued`"]);
}

#[test]
fn an_optional_input_sent_keeps_its_exact_value() {
    let ir = fixture(RELAY);
    accepted(
        &ir,
        &document(
            "",
            &act(
                0,
                "IssueTicket",
                "{ticket: T-1, ref_id: r1}",
                "    outcome: issued\n    events:\n      - event: relay.desk.Ticketed\n        \
                 payload: {ticket: T-1}\n",
            ),
            "",
        ),
    );
}

/// The `ess/27` shapes (beyond10x/ess#545): a seat whose `connect_seq` is a present mint on the
/// event and on the row, and whose `bridge_seq` the row reads from the payload's mint with
/// `{payload: relay.desk.SeatBridged.bridge_seq}`. `TakeSeat` reads the same payload field from
/// its input, so the row holds a value the act sent.
const SEATS: &str = r"
format: ess/27
system: relay
version: v1
domain: relay.desk
entities:
  - name: relay.desk.Seat
    identity: {name: seat_id, type: Uuid}
    fields:
      - {name: agent_id, type: String}
      - {name: connect_seq, type: Optional<Integer>}
      - {name: bridge_seq, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: relay.desk.SeatBridged
    fields:
      - {name: seat_id, type: Uuid}
      - {name: connect_seq, type: Optional<Integer>}
      - {name: bridge_seq, type: Integer}
  - name: relay.desk.SeatTaken
    fields:
      - {name: seat_id, type: Uuid}
      - {name: bridge_seq, type: Integer}
views:
  - name: relay.desk.Seats
    source: relay.desk.Seat
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: Uuid}
      - {name: agent_id, type: String}
      - {name: connect_seq, type: Optional<Integer>}
      - {name: bridge_seq, type: Integer}
commands:
  - name: relay.desk.Bridge
    input:
      - {name: agent_id, type: String}
    outcomes:
      - name: bridged
        creates: relay.desk.Seat
        instance: seat_id
        emits: [relay.desk.SeatBridged]
        payload:
          relay.desk.SeatBridged:
            seat_id: {generated: true}
            connect_seq: {generated: true, present: true}
            bridge_seq: {generated: true}
        sets:
          agent_id: input.agent_id
          connect_seq: {generated: true, present: true}
          bridge_seq: {payload: relay.desk.SeatBridged.bridge_seq}
  - name: relay.desk.TakeSeat
    input:
      - {name: agent_id, type: String}
      - {name: bridge_seq, type: Integer}
    outcomes:
      - name: taken
        creates: relay.desk.Seat
        instance: seat_id
        emits: [relay.desk.SeatTaken]
        payload:
          relay.desk.SeatTaken: {seat_id: {generated: true}, bridge_seq: input.bridge_seq}
        sets:
          agent_id: input.agent_id
          bridge_seq: {payload: relay.desk.SeatTaken.bridge_seq}
";

/// The `Bridge` act, claiming `SeatBridged` with the `payload` flow mapping.
fn bridged(payload: &str) -> String {
    act(
        0,
        "Bridge",
        "{agent_id: a1}",
        &format!(
            "    outcome: bridged\n    events:\n      - event: relay.desk.SeatBridged\n        \
             payload: {payload}\n"
        ),
    )
}

#[test]
fn an_exact_value_on_a_present_mint_is_refused_on_the_event_and_the_row() {
    let ir = fixture(SEATS);
    let text = pinned(&ir, &document("", &bridged("{connect_seq: 7}"), ""));
    names(
        &text,
        &[
            "the payload of `relay.desk.SeatBridged`",
            "`connect_seq`",
            "`relay.desk.Bridge/bridged`",
            "`{generated: true, present: true}`",
        ],
    );
    let text = pinned(
        &ir,
        &document(
            "",
            &bridged("{}"),
            "assert:\n  - view: relay.desk.Seats\n    contains: {agent_id: a1, connect_seq: 7}\n",
        ),
    );
    names(
        &text,
        &[
            "a row of `relay.desk.Seats`",
            "`connect_seq`",
            "`{generated: true, present: true}`",
        ],
    );
}

#[test]
fn an_exact_value_on_a_stored_field_read_from_a_minted_payload_field_is_refused() {
    let ir = fixture(SEATS);
    let text = pinned(
        &ir,
        &document(
            "",
            &bridged("{}"),
            "assert:\n  - view: relay.desk.Seats\n    contains: {agent_id: a1, bridge_seq: 7}\n",
        ),
    );
    names(
        &text,
        &[
            "a row of `relay.desk.Seats`",
            "`bridge_seq`",
            "`relay.desk.Bridge/bridged`",
            "`{payload: relay.desk.SeatBridged.bridge_seq}`",
            "`{generated: true}`",
        ],
    );
}

#[test]
fn a_stored_field_read_from_a_payload_field_the_input_fills_keeps_its_exact_value() {
    let ir = fixture(SEATS);
    accepted(
        &ir,
        &document(
            "",
            &act(
                0,
                "TakeSeat",
                "{agent_id: a1, bridge_seq: 7}",
                "    outcome: taken\n    events:\n      - event: relay.desk.SeatTaken\n        \
                 payload: {bridge_seq: 7}\n",
            ),
            "assert:\n  - view: relay.desk.Seats\n    contains: {agent_id: a1, bridge_seq: 7}\n",
        ),
    );
}
