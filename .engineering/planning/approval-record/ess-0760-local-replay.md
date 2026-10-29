---
format: aep.planning-md/3
id: approval-record:ess-0760-local-replay
kind: approval-record
status: approved
title: Tag 0.76.0 after a local replay on the neutral reproductions and five adopters
summary: 'D177: local replay against the 0.75.0 binary on 23 neutral reproductions and the ESS commands of loom, takt, substrate, harness and metaharness; pass'
tags:
- ess-0.76.0
- non-interactive
relations:
- decides: approval-record:ess-issues-batch-1
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T00:15:33Z", actor: "human:timo", revision: 2}
---
## Decision

Relayed by the orchestrator on 2026-10-29 (decision D177): 0.76.0 is confirmed. The reviewer
session is not running, so the replay is done locally in the D140 form, plus the real adopters.
Base is the 0.75.0 release binary; head is `ess` built from d6174f931, whose tree is the batch 1
merge tree. Pass: no new error, and only differences the CHANGELOG names.

## Result

**Pass.**

- Neutral reproductions: 23 runs. No selected-scenario count, refusal count, refusal code or
  interpreted result moved, and every suite and output is byte-identical once versions are
  normalized.
- Real adopters, `origin/main` exported read-only, each repository's ESS commands from its
  Taskfile and xtask (validate, compile, synthesize, schema, conformance synthesize and run,
  types, cli): loom b492afe76, takt 26ed4ac1, substrate e08fe3dd, harness 3eca074 and
  metaharness dfee56e, 36 commands. No exit code moved, and no new error or warning appeared.
  Printed output differs only in the output path that the replay itself chose. Synthesized
  crates, schemas and suites are identical apart from the per-run `.ess-output` state.
- One projection moved, and the CHANGELOG names it (beyond10x/ess#685): substrate's
  `spec/workspaces` types now carry `minLength: 1` and `maxLength: 255` on the root name. The
  generated `types.rs` differs only in its projection-digest header line, so substrate
  regenerates `crates/substrate-wire/src/generated/workspace.rs` when it moves its pin.
