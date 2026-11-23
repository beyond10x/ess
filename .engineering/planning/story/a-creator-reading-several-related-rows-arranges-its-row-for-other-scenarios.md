---
format: aep.planning-md/3
id: story:a-creator-reading-several-related-rows-arranges-its-row-for-other-scenarios
kind: story
status: active
title: A creating branch that reads several related rows arranges its row for other scenarios wherever its own scenario is witnessed
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:a-creating-branch-that-reads-several-related-rows-arranges-its-row-for-other-scenarios-wherever-its-own-scenario-is-witnessed
- depends_on: story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T20:22:05Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-22T20:22:07Z", actor: "human:timo", revision: 9}
---
## Outcome

- A creating branch that reads two or more related rows through its input, and whose run the strict setup path cannot arrange, is arranged for other scenarios exactly as its own scenario is. The strict setup path is `drive_several`, which arranges each row beside for every input.
- In the new fixture `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml`, `ess verify conform synthesize` writes 17 scenarios and 0 refusals, and every scenario passes on the interpreter. On the 0.97.4 build today it writes 10 scenarios and 7 refusals. The same holds for the in-test variant where the guard over the row beside reads the input.
- Every setup run that succeeds today is arranged as before. The fallback runs only where the strict path returns an error, and when it fails too the strict path's error is returned unchanged. So no refusal text changes, and no row of the three pin tables moves except the new fixture's own rows.
- No format version changes, no `ValidationCode` changes, no refusal text changes.

## What to change

Line numbers are at `0bda31d1c4` (the wave branch after unit S1 merged). Record the base before you start: `BASE=$(git rev-parse HEAD)`. One commit. All code changes are in one file, `crates/verify/ess-conformance/src/synthesize/related_guard.rs`.

Why: a command reading several related rows gets its own scenario from `arranged` (line 3563). `arranged` admits the rows beside without an input (`beside(.., (BESIDE, false))`, lines 3582-3589). It chooses the input with them through `arranged_beside` (3628), arranges each row whose guards read the input again for that input (`steered_beside`, 3685-3732), and shares an owner between the rows (`arranged_at` → `own_arrangement`). A setup run of the same branch instead goes through `drive` (2597) → `drive_several` (2830-2940). That path arranges each row beside strictly, for every input (`beside_row(.., (at, true))`, lines 2858-2864), and returns `beside_refused` at 2876-2881 when a predicate over that row reads the input. It re-arranges a row only for an input the caller already chose (`resteer_beside`, 2887-2895), and shares no owner. Its error reaches `synthesize.rs` `created_owned_made` (8769, `.map_err(|_| Unreachable::Unwitnessable {` at 8802), which prints "the route runs through `<branch>`, which no input reaches". That claim is false here, because the branch's own scenario is in the suite.

1. Rename `fn drive_several` (line 2830) to `fn drive_several_strict`. Keep its signature, its body and its doc comment (lines 2819-2827) unchanged. Its only caller, `drive` at line 2615 (`return drive_several(`), keeps calling `drive_several`, which step 3 adds.

2. Directly above the doc comment of the renamed function, add a re-entrancy guard modelled on `opening` (lines 1327-1380):

   ```rust
   thread_local! {
       /// The setup runs arranged as their branch's own scenario open on this thread, innermost
       /// last ([`own_opening`]).
       static OWN_OPEN: RefCell<Vec<(*const EssIr, QualifiedName, String)>> =
           const { RefCell::new(Vec::new()) };
   }

   /// One setup run arranged as its branch's own scenario, open until dropped ([`own_opening`]).
   struct OwnOpened;

   impl Drop for OwnOpened {
       fn drop(&mut self) {
           OWN_OPEN.with(|open| {
               open.borrow_mut().pop();
           });
       }
   }

   /// Opens the arrangement of `driver`'s run as its own scenario ([`as_own_scenario`]), or `None`
   /// where that same arrangement is already open further up: its own scenario's rows would need
   /// the row it is creating, and the arrangement would never end.
   fn own_opening(ir: &EssIr, driver: &super::Driver<'_>) -> Option<OwnOpened> {
       OWN_OPEN.with(|open| {
           let mut open = open.borrow_mut();
           let run = (
               std::ptr::from_ref(ir),
               driver.command.name.clone(),
               driver.outcome.name.to_string(),
           );
           if open.contains(&run) {
               return None;
           }
           open.push(run);
           Some(OwnOpened)
       })
   }
   ```

