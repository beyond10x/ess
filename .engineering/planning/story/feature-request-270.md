---
format: aep.planning-md/3
id: story:feature-request-270
kind: story
status: active
title: A related sets value beside a when_related guard witnesses success
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#270
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-10-01T11:17:35Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":2,"verification":1}}}
---
## Outcome

A command that sets a field from a related row and also guards on that row has its success branch witnessed.

## Acceptance

- `<command>/outcome/<success>` is synthesized: the related row satisfying the guard is created first, and the expectation carries the copied value.
- No ESS-SYNTH-008 for that branch.

## Origin

beyond10x/ess#270, reproduced on 0.48.0.

## Scope

Derived 2026-09-30 by `aep:story-scoper`. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (synthesis of `when_related` branches) — inferred
- **Files:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs:486` (`prepare_at`) — inferred, the branch arrangement every `when_related` success scenario goes through
- **Files:** `crates/verify/ess-conformance/src/synthesize/related.rs:122` (`arrange`), `:417` (`point_at`) — inferred
- **Symbols:** `related_guard::prepare_at`, `related_guard::unarranged`, `related::arrange`, `related::point_at`, `related::key` — inferred
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs:2321-2340`, `:5861` — inferred
- **Also likely:** a new test in `crates/verify/ess-conformance/tests/` — inferred
- **Documents:** `CHANGELOG.md` — inferred
- **Confidence:** medium
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 229 on `ess-conformance/src/synthesize/related_guard.rs`; 257 on `ess-conformance/src/synthesize/related.rs`; 271 on `ess-conformance/src/synthesize/related_guard.rs`; 272 on `ess-conformance/src/synthesize/related_guard.rs`
- **Safety fact:** the refusal comes from `prepare_at`'s `setup.bound.contains_key(field)` check (`related_guard.rs:536`): `related::arrange` → `point_at` already bound the input the guard's `via` reads, so `prepare_at` returns `unarranged()` before `with_row` runs — walked, not run

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: success witness when `sets:` copies from the row a `when_related` guard reads. Class: defect (synthesis only).

## Decisions

- **accept as proposed.**
