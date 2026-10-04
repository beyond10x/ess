# ESS bundle continuation — wave 2 (2026-10-04)

AEP implementing skill version 0.19.2, wave mode. Same coordinator, integration branch and delivery as wave 1
(`ess-bundle-continuation-wave-1.md`). Integration head at dispatch: `5fc5f623c`.

## Authority

On 2026-10-04 the operator asked how many bundle issues remain and said to use up to 6 parallel agents. The
coordinator read that as the go-ahead for option A of the wave-1 decision: accept the remaining bundle stories,
which the operator had already approved as scope in `runbook:ui-live-apps-serial-integration-20261003`, and run them
in waves of at most 6 concurrent agents. The coordinator moved the seven wave-2 stories `draft → proposed → active`
on that basis. This is a coordinator decision, not a recorded operator acceptance of each story.

## Remaining bundle issues at dispatch

Of the 50: 9 integrated and waiting for final checks (#282, #292, #293, #307, #312, #314, #318, #347, #360);
2 in flight from wave 1 (#304 slice 1 in review, #391); 1 deferred by decision (#197); 38 with work left.

## Units

Selection by reading crate surfaces pairwise, because the seven stories carried no typed scope entries
(`aep plan artifact waves` reported them unassessed). Expected overlap: `crates/edge/ess-cli` (W2-1 and W2-4 both
add CLI surface); resolved at merge with `git merge-tree` before the second lands.

| unit | stories | issues | worktree | build dir | scratch | stage |
|---|---|---|---|---|---|---|
| W2-1 | `feature-request-290` | #290 | `<worktrees>/ess/ess-w2-290-diff-breaking-20261004` | `<cache>/b10x-target/ess-w2-290-diff-breaking` | `<cache>/ess21-completion-20261003/claude-continuation/w2-290-diff-breaking` | implementor running |
| W2-2 | `feature-request-328`, `feature-request-330` | #328, #330 | `<worktrees>/ess/ess-w2-328-330-ui-choices-20261004` | `<cache>/b10x-target/ess-w2-328-330-ui-choices` | `<cache>/ess21-completion-20261003/claude-continuation/w2-328-330-ui-choices` | implementor running |
| W2-3 | `feature-request-221`, `feature-request-223` | #221, #223 | `<worktrees>/ess/ess-w2-221-223-explorer-draw-20261004` | `<cache>/b10x-target/ess-w2-221-223-explorer-draw` | `<cache>/ess21-completion-20261003/claude-continuation/w2-221-223-explorer-draw` | implementor running |
| W2-4 | `feature-request-212`, `feature-request-236` | #212, #236 | `<worktrees>/ess/ess-w2-212-236-mutation-20261004` | `<cache>/b10x-target/ess-w2-212-236-mutation` | `<cache>/ess21-completion-20261003/claude-continuation/w2-212-236-mutation` | implementor running |

Concurrent with wave-1 U2 (adversary) and U3 (#391 implementor): 6 agents.

## Pre-flight

Free disk 25G against a 10G floor; one full `ess-conformance` test build measured 11–12G. Units therefore build only
named test targets and clean their build dir when done; the coordinator runs full packages at merge.

## Commits this wave authorises

One bot commit per unit on the integration branch, review-correction commits, and one closing planning-store commit.
Push, bundle PR, merge to `main` and release stay with the bundle's existing authorisation.