3. Directly below the guard, add the new `drive_several` and its two helpers:

   ```rust
   /// [`drive`] on a command reading several rows (beyond10x/ess#283): [`drive_several_strict`],
   /// and where that finds no run for a creating branch, the run arranged as the branch's own
   /// scenario is ([`as_own_scenario`]). A row beside whose guard reads the input holds for no
   /// input but some, and a link of the row the creation is filed under compared with the input
   /// naming another row needs both rows under one owner. The strict search admits neither, and
   /// the branch's own scenario arranges both. Where both fail, the strict search's refusal is the
   /// answer, so every run the strict search finds, and every refusal it gives, stays as it was.
   // One argument per thing an arranging run is told, as [`drive`] takes them.
   #[allow(clippy::too_many_arguments)]
   fn drive_several(
       ir: &EssIr,
       driver: &super::Driver<'_>,
       instance: Option<&crate::scenario::InstanceName>,
       actors: &BTreeMap<QualifiedName, ActorRef>,
       distinction: Distinction,
       (bound, known): (
           &BTreeMap<String, crate::scenario::InstanceName>,
           &[super::ScenarioStep],
       ),
       input: Option<&BTreeMap<String, Node>>,
       arranging: &[&EntityHandle],
   ) -> Result<super::Invocation, RefusalCause> {
       let strict = drive_several_strict(
           ir,
           driver,
           instance,
           actors,
           distinction,
           (bound, known),
           input,
           arranging,
       );
       match strict {
           Err(cause)
               if instance.is_none()
                   && input.is_none()
                   && known.is_empty()
                   && own_scenario_applies(driver, bound, arranging) =>
           {
               as_own_scenario(ir, driver, actors, distinction, bound).map_err(|_| cause)
           }
           other => other,
       }
   }

   /// Whether a setup run of `driver` may be arranged as its own scenario: a creating branch,
   /// every row it reads named by an input the caller has not bound, of an entity no arrangement
   /// around the run is already arranging.
   fn own_scenario_applies(
       driver: &super::Driver<'_>,
       bound: &BTreeMap<String, crate::scenario::InstanceName>,
       arranging: &[&EntityHandle],
   ) -> bool {
       matches!(driver.effect, ResolvedEffect::Creates)
           && rows(driver.command).into_iter().all(|(via, entity)| {
               matches!(via, ResolvedRelatedVia::Input { .. })
                   && !bound.contains_key(via.field())
                   && !arranging.contains(&entity)
           })
   }

   /// The run of `driver` arranged as its own scenario is ([`arranged`]), at the caller's
   /// `distinction`: the rows the arrangement names joined to `bound`, its steps first, then the
   /// send of the input it chose. Refused where the same arrangement is already open
   /// ([`own_opening`]) or names a row through a field `bound` already binds.
   fn as_own_scenario(
       ir: &EssIr,
       driver: &super::Driver<'_>,
       actors: &BTreeMap<QualifiedName, ActorRef>,
       distinction: Distinction,
       bound: &BTreeMap<String, crate::scenario::InstanceName>,
   ) -> Result<super::Invocation, RefusalCause> {
       let _open = own_opening(ir, driver).ok_or_else(unarranged)?;
       let models = super::caller::InvocationModels::plain(ir);
       let (setup, input, _) = arranged(
           &models,
           driver.command,
           driver.outcome,
           actors,
           distinction,
           None,
       )?;
       let mut bound = bound.clone();
       for (field, named) in &setup.bound {
           if bound.insert(field.clone(), named.clone()).is_some() {
               return Err(unarranged());
           }
       }
       let mut invocation = super::invoke_with(ir, driver, None, actors, &bound, &input);
       let mut steps = setup.steps;
       steps.append(&mut invocation.steps);
       invocation.steps = steps;
       invocation.source.extend(setup.source);
       Ok(invocation)
   }
   ```

   Every name used is already in scope in `related_guard.rs`:
   - Imports: `RefCell` (line 33), `BTreeMap` (34), `EssIr`, `ResolvedEffect`, `ResolvedRelatedVia`, `EntityHandle` (36-40), `QualifiedName` (43), `Node` (45), `ActorRef`, `Distinction`, `RefusalCause` (48-51).
   - `rows` (1005), `arranged` (3563) and `unarranged` (453) are functions of this file. `unaccompanied` (line 3795) already calls `arranged(&super::caller::InvocationModels::plain(ir), …)`.
   - `super::invoke_with` (`synthesize.rs` 9586, `fn invoke_with(ir, driver, instance, actors, bound, input) -> Invocation`) is already called at lines 2678 and 3036.
   - `Invocation`'s and `Setup`'s fields are private to `synthesize`, and this child module already reads and writes them (`invocation.steps`, `setup.bound`).
   - `Driver::effect` is `&ResolvedEffect` (`ess-compiler/src/ir.rs` 3306-3312), and `matches!(driver.effect, ResolvedEffect::Creates)` is the form `synthesize.rs` uses.

   If a name does not resolve, fix the import in this file only.

