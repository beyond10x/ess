---
format: aep.planning-md/3
id: story:a-set-guard-fed-by-a-subject-payload-is-arranged
kind: story
status: draft
title: A set guard fed by a subject payload is arranged
summary: 'ESS-SYNTH-010 refuses a binding whose invoked set guard reads an input the trigger fills from {subject: field}, although arrangeable (#705)'
tags:
- ess-batch-7
- ess-next
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#705
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_set_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/binding-set-guard/
revision: 7
---
## Need

A binding's invoked command chooses its branch by a row-set guard,
`when_related: {entity:, where: <field> == input.<x>, exists: true}`, with a default branch after it.
The trigger fills the event field that feeds `input.<x>` from its own row, `{subject: <field>}`.
Synthesis refuses the binding's flow, delivery and on-failure scenarios with ESS-SYNTH-010. The case
is arrangeable: the trigger's row was created by a step of the trigger's own arrangement, and that
creating command stores the field from one of its inputs, so the value is the scenario's to choose.

Minimal reproduction (four files, `format: ess/22`), checked with `ess 0.92.0`:

`system.yaml`

```yaml
format: ess/22
system: example
version: v1
summary: Reconcile on removal.
domains: [example.grp, example.res]
```

`components.yaml`

```yaml
components:
  - component: svc
    summary: One service.
    owns: {domains: [example.grp, example.res]}
    accepts: {commands: [example.grp.Join, example.grp.Leave, example.res.Issue, example.res.Reconcile]}
    publishes: {events: [example.grp.Joined, example.grp.Left, example.res.Issued, example.res.Kept, example.res.Released]}
bindings:
  - id: reconcile-on-leave
    summary: Release the lease when no eligible membership is left.
    when: {event: example.grp.Left}
    invoke: {command: example.res.Reconcile}
    mapping: {member_id: event.member_id}
    delivery: at_least_once
    on_failure: retry
```

`domains/grp.yaml`

```yaml
domain: example.grp
summary: Memberships.
types:
  - {name: example.grp.MembershipId, kind: newtype, of: Uuid}
  - {name: example.grp.MemberRef, kind: newtype, of: Uuid}
entities:
  - name: example.grp.Membership
    identity: {name: membership_id, type: example.grp.MembershipId}
    fields: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]
    lifecycle: {initial: Active, states: [Active, Removed], terminal: [Removed], transitions: [{name: remove, from: [Active], to: Removed}]}
errors:
  - {name: example.grp.Conflict, summary: Already removed.}
actors:
  - {name: example.grp.Admin, may: [example.grp.Join, example.grp.Leave]}
commands:
  - name: example.grp.Join
    input: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]
    outcomes:
      - name: joined
        creates: example.grp.Membership
        instance: membership_id
        emits: [example.grp.Joined]
        payload: {example.grp.Joined: {membership_id: {generated: true}}}
        sets: {member_id: input.member_id, eligible: input.eligible}
  - name: example.grp.Leave
    input: [{name: membership_id, type: example.grp.MembershipId}]
    outcomes:
      - name: removed
        moves: example.grp.Membership.remove
        instance: membership_id
        emits: [example.grp.Left]
        payload: {example.grp.Left: {member_id: {subject: member_id}}}
      - {name: wrong-state, wrong_state: true, error: example.grp.Conflict}
events:
  - {name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}]}
  - {name: example.grp.Left, fields: [{name: member_id, type: example.grp.MemberRef}]}
views:
  - name: example.grp.MembershipRecord
    source: example.grp.Membership
    consistency: read_your_writes
    fields: [{name: membership_id, type: example.grp.MembershipId}, {name: state, type: example.grp.Membership.State}]
```

`domains/res.yaml`

