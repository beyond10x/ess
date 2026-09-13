# The Linker Never Chooses — Gap Register D-2

This page is the home of gap register **D-2**. It states the rule once, and under `docs/` it is the
only full statement of it — that much is checked mechanically.

A document that applies the rule cites it and links here rather than restating it. Under `docs/` a
link resolves. Outside `docs/` the honest word is **cites**, not links: `docs/` is the engineering
record and is never published, so a link to it from adopter-facing source under `website/` would
not resolve for the reader that source was written for.

Code that enforces D-2 restates the rule where it binds, which is where a linker's contract
belongs. This page does not enumerate those copies and makes no claim about what tests them.

## The rule

> **D-2. The linker never chooses.** Zero implementations offered for an obligation is an
> unsatisfied obligation. Two is an ambiguity error naming both. Selection among alternatives is
> `Realization` material and stays proposed with it.

## What it forbids, and what it does not

D-2 constrains **the linker**, and what it forbids is **selection among alternatives**. It is the
rule that stops the machinery silently picking one of two candidate implementations and shipping
it — because the moment a build can choose, a deployment carries behaviour nobody named, and the
specification stops being the account of what runs.

Both halves of the rule are refusals, and they refuse in opposite directions on purpose:

| Offers for one obligation | The linker's answer |
|---|---|
| zero | an unsatisfied obligation — the plan still owes it, and says so |
| exactly one | resolved; the offer is who answers, not a decision about what should |
| two or more | an ambiguity error **naming every claimant** |

The many case names both rather than reporting a count, because a person resolving an ambiguity
needs to know which two things collided. The zero case is not an error at all: an obligation the
plan owes and nobody has filled is the normal state of a specification that has been written and
not yet implemented, and collapsing it into a failure would make the two indistinguishable.

It is **not** a rule that an implementation may not exist, and it cannot be — this repository
contains hand-written implementations, and they do not violate D-2. They do not violate it because
no build silently selected them: a person names one, explicitly, at the point of use. The question
D-2 asks is never *may behaviour be chosen* but **who is choosing, and for what**:

| | chooses | for | D-2 |
|---|---|---|---|
| the linker | nothing — refuses zero, refuses two | code somebody ships | governs it |
| a synthesis target | nothing — emits obligations | code somebody ships | governs it |
| an operator naming a target | the operator, explicitly | checking a suite | untouched |

A tool that assembles something shippable is machinery and D-2 governs it. An operator naming one
target out of several, in an argument, is not the machinery choosing.

## Where it is enforced

The rule is executed, not asserted. Three places where it is executed, each read and checked when
this page was written — **not a census**, and this page does not attempt one:

- `examples/billing-realization/src/linker.rs` — a test for the zero case and a test for the many
  case, at `:543` and `:565`.
- `examples/gatepass-realization/src/linker.rs` — the same pair, at `:317` and `:335`.
- `crates/generate/ess-synth/tests/synthesis.rs` — the many case reaching delivery: two components
  accepting one command is a refusal naming both, rather than a choice between them.

## Provenance

D-2 was decided in `docs/plan/gap-register.md`. **That file no longer exists**, and this page
replaces it as the constraint's home rather than restoring the register. The rule outlived the wave
that recorded it, which is precisely why it could not stay in a wave plan: a plan describes a wave
and is finished when the wave is, and a constraint whose home is finished is one the next reader
re-litigates.

Extending, narrowing or overturning D-2 is a change to this page. A page that merely applies it
links here.
