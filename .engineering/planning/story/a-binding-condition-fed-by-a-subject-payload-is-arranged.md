---
format: aep.planning-md/3
id: story:a-binding-condition-fed-by-a-subject-payload-is-arranged
kind: story
status: draft
title: A binding condition fed by a subject payload is arranged
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#705
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_condition.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/binding_condition_related.rs
revision: 4
---
## Need

A binding invokes only when its cause condition holds, `when: {event:, where: event.<member> == ...}`.
Where the trigger fills that event member from its own row, `{subject: <field>}`, synthesis refuses
every scenario of the binding with ESS-SYNTH-010, although the trigger's row was created by a step of
the trigger's own arrangement and the creating command stores the field from one of its inputs. The
same payload source is followed for a set guard by
`story:a-set-guard-fed-by-a-subject-payload-is-arranged` and already for a `via:` guard; the cause
condition is the remaining sibling.

Minimal reproduction, checked with `ess 0.92.0`. `system.yaml`:

```yaml
format: ess/27
system: example
version: v1
summary: Reconcile on removal.
domains: [example.grp, example.res]
```

`components.yaml`:

```yaml
components:
  - component: svc
    summary: One service.
    owns: {domains: [example.grp, example.res]}
    accepts: {commands: [example.grp.Join, example.grp.Leave, example.res.Issue, example.res.Reconcile]}
    publishes: {events: [example.grp.Joined, example.grp.Left, example.res.Issued, example.res.Released]}
bindings:
  - id: reconcile-on-leave
    summary: Release the lease when an eligible membership is removed.
    when: {event: example.grp.Left, where: event.eligible == true}
    invoke: {command: example.res.Reconcile}
    mapping: {member_id: event.member_id}
    delivery: at_least_once
    on_failure: retry
```

`domains/grp.yaml`:

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
        payload: {example.grp.Left: {member_id: {subject: member_id}, eligible: {subject: eligible}}}
      - {name: wrong-state, wrong_state: true, error: example.grp.Conflict}
events:
  - {name: example.grp.Joined, fields: [{name: membership_id, type: example.grp.MembershipId}]}
  - {name: example.grp.Left, fields: [{name: member_id, type: example.grp.MemberRef}, {name: eligible, type: Boolean}]}
views:
  - name: example.grp.MembershipRecord
    source: example.grp.Membership
    consistency: read_your_writes
    fields: [{name: membership_id, type: example.grp.MembershipId}, {name: state, type: example.grp.Membership.State}]
```

`domains/res.yaml`:

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
      - name: removed
        moves: example.res.Lease.remove
        instances: {where: member_id == input.member_id}
        emits: [example.res.Released]
        payload: {example.res.Released: {member_id: input.member_id}}
events:
  - {name: example.res.Issued, fields: [{name: lease_id, type: example.res.LeaseId}]}
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
  `reconcile-on-leave` invokes only when its event-payload condition holds, and no publishing branch and value set this synthesis tries makes it hold (`event.eligible == true`); `removed` fills `event.eligible` from neither an input, a literal nor a field of a row an input names
  help: let the trigger copy every event member the condition reads from an input it does not guard on, or cover the condition with an authored scenario (ess-scenario/1)
(the same for `/binding/mapping`, `/binding/delivery`, `/binding/on-failure`; `/binding/condition-false` likewise)
8 scenario(s) (0 authored), 5 refusal(s), nothing written
```

Control: give `example.grp.Leave` an input `eligible` and publish `eligible: input.eligible`. Then
`13 scenario(s) (0 authored), 0 refusal(s)`.

Cause: the condition's member sources in `synthesize/binding_condition.rs` are `Source::Input`,
`Source::Literal` and `Source::Related` (a field of the row a trigger input names, varied by creating
that row with another value). A `ResolvedPayloadValue::SubjectField` has no source and falls to the
refusal at `binding_condition.rs:427`.

Design: a fourth source, the trigger's own row: the event member is filled from a stored field that
the step creating the trigger's row stored, unconverted, from one of its inputs. It is varied the way
`Source::Related` is, by sending the creating step another value, and it is found by the rule
`stored_creation` (`synthesize/binding_effects.rs`) already applies. The payload the condition is
evaluated over reads the value that step sent. No authored surface, no new diagnostic, no new step
kind.

## Acceptance

