---
format: aep.planning-md/1
id: verification-report:normalization-binary64-integrated
kind: verification-report
status: draft
title: Binary64 model and normalization integrated verification
relations:
- verifies: story:model-binary64-fields
revision: 2
---
## Outcome

Authored ess/2 models can declare finite Binary64 separately from Integer and exact Decimal. Compiler-owned numeric locations authorize format-5 normalization; explicit policies and typed literals preserve signed zero, subnormals and nearest-even rounding through the reference, actual CLI and generated Rust/Go adapters. Complete legacy format-1 through format-4 adapter maps remain unchanged at the same generator version. Standalone native structural codecs and TypeScript normalization remain separately bound follow-ups; whole-system synthesis and conformance explicitly refuse unsupported Binary64 before publication or execution.

## Source and integration

Implementation bf16e504ccad68b2ee67607ba39606aadf07f627 and independent added tests79f062f9a17ca3d97ec8cd052cf69550b2f42ff9 were integrated at242adef716f64473ed525e39cfa007556ed6ea2a. Incoming count-writer main87d9945051f5b2e6296cea8b4cbb82ad2e50aeb3 was then merged as12fb11a9225cea39975ad3303edc907046634238. The combined admission error retains every located issue, both run and try_run return checked errors before clock/target access, and run_admitted retains the sealed executed-suite digest. Original-byte admission, sparse model refusals, typed Serde guards, count report rendering and the existing generated Go admission order remain intact. New fallible emitter callers and authored cause locations were adapted without removing assertions.

The final executable/test content was committed in33215d2 after the gates below. A subsequent metadata-only merge d3401f1 retains published count closurebe0eefd7ec125d46bb3b664c4b95b8d638a2b1fe; it changes no executable source. Both append-only planning journal parent sequences were preserved byte-for-byte in order during Git conflict resolution and validated through AEP. No fabricated planning event was inserted.

## Review and corrections

The immutable review-result:normalization-binary64-adversary-pass1-public body has SHA-2560efd4fe5a026d29d2e74315ddb603a37ae9b230cc71725c2c30ba3a6b1419fd2. It found no product issue at the frozen unit and retained all926 original tracked files with five added tests. Scoped implementation qualification reported1,036 distinct tests and70 numeric vectors per reference/CLI/native lane; a same-command full-base count was not measured.

The immutable review-result:normalization-binary64-count-integration-pass1 body has SHA-256f549112de55ae8e9288c61c8f8847a0eec7da824fd14c7d35303c9e536339127. It found no product issue at12fb11a and retained all957 tracked files with two added test files. The focused conformance lane rose8→11 and the CLI lane passed6 cases, including actual generated-Go positive controls and refusals. Both review outcomes are no-op; no human approval or verifier independence is claimed.

The first whole gate on242adef exited201 after1,533 passes and one layout-test failure: its dated opening wave output quoted two proposed filenames that did not materialize. The complete unchanged computation moved to the established historical review area and the live plan points to the actual implementation. No path assertion was weakened. The initial merge compilation exited101 for an unadapted authored error field; the corrected all-workspace/all-targets compilation passed.

The first combined whole gate exited201 because the new native CLI test assumed ESS_GO_COMPILER was always set. review-result:binary64-count-default-gate-harness records this introduced test blocker before correction. The harness now honors an explicit absolute compiler override and otherwise invokes Go on PATH, matching ordinary CI discovery. Its isolated native case passed with the override absent, retaining both positive controls and all eight refusal/destination checks. The finding's outcome is fixed. Initial adversary-only harness errors (a strict Go version spelling check and a missing test-double method) remain in the original reports; no product correction is attributed to them.

## Integrated gates

| Command | Exit | Seconds | Finished UTC |
| --- | --- | --- | --- |
| task check | 0 | 99.232 | 2026-09-06T11:32:11+00:00 |
| task site-build | 0 | 16.906 | 2026-09-06T11:33:06+00:00 |

The literal full workspace gate executed1,945 Rust cases in163 result groups, with zero failed or ignored. Formatting, strict Clippy, workspace tests, rustdoc, examples, generated projections, release consistency and action checks all exited zero. ESS_GO_COMPILER was absent, matching default gate behavior. The site gate executed its WASM/browser-lab checks, pinned dependency installation and production documentation build. Final metadata-merge layout qualification passed all5 cases. No executable/source changes followed the full gates.

## Resources and publication boundary

All owned Cargo/native processes are terminal. The final independent merge review measured30,816,636,928 bytes in the existing coordinator target and30,552,416,256 free filesystem bytes, above the8GiB reserve. The separate Binary64 unit target used about3.23GB. Build directories remain managed for the continuing authorized sequence and final cleanup; no unrelated tree was retired. Agent token/cost telemetry was unavailable.

This is a validated main-source checkpoint, not a release tag or a completed adopter cutover. Released ESS remains0.19.0. Native structural codecs, positional recipe6, TypeScript normalization and the final release/adopter/Atlas publication sequence remain pending. Existing decoder and runtime-consumer contracts are not claimed solved by typed model declarations.

## Published checkpoint and Atlas delivery

Published ESS main6c6620b3783cbd41ca31a998805bc8e51e0c3515 has successful CI34030782738, documentation validation34030782692 and passive source bundle34030782607. The exact artifact9988526231 has digest sha256:499c414e338f613a126f5582b77bded4feaae39b9400b6c13a798c0cd668887c. Independent download verification checked all40 passive source files and their manifest against this commit. No new release tag was created.

Normal Atlas publication34031276932 completed successfully under Atlas control34fa907ff3ffcbdbb6bbf37d4dd674c05904bb20. Fresh live PROVENANCE.json at2026-09-06T12:00:16Z records the exact ESS source and producer/artifact above, source set a934aa1d0c586d07076b660a45b4d5fa1f1468a8b7d870900db65b8310e44058 and bundle8dabaa3755e1566eabb7f21133c40aa4983702b3cf8e148084701599515a6a30. Website runtime remainsfc4571534765c098ed861bc326da4d3da0d1df63. Its build and independent artifact-verification stages establish this content delivery; no full Website runtime gate is claimed.

Publication: https://github.com/beyond10x/atlas/actions/runs/34031276932 . Live receipt source: https://beyond10x.github.io/PROVENANCE.json . Native structural codecs, positional input, TypeScript normalization and the final adopter/release sequence remain separate pending work.
