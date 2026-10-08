---
format: aep.planning-md/3
id: review-result:one-selection-plan-design-round-2
kind: review-result
status: active
title: 'Plan critic (design), round 2: one selection plan'
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
story:selection-plan-design-and-type — the seam builds a plan with a given phase order, but the consumers build the plan inside their own entry points, so it never reaches them; the story should make the seam an override the constructor itself honours (scoped, `#[doc(hidden)]`), which would remove the competing seams in `story:synthesis-reads-selection-plan` (a scoped thread-local), `story:entity-runtime-lowering-reads-selection-plan` (a "crate-private seam", which `tests/selection_precedence.rs` cannot reach) and `story:validation-reads-selection-plan` (a `#[cfg(test)]` unit) — `.engineering/planning/story/selection-plan-design-and-type.md:51-54` against `synthesis-reads-selection-plan.md:101`, `entity-runtime-lowering-reads-selection-plan.md:80`, `validation-reads-selection-plan.md:71`
story:selection-plan-design-and-type — item 3 says the seam is "reachable from `ess-domain`'s own validation", but the plan type and constructor live in `ess-compiler`, which depends on `ess-domain`, and no acceptance line covers an `ess-domain`-side seam; the body should state the seam at the classification (phase order) level in `ess-domain` and add an acceptance line for it — `.engineering/planning/story/selection-plan-design-and-type.md:51-53` against `:67-68`

What I read: 8 artifacts in full with `aep plan artifact show`, plus `aep plan artifact relations`, `aep plan artifact graph` (I read the whole graph's edges, including those to `vision:O2` and the four round-1 review-results) and `aep plan artifact validate`. I also read `review-result:one-selection-plan-design-round-1` to compare. The graph is a star on `story:selection-plan-design-and-type`, plus `story:overlap-witnesses-per-phase` depends_on `story:synthesis-reads-selection-plan`. I found no cycle and no serialising chain.

What I could not establish:
- Whether the lowering test can inject a phase order at all: `lower_component` and `lower_command` build the plan internally and no body names an entry point that takes one. That is the first finding again, seen from the consumer.
- `aep plan artifact validate` printed `valid` after warnings about unrecorded review outcomes on older artifacts. None concern this set.
- Out of my lane (scope, acceptance): `story:overlap-witnesses-per-phase` and `story:synthesis-reads-selection-plan` still carry "Could not establish" lines about whether the plan reproduces today's order. They set no part of my verdict.

```findings
- file: .engineering/planning/story/selection-plan-design-and-type.md
  line: 51
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the seam builds a plan with a given phase order, but the consumers build the plan inside their own entry points, so it never reaches them; the story should make the seam an override the constructor itself honours (scoped, `#[doc(hidden)]`), which would remove the competing seams in story:synthesis-reads-selection-plan (a scoped thread-local), story:entity-runtime-lowering-reads-selection-plan (a crate-private seam, which tests/selection_precedence.rs cannot reach) and story:validation-reads-selection-plan (a `#[cfg(test)]` unit)"
- file: .engineering/planning/story/selection-plan-design-and-type.md
  line: 51
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "item 3 says the seam is reachable from `ess-domain`'s own validation, but the plan type and constructor live in `ess-compiler`, which depends on `ess-domain`, and no acceptance line covers an `ess-domain`-side seam; the body should state the seam at the classification (phase order) level in `ess-domain` and add an acceptance line for it"
```
