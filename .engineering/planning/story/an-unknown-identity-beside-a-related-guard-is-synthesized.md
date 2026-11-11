---
format: aep.planning-md/3
id: story:an-unknown-identity-beside-a-related-guard-is-synthesized
kind: story
status: active
title: An unknown identity beside a stored-field pair is synthesized
summary: ESS-SYNTH-008 refuses an unknown_instance outcome that reads no related row (#554 triage)
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- serves: vision:O2
- decomposes: epic:generic-guard-precedence
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/unknown_instance_not_found.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:56Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:57Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

An `unknown_instance:` branch is selected by the absence of the addressed row and is answered at
step 2 of the precedence rule, before anything the row holds is read
(`docs/design/generic-language-features.md:482-526`). A scenario that sends an identity no row
carries reaches it whatever the other branches read. Synthesis refuses that scenario as
ESS-SYNTH-008 when the command's other branches are a stored-field pair that partitions every row
with no default (`when_subject: {predicate: defined(note)}` and
`when_subject: {predicate: not defined(note)}`, admitted without a default since beyond10x/ess#644):
the command-wide `observe_subject_fact` strategy is applied to a branch that reads no stored field.

The shape this story was filed for, `unknown_instance:` beside `when_related: {via: input.<id>,
exists: false}` and an updating branch, no longer reproduces: on 0.92.0 `demo.ledger.Link` in
`repro/a-original` synthesizes `Link/outcome/unknown-item` and it passes on the interpreted target
(run.txt § A).

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/d-unknown-beside-pair` of the triage of
beyond10x/ess#554):

```yaml
  - name: demo.desk.Touch
    input:
      - {name: item_id, type: demo.desk.ItemId}
    outcomes:
      - {name: unknown-item, unknown_instance: true, error: demo.desk.NotFound}
      - name: touched-noted
        when_subject: {predicate: defined(note)}
        updates: demo.desk.Item
        instance: item_id
      - name: touched-bare
        when_subject: {predicate: not defined(note)}
        updates: demo.desk.Item
        instance: item_id
```

```text
$ ess verify conform synthesize --path d-unknown-beside-pair --out d-suite.yaml
refused: refusal[ESS-SYNTH-008]: outcome demo.desk.Touch/unknown-item has no scenario `demo.desk.Touch/outcome/unknown-item`
  its strategy is `observe_subject_fact`: the subject's stored fields select it, and this scenario asked for an input that reaches it
6 scenario(s) (0 authored), 1 refusal(s), written to d-suite.yaml
```

## Acceptance

- `ess verify conform synthesize --path d-unknown-beside-pair` reports 0 refusals and writes
  `demo.desk.Touch/outcome/unknown-item`, which sends an identity no row carries and expects
  `NotFound` with no event; `ess verify conform run --target interpreted --report-format 2` reports
  7 of 7 passed.
- The same holds with a Boolean pair (`flag == true` / `flag == false`) in place of the
  `defined`/`not defined` pair.
- `ess verify conform synthesize --path a-original` keeps `demo.ledger.Link/outcome/unknown-item`
  (the related-guard shape stays fixed).

Pinned by `crates/verify/ess-conformance/tests/unknown_instance_not_found.rs`
(`an_unknown_identity_beside_a_stored_pair_is_synthesized`,
`an_unknown_identity_beside_a_related_guard_is_synthesized`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`unknown_instances`, around line 15632; the ESS-SYNTH-008 `observe_subject_fact` cause, around line 1960)
- inferred: crates/verify/ess-conformance/src/synthesize/existence.rs
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/tests/unknown_instance_not_found.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** Every branch the precedence rule admits beside `unknown_instance:` must get the
   unknown-identity scenario, which reads nothing but the identity. Reproduction above. No syntax
   proposed: found while triaging the issue (0.74.0), and the stored-pair variant in batch 1 of
   beyond10x/ess#644.
2. **Class.** Defect: the model validates (run.txt § D, exit 0) and the rule answers an unknown
   identity at step 2 (`generic-language-features.md:492`), but synthesis refuses a scenario it
   can build.
3. **Already expressible?** The model is already valid; only an authored scenario can stand in
   today, which the refusal's help suggests. No idiom answers a synthesis defect.
4. **Fit.** No language change, no grid cell moves. The fix takes the unknown-identity scenario out
   of the command-wide strategy, as `unknown_instances` already does for the related-guard shape
   (run.txt § A, `Link`). Targets: synthesis only.
5. **Second adopter.** Any "touch" or "refresh" command that answers differently for a row with or
   without an optional field and refuses an unknown identity: a profile with or without an avatar,
   an order with or without a coupon.
6. **Cost.** No format, no key, no diagnostic code; suites of models with this shape gain one
   scenario and lose one refusal.
7. **Alternatives.** (a) Change nothing: adopters author the scenario by hand. (b) Require a
   default branch: contradicts the beyond10x/ess#644 admission. (c) Chosen: synthesize the
   unknown-identity scenario independently of the strategy that selects the other branches.

## Decisions

**Accept as proposed** (a defect; no requester design involved). Retitle to "An unknown identity
beside a stored-field pair is synthesized"; the related-guard shape is kept as a regression test.
No format change.

## Source

github:beyond10x/ess#554