4. `open_key` (line 1396) keys what a search makes (`created_memo.rs` 130, `run_memo.rs` 118, `first_kept.rs` 71, `subject_fact.rs` 2683, 5683 and 6852 call it). A search inside an open own-scenario arrangement can be cut by `own_opening`, so it must not share a memo entry with the same search outside one. Append the open own-scenario arrangements to the key, and only when there is one, so every key is byte for byte as before while none is open:

   ```rust
   pub(super) fn open_key() -> String {
       let beside = BESIDE_OPEN.with(|open| {
           open.borrow()
               .open
               .iter()
               .map(|(model, command, field)| format!("{model:p}:{command}:{field}"))
               .collect::<Vec<_>>()
               .join(",")
       });
       OWN_OPEN.with(|open| {
           let open = open.borrow();
           if open.is_empty() {
               beside
           } else {
               let own = open
                   .iter()
                   .map(|(model, command, outcome)| format!("{model:p}:{command}/{outcome}"))
                   .collect::<Vec<_>>()
                   .join(",");
               format!("{beside}|own:{own}")
           }
       })
   }
   ```

   Keep its doc comment, and add one sentence to it: "An arrangement of a setup run as its own scenario ([`own_opening`]) cuts the same arrangement inside it, so the open ones are part of the key too."

5. The 0.97.4 related-row memo. `chooses` (398) reads `row_picks` (432), whose key is the command name, the entity, the row's `state`, `settled` and `unwritten` (lines 445-448), and then the whole input (`RowPicks::answer`). It caches `selects` (2064), which reads only the model, the command, the row and the input. `as_own_scenario` reaches `selects` only through `arranged`, with the same model, so every answer it reads is the answer `selects` gives for that key: a stale answer is not possible. `row_picks` is used only for the command the model declares (`std::ptr::eq` at 436-439), and `arranged` runs on `projection` clones, which bypass it.

   Do not read any context (`OWN_OPEN`, `BESIDE_OPEN`, the arranging chain, a distinction) inside `selects`, `chooses` or `row_picks`, and do not change their keys.

6. Add the fixture `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml` with exactly the content under "Fixture" in `## Tests first`, the two tests, and the table rows (`## Commands` 5-7).

   Commit message file:

   ```text
   fix(conformance): a creating branch reading several related rows is arranged for other scenarios as its own scenario is

   A setup run of a creating branch that reads two or more related rows arranged every row beside
   for every input and shared no owner between the rows. A guard over a row beside that reads the
   input, or a link of the row the creation is filed under compared with the input naming another
   row, therefore refused the run, and every scenario that needed the row it creates was refused
   as ESS-SYNTH-004 "which no input reaches" or ESS-SYNTH-003, although the branch's own scenario
   was in the suite. Where the strict search finds no run, the run is now arranged as the branch's
   own scenario is. Every run the strict search finds, and every refusal text, is unchanged.
   ```

## Tests first

Write the fixture and the tests, run them (`## Commands` 1), and see both tests fail before you change `related_guard.rs`.

Fixture: `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml`. Use exactly this content (198 lines). The `label` input of `IssueGrant` is unused here on purpose: the variant test reads it.

