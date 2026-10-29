---
format: aep.planning-md/3
id: review-result:ess-issues-1-u2-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, open-issue batch 1 unit U2 (presence guards and the identity hint)
summary: 'NEEDS-CHANGE then fixed: A1, A2 and A3 fixed in e6fcee57f'
tags:
- ess-issues-1
relations:
- reviews: story:when-subject-reads-the-addressed-identity
- reviews: story:defined-and-not-defined-branches-are-exhaustive
revision: 1
---
## Scope

One adversary pass over unit U2 of open-issue batch 1 (branch `unit/ess-issues-1-u2`, change
568a319a03..a1c56d78c: #643, #644). Adversary cases committed as f48900a78 in
`ess-domain/tests/adversary_ess_issues_1_u2.rs` and `ess-conformance/tests/adversary_ess_issues_1_u2.rs`.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | `the_identity_hint_names_the_input_that_addresses_the_row` | fixed in e6fcee57f: the hint names the input the outcome's `instance:` gives |
| A2 | warning | `an_absent_optional_reference_answered_by_not_defined_validates` | fixed in e6fcee57f: `validate_absent` skips cases where the `via` input is present |
| A3 | note | `an_open_guard_beside_an_identity_read_keeps_its_coverage_refusal` | fixed in e6fcee57f: the coverage proof runs over the branches that read no unstored root, so an open guard keeps its own ESS-COMMAND-005 |

None was a soundness hole. Held: no command validated with a reachable case unanswered across 20
shapes; synthesis and the interpreted target agree with the proof for every shape tried.

## Result

At f48900a78: ess-domain adversary 12 passed, 3 failed. At e6fcee57f: 15 passed; ess-conformance
adversary 8, subject_guard_input 13, negated_defined_beside_when_subject 6, when_subject_witness 11,
finite_enum 37 passed.
