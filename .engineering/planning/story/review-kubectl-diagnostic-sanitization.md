---
format: aep.planning-md/1
id: story:review-kubectl-diagnostic-sanitization
kind: story
status: implemented
title: Keep untrusted kubectl stderr out of ESS diagnostics
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- informed_by: review-result:review-boundaries-1-secret-adversary-pass-1
scope:
- confidence: cited
  path: crates/infra/ess-kubernetes
- confidence: cited
  path: crates/infra/ess-kubernetes/src/lib.rs
- confidence: cited
  path: crates/infra/ess-kubernetes/tests/fixtures/fake_command.rs
- confidence: cited
  path: crates/infra/ess-kubernetes/tests/secret_boundary.rs
revision: 10
---
## Finding and source

Additional F05 credential-boundary finding from `review-result:review-boundaries-1-secret-adversary-pass-1`. The test at `crates/infra/ess-kubernetes/tests/secret_boundary.rs:205` makes both Secret kubectl invocations fail with a synthetic sentinel in stderr. The live caller path is `scan` -> `kubectl` at `crates/infra/ess-kubernetes/src/lib.rs:74` / `:34`; the wrapper includes raw subprocess stderr in its returned error, printed by the binary.

Coordinator baseline replay on 2026-09-05: the coordinator package source had an empty diff against opening `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b`. `cargo build --locked -p ess-kubernetes --bin ess-kubernetes` exited 0. With the adversary's Rust fake command and `ESS_TEST_SECRET_FAILURE=SYNTHETIC-MALFORMED-SECRET-SENTINEL`, baseline scan exited 1 and printed:

```text
error: kubectl --context synthetic-context get secrets -o json failed: malformed synthetic Secret response: SYNTHETIC-MALFORMED-SECRET-SENTINEL
```

The existing destination retained SHA-256 `374d012df97860f3bf7377c7d9c71906203a296bad6089188490fe944b4e9747`. This establishes pre-existing origin; the immutable adversary report correctly leaves its own origin undecided. The probe uses synthetic input and does not claim a live-cluster disclosure was observed. Private replay logs are retained under the coordinator's `target/review-boundaries-1/baseline-probe/`.

## Acceptance

A failed kubectl process cannot copy untrusted stderr contents into ESS diagnostics, including the failed Secret retry path.

## Implementation boundary

The credential-edge subprocess helper must produce useful value-free refusal context (operation and exit status) without echoing response values or untrusted stderr. Apply the rule to every caller of this shared helper so another invocation does not retain the same bypass. Preserve valid collection output, explicit context selection, retry policy and observation format. Do not change unrelated downstream libraries or reach a live cluster.

## Validation

Retain the adversary's original failing assertion as the red reproduction. Cover Secret all-namespaces and fallback failures plus another helper caller, synthetic strings/invalid UTF-8 diagnostic bytes, and preserved destination contents. Valid scans and existing mutation/redaction guards stay green. Run the full ess-kubernetes package suite, formatter and strict Clippy with actual runner counts; integrated offline gate decides landing.

## Scope

Confirmed 2026-09-05 from the implementor table in docs/reviews/2026-09-05-review-boundaries-2-diagnostic-implementation.md.

- **Primary surface:** crates/infra/ess-kubernetes — cited; actual production diff is src/lib.rs, with tests/secret_boundary.rs and tests/fixtures/fake_command.rs.
- **Mechanism:** static adapter operation labels and exit status replace stderr and complete argument rendering at kubectl and its four call sites — cited; the direct mechanism measurement changed two disclosure failures into 11 package passes.
- **Caller inventory correction:** cluster metadata uses the same KINDS resource loop, not a separate helper — cited; the regression matrix derives resource cases from KINDS and covers contexts/current-context separately.
- **Persisted bytes and documents:** no new document/format needed — cited; valid observation golden bytes and successful retry output/order remain equal, with no sanitizer production diff.
- **Confidence:** high — cited; package gates and actual changed paths confirm the earlier inferred mechanism and package reservation.
- **Would collide with:** edits to this package, especially shared subprocess and fixture handling — inferred; typed package/file scopes remain reserved.

## Wave 2 execution evidence

Implementation commit b26829a571c0569ba2f63a5da495b987397b43a4. The implementor observed two initial red disclosure assertions, final package 8→11 with zero failures, fmt and strict Clippy exit 0; the credential mutation failed and was restored. review-result:review-boundaries-2-diagnostic-adversary-pass-1 preserves the tests-only attack verbatim: 11→14, no findings, formatter and Clippy exit 0. No adversarial correction was needed. The story remains active until the complete integrated gate is observed.
