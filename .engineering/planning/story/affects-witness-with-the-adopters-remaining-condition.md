---
format: aep.planning-md/3
id: story:affects-witness-with-the-adopters-remaining-condition
kind: story
status: draft
title: An affects witness holds under the adopter's remaining condition
summary: 'Follow-up to the 0.71.0 not-found-branch fix: the full adopter specification still refuses its affects: outcome; the reduced reproduction dropped a condition.'
tags:
- ess-0.72.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

story:an-affects-row-is-arranged-past-a-creators-emitting-not-found-branch (0.71.0) arranges an
`affects:` row past a creator whose `exists: false` branch emits an event, and its reduced
reproduction now synthesizes every scenario. A replay of the 0.71.0 integration head against the
full adopter specification the reproduction came from still refuses that adopter's `affects:`
outcome:

> no witness: `<outcome>.affects[0]` is `<Entity>` and created only by `<Creator>/<branch>`, which
> no arrangement takes for a row the filter selects with the rows the scenario holds when it is sent

A second reduction names the dropped condition. A deleting command removes `Ticket` rows through
`affects:` (`where: member_id == input.member_id`). The only creator, `OpenTicket/opened`, sits
behind the existence guard and a second branch guarded by
`when_related: {via: input.member_id, predicate: kind == Linked}`, where `kind` comes from the
member creator's input. In the same refusing run, synthesis writes `OpenTicket/outcome/opened` by
creating members with `kind` Linked and Plain and opening the ticket for the Plain one, so the
creating branch can be taken. The `affects:` arrangement does not choose the related row's field
value that steers past the predicate guard. Deleting the predicate branch, and only that, clears
the refusal. Turning either guard into an `error:` does not.

## Acceptance

- Named conformance case `affects_past_a_creators_predicate_guard` (in
  `tests/affects_past_emitting_not_found.rs`, with a neutral `demo.*` fixture of the shape above):
  the deleting command's `<command>/outcome/<branch>` scenario is synthesized with 0 refusals. It
  arranges the related row with a field value that passes the predicate guard, and the scenario
  passes the interpreted target.
- The same fixture without the predicate branch keeps its 0.71.0 suite bytes, and so do the 0.71.0
  fixtures `affects-past-emitting-not-found.yaml` and `affects-past-erroring-not-found.yaml`.
