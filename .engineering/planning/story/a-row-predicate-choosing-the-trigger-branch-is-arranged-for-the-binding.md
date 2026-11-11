---
format: aep.planning-md/3
id: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
kind: story
status: implemented
title: A row predicate choosing the trigger branch is arranged for the binding
summary: ESS-SYNTH-010 refuses flow, delivery and a policy refusal when one related row decides both the trigger and the invoked command; the refusal contradicts its help (#706)
tags:
- ess-batch-4
- ess-next
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#706
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t1_lane5_pass2.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs
- confidence: cited
  path: website/docs/guides/specify/binding-conditions.md
- confidence: cited
  path: website/docs/guides/specify/binding-stored-state.md
- confidence: cited
  path: website/docs/guides/specify/bindings-and-components.md
- confidence: cited
  path: website/sidebars.ts
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:31Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:32Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:44Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Need

A binding's invoked command chooses its branch by `when_related:` guards over the row one input
names. Synthesis arranges that row before the trigger and sends the trigger naming it, provided the
trigger "keeps its branch" for the new value. Today the trigger keeps its branch only if its own
guards over that input are `exists: false` refusals. A trigger whose *emitting* branch is chosen by
a predicate over the same row is rejected outright, so the binding loses `binding/flow`,
`binding/delivery` and every `binding/refusal/<predicate refusal>` scenario, although one row
decides both commands and a scenario can choose it.

Minimal reproduction (`ess 0.92.0`, source format `ess/22`, three files, one domain `demo.r`):

```yaml
# domains/r.yaml, the two commands that matter; DefineTier(eligible) creates demo.r.Tier with
# sets: {eligible: input.eligible}; Join creates demo.r.Membership
  - name: demo.r.Refresh
    input: [{name: membership_id, type: demo.r.MembershipId}, {name: tier_id, type: demo.r.TierId}]
    outcomes:
      - {name: tier-missing, when_related: {via: input.tier_id, exists: false}, error: demo.r.NoTier}
      - {name: eligible, when_related: {via: input.tier_id, predicate: eligible == true},
         updates: demo.r.Membership, instance: membership_id, emits: [demo.r.EligibilityChanged],
         payload: {demo.r.EligibilityChanged: {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: true}},
         sets: {tier_id: input.tier_id, eligible: true}}
      - {name: not-eligible, updates: demo.r.Membership, instance: membership_id, emits: [demo.r.EligibilityChanged],
         payload: {demo.r.EligibilityChanged: {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: false}},
         sets: {tier_id: input.tier_id, eligible: false}}
  - name: demo.r.Issue
    input: [{name: member_id, type: demo.r.MemberRef}, {name: tier_id, type: demo.r.TierId}]
    outcomes:
      - {name: tier-missing, when_related: {via: input.tier_id, exists: false}, error: demo.r.NoTier}
      - {name: not-eligible, when_related: {via: input.tier_id, predicate: eligible == false}, error: demo.r.NoTier}
      - {name: issued, creates: demo.r.Lease, instance: lease_id, emits: [demo.r.Issued],
         payload: {demo.r.Issued: {lease_id: {generated: true}}}, sets: {member_id: input.member_id}}
```

```yaml
# components.yaml
bindings:
  - id: issue-on-eligibility-change
    when: {event: demo.r.EligibilityChanged}
    invoke: {command: demo.r.Issue}
    mapping: {member_id: event.member_id, tier_id: event.tier_id}
    delivery: at_least_once
    on_failure:
      drop: [tier-missing, not-eligible]
      retry: {except: [tier-missing, not-eligible]}
```

`ess specify validate --path .` answers `demo v1 — 3 file(s), valid`.
`ess verify conform synthesize --path .` answers `10 scenario(s) (0 authored), 4 refusal(s)`:

```
refused: refusal[ESS-SYNTH-010]: binding issue-on-eligibility-change has no scenario `issue-on-eligibility-change/binding/flow`
  `issue-on-eligibility-change` invokes `demo.r.Issue`, whose branch is decided by an input the event fills: tier-missing, not-eligible, issued
  help: leave the invoked command one branch an input does not choose, or declare the others `external:` so a scenario can force them
refused: ... `issue-on-eligibility-change/binding/delivery`            (same text)
refused: ... `issue-on-eligibility-change/binding/refusal/not-eligible`
  ... no row the predicate of the refusal `demo.r.Issue/not-eligible` refuses can be arranged before the trigger
  help: ... synthesis arranges one itself where a command creating the row takes each field the predicate reads as input, and the trigger copies the row's identity unconverted from an input of its own
refused: ... `issue-on-eligibility-change/binding/refusal/tier-missing`  (correct: every publisher refuses the missing row before it publishes)
```

Both conditions the `not-eligible` help states hold (`DefineTier` takes `eligible` as input;
`Refresh` copies `tier_id` from `input.tier_id`), so the refusal contradicts its own help. The
flow refusal's help asks for `external:` on branches a row decides, which is not the cause either.

Control: replacing the two emitting branches of `Refresh` by one unguarded branch that copies the
flag (`eligible: {related: {via: input.tier_id, field: eligible}}` in `sets:` and `payload:`) gives
`12 scenario(s), 1 refusal(s)`: `binding/flow`, `binding/delivery` and
`binding/refusal/not-eligible` are written, and only `binding/refusal/tier-missing` is refused.

Cause, in the code: `related_arranged` gives up at `keeps_branch_sent(sender, &from, Some(entity), false)`
(`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1561`), and `keeps_branch_sent`
accepts a `when_related` guard through `from` only when its test is `Absent` with an error
(`binding_effects.rs:1907-1912`). A `Holds { predicate }` guard through `from` over the same entity
returns `false` regardless of what the arranged row would make the trigger do.

Design. The row arranged for the invoked command's branch carries known field values
(`Arrangement.settled`, `crates/verify/ess-conformance/src/synthesize.rs:6878`). So the question
"which branch does the trigger take when sent this row" has an answer synthesis can compute:

1. Arrange the row for the invoked command's target branch (accepting branch for flow, mapping and
   delivery; the refusal for `binding/refusal/<outcome>`), as today.
