---
format: aep.planning-md/3
id: review-result:one-selection-plan-scope-round-1
kind: review-result
status: active
title: 'Plan critic (scope), round 1: one selection plan'
relations:
- reviews: epic:one-selection-plan
- reviews: story:selection-plan-design-and-type
- reviews: story:interpreter-reads-selection-plan
- reviews: story:entity-runtime-lowering-reads-selection-plan
- reviews: story:generated-behaviour-reads-selection-plan
- reviews: story:synthesis-reads-selection-plan
- reviews: story:overlap-witnesses-per-phase
- reviews: story:validation-reads-selection-plan
revision: 1
---
needs-revision
story:overlap-witnesses-per-phase — the epic's "mutation" consumer is claimed in "What it delivers" (:44-45) but design decision 5 makes it optional ("Decide whether `precedence_sites` reads the plan") and no acceptance requires it, and the Scope line "no story in its wave table owns it" contradicts the epic's wave 5 row, which assigns it here; the body should state that `precedence_sites` reads the plan and drop the contradictory line — .engineering/planning/story/overlap-witnesses-per-phase.md:73,87 against .engineering/planning/epic/one-selection-plan.md:40,63
story:entity-runtime-lowering-reads-selection-plan — the acceptance lets lowered Entity Runtime definitions of repository models change for any command with a held-state branch declared after a disjoint accepting branch, but the epic permits that change only "where a story names an addition (#472)", and this exception is not #472; the story should either hold the bytes identical or have the epic widen its exception — .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md:39-42 against .engineering/planning/epic/one-selection-plan.md:47-49

**What you read:** 8 artifacts (the epic and its 7 stories) in full, with `aep plan artifact show` on each and `aep plan artifact graph`. The graph shows no other artifact claiming this ground. I read the critic procedure and rubric, and checked line numbers with `sed` and `grep` in `.engineering/planning`. I did not run `aep plan artifact kinds` or `relations`. The `decomposes` edge is how the epic reaches each story, and the seven stories carry it. I extracted 13 promises from the epic: the plan with ordered phases, six consumer outcomes, the new-guard-kind outcome, the three constraints, the first story's design decision on where the classification lives, and the wave placement. I traced all 13 to an item. Two of them (mutation, and bytes identical except #472) are traced with the narrowing or widening described in the findings above.

**What you could not establish:**
- Whether `story:generated-behaviour-reads-selection-plan` also widens the byte constraint. Its acceptance allows the same disjoint-held-state exception, but only for the slow per-model probe digests. The epic names `generated/` bytes and the lowered definitions, not those digests, so I did not file it.
- Out of my lane, for the other critics:
  - Validation's "start after #487 is on `main`" (epic :52) appears only as design decision 1 in `story:validation-reads-selection-plan`. It is not a `depends_on` relation or an acceptance line (parallel-safety).
  - `story:synthesis-reads-selection-plan` and `story:overlap-witnesses-per-phase` both edit `synthesize.rs` and `subject_fact.rs`. They are ordered by `depends_on` (parallel-safety).
  - The source acceptance "adding a new guard type does not require reimplementing precedence" rests on the no-wildcard match in `story:selection-plan-design-and-type`. The `OutcomeCondition` side of that match is only a design decision, not an acceptance line (acceptance).

```findings
- file: .engineering/planning/story/overlap-witnesses-per-phase.md
  line: 73
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the epic's mutation consumer is claimed in 'What it delivers' (:44-45) but design decision 5 (:87) makes it optional, no acceptance requires precedence_sites to read the plan, and the Scope line says no wave-table story owns it although the epic's wave 5 row (epic/one-selection-plan.md:63) assigns it to this story; the body should state that precedence_sites reads the plan and drop the contradictory line"
- file: .engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md
  line: 39
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance lets lowered Entity Runtime definitions of repository models change for held-state-after-disjoint-accepting commands, but the epic permits a byte change only 'where a story names an addition (#472)' (epic/one-selection-plan.md:47-49) and this exception is not #472; the story should hold the bytes identical or the epic should widen its exception"
```
