---
format: aep.planning-md/3
id: approval-record:ess-0750-wave-1
kind: approval-record
status: approved
title: 'Wave of 0.75.0: gap R, gap S and two adversary follow-ups'
summary: 'D126: four units, one integration branch and one pull request, released as 0.75.0.'
tags:
- ess-0.75.0
- non-interactive
relations:
- decides: story:a-seed-says-it-does-not-serve-a-set-move-filter
- decides: story:the-workspace-compiles-on-its-declared-rust-version
- decides: story:every-set-move-only-refusal-names-the-state
- decides: story:a-seeded-scenario-notes-its-skipped-reversed-caller-run
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T08:41:52Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-28 (decision D126): "open the 0.75.0 wave from
integrate/ess-0750: gap R story (seed refusal/help names the set-move filter limit), gap S story
(workspace compiles on declared rust-version + an MSRV CI job), and the two adversary follow-up
drafts. Wave cap 4 units". Approved under the orchestrator's grant (class C: wave, bot commits, PR,
merge, release).

## Process

The 0.74.0 process applies unchanged:
- one integration branch and one pull request;
- targeted tests per unit (D83, D86), with every cargo call through the guarded wrapper, one tree
  building at a time (D96 flags, D108);
- one adversary pass on the integrated branch;
- the adopter replay of the pushed head before the tag;
- tag at green CI, then verify-release.

If the adopter session is still unreachable at tag time, the orchestrator decides.

Stop rule: stop on the same stop reason twice, on a red base at open, or on a gate failing twice
on one cause.

## Units, measured on the 0.74.0 release

| unit | story | base |
|---|---|---|
| U1 (R) | `a-seed-says-it-does-not-serve-a-set-move-filter` | Released binary on the reduced reproduction with one seed row: 2 selected, 1 refused. The output never mentions the seed. |
| U2 (S) | `the-workspace-compiles-on-its-declared-rust-version` | A downstream MSRV 1.85 job fails to compile `ess-conformance` with E0716. ESS CI has no MSRV job. |
| U3 | `every-set-move-only-refusal-names-the-state` | Inferred from the code. The unit first builds a case that shows the old text. |
| U4 | `a-seeded-scenario-notes-its-skipped-reversed-caller-run` | Open question. The unit first decides whether a run is skipped. |

U1 and U3 both change refusal texts in `synthesize/set_effects.rs`, so they merge one after the
other.
