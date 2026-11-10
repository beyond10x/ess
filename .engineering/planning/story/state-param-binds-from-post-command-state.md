---
format: aep.planning-md/3
id: story:state-param-binds-from-post-command-state
kind: story
status: draft
title: A view parameter compared with the lifecycle state is bound from the asserted state
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis.rs
revision: 6
---
## Outcome

A view parameter compared with the entity's lifecycle state (`state == param.p`) is bound by
synthesis from the state the generated scenario asserts after its command, so such a view no longer
blocks the outcome and transition scenarios that reach the entity.

## Problem

Reported by an adopter on 0.59.0, reproduced on 0.60.0 and again on 0.92.0. Minimal reproduction:
the workbench base example (`.engineering/repro/workbench-2026-11-09/examples/00-base/spec`) with
one more view, listed in the actor's `may:`:

```yaml
- name: gap.demo.ItemsByParentAndState
  source: gap.demo.Item
  consistency: read_your_writes
  params:
  - {name: parent_id, type: String}
  - {name: state, type: gap.demo.Item.State}
  filter:
    all:
    - parent_id == param.parent_id
    - state == param.state
  fields:
  - {name: id, type: String}
  - {name: state, type: gap.demo.Item.State}
```

`ess verify conform synthesize --path spec --out suite.yaml` (exit 0):

```text
refused: refusal[ESS-SYNTH-005]: outcome gap.demo.CreateItem/created has a scenario `gap.demo.CreateItem/outcome/created` that skips its read of `gap.demo.ItemsByParentAndState`
  `gap.demo.ItemsByParentAndState` filters on `(parent_id == param.parent_id and state == param.state)`, which is undecided for an entity in `Open`
    - `param.state` is bound by nothing a scenario knows
  help: filter the view on the entity's state, which is what a generated scenario knows after the command it ran
8 scenario(s) (0 authored), 3 refusal(s)
```

The same refusal names `FinishItem/outcome/finished` and `Item/transition/finish/...` (entity in
`Done`). The same view without the `state` parameter synthesizes 10 scenarios with 0 refusals.

`bound()` (`crates/verify/ess-conformance/src/synthesize.rs:14454`) binds a parameter from a text
comparison, an identity, a period bound or a settled field, never from the lifecycle state the caller
already knows. The help text (`synthesize.rs:1550`, `:1708`) suggests filtering on the state, which
the view already does.

## Fit review

1. **Need.** A view selecting rows by a state the caller passes is read by generated scenarios.
2. **Class.** Defect: the parameter's value is the state the scenario asserts, which synthesis
   knows (`ViewUndecidable` help, `synthesize.rs:1550`, says so).
3. **Already expressible?** Only by dropping the parameter, which changes the view's contract.
4. **Fit.** No new surface; one more source in `bound()` beside `text_param`, `identity::param`
   and `period_read`.
5. **Second adopter.** A task list read by `project_id` and `status`.
6. **Cost.** No format. Suites of specifications with such a view gain scenarios; pinned digests of
   models without one do not move.
7. **Alternatives.** (a) Change nothing: the refusal blocks every outcome reaching the entity.
   (b) This story.

## Decisions

- Accept as a defect fix; no authored surface changes.

## Acceptance

- The reproduction above synthesizes with 0 refusals, and `ess verify conform run --target
  interpreted --path spec --report-format 2` passes every scenario
  (`state_param_bound_from_asserted_state_on_create`, `state_param_bound_after_transition`).
- A generated read of the view with the asserted state shows the row; a read with another state of
  the lifecycle does not (`state_param_view_excludes_row_in_other_state`).
- Where ESS-SYNTH-005 still fires for a parameter compared with the state, its help does not tell the
  author to filter on the state.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`bound()`, `ViewUndecidable` help)
- inferred: `crates/verify/ess-conformance/tests/synthesis.rs`
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (only if a pinned model has such a view)

confidence: high
