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
revision: 4
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

- A conformance case named `a_binding_destination_arrangement_keeps_the_trigger_on_its_branch`, on
  the reproduction's model (or a smaller one with the same shape). Synthesis either keeps the
  trigger on its branch, so that every binding scenario passes `--target interpreted`, or refuses
  the binding scenarios with `ESS-SYNTH-010`, naming the trigger's guard (`Announce/announced`'s
  `when_related`). No binding scenario of that model fails on the interpreted target.
- A second case with a trigger input guard reading the same input (`when: account_id != …`)
  behaves the same way, because `keeps_branch_sent` covers both.
- No model in the suite-bytes tables moves.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/binding_effects.rs` (`arranged_destination`, `reads_stored`, `keeps_branch_sent`)
- inferred: `crates/verify/ess-conformance/tests/` (new case beside the binding-effects tests)

confidence: high
