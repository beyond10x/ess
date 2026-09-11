---
format: aep.planning-md/1
id: review-result:runtime-view-report-adversary-pass1
kind: review-result
status: active
title: Review compiled view delivery and separate report counts
relations:
- reviews: story:verify-compiled-view-delivery
- reviews: story:adopt-truthful-conformance-counts
revision: 1
---
verdict: needs-revision
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 6d390c896da647994f6ab7e637a8596aa2665f6570072d7296111ea4fd83c8d9, retained as local-evidence:runtime-gaps/publication-replay/snapshots/6d390c896da647994f6ab7e637a8596aa2665f6570072d7296111ea4fd83c8d9.md. Source creation recorded at 2026-09-11T05:16:30Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 4a4f89416846f86f6ea32e60a427ac6c34dbec4392e1d259316727a4b3d079ac, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/4a4f89416846f86f6ea32e60a427ac6c34dbec4392e1d259316727a4b3d079ac-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->
scope: root gap11 view-delivery and gap12 report adoption; CI expected-cause correction
review: independent source and retained-evidence review
tests-executed-by-reviewer: 0
origin: retained actual adopter run and exact embedded-suite bytes
mutations: this private review report only

| ID | Severity | Finding | Evidence | Required correction |
|---|---|---|---|---|
| ROOT-12-1 | P2 | The new `spec:conform:run` task selects report/2 but discards the separate-count report by default. | Adopter `Taskfile.yaml:417–425` sets `GOWORK`, `ESS_CONFORMANCE` and `ESS_REPORT_FORMAT` only. Its existing generated `runtime.go:3821–3847`, `writeCountReport`, writes the encoded report only when `ESS_REPORT_OUT` is nonempty; it otherwise emits no count summary. An ordinary task invocation with that variable unset therefore produces no report/2 artifact despite its description promising separate failure and skip counts. | Supply an explicit task-relative output path, optionally caller-overridable; ensure its parent exists and expose the output path before invoking the test so a failing run still leaves discoverable evidence. Preserve the test's failing exit and all category counts. |

The finding concerns repeatable task delivery. It does not invalidate the retained report/2 execution or successful AEP import.

Verified scope and evidence:

- The ESS `target_failure.rs` change is exactly two expected cause strings: Rust and Web now expect `path-collision`. The fixture changes domain `accessor.core` to `accessor.lib`, whose module artifact collides with the package `src/lib.rs`; `lib` is a valid identifier. All existing status, empty-stderr, format `/3`, target, destination-preservation and absent-output assertions remain. Retained `ci-target-failure.log` records the focused test passing.
- Gap11 is an adopter reader mistake, not missing compiler output. Both retained IR files contain 26 top-level full view declarations and 26 domain view references, with no unresolved view references. The pinned and candidate files are byte-identical, SHA256 `934e5d648599c443cee6567591404b83fe77a7d9fcdd7602fb9522001c8015a2`. Sample declarations include wire names, source entities, fields and consistency; all inspected sources resolve.
- The adopter's `conformance_command_test.go:72–103` now decodes top-level `views`, derives its required wire-name lookup from each declaration's `naming.wire`, and rejects an empty view table. Its previous sidecar reader was removed. The matching Taskfile change removes the independent YAML-to-`views.json` extractor. These changes consume the existing IR authority and do not create a new representation.
- ESS `compiled_view_delivery.rs` exercises actual CLI stdout and `--out` disk bytes, requiring them equal, then checks the domain reference resolves to the complete top-level view with source entity, wire name, consistency, field order and type references. Its fixture has exactly the three expected view fields. `ResolvedView` serialization carries these keys; CLI `main.rs:2018–2038` writes and prints the same canonical string, without an extra stdout newline. Root's retained `compiled-views-test.log` shows this new test passed 1/1; `root-cli-clippy.log` shows scoped lint finished successfully. The reviewer did not rerun either.
- Gap12's retained report is honest: total 340 = 96 passed + 16 failed + 228 skipped + 0 error + 0 unsupported. Outcome array lengths match every category, all 340 IDs are unique, and the exact generated embedded suite contains 340 scenarios. Its SHA256 `bb0222766d5adb0e98c5b3cb5a53f309b1a2578de39c9a035b115ba3cbe936d9` equals the report's suite identity. Both execution and conformance status remain failed, coverage knowledge remains unknown, and the run log retains 15 backend payload failures plus one campaign outcome failure. No passing-conformance claim follows from report delivery.
- The adopter AEP journal records `ess_conformance_v2` evidence for its candidate migration plan against that exact report and suite, preserving separate counts and failed statuses. This independently corroborates the coordinator's reported successful import; no AEP mutation was performed by the reviewer.

Candidate association before correction:

| File | SHA256 |
|---|---|
| ESS `crates/edge/ess-cli/tests/target_failure.rs` | `96d4fd8e587bf263267d6d8f250c2e0f1cd56027f789405d8e4e56a8386acd89` |
| ESS `crates/edge/ess-cli/tests/compiled_view_delivery.rs` | `6f3267006451e525b2233fd7c1ce6dc1ab5f18548d7ed63b55263f1a70e399f7` |
| Adopter `Taskfile.yaml` | `141c047a16891741b0b44b6e287b5eb0cca7b2c304ddbc7ff277bcb5ecfb66b8` |
| Adopter command reader test file | `b9410e1dd5bb80e773bf154ea65795b2f0a3a4c56fc9e576d17c8a9c8f307a28` |
| Retained `adopter-report2.json` | `98a3b41364136f764604ee93fc069b7aa993c6855e14c0e20b73e1e33ff066d0` |
| Retained `adopter-report2-run.log` | `5a6009f8612b7a51fc96f203aa722c20cb80b8f2fcf323698405b5c3e6904fcc` |

Reviewed managed trees: ESS `wt-643e4be649fc` and isolated adopter `wt-19e0cf57edd6`. Only the two named ESS tests and the two named adopter changes were judged. Copied primary user changes, source/4 and other partial implementation were excluded. No source edit, build, test execution, Git operation that writes state, planning mutation or lease clearing occurred. No lease was needed for this read-only source review; the coordinator's lease was untouched. Root must record this finding before correction; any correction verification belongs in a separate record, leaving these review bytes immutable.

```findings
- file: consumer-specification/Taskfile.yaml
  line: 417
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: The adopter task selects report2 but omits ESS_REPORT_OUT and discards count evidence on ordinary invocation.
```