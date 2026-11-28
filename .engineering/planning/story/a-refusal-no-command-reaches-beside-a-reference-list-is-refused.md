---
format: aep.planning-md/3
id: story:a-refusal-no-command-reaches-beside-a-reference-list-is-refused
kind: story
status: draft
title: A refusal on a stored field no command writes, beside a list of references, is refused with its disclosure cells instead of expected on a row that does not hold it
summary: A refusal on a stored field no command writes, beside a list of references, got a scenario on a row that does not hold the guard, and the scenario and its disclosure cells failed on every target, the interpreted one included
tags:
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Summary

A command that refuses a missing row named in a list of references (`when_related: {via:
input.<list>, exists: false}`) and also refuses on a stored field of the row it names
(`when_subject: {predicate: <field> == <value>}`) got a synthesized scenario for the second
refusal on a row the creating command stored, which does not hold the guard. Where no command
writes the field to that value, the refusal is reachable by no command sequence, and the scenario
failed on every target, the model's own interpreted target included. Every one-time disclosure
cell that sends that refusal after the origin inherited the scenario and failed the same way.

Where the creating command is itself a row-set command, the ordinary scenario was withdrawn by the
row-set check and refused for a row-set reason, while the disclosure cell, built after that check,
stayed in the suite: the scenario refused, its cell synthesized and failing.

## Acceptance

- For such a model, `ess verify conform synthesize` writes no scenario that expects the stored-field
  refusal: neither the outcome's own scenario nor any disclosure cell that sends it.
- The outcome's own scenario and each of its disclosure cells are refused (`ESS-SYNTH-003`) naming
  the stored field no arrangement held (`` `<Entity>` stored <field> selecting this branch``), the
  refusal the same model gets without the list of references.
- The synthesized suite passes on the interpreted target with 0 failed.
- Where a declared command does write the field to the guarded value, the refusal is arranged on
  such a row, as without the list.

## Fit review

A downstream application reported it with a reproduction on 0.98.0 and 0.98.2 and first saw it on
0.92.0: 16 scenarios, 15 passed, 1 failed (`ESS-CF-OUTCOME`, `ESS-CF-ERROR`, `ESS-CF-NO-EVENT`) on
the interpreted target, a disclosure cell expecting an outcome whose own scenario was refused.
Variants in the report: without the list-of-references refusal the outcome and its cell are refused
and 14 of 14 pass; without the row-set guard on the creating command the ordinary scenario is
synthesized too and both fail. All three share one cause: the list-of-references path arranged a
subject-less refusal on any row the acting sibling acts on, ignoring the refusal's own stored-field
guard. A defect of synthesis, adopted as reported; no request for new syntax.

## Decisions

- The disclosure cell is not owed for an outcome no declared command sequence reaches:
  `docs/design/one-time-response-values.md` forks a follow-up cell "for each declared
  command/outcome the existing arrangement machinery can reach" and refuses an unreachable cell
  "with their reason". The cell is refused beside the scenario with the same cause.
- The fix is in the arrangement, not in the disclosure inventory: the list path sends a refusal
  guarded on stored fields through the same arrangement as a command without the list, so the
  ordinary scenario and every cell that reaches the branch through it agree.
- Not changed here: disclosure cells are built after the row-set check of the ordinary suite and are
  not walked by it. Where that check withdraws a reachable branch's scenario, its cell can stay;
  the one such cell seen (the list-of-references refusal of the reported model) passes on the
  interpreted target. It is named in the commit and left.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`: `run_as`, the `related_list::routes` branch.
- `crates/verify/ess-conformance/tests/one_time_followups.rs`: the regression test
  `a_followup_no_command_sequence_reaches_is_refused_in_its_disclosure_cell`.
- `crates/verify/ess-conformance/tests/fixtures/one-time-followup/unreachable-subject-guard.yaml`:
  its fixture.
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`,
  `synthesis-suite-bytes-table.tsv`: the stamp and the new fixture's row.
- `CHANGELOG.md`: one `### Fixed` bullet.
