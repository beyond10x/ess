---
format: aep.planning-md/3
id: story:input-guard-overlap-precedence
kind: story
status: active
title: An input-guarded refusal that overlaps an accepting branch is ordered or refused
refs:
- provider: github
  reference: beyond10x/ess#178
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/decision.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: docs/design/input-guard-overlap-precedence.md
- confidence: inferred
  path: website/docs/guides/write-a-specification.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T20:19:59Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T20:21:11Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

Either a stated precedence (an input-guarded refusal wins over an accepting branch it overlaps, witnessed at the overlap point) or `validate` refuses the overlap naming both outcomes.

## Acceptance

The #178 specification either synthesizes a scenario sending `{ticket_id: "", open: false}` that requires `id-required`, or is refused naming `closed` and `id-required`.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Chosen option:** stated precedence (an input-guarded refusal wins over an accepting branch it overlaps); smaller than refusing the overlap, which needs new text-guard overlap analysis, a format bump, and leaves no repair — inferred from the code below
- **Primary surface:** `crates/verify/ess-conformance` synthesis — inferred
- **Files:** `synthesize.rs` `admits_plain` (3520, checks siblings only without a default), `plain_guards` (3483), `boundary_inputs` (6201, `ConstructInput` arm 6218-6241) — cited; edit sites inferred
- **Unchanged:** `ess-domain/src/command.rs` ESS-COMMAND-003 (2297-2321, 2369-2383); overlap only checked without an unconditional outcome and over closed enums (2440-2444, 2503-2531) — cited
- **Generated branch order:** none; command decision is an owed method (`ess-synth/src/rust/obligation.rs:192`, `go/obligation.rs:254`) — cited
- **Also likely:** new #178 fixture in `tests/`; `ess-entity-runtime/src/lib.rs` (1760-1840) tie-break unknown — inferred
- **Documents:** `website/docs/reference/predicates.md` (700-720), `website/docs/guides/write-a-specification.md` (396-399), new `docs/design/input-guard-overlap-precedence.md` — inferred
- **Confidence:** medium
- **Would collide with:** `synthesize.rs` witness and boundary selection
