---
format: aep.planning-md/3
id: story:a-payload-may-leave-an-optional-event-field-absent
kind: story
status: draft
title: An outcome's payload may state an Optional event field as absent
summary: An absent Optional event field is refused with ESS-COMMAND-005/001 (a2a wave 1, G3).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G3** (a2a wave-1 gaps list, line 12): an outcome cannot give an Optional event field an absent
  value. Refused with ESS-COMMAND-005 / ESS-COMMAND-001 (the adopter's rendered lines were not
  relayed).

Minimal reproduction (brand-free, to be confirmed by the implementor): an event
`JobUpdated { job_id, note: Optional<Text> }` emitted by an outcome whose `payload:` must state
that `note` is absent on this branch.

## Where it lives

Not traced. Related, not the same: `story:payload-fields-have-one-filling-rule` (every payload field
mapped) and `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued` (an absent
Optional input at run time; this gap is at validation).

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch. First
question: which payload spelling (if any) states "absent" today, and whether
`story:payload-fields-have-one-filling-rule` should own the answer.

## Acceptance (provisional)

- An outcome can state an Optional event field as absent; validation admits it, generated targets
  publish the event without the field, and synthesis checks its absence.
- A required event field stated absent is refused by name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1.
