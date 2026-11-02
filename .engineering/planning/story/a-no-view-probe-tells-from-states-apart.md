---
format: aep.planning-md/3
id: story:a-no-view-probe-tells-from-states-apart
kind: story
status: draft
title: The no-view arranged probe tells apart the states its branch runs from
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by adversary pass 1 of wave D357 U3 (`review-result:adversary-d357-u3-pass-1`, F1; the claim was narrowed at 1962a5ff0 instead): the no-view probe of an arranged input-refusal half sends only the addressing branch's valid input, so a target that moves the record on refusal into another state that branch also runs from passes, even when another command's wrong-state branch would show the move. The red case (a `Staged` state from which `configure` also runs, a target swapping `New` and `Staged` on refusal) is kept by the wave lead for this story.

## Acceptance

- Where the probing branch runs from several states, the arranged half also sends a branch that tells the arranged state apart from the others (a wrong-state branch of another command), or the scenario is refused with a cause naming the states it cannot tell apart.
- The red case passes for the correct target and fails for the swapping target; the narrowed sentences in `existence.rs`, `docs/design/outcome-shapes.md` and CHANGELOG are widened again only as far as the new probe reaches.
- Committed models keep their suite bytes unless a model has such a branch, which is named.
