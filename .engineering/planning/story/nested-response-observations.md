---
format: aep.planning-md/3
id: story:nested-response-observations
kind: story
status: draft
title: Observe nested response mappings across every conformance runtime
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
---
## Outcome

Every source-admitted nested ResponseField destination has a synthesized relationship observation and the same healthy/fault verdict on native, generated Go, TypeScript and browser/WASM runtimes.

## Fit review

Internal coverage gap discovered during native response scoping under the operator's all-features requirement. value_expression.rs admits nested ResponseField and resolve.rs recursively resolves struct leaves, while response.rs::Observation::of recognizes only immediate top-level ResponseField. Source evidence is recorded in interpreted-response-next-scope.md SHA d829db314ae24ea766ab8bd1de3da2cf9d8c5d8ad6396453c6e568c507ab5a9c. No probe or implementation is claimed yet.

## Acceptance

First compile an admitted nested Packet.value destination and measure whether response.value=37/event.packet.value=38 evades the relationship assertion while all shapes remain valid. Preserve that red and a healthy control. Represent nested target authority explicitly, retain complete closed validation, whole-value equality, exact invocation association and optional presence, and execute independent wrong-response/wrong-event controls on all four runtime surfaces. Native direct execution alone is insufficient.

## Decisions pending

Inspect existing typed path vocabulary before choosing representation. Do not reinterpret dotted strings in an existing envelope. Any persisted semantic change requires coordinated format allocation, old-reader refusal and preserved top-level bytes where possible. This draft reserves no version and authorizes no implementation. It remains required full-backlog work after the bounded native response unit.

## Scope

Inferred response.rs, synthesize.rs, scenario/admission modules, Go/TypeScript response runtime assets and generation entry points, corresponding parity/WASM tests. Root must record exact typed scope and binding design after the red-capable probe and coordinate overlapping synthesize.rs with312 and the serial282/304 owner.

## Scope inspection, 2026-10-03

Read-only report SHA256 2bd62ad5b85e0001b5cd6bb45889d3463bfea378448dd849f8a421dc3264c371 confirms immediate-only mappings. Prefer explicit destination segment arrays; never reinterpret existing dotted keys. Existing AccessorPlan limits paths to three segments and cannot cover the admitted32-level construction grammar unchanged. Decide between an additive omitted collection and a separate closed DTO after an actual red-capable source probe. Preserve old bytes, exact invocation association, Optional leaf semantics, whole-value equality and historical admission. Full-root declarations must not accidentally reject source-admitted Binary64 or non-String-map generated siblings; choose a structural path certificate or explicitly implement the required profile support with parity. No version is reserved. Synthesis recursion appears containable in response.rs without touching the concurrently owned synthesize.rs; this is source inspection, not execution evidence.

Browser web.rs currently refuses every response-bearing suite and its player navigates declarations only. This separate required gap is recorded as browser-response-conformance. Actual standalone WASM Runner execution does not close the browser product acceptance. Both remain required full-backlog work.

## Measured relationship defect, 2026-10-03

Confirmed defect: nested response relationships can pass the complete real Runner with unequal values.

Source authority: managed tree `ess-backlog-next-20261002`, HEAD `8b6c95c7e54d571723304fe689dec8965004dcb4` (latest source integration `9e8894b7f`). HEAD was identical before and after execution, and `git diff HEAD -- crates` was empty. Exact dependency Git objects are retained in `dependency-source-git-objects.txt`. No production source, AEP artifact, persisted format or remote artifact was changed.

Probe authority: private diagnostic directory `ess-nested-response-probe-20261003`, source `src/main.rs` SHA256 `16552d60ad91f0a340af130288f4493da299d3f0c97adcd6e7b3f2238d705fcd`; model `model.yaml` SHA256 `08abfdf512a339ef4470636addb97fed52fc13c4c946d9469a8e237fbb229875`. All runnable probe code is Rust. It calls Specification::assemble, compile, synthesize, AdmittedSuite::from_suite and the real Runner with an independent ConformanceTarget. No interpreter or authored expected response value supplies its verdict.

The ess/14 source declares Packet.value: Integer, response.value: Integer and event.packet.value sourced from that response. An independent generated receipt: String keeps a separate event assertion active. Source parse, assembly and compilation succeed. Synthesis emits exactly one scenario, `demo.api.Read/outcome/returned`, with zero refusals and zero notes. The original canonical suite is retained as `suite.json`, SHA256 `39380b6a20ad97e89587879d08280c8b7bcd51c932cacd6e060dd968c868e0b9`, under ess-conformance/34 with empty initial state.

