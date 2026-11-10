---
format: aep.planning-md/3
id: approval-record:zero-issues-turn-2-proposal
kind: approval-record
status: draft
title: Turn 2 of the zero-open-issues plan run with no operator present
tags:
- non-interactive
relations:
- decides: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie
- decides: story:an-absent-optional-under-false-and-unknown-is-arranged
- decides: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried
- decides: story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state
- decides: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row
- decides: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence
- decides: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable
- decides: story:a-declared-field-format-renders-or-is-refused
- decides: story:a-set-guard-fed-by-a-subject-payload-is-arranged
- decides: story:the-react-renderer-draws-the-declared-tokens
- decides: story:a-ui-test-declares-how-each-read-and-command-is-answered
- decides: story:an-authored-scenario-retires-the-refusal-it-covers
- decides: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
- decides: story:the-date-time-agreement-oracle-refuses-non-digits
- decides: story:the-specification-checks-command-response-wire-names
revision: 1
---
## Stop

The stage-1 proposal of turn 2 of 32 of the zero-open-issues plan
(`.engineering/waves/zero-open-issues.md` § The turns; turn page
`.engineering/waves/zero-open-issues-t2.md`). Skill version 0.26.0 (`aep:implementing`, wave mode).

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

- lane (one branch `lane/t2-synthesize`, tree `zi-t2-0`, merged in this order; stories whose
  symbols do not overlap run in their own tree branched from the lane head, per the larger-turns
  rule of `approval-record:zero-issues-larger-turns`):
  1. `story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie` (#708)
  2. `story:an-absent-optional-under-false-and-unknown-is-arranged` (#728)
  3. `story:an-arrangement-refusal-names-the-cause-of-every-creator-tried` (#728)
  4. `story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state` (#739)
  5. `story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row` (#733)
- units (parallel, one tree each, `zi-t2-1` .. `zi-t2-10`):
  `story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence` (#765),
  `story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable` (#731),
  `story:a-declared-field-format-renders-or-is-refused` (#723),
  `story:a-set-guard-fed-by-a-subject-payload-is-arranged` (#705),
  `story:the-react-renderer-draws-the-declared-tokens` (#723),
  `story:a-ui-test-declares-how-each-read-and-command-is-answered` (#722),
  `story:an-authored-scenario-retires-the-refusal-it-covers` (#735),
  `story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation`,
  `story:the-date-time-agreement-oracle-refuses-non-digits`,
  `story:the-specification-checks-command-response-wire-names`.

Check path: `aep plan artifact waves --kind story --status active --format json` (aep 0.79.0).
Every story already carried a typed Scope, so no story was re-scoped. Cycles `[]`, unassessed 0.
Its placements of the turn's stories, verbatim:

```
wave 1: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation inferred=true
wave 1: story:a-ui-test-declares-how-each-read-and-command-is-answered inferred=true
wave 1: story:an-authored-scenario-retires-the-refusal-it-covers inferred=true
wave 1: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable inferred=true
wave 1: story:the-date-time-agreement-oracle-refuses-non-digits inferred=true
wave 1: story:the-specification-checks-command-response-wire-names inferred=true
wave 2: story:a-declared-field-format-renders-or-is-refused inferred=true
wave 2: story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence inferred=true
wave 2: story:the-react-renderer-draws-the-declared-tokens inferred=true
wave 3: story:a-set-guard-fed-by-a-subject-payload-is-arranged inferred=true
wave 12: story:a-tie-breaking-sort-key-is-pinned-by-a-synthesized-tie inferred=true
wave 17: story:an-absent-optional-under-false-and-unknown-is-arranged inferred=true
wave 18: story:an-arrangement-refusal-names-the-cause-of-every-creator-tried inferred=true
wave 19: story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state inferred=false
wave 25: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row inferred=true
```

The eleven collisions between stories of the turn are all between the five lane stories, ten on
`crates/verify/ess-conformance/src/synthesize.rs` and one on
`crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (all cited); colliding lane stories
merge serially. They are listed verbatim on the turn page. No unit collides with another unit or
with the lane.

Stories that left the turn: none. Every story is `active`, with no `depends_on` and no `blocked_by`.

Pre-flight (2026-11-10, 19:04): `df -h /` 30G available of 848G (97% used); `uptime` load 117.83,
92.12, 82.53. Every cargo command runs through the run's build script on two shared target slots
with sccache, so unit trees hold no `target/`.

## Commits this record authorises

One commit per lane story and per unit (plus correction commits), one `--no-ff` merge of each
parallel lane branch into `lane/t2-synthesize`, one `--no-ff` merge of `lane/t2-synthesize` and one
per unit into `integration/ess-zero-issues`, one reconcile commit per merge for the shared surfaces,
the turn page's opening store commit, one closing store commit, and pushes of
`integration/ess-zero-issues`. No tag, no release, no PR, no merge to `main`.

## What the operator would have been asked

"Run turn 2 as proposed: the five-story synthesize.rs lane and ten parallel units?"
