# zero-open-issues: turn 1

Turn 1 of 32 of the wave plan [`zero-open-issues.md`](zero-open-issues.md) (§ The turns), for
`epic:zero-open-issues-2026-11`. Skill version 0.26.0 (`aep:implementing`, wave mode; `aep
0.79.0`). Run non-interactive: no operator turn exists until the goal is met, so the stage-1 stop is
`approval-record:zero-issues-turn-1-proposal` (tagged `non-interactive`).

The turns are the operator-side planner's reordering of the 54-wave plan (2026-11-10): the stories
that edit synthesize.rs run serially in one lane, which goes against no printed collision, because
colliding stories stay serial; every other story keeps its collisions apart and its dependencies
earlier. The plan-level record is `approval-record:zero-issues-turn-plan`.

Status: running

## Units

Every unit serves `vision:O2`. Scope: every story was re-scoped by `aep:story-scoper` (opus) on the
integration tree at `e4349e157d` before this page; its `## Scope` section and typed entries were
rewritten from that. "inferred" below means the verb printed `inferred: true` (at least one typed
entry is inferred); "cited" means every entry is cited.

| role | story | issue | scope |
|---|---|---|---|
| lane 1 | `story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null` | #765 | inferred |
| lane 2 | `story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for` | #729 | inferred |
| lane 3 | `story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed` | #727 | inferred |
| lane 4 | `story:a-multi-state-wrong-state-error-branch-is-synthesized` | #732 | inferred |
| lane 5 | `story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding` | #706 | inferred |
| unit 1 | `story:a-swapped-caller-run-renames-only-the-names-it-binds` | #766 | inferred |
| unit 2 | `story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs` | none (format story) | inferred |
| unit 3 | `story:a-badge-field-takes-a-tone` | #723 | inferred |
| unit 4 | `story:caller-attributes-walk-every-caller-read-of-a-command` | #740 | cited |
| unit 5 | `story:metric-error-state-on-failed-read` | none (store defect) | inferred |
| unit 6 | `story:a-graph-editor-draws-its-edges-and-a-menu-per-node` | #723 | inferred |
| unit 7 | `story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present` | none (store defect) | inferred |
| unit 8 | `story:a-group-named-like-a-top-level-command-of-its-component-is-refused` | none (store defect) | inferred |
| unit 9 | `story:a-required-response-member-answered-as-null-fails-exploration` | none (store defect) | cited |
| unit 10 | `story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes` | none (store defect) | cited |

Scope decisions the coordinator took to keep the turn disjoint (each written into the story's
`## Scope`): `story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for` leaves out
`docs/design/generic-language-features.md` (only a stale code reference would change, no acceptance
asks for it; unit 7 edits that page); `story:a-swapped-caller-run-renames-only-the-names-it-binds`
writes its typed walk in `caller.rs`, not in `scenario.rs` (unit 2 edits `scenario.rs`);
`story:a-graph-editor-draws-its-edges-and-a-menu-per-node` keeps its styling inline and does not edit
`styles.css.tmpl`; unit 2 moves the Go and TypeScript runtime caps with the admission table.

## The verb's lines for the turn

Check path: `aep plan artifact waves --kind story --status active --format json` (the verb, not a
pairwise reading). Cycles: `[]`. Unassessed: 0. Placements of the turn's stories, verbatim:

```
wave 1: story:a-badge-field-takes-a-tone inferred=true
wave 1: story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present inferred=true
wave 1: story:a-group-named-like-a-top-level-command-of-its-component-is-refused inferred=true
wave 1: story:a-required-response-member-answered-as-null-fails-exploration inferred=false
wave 1: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes inferred=false
wave 1: story:a-swapped-caller-run-renames-only-the-names-it-binds inferred=true
wave 1: story:caller-attributes-walk-every-caller-read-of-a-command inferred=false
wave 1: story:metric-error-state-on-failed-read inferred=true
wave 2: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for inferred=true
wave 3: story:a-graph-editor-draws-its-edges-and-a-menu-per-node inferred=true
wave 4: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed inferred=true
wave 4: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs inferred=true
wave 7: story:a-multi-state-wrong-state-error-branch-is-synthesized inferred=true
wave 11: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding inferred=true
wave 18: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null inferred=true
```

