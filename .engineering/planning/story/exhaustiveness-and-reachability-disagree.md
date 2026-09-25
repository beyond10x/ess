---
format: aep.planning-md/2
id: story:exhaustiveness-and-reachability-disagree
kind: story
status: archived
title: Exhaustiveness requires an unconditional branch that synthesis then refuses as unreachable
summary: Over a closed enum whose every variant is guarded, ESS-COMMAND-005 requires a branch ESS-SYNTH-003 cannot reach; no specification satisfies both, and the workaround is a variant the producer never sends.
relations:
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

Two rules that are each correct disagree with one another, and a specification can satisfy only one.

- `ESS-COMMAND-005 non_exhaustive_branches` requires that a command's outcomes cover every input. Over a
  closed enum, that means an unconditional branch unless every variant has a `when:`.
- `ESS-SYNTH-003` refuses to synthesize a scenario for an outcome no candidate input reaches, and says so:
  *"no candidate of the 6 tried satisfies `none of: status == Offline, …`"*.

When every variant of a closed enum **is** guarded, the unconditional branch the first rule requires is
exactly the branch the second rule cannot reach. There is no way to write the command that satisfies both.

Measured in an internal specification repository, 2026-09-11, on two commands over a
six-variant status enum. Dropping the unconditional branch: `ESS-COMMAND-005` ×2. Keeping it: `ESS-SYNTH-003`
×2. Declaring a seventh variant so the branch becomes reachable: the synthesizer is satisfied and the
specification is **wrong** — the producer sends exactly six, so the branch is reachable only by an input that
never arrives, and the entity's lifecycle has no state for the value. That was tried and reverted.

That specification carries the two refusals permanently and explains them where a reader meets them.

## What would settle it

Any one of these; the design is the story's to choose:

1. **Exhaustiveness knows the enum is closed.** If every variant is guarded, no unconditional branch is
   required, and `ESS-COMMAND-005` does not fire.
2. **A branch may be declared unreachable on purpose**, with a reason, and synthesis records a named refusal
   rather than a defect — the shape `BindingGap::PolicySilent` already has for `on_failure: drop`.
3. **The subject guard** of the sibling story, which removes the need for the unconditional branch in this
   case because the branch is then guarded by the subject rather than the input.

## Acceptance

- [ ] A command whose branches cover a closed enum exhaustively needs no unconditional outcome, or has a way
      to declare one unreachable with a reason.
- [ ] `ess verify conform synthesize` over such a command reports no refusal.
- [ ] the internal specification repository loses both permanent `ESS-SYNTH-003` refusals without declaring a variant its
      producer does not send.

## Provenance

Filed 2026-09-11 from measurements in an internal application repository wave 3
(`review-result:adversary-conformance-suite-pass-1` F2; `-pass-2` findings A and B; the correction that
reverted the seventh variant).
