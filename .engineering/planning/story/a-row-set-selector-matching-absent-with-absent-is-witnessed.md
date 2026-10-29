---
format: aep.planning-md/3
id: story:a-row-set-selector-matching-absent-with-absent-is-witnessed
kind: story
status: draft
title: A row-set selector matching absent with absent is witnessed
summary: A presence disjunction over two Optionals in a row-set where starves found (ESS-SYNTH-001); plain == over the same fields is witnessed.
tags:
- defect
- downstream
- follow-up
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
revision: 2
---
## Problem

- Two Optional fields compared so that absent matches absent starve the row set's branches. Here
  `tenant` is stored on the row and `input.tenant` is sent:

  ```yaml
  any:
    - all: ["defined(tenant)", "defined(input.tenant)", tenant == input.tenant]
    - all: ["not defined(tenant)", "not defined(input.tenant)"]
  ```

- Inside a row-set `where:` this has no arrangement on 0.77.0, with or without a subject and with
  no creator guard: `ESS-SYNTH-001`, "... is `row set`, which has no arrangement of rows its
  selector selects".
  - Under an addressed subject, both `found` and `child-not-found` are refused.
  - Without one, `found` is refused.
- The same selector with plain `tenant == input.tenant` over the same Optionals synthesizes and
  passes the interpreted target. So the rows exist, and only the search does not reach them.
- Reproduced with a neutral model, in a repro outside the repository.
- Found while preparing
  `story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`. Its
  idiom for a tenant rule on a row-set reader writes the comparison in the selector, and an adopter
  writes it in this form.
- Inferred, not traced: the row search tries the input and the rows' values from the selector's
  comparisons (`crates/verify/ess-conformance/src/synthesize/row_set.rs`, `steered_inputs` `:4061`,
  `rows_tried`). A disjunct of presence tests gives it no value to try, and no candidate leaves the
  input and the row both absent.

## Acceptance

- A row set whose `where:` holds the disjunction above, beside `parent_id == input.parent_id` and
  `child_id == input.child_id`, is witnessed on both sides with no refusal. This holds with and
  without an addressed subject.
- The interpreted target passes every scenario.
- Models whose row sets 0.77.0 already synthesizes keep their suite bytes
  (`synthesis_suite_bytes_table`).
- Tests are named when the story is scoped, in `crates/verify/ess-conformance/tests/` with inline
  models.
