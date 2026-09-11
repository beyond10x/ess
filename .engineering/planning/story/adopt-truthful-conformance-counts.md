---
format: aep.planning-md/1
id: story:adopt-truthful-conformance-counts
kind: story
status: implemented
title: Route the measured adopter report through separate failure and skip counts
tags:
- priority-high
relations:
- decomposes: task:runtime-adopter-gaps-9-13
- serves: vision:O2
scope:
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 5
---
## Outcome

Measured run: 340 total,64 passed,275 skipped,1 failed; legacy report/1 exposes276 nonpassing rows. Existing implemented story a-skipped-scenario-is-not-a-failed-one adds report/2 separate counts. Verify exact generated runtime/selected report format, provide actual --report-format 2 or ESS_REPORT_FORMAT=2 adoption, and compatible reader delivery. Preserve legacy report/1 canonical bytes and documented meaning. Add only a missing focused regression if needed; do not reconstruct implemented counting. Test path is inferred; counts.rs:53, CLI main.rs:547 and Go runtime.go:1692,4070 cited by scoper. Consumer edits require separately governed isolated checkout.

## Acceptance

Address gap 12 of task:runtime-adopter-gaps-9-13 with executable evidence and actual adoption instructions. Use generic public examples; private consumer names stay in private evidence. No full local gate or unchanged passing test reruns.

## Scope

Confidence: high for cited existing symbols; new test paths are inferred.

- crates/edge/ess-cli/tests/runtime_gap_report_counts.rs (inferred implementation surface from read-only intake).

## Actual adopter execution

An isolated copy of current primary conformance changes was run with explicit ESS_REPORT_FORMAT=2 against its existing generated runtime, after fixing its view reader. Input ownership/hashes are retained privately. Actual output:340total,96passed,228skipped,16failed,0error,0unsupported. Exit1 is honest:15 new backend response failures plus one campaign outcome failure. Runtime0.681seconds. This supersedes the older340/64/275/1 snapshot for these exact input files and is not a claim that the target conforms. Separate counts already work when selected. An explicit spec:conform:run task selects report2. No unchanged rerun was performed. Report and logs retained privately.

## Completion evidence

The actual report2 run retained SHA256 98a3b41364136f764604ee93fc069b7aa993c6855e14c0e20b73e1e33ff066d0, paired with exact suite4 bytes SHA256 bb0222766d5adb0e98c5b3cb5a53f309b1a2578de39c9a035b115ba3cbe936d9. AEP evidence import accepted that exact suite/report pair and recorded its failing status and separate counts. No green conformance claim is made.

Review-result:runtime-view-report-adversary-pass1 identified that the new task did not set its report destination. The corrected task uses an absolute path, creates its parent, selects report2, and preserves the test failure exit. Review-result:runtime-report-path-correction records the default absolute-path correction and why an optional relative override was removed. Final task dry-run and hashes are retained. The existing report implementation needed adoption, not a duplicate counter. Implemented means verified adoption instructions and isolated migration are ready; upstream publication remains part of the final PR delivery.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 fb9fe53687db9d7bd80756da524a57a0e5d66612b383ddf0fdd1194c2a8a3545, retained as local-evidence:runtime-gaps/publication-replay/snapshots/fb9fe53687db9d7bd80756da524a57a0e5d66612b383ddf0fdd1194c2a8a3545.md. Source creation recorded at 2026-09-11T04:58:05Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