```yaml
format: ess/37
system: shop
version: v1
domain: shop.desk
summary: >-
  A ticket is received only beside a clerk listening on its channel and a grant that is not
  revoked; a grant is issued only to a desk of the site the caller names.
types:
  - {name: shop.desk.SiteId, kind: newtype, of: Uuid}
  - {name: shop.desk.DeskId, kind: newtype, of: Uuid}
  - {name: shop.desk.GrantId, kind: newtype, of: Uuid}
  - {name: shop.desk.ClerkId, kind: newtype, of: Uuid}
  - {name: shop.desk.TicketId, kind: newtype, of: Uuid}
  - {name: shop.desk.GrantName, kind: enum, variants: [chat, mail, push]}
  - {name: shop.desk.Mode, kind: enum, variants: [free, asked, denied]}
entities:
  - name: shop.desk.Site
    identity: {name: site_id, type: shop.desk.SiteId}
    fields:
      - {name: label, type: String}
    relations:
      - {name: desks, kind: owns, target: shop.desk.Desk, cardinality: many, via: site_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.desk.Desk
    identity: {name: desk_id, type: shop.desk.DeskId}
    fields:
      - {name: site_id, type: shop.desk.SiteId}
    relations:
      - {name: grants, kind: owns, target: shop.desk.Grant, cardinality: many, via: desk_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.desk.Grant
    identity: {name: grant_id, type: shop.desk.GrantId}
    fields:
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: site_id, type: shop.desk.SiteId}
      - {name: name, type: shop.desk.GrantName}
      - {name: mode, type: shop.desk.Mode}
    relations:
      - {name: site, kind: references, target: shop.desk.Site, cardinality: one, via: site_id}
    lifecycle:
      initial: Granted
      states: [Granted, Revoked]
      terminal: [Revoked]
      transitions:
        - {name: revoke, from: [Granted], to: Revoked}
  - name: shop.desk.Clerk
    identity: {name: clerk_id, type: shop.desk.ClerkId}
    fields:
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: channel, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.desk.Ticket
    identity: {name: ticket_id, type: shop.desk.TicketId}
    fields:
      - {name: clerk_id, type: shop.desk.ClerkId}
      - {name: request, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: shop.desk.SiteNotFound, summary: No site carries this identity., fields: []}
  - {name: shop.desk.DeskNotFound, summary: No desk carries this identity., fields: []}
  - {name: shop.desk.DeskElsewhere, summary: The desk belongs to another site., fields: []}
  - {name: shop.desk.GrantNotFound, summary: No grant carries this identity., fields: []}
  - {name: shop.desk.GrantRefused, summary: The grant is revoked or does not allow this., fields: []}
  - {name: shop.desk.ClerkNotFound, summary: No clerk carries this identity., fields: []}
  - {name: shop.desk.OtherChannel, summary: The clerk listens on another channel., fields: []}
events:
  - name: shop.desk.SiteOpened
    fields: [{name: site_id, type: shop.desk.SiteId}]
  - name: shop.desk.DeskOpened
    fields: [{name: desk_id, type: shop.desk.DeskId}]
  - name: shop.desk.GrantIssued
    fields: [{name: grant_id, type: shop.desk.GrantId}]
  - name: shop.desk.GrantRevoked
    fields: [{name: grant_id, type: shop.desk.GrantId}]
  - name: shop.desk.ClerkAdded
    fields: [{name: clerk_id, type: shop.desk.ClerkId}]
  - name: shop.desk.TicketReceived
    fields: [{name: ticket_id, type: shop.desk.TicketId}]
actors:
  - name: shop.desk.Keeper
    may: [shop.desk.OpenSite, shop.desk.OpenDesk, shop.desk.IssueGrant, shop.desk.RevokeGrant, shop.desk.AddClerk, shop.desk.ReceiveTicket]
commands:
  - name: shop.desk.OpenSite
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: shop.desk.Site
        instance: site_id
        emits: [shop.desk.SiteOpened]
        payload: {shop.desk.SiteOpened: {site_id: {generated: true}}}
        sets: {label: input.label}
  - name: shop.desk.OpenDesk
    input:
      - {name: site_id, type: shop.desk.SiteId}
    outcomes:
      - name: no-site
        when_related: {via: input.site_id, exists: false}
        error: shop.desk.SiteNotFound
      - name: opened
        creates: shop.desk.Desk
        instance: desk_id
        emits: [shop.desk.DeskOpened]
        payload: {shop.desk.DeskOpened: {desk_id: {generated: true}}}
        sets: {site_id: input.site_id}
  - name: shop.desk.IssueGrant
    input:
      - {name: label, type: String}
      - {name: site_id, type: shop.desk.SiteId}
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: name, type: shop.desk.GrantName}
      - {name: mode, type: shop.desk.Mode}
    outcomes:
      - name: no-site
        when_related: {via: input.site_id, exists: false}
        error: shop.desk.SiteNotFound
      - name: no-desk
        when_related: {via: input.desk_id, exists: false}
        error: shop.desk.DeskNotFound
      - name: elsewhere
        when_related: {via: input.desk_id, predicate: site_id != input.site_id}
        error: shop.desk.DeskElsewhere
      - name: issued
        creates: shop.desk.Grant
        instance: grant_id
        emits: [shop.desk.GrantIssued]
        payload: {shop.desk.GrantIssued: {grant_id: {generated: true}}}
        sets: {desk_id: input.desk_id, site_id: input.site_id, name: input.name, mode: input.mode}
  - name: shop.desk.RevokeGrant
    input:
      - {name: grant_id, type: shop.desk.GrantId}
    outcomes:
      - name: no-grant
        unknown_instance: true
        error: shop.desk.GrantNotFound
      - name: revoked
        moves: shop.desk.Grant.revoke
        instance: grant_id
        emits: [shop.desk.GrantRevoked]
        payload: {shop.desk.GrantRevoked: {grant_id: input.grant_id}}
      - name: wrong-state
        wrong_state: true
        error: shop.desk.GrantRefused
  - name: shop.desk.AddClerk
    input:
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: channel, type: String}
    outcomes:
      - name: added
        creates: shop.desk.Clerk
        instance: clerk_id
        emits: [shop.desk.ClerkAdded]
        payload: {shop.desk.ClerkAdded: {clerk_id: {generated: true}}}
        sets: {desk_id: input.desk_id, channel: input.channel}
  - name: shop.desk.ReceiveTicket
    input:
      - {name: clerk_id, type: shop.desk.ClerkId}
      - {name: grant_id, type: shop.desk.GrantId}
      - {name: channel, type: String}
      - {name: request, type: String}
    outcomes:
      - name: no-clerk
        when_related: {via: input.clerk_id, exists: false}
        error: shop.desk.ClerkNotFound
      - name: no-grant
        when_related: {via: input.grant_id, exists: false}
        error: shop.desk.GrantNotFound
      - name: grant-refused
        when_related:
          via: input.grant_id
          predicate:
            any:
              - state == Revoked
              - {name: {none_of: [chat]}}
              - mode == denied
        error: shop.desk.GrantRefused
      - name: ignored
        when_related: {via: input.clerk_id, predicate: channel != input.channel}
        error: shop.desk.OtherChannel
      - name: received
        creates: shop.desk.Ticket
        instance: ticket_id
        emits: [shop.desk.TicketReceived]
        payload: {shop.desk.TicketReceived: {ticket_id: {generated: true}}}
        sets: {clerk_id: input.clerk_id, request: input.request}
views:
  - name: shop.desk.Tickets
    source: shop.desk.Ticket
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: shop.desk.TicketId}
      - {name: clerk_id, type: shop.desk.ClerkId}
  - name: shop.desk.Grants
    source: shop.desk.Grant
    consistency: read_your_writes
    fields:
      - {name: grant_id, type: shop.desk.GrantId}
      - {name: mode, type: shop.desk.Mode}
```

