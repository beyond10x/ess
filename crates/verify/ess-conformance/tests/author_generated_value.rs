//! An authored exact value on a field the model marks `{generated: true}` (beyond10x/ess#544).
//!
//! The implementation mints a generated field, and synthesis checks only its presence and type. An
//! authored act that pins an exact value on one — in an event's payload, an error's fields, or a
//! view row over a stored field a `sets:` entry generates — passes only when the implementation
//! happens to mint that value, and fails a conforming one. `author` refuses it
//! (`ESS-AUTHOR-042`), naming the field. A field the outcome fills from the input or a literal
//! keeps its exact value, and a generated field left out of the expectation is accepted.
//!
//! From `format: ess/30` (beyond10x/ess#623) the value is the scenario's own claim: a scenario's
//! earlier steps can determine what the implementation mints (a revision counting its commits, a
//! cursor after the page it asked for), so the act is accepted and the suite checks the value as
//! written. An implementation minting another value fails it. Below `ess/30` the refusal reads as
//! before, its help naming `ess/30`. Synthesis still never pins a minted value.

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
         only where it happens to mint that one: in an event's payload or an error's fields, \
         drop the field from the expectation, and the suite still checks its presence and type \
         there, or relate it to another value with `capture:` on the event; in a view row, drop \
         it and assert `satisfies: defined(<field>)` on the view, which requires it on every row \
         and at least one row, or state its relation to an event's value with `{$observed: \
         {event: …, field: …}}`; from `format: ess/30` an exact value there is the scenario's \
         own claim, which the suite checks as written"
    );
}

