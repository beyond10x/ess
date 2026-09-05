---
format: aep.planning-md/1
id: story:review-kubectl-diagnostic-sanitization
kind: story
status: draft
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
revision: 2
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

Derived 2026-09-05 from adversary test and coordinator baseline replay.

- **Primary surface:** `crates/infra/ess-kubernetes` — cited; shared subprocess helper, binary error printing and synthetic process-boundary tests.
- **Symbols:** `kubectl`, `scan`, `contexts`, and `failed_secret_subprocess_diagnostics_do_not_echo_secret_values` — cited.
- **Documents:** no new persisted construct or format is required — inferred; value-free subprocess refusal is an existing credential-boundary requirement.
- **Confidence:** high — cited; the compiled baseline replay reached the exact error path.
- **Would collide with:** any unit editing `crates/infra/ess-kubernetes` — inferred; serialize with the sanitizer story and future observation-completeness work.
