---
format: aep.planning-md/3
id: story:a-binding-destination-arrangement-keeps-the-trigger-on-its-branch
kind: story
status: draft
title: A binding destination arrangement keeps the trigger on its branch
summary: arranged_destination re-checks the trigger branch after arranging, as related_decided does since 0.72.0
tags:
- defect
- ess-0.73.0
- severity-high
- zero-open-issues
relations:
- informed_by: review-result:ess-0720-wave-1-adversary-pass-1
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/a_binding_destination_arrangement_keeps_the_trigger_on_its_branch.rs
revision: 9
---
## Problem

When a binding's destination row is named by the trigger's own input, `arranged_destination`
(`crates/verify/ess-conformance/src/synthesize/binding_effects.rs:2632`) arranges the destination
row first (`eligible_row`) and then rewrites the trigger's input to name that row
(`:2685-2694`). The only check beforehand is `reads_stored` (`:2678`, `:2961`), and it sees only a
`when_related` predicate over stored fields (`ResolvedRelatedTest::Holds`). It never re-checks that
the trigger, sent naming an existing row, still takes the branch that publishes the event. The
sibling paths do re-check: `related_decided`/`related_arranged` (`:1561`), the key path (`:2118`) and
`related_missing` (`:2531`) all call `keeps_branch_sent` (`:1873`). The 0.72.0 adversary pass
(review-result:ess-0720-wave-1-adversary-pass-1) found this site by reading the code.

Reproduction on 0.92.0 (`format: ess/29`, domain `demo.acct`). `Open` creates an `Account`.
`Announce(note_id, account_id)` publishes `Announced` only on its branch
`announced: {when_related: {via: input.account_id, exists: false}, creates: Note, ...}`, with the
default `taken` refusing. The binding on `Announced` invokes `Activate`, which moves the `Account`
that `event.account_id` names.

```text
$ ess verify conform synthesize --path system.yaml --out suite.json
12 scenario(s) (0 authored), 0 refusal(s), written to suite.json
# activate-on-announce/binding/mapping: Open {account_id: "account_id"} -> capture "account";
#   Announce {account_id: <instance account>} ; expect_outcome Announce/announced ; expect_invocation Activate
$ ess verify conform run --suite suite.json --path system.yaml --target interpreted --report-format 2
  failed activate-on-announce/binding/delivery
  failed activate-on-announce/binding/flow
  failed activate-on-announce/binding/mapping
  failed activate-on-announce/binding/on-failure
  12 scenarios: 8 passed, 4 failed, 0 error, 0 unsupported
```

The interpreted reference target fails every binding scenario. The arranged account makes
`Announce` take `taken`, so `Announced` is never published, and the suite fails a correct
implementation.

## Acceptance

All cases are in `cargo test -p ess-conformance --locked --test
a_binding_destination_arrangement_keeps_the_trigger_on_its_branch`.

- On the reproduction's model, `ess verify conform synthesize` reports one `ESS-SYNTH-010` refusal
  for the binding `activate-on-announce` whose message reads "invokes `Activate` on an existing row
  named by" and gives as its reason that the trigger's branch `Announce/announced` reads the input
  that names the row; the suite holds no `activate-on-announce/binding/*` scenario.
- On the same model, `ess verify conform run --target interpreted --report-format 2` over that
  suite reports `0 failed`.
- On a second model whose `Announce` branch is guarded by an input guard reading the same input
  (`when: account_id != "closed"`) in place of the `when_related`, synthesis reports the same
  `ESS-SYNTH-010` refusal for the binding and the interpreted run reports `0 failed`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no row of
  `synthesis-suite-bytes-table.tsv` changed.

## Decisions

- The binding is refused rather than arranged: on the reproduction's model the trigger publishes
  only when the row its input names does not exist, while the binding needs that row to exist, so
  no arrangement keeps the trigger on its branch, and the sibling paths that call
  `keeps_branch_sent` already keep the gap in that case.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- inferred: crates/verify/ess-conformance/tests/a_binding_destination_arrangement_keeps_the_trigger_on_its_branch.rs

confidence: high

Moves no shared surface: no suite-bytes row is re-pinned.
