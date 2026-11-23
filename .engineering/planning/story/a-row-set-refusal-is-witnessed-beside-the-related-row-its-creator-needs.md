---
format: aep.planning-md/3
id: story:a-row-set-refusal-is-witnessed-beside-the-related-row-its-creator-needs
kind: story
status: implemented
title: A row-set refusal is witnessed beside the related row its rows' creator needs
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names
- depends_on: story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-a-named-related-row.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-the-addressed-related-row.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:30:58Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-22T17:30:59Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-11-23T01:57:10Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

A row-set refusal that needs selected rows present (`exists: true`) gets its scenario where the
command creating those rows refuses a missing related row (`when_related: {via: input.x, exists:
false}`) and the selector compares `x == input.x`. This holds with no addressed subject, with rows
the command under test creates itself, and where the row-set command addresses that related row
as its subject without an `owns` relation. The two new fixture models synthesize with 0 refusals
and every new scenario passes on the interpreted target. No format version changes.

## What to change

Work order for the unit of `story:a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names`.
Line numbers are at commit `16cde5949b` of the wave branch. Your tree also holds the earlier
unit's merge, which does not touch `row_set.rs`, so these lines still hold.

There are two causes, both in `crates/verify/ess-conformance/src/synthesize/row_set.rs`. Both
were reproduced on the released 0.97.3 binary with the two fixture models below.

- **Cause 1: a literal identity, from `ess/23`.** In `Reading::new` (:853), lines 881–885 build
  the send's input with `resolved_literals` (:720) when `subject_fact::identity_selectors(ir)`
  holds (format `ess/23` or later). The send's input `shelf_id`, an arranged instance
  (`ScenarioValue::Instance`), becomes the shelf's literal identity, for example `"shelf_id-1"`.
  `Reading::closed` (:899) then writes that literal into the selector:
  `shelf_id == "shelf_id-1"`. Each arranged ticket's settled `shelf_id` still holds
  `ScenarioValue::Instance { instance: shelf-… }`. `instances_decided` (:4705) decides only
  fact-against-fact comparisons, so `Reading::truth` (:925) returns `Truth::Unknown`, and
  `consistent` (:1129, at :1205–1208) fails with "its own row set is Unknown" on every
  arrangement. With a target-generated identity (`{generated: true}`) there is no literal, and
  the same models pass today.
- **Cause 2: the addressed subject without `owns`.** `linked` (:4520) finds no `under` row,
  because `under_subject` (:3900) needs `ir.owner_of`. It then arranges a second, new related
  row with `related_row` (:4593) and binds it in `setup.bound` (:4596). `supply` gives the
  addressed subject precedence for the subject's own input field, so the send names the subject
  while the tickets name the new row. No arrangement selects them, and the branch is refused
  through `guarded_creator` (:3471) as `ROWS_GUARDED_CREATOR` (:255).

Steps:

1. `row_set.rs`, new private function right after `resolved_literals` (after line 739):

   ```rust
   /// The literal identity of the row the steps left under `instance`, where the steps named its
   /// identity by a literal (ess/23, beyond10x/ess#429).
   fn literal_identity<'r>(rows: &'r Rows, instance: &InstanceName) -> Option<&'r Node> {
       rows.held
           .iter()
           .filter(|row| row.instance.as_ref() == Some(instance))
           .find_map(|row| row.identity.as_ref().and_then(ScenarioValue::as_literal))
   }
   ```

   In `resolved_literals`, replace the lookup at lines 729–733 (`let named = rows.held.iter()…;`)
   with `let named = literal_identity(rows, instance);`. Its behaviour stays the same.

2. `row_set.rs`, `Reading::truth` (:925–961). Keep everything up to and including line 951
   (`let predicate = instances_decided(predicate, &settled, &self.supplied);`). It must run
   before the new block because it reads the instances. Between line 951 and the
   `subject_fact::row_truth_with(` call (:952), insert:

   ```rust
           // From `ess/23` a field naming an arranged row whose identity the steps named by a
           // literal holds that literal, as the send's input naming that row reads it
           // ([`resolved_literals`]): a row created naming a related row stores that row's identity,
           // so a selector comparing the field with the input decides it.
           if subject_fact::identity_selectors(self.ir) {
               for (field, held) in &row.settled {
                   let ScenarioValue::Instance { instance } = &held.value else {
                       continue;
                   };
                   if let Some(identity) = literal_identity(self.rows, instance) {
                       settled.to_mut().insert(
                           field.clone(),
                           Determined {
                               value: ScenarioValue::Literal {
                                   value: identity.clone(),
                               },
                               type_ref: held.type_ref.clone(),
                           },
                       );
                   }
               }
           }
   ```

   `row_truth_with` keeps reading `&settled`. Do not change `instances_decided`, `closed`,
   `members` or `Reading::new`.

