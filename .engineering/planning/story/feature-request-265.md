---
format: aep.planning-md/3
id: story:feature-request-265
kind: story
status: active
title: An ungranted actor gets one declared refusal, witnessed per command
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#265
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/validate_sees_what_synthesize_refuses.rs
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/actor.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/actor_grants.rs
- confidence: inferred
  path: crates/specify/ess-service-contract/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_go/mod.rs
- confidence: cited
  path: docs/design/caller-values.md
- confidence: cited
  path: website/docs/guides/synthesize.md
revision: 28
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 25}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:17Z", actor: "human:timo", revision: 26}
---
## Outcome

A server that does not check actor grants fails the synthesized suite: an ungranted sender has one declared answer, and synthesis witnesses it per command.

## Acceptance

- The served contract declares one standard refusal for an ungranted actor (for example `403` `{refused: "not granted", actor}`), the same for every command.
- Synthesis emits `<command>/grant/denied` for each command some declared actor lacks the grant for; none, and a coverage fact, where every actor holds it.
- Authoring accepts a step sent as an ungranted actor only when it expects that refusal; ESS-AUTHOR-009 keeps its meaning otherwise.
- The generated Rust and Go servers check the grant before the port runs; a mutant that skips it fails the suite.

## Origin

beyond10x/ess#265.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs (`generated/`, `suites/generated/`, `schemas/generated/`, `website/docs/reference/diagnostics.md`).

- **Authoring:** `crates/verify/ess-conformance/src/authored.rs:2476-2494` (`fn actor`, `Cause::ActorMayNot`, code 9 at :1323) — cited
- **Synthesis:** `crates/verify/ess-conformance/src/synthesize.rs:1550` (`synthesize_plain`, family registration), `:10737` (`granted_actors`), `:255` (`enum Note`); new `synthesize/grant.rs` — cited / inferred
- **Contract:** `crates/generate/ess-gen/src/http.rs:91` (`FORBIDDEN`), `:125` (`fn status`); `ess-gen/src/openapi.rs:445` (`grants`), `:709` (pattern for one standard response) — cited
- **Servers:** `crates/generate/ess-synth/src/plan.rs:314,940-970` (`NeedsCallerIdentity`, `plan_actors`), `rust/http.rs:1163-1168` (headers unread), `go/http.rs`, `rust/actor.rs` (`may()` table), `go/mod.rs:612` — cited
- **Also likely:** `ess-conformance/src/coverage_build.rs:435`, `reference.rs` (the reference target must answer the refusal), `mutate.rs:72`; `ess-service-contract/src/lib.rs:330` — inferred
- **Tests:** `ess-synth/tests/actor_grants.rs:218`, `ess-conformance/tests/authored.rs:1377`, `ess-cli/tests/validate_sees_what_synthesize_refuses.rs`, `ess-conformance/tests/support_go/mod.rs:252` — cited
- **Documents:** `website/docs/guides/synthesize.md:264-273`, `docs/design/caller-values.md:89` — cited
- **Confidence:** medium
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 266 on `ess-conformance/src/synthesize.rs`; 267 on `ess-conformance/src/synthesize.rs`; 268 on `ess-conformance/src/synthesize.rs`; 269 on `ess-synth/src/plan.rs`, `ess-conformance/src/synthesize.rs`; 273 on `ess-conformance/src/authored.rs`, `ess-conformance/tests/authored.rs`, `ess-conformance/src/coverage_build.rs`, `ess-conformance/src/synthesize.rs`
- **Safety fact:** 403 is already `FORBIDDEN` for caller-decided refusals (`ess-gen/src/http.rs:91`); a command with such an outcome would get two 403 bodies under one OpenAPI key, so the standard refusal merges with it or takes another status — unproven

## Decisions

- **accept, redesigned (coordinator, 2026-09-30; supersedes the header decision):** the shell passes a typed `Caller { actor, attributes }` into `dispatch`/`handle` after authenticating; the HTTP route never reads the actor from a header; the grant check runs on that value before the port. The standard refusal is the existing 403 status with body `{"refused": "not granted", "actor": <name|null>}`, told apart from the caller-decided error envelope by its members and documented in the contract. Runners send the actor through `SemanticCommandRequest.actor`.

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: a server with no grant check must fail the suite. Class: gap. Red flag found: the first design (an `ess-actor` request header) let a client-settable header decide authorization and contradicted `docs/design/caller-values.md:86`.