Every collision between two stories of the turn, verbatim; all ten are inside the lane, which runs
them serially:

```
story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for <> story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for <> story:a-multi-state-wrong-state-error-branch-is-synthesized on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for <> story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for <> story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed <> story:a-multi-state-wrong-state-error-branch-is-synthesized on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed <> story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed <> story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-multi-state-wrong-state-error-branch-is-synthesized <> story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-multi-state-wrong-state-error-branch-is-synthesized <> story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null on crates/verify/ess-conformance/src/synthesize.rs (cited)
story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding <> story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null on crates/verify/ess-conformance/src/synthesize.rs (cited)
```

No unit collides with another unit or with the lane. Every story of the turn is `active`, carries
no `depends_on` and no `blocked_by`.

## Stories that left the turn

None at stage 1.

## On the way out

Each story moves `active -> implemented` (`aep plan artifact lifecycle story`) after a
`verification` record (baseline vs treatment) and a `test_result` record against its merge commit.

## Pre-flight (2026-11-10, 02:12 UTC+1)

- `df -h /`: 848G size, 776G used, 64G available (93%).
- `uptime`: load average 70.69, 66.20, 67.26. Above 60, so at most 3 cargo builds at once across
  the turn: the lane plus at most 2 units.
- Two other managed trees of this repository belong to other sessions (`wt-4c2510e0c91f`,
  `wt-99ed18828730`, live leases); this turn does not touch them.
- Integration head at stage 1: `e4349e157d`, equal to `origin/integration/ess-zero-issues`.

## Agents

`aep:story-scoper` (opus) per story before this page; `aep:implementor` (sonnet; a unit failed
twice is redone once on opus), one for the lane and one per unit; `aep:adversary` (opus) per lane
story and per unit; `aep:security-reviewer` (opus) where a unit touches a guard, an authorization
check, an HTTP boundary or input parsing.

## Commits this turn authorises

One commit per lane story and per unit (plus correction commits), one `--no-ff` merge of
`lane/t1-synthesize` and one per unit into `integration/ess-zero-issues`, one reconcile commit per
merge for the shared surfaces, this page's opening store commit, one closing store commit, and one
push of `integration/ess-zero-issues`. No tag, no release, no PR, no merge to `main`.

## Trees

Each tree builds into its own `target/` inside the tree; scratch is the turn's scratch directory
`t1/<slug>/` under the run's scratch root.

| id | branch | story | stage | head |
|---|---|---|---|---|
| `zi-t1-0` | `lane/t1-synthesize` | lane 1-5 | planned | |
| `zi-t1-1` | `impl/a-swapped-caller-run-renames-only-the-names-it-binds` | unit 1 | planned | |
| `zi-t1-2` | `impl/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs` | unit 2 | planned | |
| `zi-t1-3` | `impl/a-badge-field-takes-a-tone` | unit 3 | planned | |
| `zi-t1-4` | `impl/caller-attributes-walk-every-caller-read-of-a-command` | unit 4 | planned | |
| `zi-t1-5` | `impl/metric-error-state-on-failed-read` | unit 5 | planned | |
| `zi-t1-6` | `impl/a-graph-editor-draws-its-edges-and-a-menu-per-node` | unit 6 | planned | |
| `zi-t1-7` | `impl/a-create-or-update-writer-is-no-proof-a-stored-reference-is-present` | unit 7 | planned | |
| `zi-t1-8` | `impl/a-group-named-like-a-top-level-command-of-its-component-is-refused` | unit 8 | planned | |
| `zi-t1-9` | `impl/a-required-response-member-answered-as-null-fails-exploration` | unit 9 | planned | |
| `zi-t1-10` | `impl/a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes` | unit 10 | planned | |

## Backlog drafts filed

None yet.
