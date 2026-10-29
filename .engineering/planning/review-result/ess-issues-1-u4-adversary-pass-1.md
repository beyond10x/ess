---
format: aep.planning-md/3
id: review-result:ess-issues-1-u4-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, open-issue batch 1 unit U4 (HTTP operation ids and servers)
summary: 'NEEDS-CHANGE then fixed: A1 fixed in d2c0422424; A2 (IPv6 bracket rule) accepted and fixed there'
tags:
- ess-issues-1
relations:
- reviews: story:an-http-binding-names-operation-ids-and-servers
- reviews: story:a-component-not-reached-over-the-network-has-no-http-route
revision: 1
---
## Scope

One adversary pass over unit U4 of open-issue batch 1 (branch `unit/ess-issues-1-u4`, change
568a319a03..9d1798c6c3: #653, #651). Adversary cases committed with the fix in d2c0422424 in
`ess-http/tests/adversary_ess_issues_1_u4.rs` (13 cases) and `ess-gen/tests/adversary_ess_issues_1_u4.rs` (4 cases).

## Findings

| id | severity | test area | outcome |
|---|---|---|---|
| A1 | warning | a server URL with a port but no host (`https://:8443/v2`) admitted by compile and by `HttpIr::from_json` | fixed in d2c0422424: the host is taken before any `:port`, bracket-aware, and refused when empty |
| A2 | note | an IPv6 literal whose `[` is not closed before the port or path | accepted as an extra rule and fixed in d2c0422424: refused |

## Result

At 9d1798c6c3: 2 adversary cases red. At d2c0422424: the ess-http and ess-gen adversary suites pass;
ess-http `compile`, `presentation` and ess-gen `openapi_http_presentation`, `openapi_reach` pass.
