---
format: aep.planning-md/3
id: story:check-history-reads-identity-from-response-field
kind: story
status: draft
title: check-history determines a record identity taken from a response field
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify conform check-history` determines a record's identity when the model takes it from a
response field, instead of exiting 2 with `check.model-undetermined`.

## Evidence

An adopter on ess 0.56.0: `check-history` exits 2 (`check.model-undetermined`) on a history whose
record identity comes from a response field. On `main`,
`crates/verify/ess-conformance/src/interpret/execute/history.rs:201` passes
`response::Authority::default()`, so the recorded response never reaches the model; reading a
response field then returns `NotInterpreted` (`src/interpret/response.rs:98-101`), which
`linearize.rs:296` reports as `check.model-undetermined`. Adjacent:
`story:stored-field-equals-returned-response-value`.

## Acceptance

- A `check-history` test over a history whose created record's id is a response field: it
  linearizes and exits 0; the same history with a wrong recorded id exits with the mismatch.
