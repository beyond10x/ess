---
format: aep.planning-md/2
id: story:a-report-says-why-a-scenario-was-skipped
kind: story
status: draft
title: A report says why a scenario was skipped
relations:
- serves: vision:O2
revision: 1
---
# Story: a report says why a scenario was skipped

## Outcome

Somebody reading an `ess-conformance-report` learns why each non-passing scenario did not pass,
without the test log beside it.

## Context

`story:a-skip-says-why-the-target-could-not-answer` put the target's own sentence into the **test
log**. The **document** still carries only `<status> <id>`, and it cannot carry more without a new
field:

- `ess-conformance-report/1`'s `failed_scenarios` is a `Vec<String>` and `evidence.rs:99-104` reads
  everything after the first space as the scenario identity. Appending a reason would corrupt that
  field for every reader, silently.
- `ess-conformance-report/2` (`counts.rs`) adds per-status counts and no per-scenario detail, so it
  is no help either.

This matters because the exit code is already not the verdict — an adopter had to write *"read the
report document, not the exit code"* into its own contributor instructions — and the document a
workflow system reads is therefore the only durable record. A record that says a scenario did not
pass and cannot say why is one somebody has to reproduce locally to act on.

## Acceptance

- A report format carries, per non-passing scenario, the status, the identity, and the reason the
  runner recorded — as separate fields, not one string to be split.
- `evidence.rs` reads the identity from its own field rather than by position, so a reason containing
  a space cannot corrupt it.
- Both runners write it. The Rust runner already holds the reason as a `Diagnostic`
  (`report.rs:487-493`); the Go runtime holds it at every `r.skip` and `r.fail` call site.
- A test asserts a wrapped target message reaches the document, not only the log.

## Out of Scope

Changing what makes a scenario skip. The exit-code defect, which is its own story.

## Ambiguities

- `requires-stakeholder-input` — whether this extends `ess-conformance-report/2` or introduces `/3`.
  `/2` is already published and read by `aep artifact evidence --from`, so extending it changes bytes
  a consumer may compare; the format owner decides.

## Open Questions

None.
