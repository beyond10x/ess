---
format: aep.planning-md/3
id: story:feature-request-273
kind: story
status: active
title: An event expectation checks identity fields against captured instances
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#273
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/authored_structured_instances.rs
- confidence: cited
  path: website/docs/guides/verify/author-scenarios.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:17Z", actor: "human:timo", revision: 13}
- {from: "proposed", to: "active", at: "2026-10-04T14:01:06Z", actor: "human:timo", revision: 16}
---
## Outcome

An implementation that drops an identity field from an event fails a conformance scenario: authored event expectations can compare an identity-typed field with a captured instance, and synthesis asserts the identity fields it can determine.

## Acceptance

- An authored `expect_event` may write `{$instance: name}` for an identity-typed payload field; it is resolved the way command inputs resolve it (0.43), and ESS-AUTHOR-021 `NotComparable` no longer refuses it there. A field of another type still refuses as today.
- Synthesized scenarios assert an identity-typed event payload field wherever the arrangement determines its value (the subject's identity, an input identity, a related row's identity).
- A mutant that drops such a field from the event fails both an authored and a synthesized scenario.

## Origin

beyond10x/ess#273, reported downstream on 0.48.0; the refusal is `NotComparable` at `crates/verify/ess-conformance/src/authored.rs:1553`.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the story or the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs (`generated/`, `suites/generated/`, `schemas/generated/`, `website/docs/reference/diagnostics.md`).

- **Authoring:** `crates/verify/ess-conformance/src/authored.rs` — cited: `fn event` :2516 sends non-fixture payload values to `fn literals` :3114, which raises `Cause::NotComparable` for `Written::Instance` (:3140) and a nested reference (:3131); the fix routes an identity-typed instance through `values`/`instance_at` (:3163) and emits `ScenarioStep::ExpectEventValues` (:2552). `:1553` is only the `Display` text.
- **Synthesis:** `crates/verify/ess-conformance/src/synthesize.rs` — cited: `determined_payload` :5055 / `determined_fields` return only literals (its doc states the identity exclusion); callers :2110, :2808, :8721.
- **Also likely:** `synthesize/set_effects.rs:569`, `fixtures.rs` (`event_values` :129, `used_by` :226), `scenario.rs` (`select_fresh_format` :157), `web.rs:297`, `coverage_build.rs:508` — inferred
- **Runners:** unchanged — cited: Rust `runner.rs:2523`, Go `go/runtime.go:2865` already resolve `instance` in `expect_event_values`; TypeScript `ts/runtime.ts` not read
- **Tests:** `tests/authored.rs:1543` flips (asserts `NotComparable` on `invoice_id`) — cited; `tests/authored_structured_instances.rs:580` — inferred
- **Documents:** `website/docs/guides/verify/author-scenarios.md:170-176` — cited
- **Confidence:** high for authoring; medium for synthesis
- **Would collide with (in-epic pairs by file):** 265 on `ess-conformance/src/authored.rs`, `ess-conformance/tests/authored.rs`, `ess-conformance/src/coverage_build.rs`; 266, 267, 268 on `ess-conformance/src/synthesize.rs`; 269 on `ess-conformance/src/synthesize.rs`, `ess-conformance/src/scenario.rs`
- **Safety fact:** synthesized `ExpectEventValues` makes `fixtures::used_by` true for nearly every suite, raising the fresh format from /4 to /18+ and making browser replay refuse it (`web.rs:297`) — read, unproven

## Decisions

- **accept, redesigned (coordinator, 2026-09-30; supersedes the `used_by` decision):** a suite that carries instance-valued `expect_event_values` is written at /18 or later (Go and TypeScript already admit it), and browser replay accepts instance-valued `expect_event_values` instead of refusing the suite (`web.rs:297`).

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: an identity field dropped from an event must fail a scenario. Class: defect. Authoring fits (the runners already resolve instances in `expect_event_values`). Red flag in the first format decision: narrowing `fixtures::used_by` would write /18-only steps under older suite headers (`ess-conformance/src/fixtures.rs:226-249`).