- Over the reproduction above, `ess verify conform synthesize --path .` prints
  `13 scenario(s) (0 authored), 0 refusal(s)` and no `ESS-SYNTH-010`; the scenario ids are those of
  the control, including `reconcile-on-leave/binding/condition-false`.
- `ess verify conform run --path . --target interpreted --report-format 2` over the reproduction
  prints `13 scenarios: 13 passed, 0 failed, 0 error, 0 unsupported`.
- In `reconcile-on-leave/binding/condition-false` the step creating the membership sends
  `eligible: false`, and the scenario asserts no invocation of `example.res.Reconcile`; a test target
  that invokes the binding whatever the condition fails that scenario.
- Negative: with `example.grp.Join` storing `eligible: true` as a literal and declaring no `eligible`
  input, the five `ESS-SYNTH-010` refusals remain with their current message and help.
- The control specification synthesizes the same suite bytes as on 0.92.0, and
  `cargo test -p ess-conformance --test synthesis_suite_bytes_table` passes with no row re-pinned.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/binding_condition.rs
- cited: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- inferred: crates/verify/ess-conformance/tests/binding_condition_related.rs
- inferred: CHANGELOG.md

confidence: medium

## Fit review

1. **The need.** A binding with a cause condition over an event member the trigger publishes from
   its own row gets no scenario at all, the condition-false scenario included. Reproduction and
   output are in `## Need` (`8 scenario(s) (0 authored), 5 refusal(s)` against
   `13 ... 0 refusal(s)` for the control, `ess 0.92.0`). The requester of
   `github:beyond10x/ess#705` reported the set-guard position only; their proposal, labelled as
   theirs, is to resolve a `{subject: field}` payload source whose field an arranging command wrote
   from its input the way an input is resolved. This story applies it to the sibling position.
2. **Class: gap, in synthesis only.** The specification validates
   (`ess specify validate`: `example v1 — 4 file(s), valid`); the refusal states its own rule
   (`crates/verify/ess-conformance/src/synthesize/binding_condition.rs:427`). The binding's policies
   and its condition are unchecked. No authored surface is added.
3. **Can it already be expressed?** By the control (an extra trigger input, which lets the caller
   contradict the row) or by an authored `ess-scenario/1`, which the help names
   (`crates/verify/ess-conformance/src/synthesize.rs:2596-2598`); not run here. The condition
   already follows a field of a row a trigger input names
   (`binding_condition.rs:94-101`), a row the arrangement created; the trigger's own row is created
   by the same arrangement, so the idiom would leave the nearer row the one that cannot be read.
4. **Fit.** Grid R: `{subject: field}` in an event `payload:` is admitted
   (`docs/design/language-grids.md:35`) and the binding cause `where:` reads its `event` root
   (`docs/design/language-grids.md:37`); no cell moves. Vocabulary: none added. Composition: the
   creating step keeps its branch under the check `Source::Related` already applies to the creating
   command ("must not guard on that input", `binding_condition.rs:1004-1007`). Siblings: `via:`
   guard and destination follow a subject payload already
   (`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:413-420`, `1554-1557`); the set
   guard is `story:a-set-guard-fed-by-a-subject-payload-is-arranged`; with this story every binding
   arrangement position that follows a trigger input follows the trigger's created row too. Targets:
   only existing step kinds are written (the control's suite), so no target, `ess verify diff`,
   entity runtime or generated docs change.
5. **A second adopter.** A ticket is closed; the event carries `{priority: {subject: priority}}`;
   a binding notifies only `where: event.priority == urgent`. The priority was stored when the
   ticket was opened, from the opening command's input.
6. **Cost.** No source format bump, no suite format bump, no keyword, no diagnostic, no migration,
   no diff classification, no generated-API change. A specification of this shape gains scenarios
   and its suite digest changes on the next synthesis.
7. **Alternatives.** (a) Change nothing: the cause condition stays the one binding position where a
   subject payload is not followed, after the set-guard story lands. (b) Fold this into the
   set-guard story: two files and two test fixtures with separate review surfaces; kept apart so each
   is one reviewable change. (c) Chosen: a fourth `Source` found by the existing `stored_creation`
   rule.

## Decisions

accept as proposed.

The requester's rule (a `{subject: field}` payload whose field the arranging creation stored from
its input is followed as an input) is applied unchanged to the binding cause condition, the sibling
position with the same refusal. The refusal's message and help keep their text for every source the
rule does not cover.

## Source

github:beyond10x/ess#705