3. `row_set.rs`, new private function right after `linked` (after line 4608):

   ```rust
   /// The subject the send addresses, as the setup's steps leave it, where the input `link`
   /// compares names it: a branch of `command` addresses an existing row of the linked entity
   /// through that input (`instance: <input>`). `None` for every other selector, whose related row
   /// [`linked`] arranges of its own.
   fn addressed_related(
       ir: &EssIr,
       command: &ResolvedCommand,
       link: &Link,
       setup: &Setup,
   ) -> Option<Arrangement> {
       let instance = setup.instance.as_ref()?;
       let addresses = command
           .outcomes
           .iter()
           .filter_map(|outcome| outcome.subject.as_ref())
           .any(|subject| {
               subject.effect != ResolvedEffect::Creates
                   && subject.entity == link.entity
                   && matches!(&subject.instance,
                       ResolvedInstance::Supplied { field } if field.name == link.input)
           });
       if !addresses {
           return None;
       }
       let (state, settled, unwritten) = walked_row(ir, &setup.steps, &link.entity, instance)?;
       Some(Arrangement {
           instance: instance.clone(),
           state,
           steps: Vec::new(),
           source: BTreeSet::new(),
           settled,
           unwritten,
           unobserved: BTreeSet::new(),
       })
   }
   ```

   The steps are empty because the subject's steps are already in `setup.steps`. The `Arrangement`
   fields are those `under_subject` fills at :3937–3945.

4. `row_set.rs`, `linked` (:4520–4608). Leave everything before line 4585 as it is, including
   the `related_guard::uses(command)` early return at :4582–4584. Replace the `let row = if let
   Some(row) = created.get(&link.input) { … } else { … };` at :4585–4599 with three arms:

   ```rust
               let row = if let Some(row) = created.get(&link.input) {
                   row.clone()
               } else if let Some(row) = addressed_related(ir, command, &link, setup) {
                   // The send's own subject is the related row: the rows are created naming it as
                   // the steps so far leave it, never a second row the send does not name.
                   if !passes_all(ir, &link, ir.commands().get(&link.creator), &row) {
                       return None;
                   }
                   created.insert(link.input.clone(), row.clone());
                   row
               } else {
                   // unchanged: the body of today's `else` arm, lines 4588–4598
               };
   ```

   The `Some(Linked { … })` built at :4600–4605 stays as it is. Add one sentence to `linked`'s
   doc comment (:4510–4519): "Where the command addresses that related row as its subject
   through the input the selector compares, the rows are created naming the subject
   ([`addressed_related`])." In `prepare`'s doc comment, append to the sentence ending
   "(beyond10x/ess#682)." on line 3199: " Where the send addresses that related row as its
   subject, the rows name the subject."

