---
format: aep.planning-md/3
id: review-result:ess-issues-2-u9-adversary
kind: review-result
status: active
title: Adversary passes, open-issue batch 2 unit U9 (precedence-swap equivalence)
summary: 'NEEDS-CHANGE then fixed: A1 in 8aa8d7839; pass 2 found nothing'
tags:
- ess-issues-2
relations:
- reviews: story:precedence-swap-of-disjoint-guards-is-equivalent
revision: 1
---
## Scope

Two adversary passes over unit U9 of open-issue batch 2 (branch `unit/ess-issues-2-u9`, change
552bb2b20, 8aa8d7839 and bf9bbc1bc: #703). Cases in
`ess-conformance/tests/adversary_ess_issues_2_u9.rs`, pass 1 eeb11b55b, pass 2 a18f4e538.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | `an_enum_of_seventy_variants_pinned_to_two_values_is_equivalent` | fixed in 8aa8d7839: an enum compared only by equality or membership is decided over the variants its guard names plus one stand-in |
| A2 | note | `read_through` branch at `mutate.rs` keeping every variant is unreachable, since attribute reads are lowered to membership first | kept; harmless |

Pass 2 found no unsound `equivalent`: for every swap so scored, the interpreter ran the original
and the swapped model on every variant (and absence for an Optional enum) and no input told them
apart. Held: `!=`, `not in`, `any:` and `not` with membership, Optional enums, two enum inputs of
one type, an earlier refusal taking part of the overlap, accepting pairs, the ess-diff
declared-values reading, and the guide text.

## Result

Pass 1 at eeb11b55b: one red. Pass 2 at a18f4e538: 16 adversary cases green, with
`precedence_swap_disjoint_guards` 14, `mutation_arms` 14, ess-diff `adversary_514_pass1` 8 and
`narrowed_acceptance` 12.
