---
format: aep.planning-md/3
id: review-result:ess-issues-2-u8-adversary
kind: review-result
status: active
title: Adversary pass, open-issue batch 2 unit U8 (input and response fields of one name)
summary: 'NEEDS-CHANGE then fixed: A1, A2 in 499cab2f7'
tags:
- ess-issues-2
relations:
- reviews: story:a-command-input-and-response-field-may-share-a-name-in-rust
revision: 1
---
## Scope

One adversary pass over unit U8 of open-issue batch 2 (branch `unit/ess-issues-2-u8`, change
b45fdb8eb: #659). Cases in `ess-synth/tests/adversary_ess_issues_2_u8.rs`, committed as b82311a55.

## Findings

| id | severity | test | outcome |
|---|---|---|---|
| A1 | warning | a declaration named `demo.core.Send.response` shared the response field scope by spelling | fixed in 499cab2f7: the response scope is keyed `response fields:{command}`, a key no declaration name can spell |
| A2 | warning | two response fields with one wire name reached the encoder, which wrote the JSON member twice | fixed in 499cab2f7: refused as a `wire-collision`, as input fields already were |

Held: input and response sharing struct, List, Optional, keyword and case-mangled names for Rust
and wasm32 web; a type spelled `{Command}Response` keeps the `naming: { code: … }` remedy.

## Result

At b82311a55 two deliberate reds; at 499cab2f7 every adversary case and the unit's lanes green.