5. New file `crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-a-named-related-row.yaml`,
   exactly (92 lines, no comments):

   ```yaml
   format: ess/37
   system: shop
   version: v1
   domain: shop.store
   summary: A ticket is pinned only to a shelf that exists, in rising order; a shelf is cleared only while no ticket is pinned to it.
   types:
     - {name: shop.store.ShelfId, kind: newtype, of: String}
     - {name: shop.store.TicketId, kind: newtype, of: String}
   entities:
     - name: shop.store.Shelf
       identity: {name: shelf_id, type: shop.store.ShelfId}
       fields:
         - {name: title, type: String}
       lifecycle: {initial: Standing, states: [Standing], terminal: [Standing]}
     - name: shop.store.Ticket
       identity: {name: ticket_id, type: shop.store.TicketId}
       fields:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: seq, type: Integer}
       lifecycle: {initial: Pinned, states: [Pinned], terminal: [Pinned]}
   errors:
     - {name: shop.store.ShelfExists, summary: A shelf already carries this identity., fields: []}
     - {name: shop.store.NoSuchShelf, summary: No shelf carries this identity., fields: []}
     - {name: shop.store.OutOfOrder, summary: A ticket on the shelf has this sequence number or a later one., fields: []}
     - {name: shop.store.TicketsPinned, summary: A ticket is still pinned to the shelf., fields: []}
   events:
     - name: shop.store.ShelfBuilt
       fields: [{name: shelf_id, type: shop.store.ShelfId}]
     - name: shop.store.TicketPinned
       fields: [{name: ticket_id, type: shop.store.TicketId}]
   actors:
     - name: shop.store.Clerk
       may: [shop.store.BuildShelf, shop.store.PinTicket, shop.store.ClearShelf]
   commands:
     - name: shop.store.BuildShelf
       input:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: title, type: String}
       outcomes:
         - {name: shelf-exists, existing_instance: true, error: shop.store.ShelfExists}
         - name: built
           creates: shop.store.Shelf
           instance: shelf_id
           emits: [shop.store.ShelfBuilt]
           payload: {shop.store.ShelfBuilt: {shelf_id: input.shelf_id}}
           sets: {title: input.title}
     - name: shop.store.PinTicket
       input:
         - {name: ticket_id, type: shop.store.TicketId}
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: seq, type: Integer}
       outcomes:
         - name: no-shelf
           when_related: {via: input.shelf_id, exists: false}
           error: shop.store.NoSuchShelf
         - name: out-of-order
           when_related:
             entity: shop.store.Ticket
             where: {all: [shelf_id == input.shelf_id, seq >= input.seq]}
             exists: true
           error: shop.store.OutOfOrder
         - name: pinned
           creates: shop.store.Ticket
           instance: ticket_id
           emits: [shop.store.TicketPinned]
           payload: {shop.store.TicketPinned: {ticket_id: input.ticket_id}}
           sets: {shelf_id: input.shelf_id, seq: input.seq}
     - name: shop.store.ClearShelf
       input:
         - {name: shelf_id, type: shop.store.ShelfId}
       outcomes:
         - name: tickets-pinned
           when_related:
             entity: shop.store.Ticket
             where: "shelf_id == input.shelf_id"
             exists: true
           error: shop.store.TicketsPinned
         - {name: cleared, accepts: nothing}
   views:
     - name: shop.store.Tickets
       source: shop.store.Ticket
       consistency: read_your_writes
       fields:
         - {name: ticket_id, type: shop.store.TicketId}
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: seq, type: Integer}
     - name: shop.store.Shelves
       source: shop.store.Shelf
       consistency: read_your_writes
       fields:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: state, type: shop.store.Shelf.State}
   ```

   On 0.97.3 it prints `5 scenario(s) (0 authored), 2 refusal(s)`. Both refusals are
   ESS-SYNTH-001, for `shop.store.ClearShelf/tickets-pinned` and
   `shop.store.PinTicket/out-of-order`, with the text "`shelf_id` created by
   `shop.store.PinTicket`, whose `no-shelf` refuses a missing `shop.store.Shelf`". Expected after
   the change: 7 scenarios, 0 refusals. The tests assert only the refusals and the two
   scenarios, not the count.

