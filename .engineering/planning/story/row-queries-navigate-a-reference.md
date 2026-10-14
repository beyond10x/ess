---
format: aep.planning-md/3
id: story:row-queries-navigate-a-reference
kind: story
status: draft
title: 'A declared reference is navigated as a value: a related row''s fields and state in values, guards, view fields and filters'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#557
- provider: github
  reference: beyond10x/ess#564
relations:
- serves: vision:O2
- decomposes: epic:generic-row-queries
revision: 1
---
## Outcome

A declared reference is navigated as a value: from the addressed record, from an input identity or
from a view's source row, one hop along a declared `references`/`owns` relation, the far row's
fields **and its lifecycle state** are readable in `payload:`, `sets:`, response fields, guards,
view fields and view filters, read at the time the position is evaluated (a view reads the far
row's current value). An absent reference or a missing far row reads as absent, by one rule in
every position.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 1, navigation.
Parent: `epic:generic-row-queries`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/557 and
https://github.com/beyond10x/ess/issues/564 (both from a downstream incident-ledger specification,
ess 0.59.0, `ess/24`).

1. **Need.** (a) A refusal names the state of the related row that blocked it; guards already read
   that state (`predicate: state == Resolved`) but a value cannot
   (https://github.com/beyond10x/ess/issues/557: `ESS-COMMAND-001 "state is not a field of
   ledger.Incident"`). (b) A view over one entity filters and projects on the current fields of the
   row its reference names: a window counts only while its incident is resolved; a window without an
   end ends with its incident (https://github.com/beyond10x/ess/issues/564: `source:` takes one
   entity, `field: {related: …}` is an unknown key). Requester's syntax (theirs):
   `{related: {via, field: state}}` in a payload; `field: {related: {via, field}}` on a view field.
2. **Class.** Gap. A related row's state is readable in guards and not in values; a view cannot read
   a value that changes after the row is written.
3. **Already expressible?** Partly: copy-at-write (`{related: {via, field}}` in `sets:`) states the
   value at write time (`docs/design/read-api-view-idioms.md:174-196`); the incident's later change is
   the fact the reporter needs, so the idiom moves the figure out of conformance. A view field derived
   from the subject's own state exists (https://github.com/beyond10x/ess/issues/457); the related
   row's state does not.
4. **Fit.** One navigation rule instead of a new root per position: the `{related:}` source
   (`story:related-record-value-source`), `when_related` predicates and view fields read the same
   path. The #447 decline already recorded this exact shape as the starting point if a live join is
   needed (`story:feature-request-447`, alternative (c)); this is its third adopter. Targets:
   interpreter, synthesis (arrange the far row changed after the near row), Rust and Go generated
   view queries, Entity Runtime lowering, `ess verify diff`, generated docs; each handles it or
   refuses it by name.
5. **Second adopter.** An order list showing the customer's current name; the earlier metrics read
   API of https://github.com/beyond10x/ess/issues/447; the uuid-to-key filter of
   https://github.com/beyond10x/ess/issues/492 gap 4.
6. **Cost.** A source format version for the new positions; one diff classification per position;
   a view query that reads two entities in every generated target. No new diagnostic code expected
   (refusals reuse `undeclared_reference`/`type_mismatch`).
7. **Alternatives.** (a) Change nothing: the idioms stand and the figures stay unchecked. (b) The
   requester's per-position keys: one more cell per request. (c) Navigation as one rule in the
   expression language's position table (chosen).

## Decisions

Accept, redesigned into the generic feature: the requested keys become instances of one navigation
rule. The declines of https://github.com/beyond10x/ess/issues/447 and gap 4 of
https://github.com/beyond10x/ess/issues/492 are reopened by the 2026-10-13 decision to build the four
generic features; `docs/design/read-api-view-idioms.md` § "A view has one source" is revised with
the binding design.

## Acceptance

- `related_state_reads_as_a_payload_value`: a refusal's payload reads `{related: {via, field: state}}`
  and the synthesized scenario asserts the blocking state.
- `view_projects_the_referenced_rows_current_field`: a view field reads the far row; a scenario
  changes the far row after the near row is written and the view answers the new value.
- `view_filters_on_the_referenced_rows_current_field`: a filter over the far row's state selects
  only rows whose far row is in that state, witnessed with a decoy.
- `absent_reference_reads_absent_everywhere`: an absent reference reads absent in a payload, a view
  field and a filter.
