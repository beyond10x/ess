---
format: aep.planning-md/3
id: approval-record:ess-0770-local-replay
kind: approval-record
status: approved
title: Tag 0.77.0 after a local replay on the neutral reproductions and five adopters
summary: 'D177 form: 0.76.0 binary against 84671d6b8 on 23 neutral reproductions and 36 ESS commands of loom, takt, substrate, harness and metaharness; no adopter refused; pass'
tags:
- ess-0.77.0
- non-interactive
relations:
- decides: approval-record:d186-downstream-defects-in-batch-2
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T10:39:53Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Decision

Batch 2 (decision D183, with D186 adding U9 and U10) ships in the D177 form: a local replay
against the 0.76.0 release binary on the neutral reproductions and on the ESS commands of five
adopters. U10 adds a refusal, so the replay also had to show that no adopter is refused; had one
been, the refusal would have shipped as a warning. Head is `ess` built from 84671d6b8, the batch 2
release head. Pass: no new error, and only differences the CHANGELOG names.

## Result

**Pass.**

- Neutral reproductions: 23 runs. No selected-scenario count, refusal count, refusal code or
  interpreted result moved, and every suite and output is byte-identical once versions are
  normalized.
- Real adopters, `origin/main` exported read-only: loom b94c03f1, takt 463c349f, substrate
  07396286, harness 3eca074 and metaharness 9855ebe, 36 commands (validate, compile, synthesize,
  schema, conformance synthesize and run, types, cli). No exit code moved and no drift against a
  committed projection moved. No adopter is refused as `unreachable_branch`, so U10 ships as a
  refusal.
- Differences: the output path the replay chose, the `requires ess 0.76` newer-release warning
  each adopter that pins 0.76 now prints, the generator version in generated headers and type
  reports, and the per-run `.ess-output` state.
- The two commits after 84671d6b8 (94826e95d, c5a17aab7) change test files and their fixtures only, so the replayed binary is the one the release ships.