6. New file `crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-the-addressed-related-row.yaml`,
   exactly (100 lines, no comments):

   ```yaml
   format: ess/37
   system: shop
   version: v1
   domain: shop.store
   summary: A ticket is pinned only to an open shelf that exists; a shelf is closed only while no ticket is pinned to it.
   types:
     - {name: shop.store.ShelfId, kind: newtype, of: String}
     - {name: shop.store.TicketId, kind: newtype, of: Uuid}
   entities:
     - name: shop.store.Shelf
       identity: {name: shelf_id, type: shop.store.ShelfId}
       fields:
         - {name: title, type: String}
       lifecycle:
         initial: Open
         states: [Open, Closed]
         terminal: [Closed]
         transitions:
           - {name: close, from: [Open], to: Closed}
     - name: shop.store.Ticket
       identity: {name: ticket_id, type: shop.store.TicketId}
       fields:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: note, type: String}
       lifecycle: {initial: Pinned, states: [Pinned], terminal: [Pinned]}
   errors:
     - {name: shop.store.NoSuchShelf, summary: No shelf carries this identity., fields: []}
     - {name: shop.store.ShelfClosed, summary: The shelf is closed., fields: []}
     - {name: shop.store.TicketsPinned, summary: A ticket is still pinned to the shelf., fields: []}
     - {name: shop.store.ShelfStateConflict, summary: The shelf is not open., fields: []}
   events:
     - name: shop.store.ShelfOpened
       fields: [{name: shelf_id, type: shop.store.ShelfId}]
     - name: shop.store.TicketPinned
       fields: [{name: ticket_id, type: shop.store.TicketId}]
     - name: shop.store.ShelfClosedEvent
       fields: [{name: shelf_id, type: shop.store.ShelfId}]
   actors:
     - name: shop.store.Clerk
       may: [shop.store.OpenShelf, shop.store.PinTicket, shop.store.CloseShelf]
   commands:
     - name: shop.store.OpenShelf
       input:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: title, type: String}
       outcomes:
         - name: opened
           creates: shop.store.Shelf
           instance: shelf_id
           emits: [shop.store.ShelfOpened]
           payload: {shop.store.ShelfOpened: {shelf_id: input.shelf_id}}
           sets: {title: input.title}
     - name: shop.store.PinTicket
       input:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: note, type: String}
       outcomes:
         - name: no-shelf
           when_related: {via: input.shelf_id, exists: false}
           error: shop.store.NoSuchShelf
         - name: shelf-closed
           when_related: {via: input.shelf_id, predicate: state == Closed}
           error: shop.store.ShelfClosed
         - name: pinned
           creates: shop.store.Ticket
           instance: ticket_id
           emits: [shop.store.TicketPinned]
           payload: {shop.store.TicketPinned: {ticket_id: {generated: true}}}
           sets: {shelf_id: input.shelf_id, note: input.note}
     - name: shop.store.CloseShelf
       input:
         - {name: shelf_id, type: shop.store.ShelfId}
       outcomes:
         - {name: no-shelf, unknown_instance: true, error: shop.store.NoSuchShelf}
         - name: tickets-pinned
           when_related:
             entity: shop.store.Ticket
             where: "shelf_id == input.shelf_id"
             exists: true
           error: shop.store.TicketsPinned
         - name: closed
           moves: shop.store.Shelf.close
           instance: shelf_id
           emits: [shop.store.ShelfClosedEvent]
           payload: {shop.store.ShelfClosedEvent: {shelf_id: input.shelf_id}}
         - {name: wrong-state, wrong_state: true, error: shop.store.ShelfStateConflict}
   views:
     - name: shop.store.Tickets
       source: shop.store.Ticket
       consistency: read_your_writes
       fields:
         - {name: ticket_id, type: shop.store.TicketId}
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: note, type: String}
     - name: shop.store.Shelves
       source: shop.store.Shelf
       consistency: read_your_writes
       fields:
         - {name: shelf_id, type: shop.store.ShelfId}
         - {name: state, type: shop.store.Shelf.State}
   ```

   On 0.97.3 it prints `8 scenario(s) (0 authored), 1 refusal(s)`: ESS-SYNTH-001 for
   `shop.store.CloseShelf/tickets-pinned`, with the same "created by `shop.store.PinTicket`,
   whose `no-shelf` refuses a missing `shop.store.Shelf`" text. This model needs both causes
   fixed. The test variant with a generated identity, `generated_shelf()` below, needs only
   cause 2. On 0.97.3 it also prints 8 scenarios and 1 refusal. With an `owns` relation from
   shelf to ticket and a generated identity, it already prints 0 refusals today. Expected after
   the change: 9 scenarios and 0 refusals for both texts.

7. Re-pin the three tables for the two new files (see `## Commands`). No other file changes.

## Tests first

All tests go in `crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs`.
Append them after the last test (file end, line 304). Reuse its helpers: `run` (:165),
`assert_passes` (:197) and `Run` (:160). Add one paragraph to the module doc (after line 15):
"Two fixture models (`tests/fixtures/row-set-refusal-beside-*.yaml`) hold the refusing branch of
the same shape: the shelf named by a literal identity the scenario sends, with no subject, with
the rows created by the command under test, and as the subject the row-set command addresses
(story:a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names)."

New items, written exactly:

