---
format: aep.planning-md/3
id: review-result:one-selection-plan-scope-round-2
kind: review-result
status: active
title: 'Plan critic (scope), round 2: one selection plan'
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
story:overlap-witnesses-per-phase — design decision 4 and Gap 3 add a `Note` for unsent refusal pairs, but neither "What it delivers" nor any acceptance line claims it, and the epic's only permitted additions are "overlap sends"; drop the decision, or name the `Note` in the deliverable and acceptance and widen the epic's exception — .engineering/planning/story/overlap-witnesses-per-phase.md:101 against .engineering/planning/epic/one-selection-plan.md:50

**Coverage:** I extracted 14 promises from the epic and traced all 14 to an item.
- Outcome: one plan, with phases and ordering inside each phase. Consumers: interpreter, lowering, emitters, synthesis, mutation, validation, so six.
- The "new guard kind is placed in the plan" promise.
- Four constraints:
  - derived and never persisted;
  - no behaviour change, with exception 1 (overlap sends) and exception 2 (held-state reorder);
  - the plan takes a new name;
  - the classification's home is decided in the first story.
- Wave placement and the #487 base, taken together as one promise.

Of the four source acceptance clauses, the new-guard-type clause rests on the no-wildcard acceptance lines in the design story, so all four are covered.

**Round-1 findings:** both are resolved.
- Mutation: `precedence_sites` reading the plan is now an acceptance line in the overlap story (:148-151).
- Byte exception: the epic now names the held-state reorder as exception 2 (epic:51), and the lowering and emitter acceptances match it.

**What you read:** 8 artifacts, the epic and its 7 stories, whole bodies. Commands: `aep plan artifact show epic:one-selection-plan` and `aep plan artifact show story:<each>`, `aep plan artifact graph`, and the round-1 scope review-result. The graph shows no other artifact claiming this ground. I did not run `kinds` or `relations`.

**What I could not establish:**
- Whether the synthesis story's "every later unit leaves that table identical" holds if exception 2 reorders a held-state branch. The epic limits that exception to generated and lowered bytes, so I did not file it. The synthesis story itself lists this as unestablished.
- Out of my lane (acceptance and design):
  - `story:interpreter-reads-selection-plan` Scope (:79) says the Go explorer is "Not listed in the epic", but the epic now lists it and puts it out of scope.
  - `story:validation-reads-selection-plan` design decision 3 says to "read whether the plan has a present-related refusal phase". The plan type lives in `ess-compiler`, which `ess-domain` cannot reach (epic Constraints).
  - `row_set.rs:401-409` is left unchanged in validation. The story names it, and the epic does not promise it, so it is not a gap.

```findings
- file: .engineering/planning/story/overlap-witnesses-per-phase.md
  line: 101
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "design decision 4 and Gap 3 add a Note for unsent refusal pairs, but neither 'What it delivers' nor any acceptance line claims it and the epic's only permitted addition is 'overlap sends' (epic/one-selection-plan.md:50); the story should drop the decision, or name the Note in the deliverable and acceptance and have the epic widen its exception"
```
