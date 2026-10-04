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
| W2-1 | `feature-request-290` | #290 | `<worktrees>/ess/ess-w2-290-diff-breaking-20261004` | `<cache>/b10x-target/ess-w2-290-diff-breaking` | `<cache>/ess21-completion-20261003/claude-continuation/w2-290-diff-breaking` | implementor green (ess-diff 286 passed + 1 pre-existing failure; verify_diff_gate 7/7; new format ess-diff/14, opt-in flags, exit 4 on gate failure); merged `49fd82703` after one adversary pass (2 findings fixed; 6 adversary cases pass); ess-diff 294 passed + 1 pre-existing failure routed to W2-6 |
| W2-2 | `feature-request-328`, `feature-request-330` | #328, #330 | `<worktrees>/ess/ess-w2-328-330-ui-choices-20261004` | `<cache>/b10x-target/ess-w2-328-330-ui-choices` | `<cache>/ess21-completion-20261003/claude-continuation/w2-328-330-ui-choices` | implementor green (UI packages 526→555 passed; 5 failures before and after are a missing esbuild; ess-cli ui_commands 16/16); merged `5064707da` after one adversary pass (5 findings fixed); UI packages 567 passed, 5 failed only for missing esbuild; ui_commands 17/17 |
| W2-3 | `feature-request-221`, `feature-request-223` | #221, #223 | `<worktrees>/ess/ess-w2-221-223-explorer-draw-20261004` | `<cache>/b10x-target/ess-w2-221-223-explorer-draw` | `<cache>/ess21-completion-20261003/claude-continuation/w2-221-223-explorer-draw` | implementor green (explore_stored_rows 6/6, 12 existing explorer lanes unchanged); merged `9648a3d3a` after one adversary pass (4 findings fixed); 15 explorer targets 75 passed 0 failed |
| W2-4 | `feature-request-212`, `feature-request-236` | #212, #236 | `<worktrees>/ess/ess-w2-212-236-mutation-20261004` | `<cache>/b10x-target/ess-w2-212-236-mutation` | `<cache>/ess21-completion-20261003/claude-continuation/w2-212-236-mutation` | merged `cf95678b8` after one adversary pass (2 blockers, 1 warning fixed; scope by site ownership); conformance 300/0, ess-cli mutate 80/0 |
| W2-5 | `related-via-stored-reference` | #304 slice 2 | `<worktrees>/ess/ess-w2-304-stored-reference-20261004` | `<cache>/b10x-target/ess-w2-304-stored-reference` | `<cache>/ess21-completion-20261003/claude-continuation/w2-304-stored-reference` | merged `d1026d1f0` after one adversary pass (1 blocker, 2 warnings fixed); domain+compiler 1357/0, conformance 506/0, synth 21/0 |
| W2-6 | integration repair (no story) | pre-existing CI failures | `<worktrees>/ess/ess-w2-integration-repair-20261004` | `<cache>/b10x-target/ess-w2-integration-repair` | `<cache>/ess21-completion-20261003/claude-continuation/w2-integration-repair` | merged `a16561d9f` (63 pre-existing failures fixed or shown machine-only; ~280 Rust 1.99 lints; stable and 1.98.1 clippy clean) |
| W2-7 | `feature-request-354` | #354 | `<worktrees>/ess/ess-w2-354-ui-live-composites-20261004` | `<cache>/b10x-target/ess-w2-354-ui-live-composites` | `<cache>/ess21-completion-20261003/claude-continuation/w2-354-ui-live-composites` | implementor green; adversary pass 1 red (2 blockers, 2 warnings); back with the implementor |
| W2-8 | `feature-request-222` | #222 | `<worktrees>/ess/ess-w2-222-authored-outcome-check-20261004` | `<cache>/b10x-target/ess-w2-222-authored-outcome-check` | `<cache>/ess21-completion-20261003/claude-continuation/w2-222-authored-outcome-check` | implementor green; adversary pass 1 red (1 blocker: now-guards decided at a fixed instant); back with the implementor |

Concurrent with wave-1 U2 (adversary) and U3 (#391 implementor): 6 agents.

## Pre-flight

Free disk 25G against a 10G floor; one full `ess-conformance` test build measured 11–12G. Units therefore build only
named test targets and clean their build dir when done; the coordinator runs full packages at merge.

## Commits this wave authorises

One bot commit per unit on the integration branch, review-correction commits, and one closing planning-store commit.
Push, bundle PR, merge to `main` and release stay with the bundle's existing authorisation.

## Wave 3 units (dispatched from integration head `d1026d1f0`)

| unit | stories | issues | worktree | build dir | scratch | stage |
|---|---|---|---|---|---|---|
| W3-1 | `related-guard-behaviour` | #319 | `<worktrees>/ess/ess-w3-319-related-guard-behaviour-20261004` | `<cache>/b10x-target/ess-w3-319-related-guard-behaviour` | `<cache>/ess21-completion-20261003/claude-continuation/w3-319-related-guard-behaviour` | implementor running |
| W3-2 | `feature-request-283` | #283 | `<worktrees>/ess/ess-w3-283-multiple-related-vias-20261004` | `<cache>/b10x-target/ess-w3-283-multiple-related-vias` | `<cache>/ess21-completion-20261003/claude-continuation/w3-283-multiple-related-vias` | implementor running |
| W3-3 | `feature-request-266`, `feature-request-267` | #266, #267 | `<worktrees>/ess/ess-w3-266-267-binding-arrangement-20261004` | `<cache>/b10x-target/ess-w3-266-267-binding-arrangement` | `<cache>/ess21-completion-20261003/claude-continuation/w3-266-267-binding-arrangement` | implementor running |
| W3-4 | `feature-request-297` | #297 | `<worktrees>/ess/ess-w3-297-process-restart-20261004` | `<cache>/b10x-target/ess-w3-297-process-restart` | `<cache>/ess21-completion-20261003/claude-continuation/w3-297-process-restart` | implementor running |

Wave 1 U3 (#391) merged `c6017982e` after one adversary pass (1 blocker, 3 warnings fixed; F6 ignored network controls left as CI-invoked).

Machine finding: empty directories `~/.git` and `~/.cache/.git` (created 2026-10-03 01:03 by an unknown process) made every Go build under the home directory fail with "error obtaining VCS status"; the coordinator removed both empty directories on 2026-10-04.
