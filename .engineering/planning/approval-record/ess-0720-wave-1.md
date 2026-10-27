---
format: aep.planning-md/3
id: approval-record:ess-0720-wave-1
kind: approval-record
status: approved
title: 'Wave 1 of 0.72.0: D, G, F, B'
summary: 'D102: the first adopter wave after 0.71.0, four units, one integration branch and one pull request.'
tags:
- ess-0.72.0
- non-interactive
relations:
- decides: story:a-set-effect-bound-to-a-fresh-input-composes-after-a-disclosure
- decides: story:a-binding-into-an-updating-command-follows-its-related-guards
- decides: story:a-binding-arranges-the-row-a-set-guard-selects
- decides: story:a-binding-whose-trigger-shares-the-row-guard-flows
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-27T14:44:57Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D102, queue row 44): after 0.71.0, run the
adopter waves back to back under the 0.71.0 process. That means at most four units per wave, one
integration branch, one pull request, targeted tests per unit (D85, D86), one adversary pass on the
integrated branch, and the adopter replay before each tag, followed by the release. Approved under
the orchestrator's grant (class C: wave, bot commits, PR, merge, release).

This is the first of the three waves D102 orders, released as 0.72.0:

| unit | story | reduced reproduction on 0.71.0 (scenarios / refusals) |
|---|---|---|
| U1 (D) | `a-set-effect-bound-to-a-fresh-input-composes-after-a-disclosure` | 9 / 1; twin with `instance:` 11 / 0 |
| U2 (G) | `a-binding-into-an-updating-command-follows-its-related-guards` | 8 / 4; twin without the guards 11 / 0 |
| U3 (F) | `a-binding-arranges-the-row-a-set-guard-selects` | 5 / 3 |
| U4 (B) | `a-binding-whose-trigger-shares-the-row-guard-flows` | 6 / 3 (one refusal stays, by its acceptance) |

U2, U3 and U4 all touch binding synthesis (`synthesize/binding_effects.rs` and its neighbours). They
run in parallel in their own trees, and conflicts are resolved on the integration branch. Builds are
serialized across the trees, with one ESS tree building at a time.

Stop rule: the same stop reason twice, or a gate failing twice on one cause.
