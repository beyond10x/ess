---
format: aep.planning-md/3
id: approval-record:ess-0730-wave-1
kind: approval-record
status: approved
title: 'Wave 2 of the adopter queue, released as 0.73.0: H, C, I, N2'
summary: 'D102: the second adopter wave, four units, one integration branch and one pull request.'
tags:
- ess-0.73.0
- non-interactive
relations:
- decides: story:a-set-effect-binding-refusal-names-each-cause
- decides: story:a-binding-no-step-triggers-leaves-a-row-set-arrangeable
- decides: story:a-predicate-refusal-policy-is-forced-through-an-arranged-row
- decides: story:affects-witness-with-the-adopters-remaining-condition
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-27T22:48:05Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D102): after 0.71.0, run the adopter waves back
to back under the 0.71.0 process. That means at most four units per wave, one integration branch,
one pull request, targeted tests per unit (D83, D86), one adversary pass on the integrated branch,
and the adopter replay of the pushed head before the tag (D115), followed by the release. Approved
under the orchestrator's grant (class C: wave, bot commits, PR, merge, release).

This is the second of the three waves D102 orders, released as 0.73.0. Each reduced reproduction
still reproduces on the released 0.72.0 binary:

| unit | story | reduced reproduction on 0.72.0 (scenarios / refusals) |
|---|---|---|
| U1 (H) | `a-set-effect-binding-refusal-names-each-cause` | 7 / 3; twin with all four causes removed 9 / 0 |
| U2 (C) | `a-binding-no-step-triggers-leaves-a-row-set-arrangeable` | 7 / 2; without the binding 5 / 0 |
| U3 (I) | `a-predicate-refusal-policy-is-forced-through-an-arranged-row` | 9 / 1; with `external:` 10 / 0 |
| U4 (N2) | `affects-witness-with-the-adopters-remaining-condition` | 4 / 1; without the predicate branch 4 / 0 |

U1 and U3 touch binding synthesis (`synthesize/binding_effects.rs`, `refusal_policy`), U2 touches
`synthesize/row_set.rs` and U4 the `affects:` arrangement. They run in parallel in their own trees,
and conflicts are resolved on the integration branch. Builds are serialized across the trees, with
one ESS tree building at a time (D108).

Stop rule: the same stop reason twice, or a gate failing twice on one cause.