Measured today on the 0.97.4 build (release binary) for this exact text:
- It writes `10 scenario(s) (0 authored), 7 refusal(s)`.
- Its scenarios: `AddClerk/outcome/added`, `IssueGrant/outcome/{elsewhere,issued,no-desk,no-site}`, `OpenDesk/outcome/{no-site,opened}`, `OpenSite/outcome/opened`, `ReceiveTicket/outcome/no-grant`, `RevokeGrant/outcome/no-grant`.
- The 7 refusals are the `DRIVEN` ids below. Two of their texts:

```text
refused: refusal[ESS-SYNTH-003]: outcome shop.desk.ReceiveTicket/received has no scenario `shop.desk.ReceiveTicket/outcome/received`
  no candidate of the 1 tried satisfies `a row of `shop.desk.Grant` for `input.grant_id` of `shop.desk.ReceiveTicket` on which no branch answering before `received` is selected, beside the row the scenario is arranged around`
refused: refusal[ESS-SYNTH-004]: outcome shop.desk.RevokeGrant/revoked has no scenario `shop.desk.RevokeGrant/outcome/revoked`
  it needs an instance of `shop.desk.Grant` to move along `revoke`, and the route runs through `shop.desk.IssueGrant/issued`, which no input reaches
```

The variant used by test 2 (the `elsewhere` guard line replaced as below) gives the same 10 scenarios and the same 7 refusals today.

Expected after the change (expected, not verified): 17 scenarios and 0 refusals for both texts, all 17 passing on the interpreter. Measured as a reference: the same model with the `elsewhere` guard written as an `external:` refusal writes 17 scenarios and 0 refusals today.

Tests: append to `crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs` (an existing binary; add no new test file). That file already has `replaced` (line 41), `ir` (57), `refusals_for` (179), `scenario` (224) and the imports `BTreeMap`, `EssIr`, `Interpreted`, `Status`, `Synthesis`, `AdmittedSuite`, `Runner` (lines 8-21). Append at the end of the file:

```rust
// ---- A creating branch reading several rows, arranged for other scenarios ----------------------

const DESK: &str = include_str!("fixtures/a-creator-reading-several-related-rows.yaml");
/// `elsewhere` as the fixture writes it: the link of the desk the grant is filed under, compared
/// with the input naming the site.
const LINK_GUARD: &str =
    "        when_related: {via: input.desk_id, predicate: site_id != input.site_id}\n";
/// The same branch reading the row beside instead: the site's label, compared with an input.
const BESIDE_INPUT_GUARD: &str =
    "        when_related: {via: input.site_id, predicate: label != input.label}\n";
/// The branch's own scenarios, synthesized before the change too.
const OWN: [&str; 4] = [
    "shop.desk.IssueGrant/outcome/elsewhere",
    "shop.desk.IssueGrant/outcome/issued",
    "shop.desk.IssueGrant/outcome/no-desk",
    "shop.desk.IssueGrant/outcome/no-site",
];
/// The scenarios that need a grant `IssueGrant/issued` creates: refused before the change.
const DRIVEN: [&str; 7] = [
    "shop.desk.ReceiveTicket/outcome/no-clerk",
    "shop.desk.ReceiveTicket/outcome/grant-refused",
    "shop.desk.ReceiveTicket/outcome/ignored",
    "shop.desk.ReceiveTicket/outcome/received",
    "shop.desk.RevokeGrant/outcome/revoked",
    "shop.desk.Grant/transition/revoke/by/shop.desk.RevokeGrant/revoked",
    "shop.desk.Grant/state/Revoked/refuses/shop.desk.RevokeGrant",
];

fn every_status(result: &Synthesis, model: EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// The model synthesizes with no refusal, the branch's own scenarios and every scenario needing
/// the row it creates among the rest, and every scenario passes against the interpreter.
fn driven(label: &str, text: &str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(
        refusals_for(&result, "shop.desk."),
        Vec::<String>::new(),
        "{label}: refusals"
    );
    for id in OWN.iter().chain(DRIVEN.iter()) {
        scenario(&result, id);
    }
    let passed = every_status(&result, model);
    for id in DRIVEN {
        assert_eq!(
            passed.get(id),
            Some(&Status::Passed),
            "{label}: {id} runs and passes: {passed:#?}"
        );
    }
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{label}: every scenario passes against the interpreter: {passed:#?}"
    );
}

#[test]
fn a_creator_whose_owned_rows_link_is_compared_with_the_input_is_driven() {
    driven("link", DESK);
}

#[test]
fn a_creator_whose_row_beside_reads_the_input_is_driven() {
    driven("beside", &replaced(DESK, LINK_GUARD, BESIDE_INPUT_GUARD));
}
```