2. Where the trigger's guards through the copied input all read that same entity, read no further
   row (`also` empty), and their predicates read the row's stored fields alone (no input), evaluate
   them in declaration order on the arranged row's settled values. The branch they select is the
   branch the honest system takes.
3. Use the arrangement only if that branch is the candidate publishing branch (`published_by`).
   Every publishing branch is already a candidate (`publishers`, `synthesize.rs:20454`), so a row
   that sends the trigger down its other publishing branch is found on that candidate. A predicate
   that evaluates `Unknown` on the settled values is no selection: the candidate is dropped.
4. Where no candidate publishes for a row that selects the target branch, the refusal stays and
   names the cause: the trigger's `when_related:` predicate over the row, not "an input the event
   fills" and not conditions that already hold.

No authored surface changes. `exists: false` on both commands (0.72.0) becomes the special case of
step 2 where every guard is `Absent`.

## Acceptance

- `ess verify conform synthesize --path .` over the reproduction as written exits 0 with
  `13 scenario(s) (0 authored), 1 refusal(s)`. The suite holds
  `issue-on-eligibility-change/binding/flow`, `.../binding/delivery` and
  `.../binding/refusal/not-eligible`; the one refusal is `.../binding/refusal/tier-missing` with its
  current text.
- In `binding/flow` and `binding/delivery` the one `demo.r.Tier` the trigger names is created with
  `eligible: true` and the scenario expects `demo.r.Issued`. In `binding/refusal/not-eligible` it is
  created with `eligible: false`, the trigger is expected on its `not-eligible` branch, and the
  scenario expects one attempt and no `demo.r.Issued`.
- `ess verify conform run --target interpreted` over that suite passes every scenario. A test
  target that never invokes `demo.r.Issue` fails `binding/flow`; a test target that retries
  `not-eligible` instead of dropping it fails `binding/refusal/not-eligible`.
- Kept refusals, each asserted by a fixture in the same test file:
  - the trigger's predicate also reads an input (`predicate: eligible == input.flag`): flow,
    delivery and the predicate refusal stay refused with `ESS-SYNTH-010`;
  - the trigger reads a row of another entity through the copied input: stay refused;
  - the trigger publishes only for rows the invoked refusal does not refuse (drop the trigger's
    default emitting branch from the reproduction): `binding/refusal/not-eligible` stays refused,
    its text names the trigger's `when_related:` predicate, and its help no longer states
    conditions that hold; flow and delivery are written.
- The control variant keeps `12 scenario(s), 1 refusal(s)` and its suite bytes.
- `synthesis_suite_bytes_table` passes with no row changed.

## Scope

Delivered in turn 1 of the zero-open-issues plan (lane story 5, guide-split correction 045e12f293). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/verify/ess-conformance/src/synthesize.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/synthesize/binding_effects.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/synthesize/refusal_policy.rs`
- cited, confirmed: `website/docs/guides/specify/bindings-and-components.md`
- cited, added: `crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs`
- cited, added: `crates/verify/ess-conformance/src/synthesize/existence.rs`
- cited, added: `crates/verify/ess-conformance/src/synthesize/related_guard.rs`
- cited, added: `crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs`
- cited, added: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`
- cited, added: `crates/verify/ess-conformance/tests/adversary_t1_lane5_pass2.rs`
- cited, added: `crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs`
- cited, added: `website/docs/guides/specify/binding-conditions.md`
- cited, added: `website/docs/guides/specify/binding-stored-state.md`
- cited, added: `website/sidebars.ts`

