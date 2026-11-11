---
format: aep.planning-md/3
id: story:generated-targets-answer-a-caller-read-in-a-row-reading-guard
kind: story
status: active
title: Generated targets answer a caller read in a row-reading guard
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#636
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:caller-attributes-are-readable-in-a-when-related-predicate
- depends_on: story:caller-attributes-walk-every-caller-read-of-a-command
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/caller_values.rs
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/http_caller.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/caller_values.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:44Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:45Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Every target must implement a caller read in a related-row predicate, a row-set guard or a set
effect's `where:`, or refuse it by name, once
`story:caller-attributes-are-readable-in-a-when-related-predicate` admits it. Today:

- `ess generate openapi` lists under `x-ess-caller` only the attributes `when:`, the subject
  predicates and values read (`crates/generate/ess-gen/src/http.rs:175-246`); a `Related` or
  `RelatedSet` condition falls into `_ => {}`, so the reproduction's `account_id` read in
  `elsewhere` would not be listed.
- Generated Rust and Go read a related predicate in an environment that includes `caller.`
  (`crates/generate/ess-synth/src/determined.rs:242-245, 848-852`) but no test pins it; row sets stay
  refused there by name (`determined.rs:209-217`).
- Entity Runtime refuses any caller read with `CallerUnsupported`
  (`crates/generate/ess-entity-runtime/src/subset.rs:596-600`); the refusal must cover the new
  positions by name.

Minimal model: the parent story's `example.g.Bind` (`missing` / `elsewhere` / `bound`).

## Acceptance

- `ess generate openapi` on the minimal model lists `account_id` under `x-ess-caller` for
  `example.g.Bind` and answers `elsewhere` with `403`.
- The generated Rust and Go behaviour for the minimal model pass the synthesized
  `example.g.Bind/outcome/elsewhere` and `/bound` scenarios (`ess verify conform run` against each).
- Entity Runtime lowering of the minimal model is refused with `CallerUnsupported` naming
  `example.g.Bind/elsewhere`; a row-set guard reading the caller keeps the generated targets'
  existing row-set refusal by name.

## Scope

- cited: crates/generate/ess-gen/src/http.rs
- cited: crates/generate/ess-synth/src/determined.rs
- cited: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: crates/generate/ess-synth/tests/caller_values.rs
- inferred: crates/generate/ess-entity-runtime/tests/caller_values.rs
- inferred: crates/generate/ess-gen/tests/http_caller.rs
confidence: medium

## Fit review

1. **Need.** A construct the parent story admits must not be ignored by a target. Reproduction: the
   parent story's model.
2. **Class.** Gap, following the parent story.
3. **Already expressible?** Not applicable: the construct does not validate today (triage run,
   ESS-COMMAND-003).
4. **Fit.** Same grid R column. Every target answers it or refuses it by name
   (`.agents/skills/assessing-external-requests/SKILL.md`, question 4, last item;
   `docs/design/caller-values.md:84-90`). Overlaps `story:caller-attributes-walk-every-caller-read-of-a-command`
   (beyond10x/ess#740), which fixes the same walk for `external:` subject guards and error
   payloads; this story extends that walk to related-row and row-set conditions.
5. **Second adopter.** As the parent story.
6. **Cost.** No format. Generated OpenAPI of a model with the construct gains `x-ess-caller` entries
   and a `403`; models without it keep their bytes.
7. **Alternatives.** (a) Leave the projections unchanged: rejected, the OpenAPI would hide a caller
   read and answer a caller refusal `404`/`409`. (b) Refuse the construct in generated targets:
   rejected for Rust and Go, which already read the caller in that environment. (c) Implement where
   the environment exists, refuse by name elsewhere: chosen.

## Decisions

accept, redesigned: as the parent story; this story carries the projections and generated targets.

## Source

github:beyond10x/ess#636
