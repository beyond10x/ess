---
format: aep.planning-md/2
id: story:witness-search-beyond-64-candidates
kind: story
status: active
title: A guard over three inputs is witnessed, not refused after 64 candidates
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary2_connective_and_source_mutants.rs
revision: 7
---
## Scope

The witness search refuses a guard over three or more inputs after 64 candidates, for example
`all: [any: [a > 10, b > 10], c > 10]` over three `Integer` inputs: `refusal[ESS-SYNTH-003]: no
candidate of the 64 tried satisfies ((a > 10 or b > 10) and c > 10)`
(`crates/verify/ess-conformance/src/synthesize.rs` ~5855). Neither the branch nor its connective
mutants get a scenario. Found by adversary pass 2 of retrofit wave 1
(`review-result:adversary-retrofit-w1-synthesis-pass-2`); the search code was not touched by that
unit.

## Acceptance

The test `adversary2_a_three_input_nested_guard_is_witnessed_and_its_mutants_killed`
(`crates/verify/ess-conformance/tests/adversary2_connective_and_source_mutants.rs`, currently
`#[ignore]` naming this story) passes with the `#[ignore]` removed: the branch is witnessed and its
connective mutants are killed.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` — cited
- **Files:** `crates/verify/ess-conformance/src/witness.rs` — cited, the candidate search and its limit
- **Files:** `crates/verify/ess-conformance/tests/adversary2_connective_and_source_mutants.rs:228-232` — cited, the `#[ignore]` to remove
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs:807-810` — cited, `RefusalCause::GuardUnsatisfiable` Display gives the refusal text; the "~5855" pointer in Scope above is off
- **Symbols:** `witness::MAX_CANDIDATES` (`witness.rs:116`, `= 64`), `witness::candidates` (`witness.rs:233-373`), `witness::enumerate` (`witness.rs:477-534`, mixed-radix walk `499-516`), `MAX_ENUMERATED_PER_CANDIDATE` (`witness.rs:569`) — cited
- **Likely change site:** the ladder order in `enumerate`: three fields' ladders never reach a combination that moves the last field within 64 — inferred
- **Also likely:** `synthesize.rs` callers capping on `MAX_CANDIDATES` (`:3194`, `:3382`, `:3432`, `:5641`, `:5664`, `:5754`, `:6121`) — inferred, only if the bound or signature changes
- **Also likely:** `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` — inferred, only if the constant is renamed
- **Confidence:** medium
- **Would collide with:** any unit touching `witness.rs` candidate search