```yaml
domain: example.res
summary: Leases.
types:
  - {name: example.res.LeaseId, kind: newtype, of: Uuid}
entities:
  - name: example.res.Lease
    identity: {name: lease_id, type: example.res.LeaseId}
    fields: [{name: member_id, type: example.grp.MemberRef}]
    lifecycle: {initial: Held, states: [Held, Removed], terminal: [Removed], transitions: [{name: remove, from: [Held], to: Removed}]}
errors:
  - {name: example.res.Unavailable, summary: Store unavailable.}
actors:
  - {name: example.res.Internal, may: [example.res.Issue, example.res.Reconcile]}
commands:
  - name: example.res.Issue
    input: [{name: member_id, type: example.grp.MemberRef}]
    outcomes:
      - name: issued
        creates: example.res.Lease
        instance: lease_id
        emits: [example.res.Issued]
        payload: {example.res.Issued: {lease_id: {generated: true}}}
        sets: {member_id: input.member_id}
  - name: example.res.Reconcile
    input: [{name: member_id, type: example.grp.MemberRef}]
    outcomes:
      - {name: unavailable, external: the store cannot be written to, error: example.res.Unavailable}
      - name: kept
        when_related: {entity: example.grp.Membership, where: {all: [member_id == input.member_id, state == Active, eligible == true]}, exists: true}
        emits: [example.res.Kept]
        payload: {example.res.Kept: {member_id: input.member_id}}
      - name: removed
        moves: example.res.Lease.remove
        instances: {where: member_id == input.member_id}
        emits: [example.res.Released]
        payload: {example.res.Released: {member_id: input.member_id}}
events:
  - {name: example.res.Issued, fields: [{name: lease_id, type: example.res.LeaseId}]}
  - {name: example.res.Kept, fields: [{name: member_id, type: example.grp.MemberRef}]}
  - {name: example.res.Released, fields: [{name: member_id, type: example.grp.MemberRef}]}
views:
  - name: example.res.LeaseRecord
    source: example.res.Lease
    consistency: read_your_writes
    fields: [{name: lease_id, type: example.res.LeaseId}, {name: state, type: example.res.Lease.State}, {name: member_id, type: example.grp.MemberRef}]
```

`ess verify conform synthesize --path .` on 0.92.0:

```
refused: refusal[ESS-SYNTH-010]: binding reconcile-on-leave has no scenario `reconcile-on-leave/binding/flow`
  `reconcile-on-leave` invokes `example.res.Reconcile`, whose branch is decided by the rows its set guard selects, `example.grp.Membership` where (member_id == input.member_id and state == Active and eligible == true): kept, removed; no row it selects can be arranged before the trigger for the value the event fills
  help: let the trigger copy every input the set guard's `where:` reads from an input of its own, unconverted, ...
(the same for `/binding/delivery` and `/binding/on-failure`)
10 scenario(s) (0 authored), 3 refusal(s), nothing written
```

Control ("toggle"): give `example.grp.Leave` an input `member_id` and publish
`member_id: input.member_id`. Then `13 scenario(s) (0 authored), 0 refusal(s)`, the flow scenario
ends in `eventually_event example.res.Kept`, and
`ess verify conform run --path . --target interpreted --report-format 2` prints
`13 scenarios: 13 passed, 0 failed, 0 error, 0 unsupported`.

Cause: `read_through` in `synthesize/binding_effects/set_guard.rs` follows the event field only to
`ResolvedPayloadValue::InputField` and returns `None` for every other source. The sibling `via:`
arrangement in `synthesize/binding_effects.rs` already follows `ResolvedPayloadValue::SubjectField`
through `stored_creation`, which finds the arrangement step that created the trigger's row and the
input that step stores the field from.

Design: `read_through` returns, per input the selectors read, where the value is sent: an input of
the trigger's own send (today), or an input of the creating step in the trigger's arrangement, found
by `stored_creation`. `alone` reads the value that step sent. `arranged_for` steers that step's input
to the value the rows were arranged against, under the same keeps-its-branch check
(`keeps_branch_sent`) the trigger's own send is held to, and `row_set::decided_after` decides the
branch again over the rows that remain. No authored surface, no new diagnostic, no new step kind.

## Acceptance

- Over the reproduction above, `ess verify conform synthesize --path .` prints
  `13 scenario(s) (0 authored), 0 refusal(s)` and no `ESS-SYNTH-010`. The scenarios
  `reconcile-on-leave/binding/flow`, `/delivery` and `/on-failure` exist and the flow scenario ends
  in `eventually_event` of `example.res.Kept`.
- `ess verify conform run --path . --target interpreted --report-format 2` over the same
  reproduction prints `13 scenarios: 13 passed, 0 failed, 0 error, 0 unsupported`.
- A test target that answers `removed` for `example.res.Reconcile` whatever rows exist fails
  `reconcile-on-leave/binding/flow` (the `failing` helper of `tests/binding_set_guard.rs`).
- Negative: with `example.grp.Join` storing `member_id: {generated: true}` and no `member_id` input,
  synthesis output is byte-identical to 0.92.0: `9 scenario(s) (0 authored), 4 refusal(s)`, the three
  `ESS-SYNTH-010` refusals with their current message and help.