/// A view read does not check a row's declared fields beyond what the scenario claims of it, so
/// the help a pinned row value gets does not tell its author the suite checks the dropped field's
/// presence and type there; it names `satisfies: defined(<field>)` and `$observed` instead, and
/// both of those are accepted for the field the refusal named (beyond10x/ess#680).
#[test]
fn the_generated_value_hint_names_satisfies_for_a_view_row() {
    let ir = fixture(RELAY);
    let text = pinned(
        &ir,
        &document(
            "",
            &selected("{agent_id: a1}"),
            "assert:\n  - view: relay.desk.Entries\n    excludes: {agent_id: a1, connect_seq: 7}\n",
        ),
    );
    names(&text, &["a row of `relay.desk.Entries`", "`connect_seq`"]);
    let help = text
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("help: "))
        .unwrap_or_else(|| panic!("the refusal carries a help line: {text}"));
    let row = help
        .split("; ")
        .find(|clause| clause.contains("view row"))
        .unwrap_or_else(|| panic!("the help speaks of a view row: {help}"));
    names(
        row,
        &[
            "`satisfies: defined(<field>)`",
            "{$observed: {event: …, field: …}}",
        ],
    );
    assert!(
        !row.contains("presence and type"),
        "the view row clause claims no presence and type check: {row}"
    );
    for clause in help.split("; ") {
        if clause.contains("presence and type") {
            assert!(
                clause.contains("an event's payload") && !clause.contains("view row"),
                "only the payload clause claims a presence and type check: {clause}"
            );
        }
    }
    accepted(
        &ir,
        &document(
            "",
            &selected("{agent_id: a1}"),
            "assert:\n  - view: relay.desk.Entries\n    satisfies: defined(connect_seq)\n",
        ),
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

// ---- from `ess/30`: the scenario's own claim (beyond10x/ess#623) -------------------------------

/// `RELAY` declared at `format`.
fn relay(format: &str) -> EssIr {
    let text = RELAY.replace("format: ess/19", &format!("format: {format}"));
    assert_ne!(text, RELAY);
    fixture(&text)
}

/// Every act this file refuses below `ess/30`, with the field it pins and the value it pins, in
/// JSON.
fn pins() -> Vec<(String, &'static str, &'static str)> {
    let row = |claim: &str| {
        document(
            "",
            &selected("{agent_id: a1}"),
            &format!(
                "assert:\n  - view: relay.desk.Entries\n    {claim}: \
                 {{agent_id: a1, connect_seq: 7}}\n"
            ),
        )
    };
    vec![
        (
            document("", &selected("{agent_id: a1, connect_seq: 7}"), ""),
            "connect_seq",
            "7",
        ),
        (row("contains"), "connect_seq", "7"),
        (row("excludes"), "connect_seq", "7"),
        (
            document(
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
            "connect_seq",
            "7",
        ),
        (
            document(
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
            "retry_token",
            "\"t-1\"",
        ),
        (
            document(
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
            "ref",
            "{\"id\":\"r1\",\"label\":\"L\"}",
        ),
        (
            document(
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
            "ticket",
            "\"T-1\"",
        ),
    ]
}

/// Every exact value the steps of `ir`'s one scenario for `text` compare, by field: an event's
/// payload, an error's fields and a view row's literals.
fn claimed(ir: &EssIr, text: &str) -> Vec<(String, ess_primitives::node::Node)> {
    use ess_conformance::{ScenarioStep, ViewExpectation};
    let compiled = authoring(ir, text);
    let mut found = Vec::new();
    for scenario in compiled.scenarios.values() {
        for step in &scenario.steps {
            let values: Vec<(&String, &ess_primitives::node::Node)> = match step {
                ScenarioStep::ExpectEvent { payload, .. } => payload.iter().collect(),
                ScenarioStep::ExpectError { fields, .. } => fields.iter().collect(),
                ScenarioStep::ExpectView {
                    expectation:
                        ViewExpectation::Contains { fields } | ViewExpectation::Excludes { fields },
                    ..
                } => fields
                    .iter()
                    .filter_map(|(field, value)| value.as_literal().map(|it| (field, it)))
                    .collect(),
                _ => Vec::new(),
            };
            for (field, value) in values {
                found.push((field.clone(), value.clone()));
            }
        }
    }
    found
}

#[test]
fn from_ess_30_an_exact_value_on_a_generated_field_is_the_scenarios_own_claim() {
    let ir = relay("ess/30");
    for (text, field, value) in pins() {
        accepted(&ir, &text);
        let claimed = claimed(&ir, &text);
        let value: ess_primitives::node::Node =
            serde_json::from_str(value).expect("the expected value is JSON");
        assert!(
            claimed.contains(&(field.to_owned(), value.clone())),
            "the step compares `{field}` to {value:?} exactly: {claimed:?}\n{text}"
        );
    }
}

#[test]
fn below_ess_30_every_pin_is_refused_as_before_and_the_help_names_ess_30() {
    let ir = relay("ess/29");
    for (text, _, _) in pins() {
        let refusal = pinned(&ir, &text);
        let (message, help) = refusal
            .split_once("\n  help: ")
            .unwrap_or_else(|| panic!("the refusal has a help line: {refusal}"));
        assert_eq!(
            pinned(&fixture(RELAY), &text)
                .split_once("\n  help: ")
                .map(|it| it.0),
            Some(message),
            "the message reads as it does at `ess/19`"
        );
        assert!(help.contains("`format: ess/30`"), "{help}");
    }
}

#[test]
fn from_ess_30_a_present_mint_and_a_minted_payload_read_keep_their_exact_value() {
    let text = SEATS.replace("format: ess/27", "format: ess/30");
    assert_ne!(text, SEATS);
    let ir = fixture(&text);
    accepted(&ir, &document("", &bridged("{connect_seq: 7}"), ""));
    for field in ["connect_seq", "bridge_seq"] {
        accepted(
            &ir,
            &document(
                "",
                &bridged("{}"),
                &format!(
                    "assert:\n  - view: relay.desk.Seats\n    contains: {{agent_id: a1, {field}: \
                     7}}\n"
                ),
            ),
        );
    }
}

#[test]
fn a_synthesized_suite_never_pins_a_generated_value_at_ess_30() {
    let ir = relay("ess/30");
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    assert!(!suite.scenarios.is_empty());
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            if let ess_conformance::ScenarioStep::ExpectEvent { payload, .. } = step {
                for minted in ["connect_seq", "entry_id", "label"] {
                    assert!(
                        !payload.contains_key(minted),
                        "{id} pins the minted `{minted}`: {payload:?}"
                    );
                }
            }
        }
    }
}

// ---- conformance: the pinned value is checked exactly ------------------------------------------

/// The interpreter, with every `connect_seq` it publishes and every one the view shows replaced by
/// `minted`: an implementation minting that value.
struct Minting {
    inner: ess_conformance::interpret::Interpreted,
    minted: i64,
}

impl Minting {
    fn mint(&self, fields: &mut std::collections::BTreeMap<String, ess_primitives::node::Node>) {
        if let Some(value) = fields.get_mut("connect_seq") {
            *value = ess_primitives::node::Node::Number(self.minted.into());
        }
    }
}

impl ess_conformance::ConformanceTarget for Minting {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(
        &self,
        scenario: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(
        &self,
        scenario: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            self.mint(&mut event.payload);
        }
        Ok(result)
    }
    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            self.mint(row);
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        let mut events = self.inner.observe_events(request)?;
        for event in &mut events {
            self.mint(&mut event.payload);
        }
        Ok(events)
    }
    fn configure_external_outcome(
        &self,
        control: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(
        &self,
        request: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// The status of `text`'s one scenario, compiled against `ir`, on an implementation minting
/// `connect_seq` as `minted`.
fn status(ir: &EssIr, text: &str, minted: i64) -> ess_conformance::report::Status {
    let compiled = authoring(ir, text);
    assert!(compiled.is_complete(), "{:#?}", compiled.refusals);
    let mut suite =
        ess_conformance::ConformanceSuite::new(ess_conformance::SuiteProvenance::of(ir));
    suite.scenarios = compiled.scenarios;
    suite.select_fresh_format_for(ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite)
        .unwrap_or_else(|error| panic!("{error}"));
    let target = Minting {
        inner: ess_conformance::interpret::Interpreted::for_model(ir.clone()),
        minted,
    };
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    let [scenario] = report.scenarios.as_slice() else {
        panic!("one scenario: {:#?}", report.scenarios);
    };
    scenario.status
}

#[test]
fn a_pinned_generated_value_passes_where_it_is_minted_and_fails_where_another_is() {
    use ess_conformance::report::Status;
    let ir = relay("ess/30");
    let on_the_event = document("", &selected("{agent_id: a1, connect_seq: 7}"), "");
    let on_the_row = document(
        "",
        &selected("{agent_id: a1}"),
        "assert:\n  - view: relay.desk.Entries\n    contains: {agent_id: a1, connect_seq: 7}\n",
    );
    for text in [on_the_event, on_the_row] {
        assert_eq!(status(&ir, &text, 7), Status::Passed, "{text}");
        assert_eq!(status(&ir, &text, 8), Status::Failed, "{text}");
    }
}
