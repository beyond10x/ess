---
format: aep.planning-md/3
id: review-result:ess-issues-2-u7-adversary
kind: review-result
status: active
title: Adversary pass, open-issue batch 2 unit U7 (declared order and box-only recursion)
summary: nothing found
tags:
- ess-issues-2
relations:
- reviews: story:generated-fields-keep-declaration-order-and-box-only-recursion
revision: 1
---
## Scope

One adversary pass over unit U7 of open-issue batch 2 (branch `unit/ess-issues-2-u7`, change
1d82569e3: #661). Cases in `schema-contract/tests/adversary_ess_issues_2_u7.rs`, committed as
0c7f1eea1.

## Findings

None. Held: box-only recursion over a three-type cycle through required fields, recursion through
a union payload, a map and an optional list, a type reached from inside and outside its cycle, a
bundle tuple position on a cycle; the generated crate builds and round-trips each shape with both
options and the other Rust options; declared order in Rust, Go and TypeScript, under short names
and wire renames; with both options off, output and report keep their bytes.

## Result

At 0c7f1eea1 every adversary case green.
