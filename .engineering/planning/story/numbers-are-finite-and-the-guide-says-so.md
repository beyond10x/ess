---
format: aep.planning-md/3
id: story:numbers-are-finite-and-the-guide-says-so
kind: story
status: active
title: Decimal and Binary64 are finite, and the guide says so
summary: State that no non-finite value reaches the domain and where its refusal belongs
tags:
- ess-0.69.0
refs:
- provider: github
  reference: beyond10x/ess#676
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T19:01:19Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T19:01:20Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
---
## Problem

An adopter tried to write NaN and infinite `Decimal` inputs in scenarios to reach a `non-finite` refusal (ess#676). Neither `Decimal` nor `Binary64` has non-finite values (`fields-and-invariants.md:317`, `docs/design/model-binary64.md:17`), but the guide never says so, so the adopter modelled an unreachable outcome with a stand-in value.

## Acceptance

- `the_guide_states_numbers_are_finite`: `website/docs/guides/specify/fields-and-invariants.md` states that `Decimal` and `Binary64` hold finite values only, that NaN and infinities never reach the domain, and that their refusal belongs to the decoder or the binding's `malformed` refusal.
- `a_non_finite_scenario_literal_is_refused_with_the_reason`: `ESS-AUTHOR-015` for `.nan`/`.inf` on a numeric input says the type has no non-finite values instead of "found null", if the parser can tell them apart; otherwise the unit records why not.
