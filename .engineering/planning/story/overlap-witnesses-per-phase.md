---
format: aep.planning-md/3
id: story:overlap-witnesses-per-phase
kind: story
status: active
title: Two overlapping branches of one phase are witnessed where both hold
tags:
- defect
- severity-high
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#472
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:synthesis-reads-selection-plan
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/adversary-244b-base-digests.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:28Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:29Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"review_outcome":5}}}
---
# Story: Two overlapping stored-field refusals of one phase are witnessed where both hold

## Why

beyond10x/ess#472 (closed 2026-10-07) reported a `precedence-swap` survivor for two input-guarded
refusals whose guards can both hold. That case is fixed: since 0.54.0 (beyond10x/ess#455,
`refusal_pair_overlaps`, `crates/verify/ess-conformance/src/synthesize.rs:18243`) synthesis sends
their overlap requiring the first declared, on the stateless path and on a command that also reads
its stored row. Verified on 0.92.0 (reproduction below, models A and B).

What is still open is the same defect one phase later: two `when_subject` refusals whose stored-field
predicates one row satisfies together. No synthesized scenario arranges a row where both hold, so
their declared order is never tested and a target that checks the second first passes the suite.
`mutate` does not see the gap either: `precedence_sites` (`src/mutate.rs:2547-2561`) only pairs
branches `guarded_by_input_alone` (`src/mutate.rs:1131-1146`, which excludes `when_subject`), so it
reports no precedence-swap site at all.

## Problem

Model C, neutral domain `demo.order`, format `ess/20`:

- Entity `Order`: identity `order_id`, fields `quantity: Integer`, `locked: Boolean`.
- Command `Place` creates `Order`, `sets: {quantity: input.quantity, locked: input.locked}`.
- Command `Ship`, input `order_id`; outcomes in order:
  - `empty`: `when_subject: {predicate: quantity <= 0}`, error `Empty`;
  - `locked`: `when_subject: {predicate: locked == true}`, error `Locked`;
  - `shipped`: `updates: Order`, `instance: order_id`, `sets: {locked: true}`.

On ess 0.92.0:

```
$ ess verify conform synthesize --path model.yaml --target ir --out suite.json
4 scenario(s) (0 authored), 0 refusal(s)
```

`Ship/outcome/empty` arranges `{quantity: 0, locked: false}`, `Ship/outcome/locked` arranges
`{quantity: 1, locked: true}`; no scenario arranges `{quantity: 0, locked: true}`, and no note names
the pair.

```
$ ess verify conform mutate --path model.yaml --target interpreted --class precedence-swap
refusal[ESS-MUTATE-003]: `precedence-swap` find(s) no site in this specification, so the audit would run nothing
exit 3
```

Controls (pass on 0.92.0, keep as regression tests): model A, the #472 shape
(`bad-quantity: quantity <= 0`, `bad-discount: discount < 0` on a creating command), and model B,
the same two input refusals on an updating command beside a `when_subject` refusal. Both send
`{quantity: 0, discount: -1}` requiring `bad-quantity`, and `mutate --class precedence-swap` reports
the mutant killed.

## What it delivers

For each pair of `when_subject` refusals the selection plan places in one phase, ordered by
declaration, whose predicates one stored row can satisfy together, synthesis arranges that row and
sends the command, requiring the first declared, appended to the first branch's own
`…/outcome/<first>` scenario (as #178, #217 and #455 do; no new scenario id). `precedence_sites`
builds a precedence-swap mutant from the same pairs. A pair synthesis cannot arrange is named in a
note, as `unwitnessed_overlaps` (`src/synthesize.rs:18456`) does for a refusal/accepting overlap.

Not in this story: related-row and row-set pairs (`story:the-first-of-overlapping-related-refusals-is-synthesized`),
and input-only pairs, already fixed.

## Acceptance

All cases below run under `cargo test -p ess-conformance --locked --test refusal_pair_overlap`.

- Models A and B (regression, green on 0.92.0): the overlap send `{quantity: 0, discount: -1}`
  requires `bad-quantity`, and `ess verify conform mutate --target interpreted --class
  precedence-swap` reports the mutant killed, exit 0.
- Model C: `ess verify conform synthesize` writes, in `demo.order.Ship/outcome/empty`, a send on a
  row arranged with `quantity: 0` and `locked: true` that requires outcome `empty` and error
  `Empty`. Red on 0.92.0.
- Model C: a target answering `locked` before `empty` fails `demo.order.Ship/outcome/empty` of the
  new suite and passes every scenario of the 0.92.0 suite.
- Model C: `ess verify conform mutate --target interpreted --class precedence-swap` lists the site
  `precedence-swap/demo.order.Ship/empty/locked`, scores it killed, and exits 0 (0.92.0: exit 3,
  `ESS-MUTATE-003`).
- Model C with `Place` setting the literal `locked: false` (only `shipped` stores `locked: true`,
  and only on a row with `quantity > 0`, so no arrangeable row satisfies both predicates):
  synthesis records a `Note::UnwitnessedOverlap` on `demo.order.Ship/outcome/empty` with
  `first: empty` and `other: locked`.
- Same model: `mutate --class precedence-swap --target interpreted` scores no mutant `survived`.
- `cargo test -p ess-conformance --locked --test external_beside_held_guard`, `--test
  adversary_244b_window_free_bytes`, `--test synthesis_suite_bytes_table` and `--test
  mutation_audit` pass, with a row or a `("precedence-swap", N)` pin moved only where its model has
  two `when_subject` refusals of one phase that one row satisfies together, the cause named beside
  the pin.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`overlaps` :18112, `refusal_pair_overlaps` :18243, `unwitnessed_overlaps` :18456)
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`overlaps` :8463, refusal x accepting only; `refusal_pair_inputs` call :6992)
- cited: crates/verify/ess-conformance/src/mutate.rs (`precedence_sites` :2547, `guarded_by_input_alone` :1131)
- cited: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs
- inferred: crates/verify/ess-conformance/tests/mutation_audit.rs (`("precedence-swap", 0)` pins)
- inferred: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv
- inferred: crates/verify/ess-conformance/tests/fixtures/adversary-244b-base-digests.tsv
- inferred: docs/design/input-guard-overlap-precedence.md

confidence: medium

Shared surfaces moved: re-pins suite-bytes rows (`synthesis-suite-bytes-table.tsv`) for models
with a same-phase `when_subject` pair; proposes the `CHANGELOG.md` sentence.

## Fit review

1. Need: two refusals over stored fields that one row satisfies together have a declared order no
   suite checks; minimal reproduction under Problem. Requester's proposal (#472): an overlap send
   and a killed precedence-swap mutant.
2. Class: defect. The one precedence rule answers refusals in declaration order
   (`docs/design/input-guard-overlap-precedence.md`), and synthesis already witnesses that order for
   input-only pairs.
3. Already expressible: yes, the model needs no new surface; only synthesis and mutate change.
4. Fit: reuses the overlap send of #178/#217/#455 and the existing note; no new key, no format.
5. Second adopter: any service refusing a record for two independent stored conditions (archived
   and over quota) whose order decides the error a client sees.
6. Cost: suite bytes move for models with such a pair; no format bump, no new diagnostic.
7. Considered: change nothing (the order stays untested), a new scenario id per pair (rejected:
   #455 appends to the first branch's scenario), this design.

## Decisions

- accept, redesigned: the input-only half of #472 shipped in 0.54.0 and is kept as regression tests;
  this story covers the `when_subject` phase the issue's reasoning also applies to.