Red before the change: both tests fail at the first `assert_eq!`. `refusals_for` returns the 7 refusals quoted above, because `drive_several` refuses the setup run of `IssueGrant/issued`.

In test 1, `Desk` owns `Grant`, so the run is arranged around the desk (`focus` 1065-1083 through `owns_through` 125-147). `elsewhere` compares the desk's `site_id` with the `site_id` the run sends, which names the site row beside, so the rows must share one owner. In test 2, the site is the row beside and its guard reads the input.

Time: each test synthesizes one 200-line model and runs it on the interpreter. The release binary synthesizes it in about 0.2 s, so each test is expected to take seconds in a test build. Time both, and name any over 5 s in your report. Nothing is compiled per test.

## Commands

From the worktree root. Bounds:
- No single test over 60 s; a test step within 120 s; time each new test and name any over 5 s.
- A pool job ends after 10 minutes unless it names its time. Name the first build of each target below and every `clippy` on `ess-conformance` with `SLOT_TIMEOUT=30m` (at most 30 minutes). Never wrap a command in `timeout`.
- Run only the named `--test` targets, one per command, each with `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0`. `cargo` goes through the `slot` shim on its own.
- Do not set `RUSTC_WRAPPER`, `CARGO_BUILD_RUSTC_WRAPPER`, `SCCACHE_DIR`, `SCCACHE_SERVER_PORT` or `SCCACHE_CACHE_SIZE`: the host's one compile cache applies.
- A script that waits or measures runs without `slot`.
- No Python anywhere. Names in code, fixtures and messages stay generic (`shop.desk.*`).

0. `BASE=$(git rev-parse HEAD)`: the wave base (`0bda31d1c4` or the merge the coordinator names).
1. Write the fixture and the two tests, then run `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test related_guard_multiple_vias`. Expected: the two new tests fail as `## Tests first` says, and every older test of the file passes.
2. Apply `## What to change` steps 1-4, then run the same command. Expected: all pass. If a new test still fails, report its assertion message and stop. In particular, if a `DRIVEN` id is withdrawn as ESS-SYNTH-001 "names one row per value" (two rows of one scenario under one identity), report it and do not change distinctions or blocks.
3. One command each, all pass unchanged:
   - `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test adversary_283_pass1`
   - `--test related_owner_shared`
   - `--test related_guard_owner_link`
   - `--test a_row_arranged_beside_is_created_once_under_its_name`
   - `--test existing_instance_beside_related_guards`
   - `--test adversary_271_pass2`
4. Run the memo bounds, one command each, all pass unchanged: `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test stored_rows_decided_once_work`, then `--test once_per_model_work`. If `a_related_row_many_refusals_read_is_decided_within_a_bound` fails, report the printed count and stop. Do not raise `RELATED_BOUND`.
5. Run `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`. It fails because the tree holds a model the table does not pin, and prints its re-pin command. Run that command as printed:
   `SYNTHESIS_SUITE_BYTES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`
   then run the plain command again: it passes.
   This target holds two tables. The fixture declares no component, so the component table (`synthesis-component-suites-table.tsv`, header `#	models=102	lines=124`) must not change. If the test prints the `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` command, stop and report instead of re-pinning it.
