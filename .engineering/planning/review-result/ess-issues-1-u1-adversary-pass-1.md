---
format: aep.planning-md/3
id: review-result:ess-issues-1-u1-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, open-issue batch 1 unit U1 (schema keywords for invariants)
summary: 'NEEDS-CHANGE then fixed: A1, A2 (soundness) and A3 fixed in 1eb389dcb'
tags:
- ess-issues-1
relations:
- reviews: story:an-entity-invariant-is-projected-into-its-schema
- reviews: story:a-binary64-bound-is-projected-as-a-numeric-keyword
- reviews: story:a-text-length-invariant-is-projected-as-min-and-max-length
- reviews: story:a-distinct-invariant-is-projected-into-its-schema
revision: 1
---
## Scope

One adversary pass over unit U1 of open-issue batch 1 (branch `unit/ess-issues-1-u1`, change
568a319a03..096f914eb: #688, #687, #685, #686). Adversary cases committed as 053d06222 in
`ess-gen/tests/adversary_ess_issues_1_u1.rs` and `schema-contract/tests/adversary_ess_issues_1_u1.rs`.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | `a_distinct_list_of_newtype_wrapped_optionals_admits_two_nulls` | fixed in 1eb389dcb: a list whose element may be null, through a newtype too, is checked only while it holds fewer than two nulls |
| A2 | warning | `a_binary64_bound_past_two_to_the_53_refuses_no_payload_the_model_admits` | fixed in 1eb389dcb: a bound of magnitude 2^53 or more publishes the integer at the edge of what rounds into range |
| A3 | note | `the_mapping_form_length_is_reported_as_the_pair_is` | fixed in 1eb389dcb: `TextConstraint::read` reads the conjuncts of the `{gte, lte}` mapping form |

A1 and A2 were soundness defects (the schema refused a value the model admits). Held: strict
operators at both ends of i64, contradictory pairs, binary64 neighbours, scalar-value lengths,
`x-ess-invariants` only on entities, agreement of the OpenAPI copy.

## Result

At 053d06222: ess-gen adversary 3 passed, 2 failed; schema-contract adversary 0 passed, 1 failed.
At 1eb389dcb: adversary 5 and 1 passed; ess-gen targeted lanes and `xtask generate --check` green.
