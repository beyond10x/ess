---
format: aep.planning-md/3
id: review-result:ess-issues-2-u5-adversary
kind: review-result
status: active
title: Adversary passes, open-issue batch 2 unit U5 (alike outcomes and route answers)
summary: 'NEEDS-CHANGE then fixed: A1-A3 fixed in 39949f233; pass 2 found nothing'
tags:
- ess-issues-2
relations:
- reviews: story:two-outcomes-answered-alike-may-share-a-status-and-body
- reviews: story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views
revision: 1
---
## Scope

Two adversary passes over unit U5 of open-issue batch 2 (branch `unit/ess-issues-2-u5`, change
64aeafb49 and ddf3c6876: #652, #673). Cases in `ess-http/tests/adversary_ess_issues_2_u5.rs` and
`ess-gen/tests/adversary_ess_issues_2_u5.rs`, pass 1 948667c46, pass 2 e3c76636c.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | an `alike` group a header sourced from `error.wire` tells apart compiled | fixed in 39949f233: refused as ESS-HTTP-025 at `routes[i].alike` |
| A2 | warning | an `alike` group of `{error: error}` bodies whose errors share a wire name but carry different fields compiled | fixed in 39949f233: refused as ESS-HTTP-025 |
| A3 | warning | for that group the one published schema did not admit the second member's body | fixed with A2; pass 2 asserts the 409 schema admits every member's answer |

Pass 2 found nothing new. Held: unknown and doubled names in groups (ESS-HTTP-022), literal empty
bodies with quotes, backslashes and non-ASCII characters, per-route refusals on views, an
`error.wire` header or challenge param beside a literal one.

Notes kept, not fixed: a single-view refusal on the same status as a bodiless `empty` is unchecked
(as for top-level refusals already); headers from `error.<field>` where one field is required and
the other Optional are not compared (as the redirect rule).

## Result

Pass 1 at 948667c46: three deliberate reds. At 39949f233 and e3c76636c: every adversary case and
the unit's lanes green.
