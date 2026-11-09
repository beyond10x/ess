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
- severity-medium
- zero-open-issues
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_sets.rs
revision: 5
---
## Problem

- Two Optional fields compared so that absent matches absent starve the row set's branches. Here
  `tenant` is stored on the row and `input.tenant` is sent:

  ```yaml
  any:
    - all: ["defined(tenant)", "defined(input.tenant)", tenant == input.tenant]
    - all: ["not defined(tenant)", "not defined(input.tenant)"]
  ```

- Inside a row-set `where:` this has no arrangement, still on 0.92.0. Under an addressed subject,
  both `found` and `child-not-found` are refused as `ESS-SYNTH-001`, "... is `row set`, which has no
  arrangement of rows its selector selects". The refusal also blames the creator's
  `parent-not-found` guard, which is wrong: the control below has the same creator and is witnessed.
- The same selector with plain `tenant == input.tenant` over the same Optionals synthesizes 6
  scenarios with no refusal and passes the interpreted target. So the rows exist, and only the
  search does not reach them.
- Minimal reproduction (0.92.0). `Child {child_id, parent_id, tenant: Optional<String>}`, created by
  `CreateChild` (refuses a missing parent, sets `tenant: input.tenant`). `GetChild` addresses the
  `Parent` by `parent_id`:

  ```yaml
  - name: child-not-found
    when_related:
      entity: demo.d.Child
      where:
        all:
          - parent_id == input.parent_id
          - child_id == input.child_id
          - any:
              - all: ["defined(tenant)", "defined(input.tenant)", tenant == input.tenant]
              - all: ["not defined(tenant)", "not defined(input.tenant)"]
      exists: false
    error: demo.d.NotFound
  - name: found
    preserves: demo.d.Parent
    instance: parent_id
  ```

  `ess verify conform synthesize` gives `4 scenario(s) (0 authored), 2 refusal(s)`: `ESS-SYNTH-001`
  on `GetChild/child-not-found` and `GetChild/found`. With the `any:` replaced by
  `tenant == input.tenant`: `6 scenario(s) (0 authored), 0 refusal(s)`, and the interpreted run
  passes all 6.
- Found while preparing
  `story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`. Its
  idiom for a tenant rule on a row-set reader writes the comparison in the selector, and an adopter
  writes it in this form.
- Inferred, not traced: the row search tries the input and the rows' values from the selector's
  comparisons (`crates/verify/ess-conformance/src/synthesize/row_set.rs`, `rows_tried` `:4193`,
  `steered_inputs` `:4289`). A disjunct of presence tests gives it no value to try, and no candidate
  leaves the input and the row both absent.

## Acceptance

- `a_row_set_selector_matching_absent_with_absent_is_witnessed`, in
  `crates/verify/ess-conformance/tests/row_sets.rs` with the inline model above: synthesis writes
  `GetChild/outcome/found` and `GetChild/outcome/child-not-found` with no refusal, and every
  scenario passes the interpreted target.
- The same holds for a reader with no addressed subject (the row set alone decides `found`).
- A target whose row-set test treats absent as unequal to absent fails at least one of those
  scenarios.
- Models whose row sets 0.92.0 already synthesizes keep their suite bytes
  (`synthesis_suite_bytes_table`).

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/tests/row_sets.rs
- cited: CHANGELOG.md

confidence: medium