Scoped but not touched:

- `crates/verify/ess-conformance/tests/binding_related_branch.rs`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Fit review

1. **Need.** A behaviour ESS gets wrong: synthesis refuses to witness a binding whose trigger and
   invoked command are both decided by a predicate over one related row, although a scenario can
   create that row with either value and the outcome of both commands follows. Reproduction above;
   command output in `repro/run.txt` of the triage (`10 scenario(s), 4 refusal(s)` as written,
   `12 scenario(s), 1 refusal(s)` for the control). The requester proposed no syntax; the request
   is that synthesis follow a predicate on the trigger as it follows `exists: false`.
2. **Class: defect.** The refusal's own help says synthesis arranges the row "where a command
   creating the row takes each field the predicate reads as input, and the trigger copies the row's
   identity unconverted from an input of its own"
   (`crates/verify/ess-conformance/src/synthesize.rs:2565-2570`), and both hold. The guide states the
   same two conditions and no third
   (`website/docs/guides/specify/bindings-and-components.md:341-347`).
3. **Already expressible?** Only by leaving the specification's shape: an authored `ess-scenario/1`
   document per cell, or the control rewrite (one unguarded trigger branch copying the flag), which
   changes what the trigger declares. Neither is an idiom for the declared shape, and
   `ess specify validate` accepts the shape as written (`3 file(s), valid`). The suite must check
   the binding of the specification the reader sees
   (`docs/design/generic-language-features.md` § Policy, policy 1).
4. **Fit.** No cell of grid R, grid O or the effect × target grid opens or closes
   (`docs/design/language-grids.md`): nothing authored changes. The change generalises one rule in
   one place, `keeps_branch_sent` (`binding_effects.rs:1873`), which every related-row arrangement
   shares: the accepting branch (`related_decided`, `:1486`), the acting-on-row variant
   (`related_on_row`, `:1694`) and the predicate refusal (`related_refused`, `:1629`). So the three
   siblings gain it together. Targets: the new scenarios use step kinds the control variant already
   emits under `ess-conformance/36`, so runners, generated Rust, Go and TypeScript packages,
   `ess verify diff` and generated docs see nothing new. The rule that a trigger must stay on the
   branch its scenario claims is kept (`binding_effects.rs:1861-1872`); it is evaluated instead of
   assumed false.
5. **Second adopter.** A stock domain: `example.stock.Reserve(order_id, item_id)` publishes
   `Reserved` on `when_related: {via: input.item_id, predicate: stocked == true}` and `Backordered`
   by default; a binding on `Reserved` invokes `example.ship.Schedule(item_id)`, which refuses
   `not-shippable` on `predicate: shippable == false` over the same item. Same shape, unrelated
   nouns.
6. **Cost.** No source format, no suite format, no keyword, no new diagnostic code, no migration,
   no diff classification, no generated-API change. One refusal text of `ESS-SYNTH-010` is reworded
   for the kept case. Suites of specifications with this shape gain scenarios in place of refusals;
   every other suite keeps its bytes (`synthesis_suite_bytes_table`,
   `AGENTS.md` § Determinism and formats: "Preserve canonical bytes").
7. **Alternatives.**
   - Change nothing, correct the help text only. Removes the contradiction, leaves an arrangeable
     binding unwitnessed and pushes three cells per binding to authored scenarios. Refused.
   - Accept any `Holds` guard on the trigger over the same row without evaluating it. Would send
     the trigger down a branch the scenario does not claim, the failure the 2026-10-26 rule exists
     to prevent (`binding_effects.rs:1867-1870`). Refused.
   - Search a row satisfying the conjunction of the trigger's and the invoked command's predicates
     in one solve. More general, a second search beside `related_guard`. Not needed: arranging for
     the invoked branch and evaluating the trigger on the settled values, across every candidate
     publisher, covers the reported shape and the second example with existing parts.
   - Chosen: evaluate the trigger's guards on the arranged row and keep the arrangement where the
     selected branch is the candidate publisher. The requester's reading ("one row decides both
     commands") is taken as stated.

## Decisions

accept as proposed

The request names a behaviour, not a syntax, and the behaviour passes question 4 unchanged. One
story, one crate. No format bump. No design page precedes it: the governing text is the guide
section "the command chooses its branch by the row"
(`website/docs/guides/specify/bindings-and-components.md:318-347`), which this story extends with
the trigger-predicate case and its three kept refusals.

## Source

github:beyond10x/ess#706
