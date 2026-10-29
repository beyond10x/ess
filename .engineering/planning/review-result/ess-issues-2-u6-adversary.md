---
format: aep.planning-md/3
id: review-result:ess-issues-2-u6-adversary
kind: review-result
status: active
title: Adversary pass, open-issue batch 2 unit U6 (open objects and integer widths)
summary: 'NEEDS-CHANGE then fixed: A1 in da0c1f65d, A2 in b198d8439'
tags:
- ess-issues-2
relations:
- reviews: story:openapi-import-admits-open-objects-as-json
- reviews: story:an-imported-integer-format-realizes-at-its-native-width
revision: 1
---
## Scope

One adversary pass over unit U6 of open-issue batch 2 (branch `unit/ess-issues-2-u6`, change
bd3e268fd and efd0800de: #647, #648). Cases in `ess-openapi/tests/adversary_ess_issues_2_u6.rs`
and `schema-contract/tests/adversary_ess_issues_2_u6.rs`, committed as 06385217c.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | `a_one_of_whose_ref_member_admits_null_stays_refused` | fixed in da0c1f65d: the nullable-pair check follows local references, so a `$ref` member that admits null keeps the `oneOf` refused |
| A2 | warning | `the_typescript_report_does_not_claim_a_decoder_asserts_the_format_range` | fixed in b198d8439: the TypeScript report leaves the format range to the consumer (`typescript_integer_range`) |

Held: every envelope 0.76.0 wrote for five accepted documents is rewritten byte for byte; the
nullable-pair edges; the `additionalProperties` matrix; without the option every target writes the
0.76.0 bytes; the Go decoder holds both edges of each width.

## Result

At 06385217c two deliberate reds; at b198d8439 every adversary case and the unit's lanes green.
