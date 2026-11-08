---
format: aep.planning-md/3
id: review-result:ess-0900-u4-literals-adversary
kind: review-result
status: active
title: 'ESS 0.90.0 U4 several-literal guard adversary: three pre-existing cases drafted'
relations:
- reviews: story:a-guard-over-several-list-literals-is-witnessed
revision: 1
---
unit: ESS 0.90.0 U4, a guard over several list literals is witnessed (commit 383ee5335fa), adversary tests 88075230d8b
verdict: unit merged; three pre-existing cases moved to a draft
origin: introduced 0 / pre-existing 3
fixes: none needed in the unit; cases moved out in fdc653cf2a0, merged 05dc7974e14

Findings:

1. Pre-existing: a guard over two lists is not witnessed (ESS-SYNTH-003, ESS-SYNTH-022 on base d91b3a039c2 without U4).
2. Pre-existing: a leaf beside the literals that its base value refutes is not witnessed (ESS-SYNTH-003, ESS-SYNTH-022 on base).
3. Pre-existing: the negated child of a list guard is never tested; the mutant dropping it passes all 10 scenarios on base. Named under Known in the 0.90.0 CHANGELOG.

All three are story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation, ranked first for 0.91.0.

Evidence: PR https://github.com/beyond10x/ess/pull/763 CI run 42328951670 green at 18e91e142b2; squash commit 9a2468ef48e on main; tag 0.90.0.