6. Do the same for the two other tables, each re-pinned with the command its own failure prints:
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`, then `SELECTION_PRECEDENCE_TABLE_WRITE="$PWD/crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-compiler --locked --test selection_precedence_table`, then the plain command: passes.
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`, then `LOWERED_DEFINITIONS_TABLE_WRITE="$PWD/crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`, then the plain command: passes.
7. Prove that no pinned row moved, for each of the three tables `T`:
   - `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`, base header `#	models=485	lines=485`;
   - `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv`, base header `#	models=403	commands=830`;
   - `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`, base header `#	models=485	lines=666`.

   Expected for each `T`:
   - `comm -23 <(git show "$BASE:$T" | sort) <(sort "$T")` prints exactly one line: the old header.
   - `comm -13 <(git show "$BASE:$T" | sort) <(sort "$T")` prints the new header and only lines that begin with `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml`.

   Expected rows (not verified):
   - one new line in the suite table and in the lowered table;
   - in the selection table, one `ir=` line plus one line per command it lists, probably `shop.desk.IssueGrant`, `shop.desk.OpenDesk`, `shop.desk.ReceiveTicket` and `shop.desk.RevokeGrant`. Accept whatever command lines the write mode adds for the new path.

   Any other added or removed line is a moved pin: stop and report it with its old and new text and the suite diff (`ess verify conform synthesize` of that model before and after). Do not re-pin it.

   Why none is expected: the fallback runs only where `drive_several_strict` fails, and its failure is returned unchanged where the fallback fails too. A scan of all 485 pinned models with the 0.97.4 build found no refusal printing "beside the row the scenario is arranged around". The five that print "which no input reaches" name a route branch that reads no related row or only one, so `drive_several` is not on their path: `crates/edge/ess-cli/tests/fixtures/parse-refusals/every-position.yaml`, `calendar-windows.yaml`, `claim-search/pick-frozen.yaml`, `external-narrowed-by-state-unreachable.yaml`, `rows-moved-by-many-commands.yaml`.

   The committed models most likely to move all have a creating branch that reads two related rows through its input:
   - `crates/verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml`
   - `beside-row-once.yaml`
   - `related-guard-ranked-candidates.yaml`
   - `related-owner/box.yaml`
   - `two-rows-one-branch.yaml`
   - `crates/specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml`

   Look at their rows first.
8. Run the byte pins of `AGENTS.md` that this change could move, one command each; all pass with no re-pin:
   - `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read`
   - `--test adversary_related_values_all_bytes`
   - `--test external_beside_held_guard`
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test served_codec_names`
   - `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-synth --locked --test go` (it needs `go` on `PATH`; if it is not there, say so)
   - `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page`

   `ess-openapi` and `ess-compiler`'s `string_alphabet_and_length` do not depend on `ess-conformance`: do not run them.
9. `SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`: no warnings. Then `cargo fmt -p ess-conformance`, then `slot task fmt-check`: clean. Never `cargo fmt --all`.
10. Commit (message in step 6 of `## What to change`).

Never the whole workspace, never `task check`, never `cargo test -p ess-conformance` without `--test`.

## Files this unit owns

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`drive_several` → `drive_several_strict`, the new `drive_several`, `own_scenario_applies`, `as_own_scenario`, `OWN_OPEN`/`OwnOpened`/`own_opening`, `open_key`)
- `crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs` (two tests and their helpers appended)
- `crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (written by the re-pin command only)
- `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` (written by the re-pin command only)
- `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (written by the re-pin command only)

Must not touch:

- In `related_guard.rs`: `selects` (2064), `chooses` (398), `row_picks` (432), `answering_refusal` (2151), `beside_refused` (1477-1492), `drive` (2597), `creates_if_absent` and the `exists: false` arm of `drive` (unit S1).
- `crates/verify/ess-conformance/src/synthesize.rs`: including `created_owned_made` (8769), `invoke` (8988), `invoke_with` (9586) and the `Unreachable` texts. The refusal text is the follow-up named under `## Out of scope`.
- `crates/verify/ess-conformance/src/synthesize/made.rs`, `src/synthesize/subject_fact.rs`, `src/synthesize/created_memo.rs`, `src/synthesize/run_memo.rs`, `src/synthesize/first_kept.rs`: the memos.
- `crates/verify/ess-conformance/src/synthesize/row_set.rs`, `src/synthesize/caller.rs`, `src/witness.rs`, `src/witness_memo.rs`.
- `crates/verify/ess-conformance/tests/stored_rows_decided_once_work.rs` and its bounds, `tests/fixtures/synthesis-component-suites-table.tsv`.
- `CHANGELOG.md`, `docs/`, `website/`, `AGENTS.md`, `Taskfile.yml`, `.github/workflows/*`, `crates/edge/ess-xtask/tests/ci_lanes.rs`, `README.md`.

## For the coordinator at integration

- `CHANGELOG.md`, under `## [Unreleased]`, `### Fixed`, append:

  ```markdown
  - `ess verify conform synthesize` arranges the row a creating branch reading two or more related rows creates, for every scenario that needs it, wherever that branch's own scenario is witnessed. Before, a guard over a second row that compared it with the input, or a link of the row the creation is filed under compared with the input naming another row, refused every such scenario as `ESS-SYNTH-004` "which no input reaches" or `ESS-SYNTH-003` "beside the row the scenario is arranged around". Suites the earlier search arranged are byte for byte unchanged.
  ```

