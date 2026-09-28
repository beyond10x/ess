---
format: aep.planning-md/3
id: story:witness-search-beyond-64-candidates
kind: story
status: draft
title: A guard over three inputs is witnessed, not refused after 64 candidates
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 1
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