```rust
/// A row-set refusal over rows whose creator refuses a missing shelf (`ess/37`): the shelf's
/// identity is a literal the scenario sends, `ClearShelf` addresses no subject, and `PinTicket`
/// refuses on a row set over the tickets it creates itself.
const NAMED: &str = include_str!("fixtures/row-set-refusal-beside-a-named-related-row.yaml");

/// The same shape where the row-set command addresses the shelf the rows name (`CloseShelf`,
/// `instance: shelf_id`), with no `owns` relation from shelf to ticket.
const ADDRESSED: &str =
    include_str!("fixtures/row-set-refusal-beside-the-addressed-related-row.yaml");

const CLEAR_PINNED: &str = "shop.store.ClearShelf/outcome/tickets-pinned";
const OUT_OF_ORDER: &str = "shop.store.PinTicket/outcome/out-of-order";
const CLOSE_PINNED: &str = "shop.store.CloseShelf/outcome/tickets-pinned";

/// `text` with `from`, which it holds once, replaced by `to`.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "written once: {from}");
    text.replacen(from, to, 1)
}

/// [`ADDRESSED`] with the shelf's identity generated by the target, not sent as a literal.
fn generated_shelf() -> String {
    let text = replaced(
        ADDRESSED,
        "  - {name: shop.store.ShelfId, kind: newtype, of: String}\n",
        "  - {name: shop.store.ShelfId, kind: newtype, of: Uuid}\n",
    );
    let text = replaced(
        &text,
        "      - {name: shelf_id, type: shop.store.ShelfId}\n      - {name: title, type: String}\n",
        "      - {name: title, type: String}\n",
    );
    replaced(
        &text,
        "{shop.store.ShelfOpened: {shelf_id: input.shelf_id}}",
        "{shop.store.ShelfOpened: {shelf_id: {generated: true}}}",
    )
}

/// Every send of scenario `wanted`, in order: the command, its input as JSON, and the outcome the
/// step after it expects.
fn sends(run: &Run, wanted: &str) -> Vec<(String, serde_json::Value, String)> {
    let (_, scenario) = run
        .synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .unwrap_or_else(|| panic!("{wanted} is synthesized"));
    let value = serde_json::to_value(scenario).unwrap();
    let steps = value["steps"].as_array().cloned().unwrap_or_default();
    steps
        .iter()
        .enumerate()
        .filter(|(_, step)| step["step"] == "execute_command")
        .map(|(at, step)| {
            let expected = steps
                .get(at + 1)
                .and_then(|next| next["outcome"]["outcome"].as_str())
                .unwrap_or_default();
            (
                step["command"].as_str().unwrap_or_default().to_owned(),
                step["input"].clone(),
                expected.to_owned(),
            )
        })
        .collect()
}

fn assert_refuses_nothing(run: &Run) {
    let all: Vec<String> = run
        .synthesis
        .refusals
        .iter()
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(all.is_empty(), "the model synthesizes without a refusal: {all:#?}");
}

/// The tickets [`CLOSE_PINNED`] pins name the shelf its `CloseShelf` send addresses, the one shelf
/// it opens.
fn assert_rows_name_the_addressed_shelf(run: &Run) {
    let sent = sends(run, CLOSE_PINNED);
    let opened = sent
        .iter()
        .filter(|(command, _, _)| command == "shop.store.OpenShelf")
        .count();
    assert_eq!(opened, 1, "one shelf is opened, the one the send addresses: {sent:#?}");
    let (last, input, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (last.as_str(), outcome.as_str()),
        ("shop.store.CloseShelf", "tickets-pinned"),
        "{sent:#?}"
    );
    let pinned: Vec<_> = sent
        .iter()
        .filter(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned")
        .collect();
    assert!(!pinned.is_empty(), "a ticket is pinned first: {sent:#?}");
    for (_, ticket, _) in pinned {
        assert_eq!(
            ticket["shelf_id"], input["shelf_id"],
            "the ticket names the shelf the send addresses: {sent:#?}"
        );
    }
}

#[test]
fn a_row_set_refusal_without_a_subject_is_witnessed_beside_the_shelf_the_send_names() {
    let run = run(NAMED);
    assert_passes(&run, CLEAR_PINNED);
    assert_refuses_nothing(&run);
    let sent = sends(&run, CLEAR_PINNED);
    assert!(
        sent.iter()
            .any(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned"),
        "a ticket is pinned before the shelf is cleared: {sent:#?}"
    );
}

#[test]
fn a_creators_own_row_set_refusal_is_witnessed_beside_the_shelf_the_send_names() {
    let run = run(NAMED);
    assert_passes(&run, OUT_OF_ORDER);
    let sent = sends(&run, OUT_OF_ORDER);
    let (last, _, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (last.as_str(), outcome.as_str()),
        ("shop.store.PinTicket", "out-of-order"),
        "{sent:#?}"
    );
    assert!(
        sent.iter()
            .any(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned"),
        "an earlier ticket is pinned first: {sent:#?}"
    );
}

#[test]
fn a_row_set_refusal_is_witnessed_beside_the_shelf_the_send_addresses() {
    let run = run(ADDRESSED);
    assert_passes(&run, CLOSE_PINNED);
    assert_refuses_nothing(&run);
    assert_rows_name_the_addressed_shelf(&run);
}

#[test]
fn with_a_generated_shelf_identity_the_rows_still_name_the_addressed_shelf() {
    let run = run(&generated_shelf());
    assert_passes(&run, CLOSE_PINNED);
    assert_refuses_nothing(&run);
    assert_rows_name_the_addressed_shelf(&run);
}
```