- The control specification ("toggle") synthesizes the same suite bytes as on 0.92.0.
- `cargo test -p ess-conformance --test synthesis_suite_bytes_table` passes with no row of
  `tests/fixtures/synthesis-suite-bytes-table.tsv` re-pinned.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs
- cited: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- cited: crates/verify/ess-conformance/tests/binding_set_guard.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/binding-set-guard/
- inferred: CHANGELOG.md

confidence: high

## Fit review

1. **The need.** A binding whose invoked command picks its branch by a row-set guard gets no flow,
   delivery or on-failure scenario when the trigger publishes the guarded value from its own row
   rather than from its input. Reproduction and output are in `## Need`; the run is recorded with
   `ess 0.92.0` (`10 scenario(s) (0 authored), 3 refusal(s)` as written, `13 ... 0 refusal(s)` with
   the control). The requester's proposal, labelled as theirs: resolve a `{subject: field}` payload
   source whose field an arranging command wrote from its input the way an input is resolved. They
   proposed no syntax.
2. **Class: gap, in synthesis only.** The specification validates
   (`ess specify validate`: `example v1 — 4 file(s), valid`) and the refusal follows its own stated
   rule (`crates/verify/ess-conformance/src/synthesize.rs:2530-2534`), so it is not a defect of the
   language. The binding's three policies are unchecked, which is a checking gap. No authored
   surface is asked for or added.
3. **Can it already be expressed?** Two ways today, neither a good answer. (a) The control: give the
   trigger an input and publish it. That changes the domain: the caller then supplies a value the
   row already holds and may contradict. (b) An authored `ess-scenario/1`, which the refusal's help
   names (`synthesize.rs:2533`); not run here. It keeps the figure checked, but the sibling `via:`
   guard needs no authored scenario for the same payload source
   (`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1554-1557`), so the idiom would
   leave two siblings answering one question differently.
4. **Fit.** Grid R already admits `{subject: field}` in an event `payload:`
   (`docs/design/language-grids.md:35`, row "event `payload:`", subject column "yes"); nothing in the
   grids moves. Vocabulary: none added. Composition: the trigger's branch is kept by the existing
   `keeps_branch_sent` (`binding_effects.rs:1873`) and the branch is re-decided by
   `row_set::decided_after` (`set_guard.rs:237`). Siblings: the `via:` guard and the binding
   destination already follow a subject payload (`binding_effects.rs:413-420`, `1554-1557`); the
   single-branch set move needs nothing (probe: the reproduction without `kept` gives
   `12 scenario(s) (0 authored), 0 refusal(s)`); the binding cause `where:` has the same gap and is
   `story:a-binding-condition-fed-by-a-subject-payload-is-arranged`. Targets: the scenarios use only
   existing step kinds (`execute_command`, `expect_outcome`, `capture_instance`, `expect_event`,
   `eventually_event`, as in the control's flow scenario), so Rust, Go, TypeScript, web, the
   interpreter, `ess verify diff`, the entity runtime and generated docs are untouched.
5. **A second adopter.** A shipment is cancelled; the event carries the order the shipment row
   holds, `{order_id: {subject: order_id}}`; a binding invokes `example.order.Settle`, which answers
   `open` while `example.ship.Shipment where order_id == input.order_id and state == Pending` exists
   and `settled` otherwise. Same shape, unrelated domain.
6. **Cost.** No source format bump, no suite format bump (the control's suite is
   `ess-conformance/34` with these step kinds already), no keyword, no diagnostic, no migration, no
   diff classification, no generated-API change. A specification of this shape gains scenarios; its
   suite digest changes on the next synthesis, as with every arrangement extension.
7. **Alternatives.** (a) Change nothing and answer with the authored-scenario idiom: leaves the
   sibling inconsistency of answer 3 and loses three scenarios per such binding. (b) Follow any
   subject field, whoever wrote it: unsound, a field written by a mint or a later update is not the
   scenario's to choose; `stored_creation` refuses those (`binding_effects.rs:2884-2898`).
   (c) Chosen: reuse `stored_creation` in `read_through`. It is the requester's proposal, taken
   unchanged, and it is the rule the `via:` sibling already applies.

## Decisions

accept as proposed.

The requester proposed no syntax, only that a `{subject: field}` payload whose field the arranging
creation stored from its input be followed as an input is. That is the rule `stored_creation` already
implements for a `via:` guard, so it is applied to the set guard unchanged. The refusal's message and
help keep their text. Placement was approved earlier (approval record D229).

## Source

github:beyond10x/ess#705
