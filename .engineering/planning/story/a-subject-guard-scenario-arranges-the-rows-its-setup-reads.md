---
format: aep.planning-md/3
id: story:a-subject-guard-scenario-arranges-the-rows-its-setup-reads
kind: story
status: draft
title: A subject-guard scenario arranges the rows its setup reads
summary: Synthesized when_subject scenario fails against the reference implementation (#554 triage)
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#554
relations:
- serves: vision:O2
- decomposes: epic:generic-guard-precedence
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A suite ESS synthesizes must pass against ESS's own interpreted target. A command whose outcomes
are, in order, `unknown_instance: true`, a `when_subject:` predicate over a stored Boolean,
`when_related: {via: input.<id>, exists: false}`, a `when_related:` predicate on that row and an
updating branch that sets the Boolean, gets a `when_subject:` scenario that fails: its setup drives
the subject into the guarded state by running the command's own updating branch, with a related
identity no row carries, so the setup run answers the `exists: false` branch, the subject never
changes, and the scenario observes the related-row refusal instead of the subject refusal.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/a-original` of the triage of
beyond10x/ess#554, command `demo.ledger.Seal`):

```yaml
outcomes:
  - {name: unknown-item, unknown_instance: true, error: demo.ledger.NotFound}
  - name: already-sealed
    when_subject: {predicate: sealed == true}
    error: demo.ledger.AlreadySealed
  - name: no-review
    when_related: {via: input.review_id, exists: false}
    error: demo.ledger.NotFound
  - name: not-approved
    when_related: {via: input.review_id, predicate: approved == false}
    error: demo.ledger.NotApproved
  - name: sealed
    updates: demo.ledger.Item
    instance: item_id
    sets: {sealed: true}
```

```text
$ ess verify conform run --path a-original --target interpreted --report-format 2
  failed demo.ledger.Seal/outcome/already-sealed
  12 scenarios: 11 passed, 1 failed, 0 error, 0 unsupported
exit 1
```

The setup step of that scenario sends `Seal` with `review_id: "review_id"`, no `Review` arranged,
expecting `sealed`.

## Acceptance

- `ess verify conform run --path a-original --target interpreted --report-format 2` reports
  `demo.ledger.Seal/outcome/already-sealed` `passed` and 12 of 12 passed: the setup that drives the
  subject into the guarded state first arranges every row the updating branch's guards read (here an
  approved `Review` created through `AddReview`), so the setup run takes `sealed`.
- Where no declared command can arrange such a row, the scenario is refused by name
  (`ESS-SYNTH-004`, naming the row) instead of being written to fail.

Pinned by `crates/verify/ess-conformance/tests/when_subject_witness.rs`
(`a_subject_guard_scenario_arranges_the_related_row_its_setup_reads`), whose fixture restates the
minimal specification.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (the setup that reaches a stored-field state through a declared branch)
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs (arranging the row an `input.<id>` names)
- cited: crates/verify/ess-conformance/tests/when_subject_witness.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A synthesized scenario must reach the state it claims; a suite that fails ESS's own
   interpreted target is a defect. Reproduction above (run.txt § A). No syntax proposed: found
   while triaging the issue's own shape (`when_related:` beside `when_subject:`, the second
   command the issue names, rewritten as `demo.ledger.Seal`).
2. **Class.** Defect: synthesis contradicts the documented rule that a generated suite holds
   ESS's reference implementation (`crates/verify/ess-conformance/src/synthesize.rs` emits the
   scenario; the interpreted target answers the precedence rule
   `generic-language-features.md:482-526` correctly, the scenario does not).
3. **Already expressible?** The specification is valid and correct (run.txt § A, validate exit
   0). An authored scenario could stand in, but the synthesized one is wrong, so no idiom
   answers the defect.
4. **Fit.** No language change; no grid cell moves. The fix composes with the existing related-row
   arrangement (`TestStrategy::ArrangeRelatedRow`, `synthesize/related_guard.rs:417`) used for the
   branch's own scenario. Targets: synthesis only; every runner executes the corrected steps.
5. **Second adopter.** Any "already approved" or "already closed" subject guard beside a required
   related row: closing a ticket that needs an existing assignee, publishing a document that needs
   an existing reviewer.
6. **Cost.** No format, no key, no diagnostic code. Suites of models with this shape change their
   setup steps (bytes move for those models only).
7. **Alternatives.** (a) Change nothing: the suite fails every correct target. (b) Withdraw the
   scenario with a refusal: loses the subject guard's check. (c) Chosen: arrange the rows the
   setup branch reads, refuse by name only where no declared command can.

## Decisions

**Accept as proposed** (a defect; no requester design involved): the setup arranges every row the
setup branch's guards read before it runs. No format change.

## Source

github:beyond10x/ess#554
