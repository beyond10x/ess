---
format: aep.planning-md/3
id: story:a-conformance-report-replaces-its-file-in-one-step
kind: story
status: draft
title: A conformance report replaces its file in one step
summary: run --report-out truncates and then writes; an interrupted run leaves a truncated file where a complete report was
tags:
- defect
relations:
- decomposes: epic:adopter-gaps-verdicts
- serves: vision:O2
revision: 1
---
## Finding

`ess verify conform run --report-out <file>` writes the report in place: `fs::write` truncates the
file and then writes it (`crates/edge/ess-cli/src/main.rs`, the write of `--report-out` in
`render_conformance_report`; line 5082 at `eb18d0e5fb`). A run that is killed or loses its disk
between the truncation and the last byte leaves a truncated file where an earlier, complete report
was. A reader that takes the file as evidence then reads a document that is not a report, or reads
nothing.

Found by reading, in review-result:outside-target-adversary-pass-2 (judgement finding 3), and
recorded there as pre-existing: the behaviour is the same before
story:a-conformance-run-drives-a-target-process-in-any-language, whose signal handling made the
window easier to name. No deterministic reproduction exists; the window is the duration of one
write.

## Acceptance

- `--report-out` is written to a new file beside the destination and renamed onto it, so the
  destination holds either the earlier bytes or the complete new report at every instant. A test
  that fails the write after the first byte (through an injected writer) leaves the earlier file's
  bytes unchanged.
- The same holds for the other documents the command writes beside a report
  (`--accounting-out`, an execution context), or the story says which are already written as new
  files only.

## Scope

- inferred: `crates/edge/ess-cli/src/main.rs` (the report write), and whatever helper the
  known-failure accounting already uses to create its outputs as new files.
