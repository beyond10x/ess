---
format: aep.planning-md/3
id: approval-record:zero-issues-turn-1-proposal
kind: approval-record
status: draft
title: Turn 1 of the zero-open-issues plan run with no operator present
tags:
- non-interactive
relations:
- decides: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null
- decides: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
- decides: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
- decides: story:a-multi-state-wrong-state-error-branch-is-synthesized
- decides: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
- decides: story:a-swapped-caller-run-renames-only-the-names-it-binds
- decides: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- decides: story:a-badge-field-takes-a-tone
- decides: story:caller-attributes-walk-every-caller-read-of-a-command
- decides: story:metric-error-state-on-failed-read
- decides: story:a-graph-editor-draws-its-edges-and-a-menu-per-node
- decides: story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present
- decides: story:a-group-named-like-a-top-level-command-of-its-component-is-refused
- decides: story:a-required-response-member-answered-as-null-fails-exploration
- decides: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
revision: 1
---
## Stop

The stage-1 proposal of turn 1 of 32 of the zero-open-issues plan
(`.engineering/waves/zero-open-issues.md` § The turns; turn page
`.engineering/waves/zero-open-issues-t1.md`). Skill version 0.26.0 (`aep:implementing`, wave mode).

## Reason: non-interactive

The run is non-interactive: no operator turn exists until the goal is met (a Stop hook holds the
session). Operator goal, verbatim (2026-11-09): "zero remaining issues on github, all bugs fixed from
planning store", narrowed to "for repo ess". Method, verbatim: "use /planning and /wave and
/worktree:managing-worktrees - but in the end provide 1 single PR to gh. make sure all are below 1
big epic for integration via bottoken from ./atlas". On wave size (operator, 2026-11-10): "each
processing a whole wave of up to 10 issues (the more the better) - run no expensive gates i the
middle, only at the end of the wave".

The turns are the operator-side planner's reordering of the 54-wave plan (2026-11-10): the stories
that edit synthesize.rs run serially in one lane, which goes against no printed collision, because
colliding stories stay serial; every other story keeps its collisions apart and its dependencies
earlier.

## The proposal

Units, each serving `vision:O2`:

- lane (serial, one tree `zi-t1-0`, branch `lane/t1-synthesize`, in this order):
  1. `story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null` (#765)
  2. `story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for` (#729)
  3. `story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed` (#727)
  4. `story:a-multi-state-wrong-state-error-branch-is-synthesized` (#732)
  5. `story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding` (#706)
- units (parallel, one tree each, `zi-t1-1` .. `zi-t1-10`):
  `story:a-swapped-caller-run-renames-only-the-names-it-binds` (#766),
  `story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs`,
  `story:a-badge-field-takes-a-tone` (#723),
  `story:caller-attributes-walk-every-caller-read-of-a-command` (#740),
  `story:metric-error-state-on-failed-read`,
  `story:a-graph-editor-draws-its-edges-and-a-menu-per-node` (#723),
  `story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present`,
  `story:a-group-named-like-a-top-level-command-of-its-component-is-refused`,
  `story:a-required-response-member-answered-as-null-fails-exploration`,
  `story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes`.

Check path: `aep plan artifact waves --kind story --status active --format json` (aep 0.79.0),
after every story was re-scoped by `aep:story-scoper`. Cycles `[]`, unassessed 0. Its placements of
the turn's stories, verbatim:

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

The ten collisions between stories of the turn are all on
`crates/verify/ess-conformance/src/synthesize.rs` (cited), between the five lane stories, which run
serially; they are listed verbatim on the turn page. No unit collides with another unit or with the
lane.

Stories that left the turn: none. Every story is `active`, with no `depends_on` and no `blocked_by`.

Pre-flight: `df -h /` 64G available of 848G (93% used); `uptime` load 70.69, 66.20, 67.26, so at
most 3 cargo builds at once (the lane plus 2 units).

## Commits this record authorises

One commit per lane story and per unit (plus correction commits), one `--no-ff` merge of
`lane/t1-synthesize` and one per unit into `integration/ess-zero-issues`, one reconcile commit per
merge for the shared surfaces, the turn page's opening store commit, one closing store commit, and
one push of `integration/ess-zero-issues`. No tag, no release, no PR, no merge to `main`.

## What the operator would have been asked

"Run turn 1 as proposed: the five-story synthesize.rs lane and ten parallel units?"
