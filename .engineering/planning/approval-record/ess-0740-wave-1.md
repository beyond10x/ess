---
format: aep.planning-md/3
id: approval-record:ess-0740-wave-1
kind: approval-record
status: approved
title: 'Wave 3 of the adopter queue, released as 0.74.0: O, P, L, J'
summary: 'D102: the third adopter wave, four stories in three units, one integration branch and one pull request.'
tags:
- ess-0.74.0
- non-interactive
relations:
- decides: story:a-binding-tries-every-publisher-of-its-trigger
- decides: story:a-synthesis-seed-is-accepted-for-a-caller-model
- decides: story:a-caller-attribute-keeps-the-binding-refusal-scenario
- decides: story:a-state-only-a-set-move-reaches-is-named-in-the-refusal
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T00:21:14Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-27 (decision D102): after 0.71.0, run the adopter waves back
to back under the 0.71.0 process. That means at most four units per wave, one integration branch,
one pull request, targeted tests per unit (D83, D86), one adversary pass on the integrated branch,
and the adopter replay of the pushed head before the tag (D115), followed by the release. Approved
under the orchestrator's grant (class C: wave, bot commits, PR, merge, release).

This is the third of the three waves D102 orders, released as 0.74.0. It branches from the 0.73.0
release head (807fd59be) while that head's adopter replay runs; a fix the replay asks for lands on
0.73.0 and is merged in here. The reduced reproductions measured on that head:

| unit | story | reduced reproduction on the 0.73.0 head |
|---|---|---|
| U1 (J) | `a-binding-tries-every-publisher-of-its-trigger` | 6 / 4, and `give-on-changed/binding/mapping` fails `--target interpreted` (already so on 0.70.0 to 0.72.0) |
| U2 (P) | `a-synthesis-seed-is-accepted-for-a-caller-model` | the seed is refused, `InvalidSynthesisSeeds [caller-model]` |
| U2 (O) | `a-caller-attribute-keeps-the-binding-refusal-scenario` | 10 / 0 already (8 / 2 on 0.72.0): the 0.73.0 caller-pass format fix answers it; the unit adds the named case |
| U3 (L) | `a-state-only-a-set-move-reaches-is-named-in-the-refusal` | 2 / 1, and the refusal names neither the state nor the set move |

O and P share one caller-model fixture and run as one unit, so the wave has three trees.

Stop rule: the same stop reason twice, or a gate failing twice on one cause.
