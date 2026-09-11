---
format: aep.planning-md/1
id: review-result:runtime-report-path-correction
kind: review-result
status: active
title: Verify fixed report artifact destination
relations:
- reviews: story:adopt-truthful-conformance-counts
revision: 1
---
verdict: default delivery corrected; optional relative override remains unsafe in inspected candidate
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 66cccdee07708274f0552a3445cd003b00de976e3c8833e4f4f2dc90c3233d95, retained as local-evidence:runtime-gaps/publication-replay/snapshots/66cccdee07708274f0552a3445cd003b00de976e3c8833e4f4f2dc90c3233d95.md. Source creation recorded at 2026-09-11T05:21:22Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 35343d64d2ba91f35d97b6ead0fd0e032a717b0522a5dcb20cb2bd871a3ea622, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/35343d64d2ba91f35d97b6ead0fd0e032a717b0522a5dcb20cb2bd871a3ea622-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->
review-of: ROOT-12-1 from immutable root-review.md
scope: report-path wiring only
tests-executed: 0
origin: source, retained task dry-run, local Go runner source
mutations: this separate observation only

The inspected correction (`Taskfile.yaml` SHA256 `ad328af373d8dc31af6c72bcebd636d54907b160864fb43359cbae381866bfac`) sets `report="${ESS_REPORT_OUT:-$PWD/../target/conformance/report.json}"`, creates its parent, prints its location before the test, and sets `ESS_REPORT_OUT="$report"` on the final `go test` command. The default absolute path addresses ROOT-12-1. The final command retains the Go test's exit status. Retained `adopter-report-task-dry.log` confirms Task emits those commands in that order. No 340-case rerun is needed to verify this wiring.

The optional override introduces a separate path-base ambiguity: for `ESS_REPORT_OUT=reports/report.json`, `mkdir` resolves relative to the Task directory, while the generated report writer runs inside the Go package directory. Local Go tool source `/usr/lib/go/src/cmd/go/internal/test/test.go:1673` sets `cmd.Dir = a.Package.Dir` before execution. This can leave the actual report parent missing. Either normalize an override to an absolute path or remove the optional override and retain the already-correct absolute default. Caller override was not required for the original finding.

The coordinator chose to remove the optional override entirely and retain the fixed absolute default. This is the exact already-verified delivery branch, not a new semantic implementation requiring another full attack. The coordinator should preserve a final dry-run/hash showing that removal. The original report and this observation remain immutable; neither claims an additional adopter execution.

```findings
[]
```