| Target mode | Response value | Event packet.value | Scenario passed / failed / other | Commands executed |
|---|---:|---:|---:|---:|
| Healthy | 37 | 37 | 1 / 0 / 0 | 1 |
| Wrong event | 37 | 38 | 1 / 0 / 0 | 1 |
| Wrong response | 38 | 37 | 1 / 0 / 0 | 1 |
| Wrong generated sibling (receipt=null) | 37 | 37 | 0 / 1 / 0 | 1 |

Every case executes the entire unmodified synthesized suite. Its only steps are ExecuteCommand, ExpectOutcome and ExpectEvent. Every report contains three checks: ESS-CF-OUTCOME, ESS-CF-EVENT and ESS-CF-PAYLOAD. The event shape checks packet.value is an Integer and receipt is a String, but no step compares packet.value with response.value. Healthy, wrong-event and wrong-response reports are byte-identical (SHA256 `46abcbfcc10041229ffd646bed23998a3a36aac6c58308a3483103e080a30c44`), as are their count reports. The receipt negative control fails ESS-CF-PAYLOAD with expected `demo.api.Returned.receipt holds a String`, observed `receipt = null`. This rules out an inert target or missing payload execution as the explanation.

The probe asserts that both unequal pairs must fail after first requiring the healthy control to pass and the receipt control to fail. Its terminal exit is **101** at the explicitly named `NESTED_RESPONSE_RELATIONSHIP_GAP` assertion. The initial measured run and a direct binary repeat produce the same verdict matrix and terminal exit. `probe.log` SHA256 `55f028e808e47ebee8440aa09f1be23a30bda0dbf50c06ac4894bf34f7e255f2`; repeat `probe-repeat.log` SHA256 `d161c0a3e792043c034ca900ddff6514d106efdff372f5d5940309c23126f8c5`. Complete individual reports/counts, source, lockfile, suite and hashes are retained in `evidence.sha256`.

Source explanation, now backed by execution: `crates/verify/ess-conformance/src/response.rs::Observation::of` scans only immediate ResponseField values and skips Struct mappings. `event_shape` retains the nested Integer leaf as an ordinary shape assertion. `src/synthesize.rs::expression_value` does not invent a response-derived literal, so the unequal shape-valid values have no relationship assertion to violate. Source admission/refusal and inactive-event hypotheses are contradicted by the measured controls.

Smallest next design proposal: retain historical flat mappings and bytes, and add explicitly typed destination segment arrays for nested relationships; do not reinterpret a dotted legacy key. Recursively discover response leaves inside Struct mappings in response.rs and attach one validated command/outcome/event observation to the existing step. Preserve existing exact invocation association and whole-value/Optional semantics. Preserve ordinary assertions for generated or input-derived siblings, filtering only the response-mapped terminal subtree from ordinary shape assertions. No new authored syntax or response-generation policy is needed.

Before implementation, root must bind the new DTO/admission authority and choose a coordinated format strategy. A path certificate must prove declared struct traversal and terminal compatibility without accidentally imposing leaf response-profile restrictions on unrelated siblings; alternatively explicitly accept and test any required schema-profile extension. Apply the same closed path authority and comparison in native, actual generated Go and TypeScript, with real WASM execution. The separately tracked browser product requirement remains required; this native probe does not discharge it. The previous scope report's compatibility, resource bounds and source-owned file list still apply; no version is reserved here.

Reproduction: from the private diagnostic directory, set CARGO_TARGET_DIR to the owned `ess-backlog-synthesis-20261002/target`, CARGO_BUILD_JOBS=1, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_DEV_INCREMENTAL=false, CARGO_INCREMENTAL=0 and TMPDIR to the private directory's `tmp`; run `cargo run --offline`. After compilation, executing that target directory's `debug/ess-nested-response-probe` reproduces directly. The direct repeat completed in under one second. Compiler: rustc 1.98.1 (48a229cea 2026-09-01); Cargo 1.98.1. The first local harness compilation rejected serialization of ExecutedRun; the retained `probe-compile1.log` records that harness-only error, corrected by serializing its public report(). It is not product failure evidence.

Resource handling: no build began below the hard 8 GiB floor. Authorized cleanup reclaimed exactly 1,452,347,392 allocated bytes from 37 inspected, idle, tagged nested Cargo output directories plus generated rustdoc and WASM output in the assigned synthesis cache. Generated fixture sources, all backlog-input evidence (verified by SHA256), and the main warm debug cache were preserved. Exact directory/size manifests are retained. The subsequent build used one job with no debug or incremental output; final available space was 16,887,271,424 bytes. Own cache lease was released after execution. No other session's output was removed.

This report omits local absolute paths for safe publication; the source manifest containing local path dependencies remains private evidence.