- `docs/design/cross-record-and-stored-field-guards.md`, § "Several related rows", lines 746-750 at `0bda31d1c4`. Replace the sentence that starts `A driver` (end of line 746) and ends `between its rows.` (line 750) with:

  ```markdown
  A driver running such a command arranges the other rows first, each where no branch answering
  before the driven one can hold whatever the input. A creating branch whose run no such
  arrangement reaches — a guard over a row beside reading the input, or a link of the row the
  creation is filed under compared with the input naming another row — is arranged as its own
  scenario is: the rows beside admitted without an input, the input chosen with them, each row
  whose guards read it arranged again for that input, and the rows filed under one owner, so every
  scenario needing its row gets one wherever its own scenario is witnessed. Any other run no such
  arrangement reaches refuses naming `arrange_related_row`, and so does an aggregate view sharing
  one related row between its rows.
  ```

- The three pin tables: every unit that adds or changes a YAML model in this wave re-pins them. Merge the units one after another, re-run the three write commands of `## Commands` 5-6 after the last merge, then run the `comm` proof against the wave base.

## Out of scope

- The refusal text (story decision D2) is a follow-up unit: "A setup run's refusal names the real cause where its branch has a scenario of its own". It is cut from this unit because it moves pinned refusal lines. It would change:
  - `created_owned_made` and `invoke` in `synthesize.rs` (`.map_err(|_| Unreachable::Unwitnessable {` at 8802, 9013, 9031, 9053 and 9078), which print "which no input reaches";
  - `beside_refused`'s constant `tried: 1` (`related_guard.rs` 1489).

  Pins it would move, each by name:
  - the `refusals=` digests in `synthesis-suite-bytes-table.tsv` of the five models listed in `## Commands` 7;
  - `tests/adversary_271_pass2.rs` line 512 (`"which no input reaches"`);
  - the doc comment at line 16 of `tests/existing_instance_beside_related_guards.rs`.

  Do not change any refusal text here.
- A `references` link where even the creating branch's own scenario fails (no `owns`; the referenced row is required by the row's creator). That is `story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed` (story decision D3).
- A row-set branch treated as always selected by `selects` (the follow-up of `story:a-row-set-refusal-is-witnessed-beside-the-related-row-its-creator-needs`), and `story:a-creator-under-a-row-set-and-when-is-arranged`.
- Setup runs of a moving branch (`instance` is `Some`), runs whose input the caller chose (`drive_toward`, the existence family), runs with an owner the caller arranged (`known` not empty), and commands with a stored-field `via`. Each keeps today's path.
- Changing `drive_several_strict`'s body, `beside_row`, `resteer_beside`, `arranged`, `steered_beside`, the blocks `BESIDE`/`BESIDE_DRIVEN`, or any distinction.
- Raising a work bound, adding a new test binary, editing a table by hand, running the whole workspace or `task check`.

## Acceptance

- `cargo test -p ess-conformance --locked --test related_guard_multiple_vias` passes, including `a_creator_whose_owned_rows_link_is_compared_with_the_input_is_driven` and `a_creator_whose_row_beside_reads_the_input_is_driven`. Both fail before the change with the 7 refusals of `## Tests first` (`## Commands` 1-2).
- `adversary_283_pass1`, `related_owner_shared`, `related_guard_owner_link`, `a_row_arranged_beside_is_created_once_under_its_name`, `existing_instance_beside_related_guards` and `adversary_271_pass2` pass unchanged (`## Commands` 3).
- `stored_rows_decided_once_work` and `once_per_model_work` pass with their bounds unchanged (`## Commands` 4).
- The three pin tables pass. The `comm` proof shows only the new header and lines for the new fixture, and no existing row moved (`## Commands` 5-7).
- The byte pins of `AGENTS.md` (`synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `served_codec_names`, `go`, `tutorial_page`) pass with no re-pin (`## Commands` 8).
- Clippy and `slot task fmt-check` are clean (`## Commands` 9). No format version changes, no `ValidationCode` changes, no refusal text changes.

## Depends on

Unit S1, `a-creating-branch-sends-every-row-it-creates-under-its-own-identity`, is merged at `0bda31d1c4`. It added `pub(super) fn creates_if_absent(outcome: &ResolvedOutcome) -> bool` and changed the `exists: false` arm of `drive` in `related_guard.rs`. This unit only reads them: nothing it adds calls them. The line numbers above already include S1's changes.

Nothing else. If another unit of this wave edits `related_guard.rs` (for example the follow-up that changes `selects` for row-set branches), the two collide on this file and must be merged one after the other.