These four tests are red before steps 1–4, each at `assert_passes` ("… is not refused"). The
reasons: on 0.97.3 `ClearShelf/tickets-pinned` and `PinTicket/out-of-order` (cause 1) and
`CloseShelf/tickets-pinned` (causes 1 and 2) are ESS-SYNTH-001, and so is the generated variant
(cause 2 alone). Expected red line: `test result: FAILED. 5 passed; 4 failed`. The five existing
tests stay green throughout. Each test synthesizes one model of three commands. In the release
build that takes 0.03 s, and the interpreted run compiles no generated code, so each test should
take a few seconds at most. Time them, and name any test over 5 s with its time. None may exceed
60 s.

If the scenario JSON has no `steps` array or no `execute_command` value, check
`serde_json::to_value` of a scenario and fit the key names in `sends`. The CLI's `suite.json`
writes `{"step":"execute_command","command":…,"input":{"shelf_id":{"kind":"instance","instance":"shelf-1"}}}`
and then `{"step":"expect_outcome","outcome":{"command":…,"outcome":"pinned"}}`. Report the
change.

## Commands

Run from the worktree root. Every cargo command carries `CARGO_PROFILE_DEV_DEBUG=false
CARGO_INCREMENTAL=0`. A pool job ends after 10 minutes unless it names its time. The test builds
of `ess-conformance` and `ess-cli` are large, so those commands carry `SLOT_TIMEOUT=30m`, the
most allowed. A killed job is not restarted unchanged. Run each command that may take over 30 s
in the background, with its log under your scratch root, and wait on its PID or log. Do not put
the `slot` prefix on a script that waits or measures; `cargo` takes its own slot through the
shim. No Python anywhere, not even for the `comm` proof. Use only named `--test` targets, one per
command. Test budget: no test over 60 s, a test step within 120 s, and every new test over 5 s
named with its time. The table targets below are pre-existing; report their wall time, and if
one exceeds 60 s, report it as an existing defect, not as yours.

1. Red: `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test a_row_set_branch_arranges_rows_past_the_creators_guard`
   after writing the tests and fixtures (steps 5 and 6) and before steps 1–4. Expected:
   `5 passed; 4 failed`. Keep the line and the first failure message.
2. Make steps 1–4, then repeat command 1. Expected: `9 passed; 0 failed`.
3. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`.
   It fails because the table lacks the two new models and the header count is off. Re-pin with
   the command its failure prints:
   `SYNTHESIS_SUITE_BYTES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`.
   The component table (`synthesis-component-suites-table.tsv`) must not change, because the new
   models declare no component. Run the `comm` proof (below), then command 3 again. Expected:
   pass.
4. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`.
   It fails for the two new models. Re-pin with
   `SELECTION_PRECEDENCE_TABLE_WRITE="$PWD/crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`
   (the command its failure prints), run the `comm` proof, then rerun. Expected: pass.
5. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`.
   It fails for the two new models. Re-pin with
   `LOWERED_DEFINITIONS_TABLE_WRITE="$PWD/crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`
   (the command its failure prints), run the `comm` proof, then rerun. Expected: pass.
6. These are the byte pins of the repository's `AGENTS.md` that a synthesis change can move. Run
   each with `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p <crate> --locked --test <target>`,
   one command each. Expected: pass, with nothing re-pinned.
   - `ess-conformance`: `synthesis_suite_bytes_table_cli_read`
   - `ess-conformance`: `external_beside_held_guard`
   - `ess-conformance`: `adversary_related_values_all_bytes`
   - `ess-cli`: `tutorial_page` (leave `ESS_TUTORIAL_NETWORK` unset)

   Not run: `ess-synth` `served_codec_names` and `go`, `ess-openapi` `adversary_ess_issues_2_u6`,
   and `ess-compiler` `string_alphabet_and_length`. They compare generated code and projections,
   which read no synthesis output, and this change touches only synthesis.
7. Row-set neighbours, one command each, all in `ess-conformance`, with the same prefix as
   command 1. Expected: pass.
   - `row_sets`
   - `row_set_struct_identity`
   - `adversary_463_pass1`
   - `adversary_ess0700a_identity_beside_row_set`
   - `row_set_upsert`
8. `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`.
   Expected: no warnings.
9. Format your two Rust files. The test file is a leaf: `rustfmt --edition 2021 crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs`.
   `row_set.rs` declares modules (`mod copied; mod related_rows;`), so after
   `rustfmt --edition 2021 crates/verify/ess-conformance/src/synthesize/row_set.rs`, run
   `git status` and `git checkout --` every changed file you do not own. Then run
   `slot task fmt-check`. Expected: exit 0.

