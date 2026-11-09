---
format: aep.planning-md/3
id: story:an-authored-success-act-claiming-an-error-is-refused
kind: story
status: draft
title: An authored success act that also claims an error is refused
summary: 'outcome: <success> plus error: compiles to a step no target can pass; refuse it at compile (W1 RUN1 review finding, pre-existing, backlog)'
tags:
- backlog
- defect
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:adversary-w1-run1-pass-1
- informed_by: review-result:security-w1-run1-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 4
---
## Need

An authored act may name a success outcome and also claim an `error:`. `authored.rs` accepts the
combination: `error()` (`crates/verify/ess-conformance/src/authored.rs:4261`) refuses only undeclared
names, and nothing refuses an error on a branch that declares none. Since wave W1,
`expected_outcome` (`authored.rs:4354-4381`) writes `expect_no_error` after the `expect_outcome` of a
branch with no error, so such an act lowers to `expect_outcome`, `expect_no_error`, `expect_error`,
which no target can pass. Before W1 it passed only a target that answered the success branch with
that error. Both W1 RUN1 review passes found it (review-result:adversary-w1-run1-pass-1,
review-result:security-w1-run1-pass-1). It predates them (b81deac78), and no document in the tree
writes such an act.

## Problem

Reproduced on 0.92.0. In the model, `Send` has `undeliverable` (`when: hold == true`, `error:
parcel.post.Undeliverable`) and `sent` (no error). The authored act:

```yaml
type: ess-scenario/6
domain: parcel.post
scenario: a-success-act-claiming-an-error
summary: An act names the success branch sent and also claims an error.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: parcel.post.Send
    input: {city: Lisbon, zip: '1000', region_code: PT, region_label: Lisboa, hold: false}
    outcome: sent
    error: {name: parcel.post.Undeliverable}
```

`ess verify conform synthesize --path spec.yaml --scenarios scenarios --out suite.json` exits 0 with
0 refusals. The authored scenario's steps are `execute_command`, `expect_outcome`,
`expect_no_error`, `expect_error`, and `ess verify conform run --target interpreted --suite suite.json
--report-format 2` fails it (exit 1).

## Acceptance

- `crates/verify/ess-conformance/tests/authored_success_act_claiming_an_error.rs`,
  `a_success_act_that_also_claims_an_error_is_refused`: compiling the act above with
  `ess_conformance::authored::compile` yields one refusal and no scenario. The refusal names the
  act, the outcome `sent` and the claimed error `parcel.post.Undeliverable`, under an existing
  `ESS-AUTHOR-*` code or a new one that is listed in the diagnostics reference. Red on 0.92.0.
- Same file, `a_refusing_act_claiming_its_error_still_compiles`: `outcome: undeliverable` with
  `error: {name: parcel.post.Undeliverable, ...}` compiles as before.
- `ess verify conform synthesize --scenarios` with the act above exits non-zero and prints the
  refusal; the committed billing suite keeps its bytes.

## Scope

- cited: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/tests/authored_success_act_claiming_an_error.rs
- inferred: website/docs/reference/diagnostics.md

confidence: high

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
