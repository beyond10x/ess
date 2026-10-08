---
format: aep.planning-md/3
id: review-result:one-selection-plan-design-round-1
kind: review-result
status: active
title: 'Plan critic (design), round 1: one selection plan'
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
story:selection-plan-design-and-type — five consumer acceptances need a plan with two phases exchanged and the type's "What it delivers" and "Acceptance" never provide that construction seam, so the interpreter, lowering, emitter and synthesis stories each invent a different one (`#[doc(hidden)] pub`, crate-private, scoped thread-local), and the validation story needs a `#[cfg(test)]` classifier injection in `ess-domain`. Add an exchanged-phase constructor, or a phase-order parameter, to the type's deliverables so the edge `depends_on` carries it — `.engineering/planning/story/selection-plan-design-and-type.md:35` (what it delivers has no such item); consumer needs at `.engineering/planning/story/interpreter-reads-selection-plan.md:41,87`, `.engineering/planning/story/entity-runtime-lowering-reads-selection-plan.md:43,75`, `.engineering/planning/story/generated-behaviour-reads-selection-plan.md:47`, `.engineering/planning/story/synthesis-reads-selection-plan.md:54`, `.engineering/planning/story/validation-reads-selection-plan.md:71`

What I read: 8 artifacts, each in full via `aep plan artifact show`. I also ran `aep plan artifact relations` and `aep plan artifact graph`. The graph is a star on the design story plus one edge from overlap to synthesis. I checked the declared edges of every id in the set, including `serves vision:O2`, and found no cycle and no serialising chain. `aep plan artifact validate` printed nothing I could use: I tailed its output and saw no report line, so I treat its output as unread.

What I could not establish:
- Whether `validate` reported anything.
- Out of my lane, scope or acceptance: `story:overlap-witnesses-per-phase` "What it delivers" says `mutate.rs` `precedence_sites` builds from the same pairs, but its design decision 5 says "Decide whether `precedence_sites` reads the plan" and its acceptance names no mutation criterion. The epic's wave table says this story owns mutation.
- Out of my lane, parallel safety: the design story and `story:validation-reads-selection-plan` both edit `ess-domain/src/command.rs`, and the edge covers that order.

```findings
- file: .engineering/planning/story/selection-plan-design-and-type.md
  line: 35
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "five consumer acceptances need a plan with two phases exchanged and the type's What it delivers and Acceptance never provide that construction seam, so the interpreter, lowering, emitter and synthesis stories each invent a different one (`#[doc(hidden)] pub`, crate-private, scoped thread-local) and the validation story needs a `#[cfg(test)]` classifier injection in `ess-domain`; add an exchanged-phase constructor, or a phase-order parameter, to the type's deliverables so the edge `depends_on` carries it"
```
