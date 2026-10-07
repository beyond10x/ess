# ESS bundle continuation — wave 1 (2026-10-04)

AEP implementing skill version 0.19.2, wave mode. Coordinator: the Claude Code session that took over
the bundle from Codex session `01a10260` on 2026-10-04. Integration branch
`batch/ui-live-apps-complete-20261003` (managed tree `ess-w7-server-public`), head `9ccbc0beb` at
proposal time. Authoritative bundle scope stays `runbook:ui-live-apps-serial-integration-20261003`.
Delivery stays one bundle PR, one bot merge into `main`, one release.

## Selection

Selected with `aep plan artifact waves --kind story --status active --format json` (aep 0.68.0):
13 waves, 333 collisions, 19 unassessed over the 69 `active` stories. Raw output is retained in
the coordinator scratch as `waves-active.json`. Bundle candidates are the `active` stories whose
work is not yet integrated.

| unit | story | serves | issue | verb wave | scope |
|---|---|---|---|---|---|
| U1 | `story:counter-reachability-arithmetic-completeness` | vision:O2 | #413 (part A) | 1 | cited |
| U2 | `story:related-via-optional-input` | vision:O2 | #304 slice 1 | 11 | cited |
| U3 | `story:feature-request-391` | see story | #391 | 1 | cited |
| U4 | `story:feature-request-347` | see story | #347 | 1 | cited |

U1 and U2 were already dispatched before this page existed (U1 final review pass 2, U2 implementor).
The verb places U1, U3 and U4 in one wave. U2 sits in wave 11 because of its collisions with
`related-via-stored-reference` and `feature-request-229`, neither of which is in this wave.
U2 has no collision with U1, U3 or U4.

Left out on purpose:

- `story:related-via-stored-reference` (verb wave 12) and `story:related-guard-behaviour` (wave 13):
  they collide with U2 on `related_guard.rs`, `interpret/execute.rs` and `related_guard_obligation.rs`,
  and depend on it. Next waves, serially.
- `story:feature-request-229`: collides with U2 (cited, `ess-domain/src/command/related_guard.rs`)
  and U1 (inferred, `synthesize/subject_fact.rs`). After U2.
- 37 bundle issues have no `active` story: 30 `draft`, 6 `proposed` (#363, #273, #266–#269), and
  three with no story (#400, #389, #314). A wave cannot implement them until they are accepted.

## Units

| unit | worktree | build dir | scratch | stage |
|---|---|---|---|---|
| U1 | `<worktrees>/ess/wt-5794b43839af` | `<cache>/b10x-target/ess-wt-5794b43839af` (11G) | `<cache>/ess21-completion-20261003/claude-continuation/413a-review2` | adversary pass 2 running; full package green 2611/0/8 |
| U2 | `<worktrees>/ess/ess-optional-input-via-20261003` | `<cache>/b10x-target/ess-optional-input-304` | `<cache>/ess21-completion-20261003/claude-continuation/304-optional-input` | implementor running |
| U3 | `<worktrees>/ess/ess-parameterized-transport-20261003` (head `da72325cb`) | `<cache>/b10x-target/ess-transport-391` | `<cache>/ess21-completion-20261003/claude-continuation/391` | not started |
| U4 | new managed tree `ess-347-guidance-20261004` | `<cache>/b10x-target/ess-347` | `<cache>/ess21-completion-20261003/claude-continuation/347` | not started |

## Pre-flight

- Free disk on `/`: 41G. Floor 10G per the operator rule; one full `ess-conformance` package build
  measured 11G (U1).
- Primary checkout `ess`: on `main`, 47 behind `origin/main`, one untracked `.agents/skills/worktree/`
  (not ours; left alone).
- 107 non-removed managed ESS worktrees from earlier waves and sessions remain registered. The skill's
  pre-flight refuses on leftover trees; they belong to earlier Codex sessions and are not cleaned by this wave.
- Agents: N=4, `aep:implementor` and `aep:adversary` subagent types, model opus.

## Approval

The operator invoked `/aep:implementing` after the stage-1 proposal on 2026-10-04 without answering
the acceptance question for the 37 bundle issues with no `active` story. The stated default applies:
this wave carries only U1–U4, and the question is asked again when the wave closes.

## Commits this wave authorises

One bot commit per unit on the integration branch (U1–U4), the adversary-correction commits that
merge into it, one closing planning-store commit, and nothing else. Push, the bundle PR, the merge to
`main` and the release stay with the bundle's existing authorisation in the runbook, after all bundle
waves close.
