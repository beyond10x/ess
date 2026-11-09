---
format: aep.planning-md/3
id: story:a-transition-taken-by-an-affects-move-has-a-cause
kind: story
status: archived
title: 'A transition taken by another command''s `affects: moves:` has a cause'
summary: A lifecycle transition moved only as a side effect is refused as missing_causation (workbench report item 39)
tags:
- backlog
- external-request
- from-consumer-report
- not-a-defect
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:51Z", actor: "human:timo", revision: 4}
---
## Need

A transition that only another command's `affects:` entry takes is refused: `ESS-ENTITY-005
\`close\` moves \`gap.demo.Parent\` to \`Closed\`, and no command outcome takes it`
(`missing_causation`). Item 39 of the 2026-11-09 workbench report; example `39`. Item 40 of the same
report (create one row and update another in one outcome) is story:create-beside-a-move-on-the-same-key.

## Acceptance

- An `affects:` entry whose `moves:` takes a transition counts as that transition's cause.
- A transition no outcome and no `affects:` entry takes is still refused.

## Scope

- inferred: `crates/specify/ess-domain` (lifecycle causation, `ESS-ENTITY-005`).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.


## Verification on 0.92.0 (2026-11-09)

Result: **not-a-defect**. Checked against main at 4ab2ea1449 and the 0.92.0 release binary by the
zero-open-issues run (epic `epic:zero-open-issues-2026-11`); this draft leaves the plan as archived.

Proof: verify/a-transition-taken-by-an-affects-move-has-a-cause/repro/run.txt; crates/specify/ess-domain/tests/related_record_effects.rs::issue_229_the_affects_move_is_the_cause_of_its_transition

Workbench example 39 declares the `close` transition but no `affects:` entry anywhere (diff against 00-base adds only the transition), so ESS-ENTITY-005 is correct for it.
With `affects: [{entity: gap.demo.Parent, where: id == subject.parent_id, moves: gap.demo.Parent.close}]` on FinishItem.finished, 0.92.0 validates (exit 0): `set_effects::performed` (crates/specify/ess-domain/src/command/set_effects.rs:1251-1267) already counts `affects:` moves as causes, used at entity.rs:1237; landed in d790d6db9d (ess/16, 2026-09-28) with test issue_229.
Both acceptance bullets already hold. Only residue: the hint at entity.rs:1331 says "give some outcome `moves: ...`" and does not mention `affects: moves:` - wording only; could be folded into a hint story, not this one.
Recommend: close as not-a-defect (report misread).
