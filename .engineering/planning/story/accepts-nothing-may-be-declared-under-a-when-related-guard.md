---
format: aep.planning-md/3
id: story:accepts-nothing-may-be-declared-under-a-when-related-guard
kind: story
status: draft
title: 'accepts: nothing may be declared under a when_related: guard'
summary: 'An idempotent ensure-create cannot accept and do nothing when a matching row exists, because accepts: nothing is admitted only under when: or as the default (adopter item 12, 2026-10-22, related to beyond10x/ess#554); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 12 of the recheck batch, the second team's limit file L2, relayed by the
orchestrator; adopter names kept out). An idempotent "ensure" command accepts and changes nothing when a matching row already
exists, and creates one otherwise. The existence test is a related-row guard (`when_related: {entity, where, exists: true}`),
but `accepts: nothing` is admitted only under `when:` or as the default (beyond10x/ess#144, closed, introduced it with that
restriction). Today the only spellings are a refusal (an error the caller must treat as success) or an input-only `when:` that
cannot see the rows.

Needed: `accepts: nothing` under a `when_related:` guard, answered in the order of the one precedence rule
(beyond10x/ess#554), with the same whole-view "nothing changed" check its scenario has today.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Tag` with `label: String`, and
`EnsureTag(label)` with `already-there` (`when_related: {entity: demo.server.Tag, where: label == input.label, exists: true}`,
`accepts: nothing`) before `created` (`creates: demo.server.Tag`). `ess specify validate --path spec.yaml`, 0.68.0 (integration
build, also at `format: ess/30`) and 0.67.0 identical:

```text
error[ESS-COMMAND-004]: outcome `already-there` declares `accepts: nothing` beside a subject, an event, an error, an
assignment, a replay or a condition other than `when:`; an accepted request that changes nothing carries none of them
help: keep `accepts: nothing` alone, under `when:` or as the default; a command with a subject says `preserves:`
```

Controls, both versions, byte-identical suites: the same guard with `error:` instead of `accepts: nothing` validates and passes
2/2; `accepts: nothing` under an input `when:` validates and passes 2/2.

## Where it lives

- `crates/specify/ess-domain/src/command/outcome_shapes.rs:74-97` (`validate_outcome`): `admitted_condition` is
  `OutcomeCondition::When(_) | OutcomeCondition::Otherwise` only, so a `when_related:` condition is refused with the subject,
  event, error, assignment and replay cases in one message.
- Not covered by `story:every-guard-combination-validates-under-the-rule` (pairs refused for want of an order) or
  `story:an-identity-guard-beside-a-row-set-validates` (ESS-COMMAND-009): this refusal is the outcome shape of
  `accepts: nothing`, not a precedence pair. `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` is the
  subject-addressed counterpart (adopter item 6).

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0 and 0.68.0. Gap. Does not block 0.68.0.
