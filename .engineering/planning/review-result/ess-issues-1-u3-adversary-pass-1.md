---
format: aep.planning-md/3
id: review-result:ess-issues-1-u3-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, open-issue batch 1 unit U3 (ess-composition/5 ends and constraints)
summary: 'NEEDS-CHANGE then fixed: F1-F6 fixed in 88829d592, tables re-pinned in 252792111'
tags:
- ess-issues-1
relations:
- reviews: story:a-composition-conformance-may-name-an-event-or-command
- reviews: story:a-type-conformance-compares-value-constraints-and-variant-attributes
- reviews: story:reference-outside-component-names-its-remedy
- reviews: story:role-input-drift-tells-a-caller-from-an-implementation
revision: 1
---
## Scope

One adversary pass over unit U3 of open-issue batch 1 (branch `unit/ess-issues-1-u3`, change
568a319a03..5f3a635d1: #664, #665, #666, #671). Adversary cases committed as 1602e08c5 in
`ess-composition/tests/adversary_ess_issues_1_u3.rs`.

## Findings

| id | test area | outcome |
|---|---|---|
| F1 | an unknown conformance-end key | fixed in 88829d592: reads as before under every format; the `/5` clause stays on the four new keys |
| F2 | a missing `service` or `type` | fixed in 88829d592: serde's missing-field message at the end's own path and line |
| F3 | invariant order and grouping | fixed in 88829d592: invariants compare as sets of conditions split at `all:` |
| F4 | alphabet order | fixed in 88829d592: an alphabet compares as a set of characters |
| F5 | remedy for a type outside the component | fixed in 88829d592: names the components that export it, including through `accepts` |
| F6 | remedy under `ess-composition/1` | fixed in 88829d592: offers no `owns` edit |

The four model tables were re-pinned for the new fixture directories in 252792111; no existing row
moved.

## Result

At 1602e08c5: 9 adversary tests red. At 88829d592: adversary 12/12, every ess-composition target and
ess-cli `compose_conformance_v5`, `compose_roles` pass.