**The `comm` proof**, for each of the three tables. `<t>` is the table's path; `<s>` is your
scratch root.

```console
git show HEAD:<t> | tail -n +2 | LC_ALL=C sort > <s>/table.before
tail -n +2 <t> | LC_ALL=C sort > <s>/table.after
LC_ALL=C comm -23 <s>/table.before <s>/table.after
LC_ALL=C comm -13 <s>/table.before <s>/table.after
```

The first `comm` must print nothing: no pinned row moved or vanished. The second must print only
lines whose first column starts with
`crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-`. That is one line per model
for the bytes and lowered-definitions tables. For the precedence table it is the model's lines
and its commands' lines, as the write mode lays them out. The header line (line 1) changes only
in its counts. Put both outputs in your report.

**Expected: no existing row moves.** Cause 1's change reads a field as a literal only where the
field names an arranged row with a literal identity, in `ess/23` or later. Cause 2's change
applies only where a row-set command addresses the linked related row through the input its
selector compares, with no `owns` relation and no `when_related: {via: …}` guard of its own.
Today such branches are refused or witnessed without selected rows. A row may still move where a
committed model has one of these shapes and an earlier arrangement failed on "its row set is
Unknown". Five committed models carry a row-set selector beside a creator that refuses a missing
related row and are the ones to check first:

- `crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml`
- `crates/verify/ess-conformance/tests/fixtures/arrangement-through-writers.yaml`
- `crates/verify/ess-conformance/tests/fixtures/caller-swap-command-under-test.yaml`
- `crates/verify/ess-conformance/tests/fixtures/caller-swap-unread-members.yaml`
- `crates/verify/ess-conformance/tests/fixtures/related-guard-ranked-candidates.yaml`

If `comm -23` prints any line, stop before re-pinning it and report:

- the model;
- its old and new `suite=` and `refusals=` counts;
- which scenario ids were added or removed, from the two suites written with the 0.97.3-built
  `ess` and your build. Use `cargo run` only if your dispatch allows it; otherwise list the ids
  from the moved row's counts.

A moved row is acceptable only if its refusals count does not rise and its suite count does not
fall. Re-pin it in the same table, and in `external-beside-held-guard-base.tsv` if
`external_beside_held_guard` names it, and name the cause beside the pin in your report. Any
other move means the change is wrong: report it as red.

## Files this unit owns

- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (edit)
- `crates/verify/ess-conformance/tests/a_row_set_branch_arranges_rows_past_the_creators_guard.rs` (edit)
- `crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-a-named-related-row.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/row-set-refusal-beside-the-addressed-related-row.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (re-pin: two rows added)
- `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` (re-pin: new rows only)
- `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (re-pin: two rows added)
- `crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv` (only if an
  existing row moves as allowed above; expected untouched)

Must not touch:

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` and
  `crates/verify/ess-conformance/src/synthesize.rs`, owned by the earlier unit of this wave;
- `crates/verify/ess-conformance/src/synthesize/witness.rs`, `witness_memo.rs`,
  `synthesize/caller.rs`, `synthesize/row_set/related_rows.rs`, `synthesize/row_set/copied.rs`;
- `crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv`, which must
  stay byte-identical;
- the earlier unit's two fixture models and their table rows;
- `.engineering/`, `CHANGELOG.md`, `AGENTS.md`, `Taskfile.yml`, `.github/workflows/*`,
  `crates/edge/ess-xtask/tests/ci_lanes.rs`, `README.md`, `docs/`, `website/`.

## For the coordinator at integration

- `CHANGELOG.md`, under `## [Unreleased]` → `### Fixed`, append after the existing bullet:

  ```text
  - Synthesis witnesses a row-set branch that needs rows present where the command creating those rows refuses a missing related row the send names (`when_related: {via: input.x, exists: false}` beside a selector comparing `x == input.x`). From `ess/23` a row's field naming a related row whose identity the scenario sends as a literal is compared with the input as that literal. Where the row-set command addresses the related row as its subject without an `owns` relation, the rows are created naming that subject, not a second row the send does not name. Such branches were refused as `ESS-SYNTH-001`. No format version changes.
  ```

- Plan store: file a draft follow-up story, "A creator guarded by a related row and a row set
  drives its rows for another command's arrangement". Need: the rows' creator `PinTicket` reads
  a related row (`no-shelf`) and also refuses on a row set over its own rows (`out-of-order`).
  A command moving such a row (`FinishTicket`, `moves: Ticket.finish`) is refused three times as
  ESS-SYNTH-004, "the route runs through `PinTicket/pinned`, which no input reaches", while
  `PinTicket/pinned`'s own scenario is in the suite. Without the row-set refusal the same model
  synthesizes with 0 refusals, as reproduced on 0.97.3. The cause, inferred from reading and not
  run: `related_guard::selects` (`related_guard.rs:2064` at `16cde5949b`) never excludes a
  `ResolvedCondition::RelatedSet` branch, so it pushes it as selected. `answering_refusal`
  (:2151) then picks it over the accepting branch, and `drive` → `searched` (:2961) →
  `with_row` (:5191) finds no row that selects `pinned`. The fix belongs in `related_guard.rs`,
  which this wave's earlier unit owns. It covers the `ESS-SYNTH-004` part of the parent story's
  second acceptance row.
- No change to `AGENTS.md`, `Taskfile.yml`, workflows, `ci_lanes.rs`, `README.md`, `docs/` or
  `website/`.

## Out of scope

- The three ESS-SYNTH-004 refusals of a move whose route runs through a creator that also refuses
  on a row set. Their cause is in `related_guard.rs` (`selects`), which this unit must not edit.
  It is the follow-up above. Do not add `FinishTicket`-like moves to the fixtures.
- Any edit to `related_guard.rs`, `synthesize.rs`, `witness.rs`, `witness_memo.rs` or `caller.rs`.
- `consistent` (:1129–1216): the sibling story
  `a-row-set-upserts-creating-branch-is-arranged-past-its-row-set-refusals` changes it. Do not
  touch it here.
- Set effects over the selector's entity (`UNFOLLOWED`, sibling
  `a-row-set-refusal-beside-a-set-effect-on-its-entity-is-witnessed`). Non-`Boolean` selectors
  (sibling `a-row-set-refusal-under-a-non-boolean-selector-is-witnessed`).
- Commands that address the subject and also carry a `when_related: {via: …}` guard. The early
  return at :4582–4584 stays.
- `under_subject`, `instances_decided`, `Reading::closed`, `Reading::new`, `guarded_creator` and
  every `ROWS_*` text: unchanged. No new diagnostic, no `ValidationCode`, no help text change.
- Below `ess/23` nothing changes. The step 2 block is guarded by
  `subject_fact::identity_selectors`.
- No format version changes. This change touches synthesis only, so suites of models with these
  shapes gain scenarios.

## Acceptance

- `a_row_set_branch_arranges_rows_past_the_creators_guard`: 4 new tests red before steps 1–4
  (`5 passed; 4 failed`), all 9 green after (commands 1 and 2).
- `ClearShelf/tickets-pinned`, `PinTicket/out-of-order` and `CloseShelf/tickets-pinned`, with
  the literal and with the generated shelf identity, are synthesized with 0 refusals and pass on
  the interpreted target. The tickets of `CloseShelf/tickets-pinned` name the shelf the send
  addresses, and only one shelf is opened (tests 1–4).
- `synthesis_suite_bytes_table`, `selection_precedence_table` and `lowered_definitions_table`
  pass after re-pinning. Each table's `comm -23` prints nothing, and each `comm -13` prints only
  the new fixtures' lines (commands 3–5).
- `synthesis_suite_bytes_table_cli_read`, `external_beside_held_guard`,
  `adversary_related_values_all_bytes`, `tutorial_page` and the row-set neighbours of command 7
  pass unchanged (commands 6 and 7).
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings` and
  `slot task fmt-check` are clean (commands 8 and 9).
- The parent story's acceptance rows map as follows:
  - Rows 1 and 4 (g5h, and rows pinned) are met by `NAMED` and the tables.
  - Row 2 (g5) is met for `out-of-order` and the addressed-subject refusal by `NAMED` and
    `ADDRESSED`. Its ESS-SYNTH-004 part is the follow-up.
  - Row 3, the active story's part A, is not run here. Its example with an addressed parent, a
    literal identity and no `owns` has the shape of `ADDRESSED`, and on 0.97.3 it is refused
    with the same text.

## Depends on

The earlier unit of this wave (story
`a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own`; it edits
`related_guard.rs`, `synthesize.rs` and adds two fixture models). This unit uses none of its
functions. It depends on that unit's merge only for the three `.tsv` tables, which both units
re-pin. Start this unit's worktree from the wave head after that merge, so that your re-pin keeps
its rows and `comm -13` shows only your two models.
