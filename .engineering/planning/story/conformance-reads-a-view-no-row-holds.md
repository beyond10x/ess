---
format: aep.planning-md/3
id: story:conformance-reads-a-view-no-row-holds
kind: story
status: draft
title: Conformance reads a view no row holds
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-may-answer-values-no-row-holds
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/view_settings.rs
revision: 4
---
## Need

Once a view may answer a literal or a fixed setting with no source row
(`story:a-view-may-answer-values-no-row-holds`), the suite must check the figure, and the
interpreted target must answer it; otherwise the value is documentation the suite never reads
(policy 1, `docs/design/generic-language-features.md:588-594`). On 0.92.0 the nearest idiom (a
constant `expr:` on a sourced view) is read only beside a stored row (`triage/674/repro/run.txt`,
run 4: the scenario reading it is `AddBook/outcome/added`); a sourceless view cannot be declared.

## Acceptance

- `ess verify conform synthesize` on the reproduction's sourceless view `Limits`
  (`max_tags: setting.max_tags` with `value: "20"`, `api_level: 1`) emits a scenario
  `example.library.Limits/answer` that reads the view on an empty store and expects exactly one row
  `{max_tags: 20, api_level: 1}`; on a sourced view with a setting field, every existing read
  expects the setting's fixed value in that field.
- `ess verify conform run --target interpreted --report-format 2` passes it (exit 0); an interpreter
  mutant answering `max_tags: 21` or zero rows fails it.
- A specification without such a view keeps its suite bytes (`synthesis_suite_bytes_table` rows
  unchanged); no suite format moves (the expectation reuses `eventually_view` `contains` with literal
  fields, as run 4's suite does).
- Tests in `crates/verify/ess-conformance/tests/view_settings.rs` (new).

## Scope

- cited: crates/verify/ess-conformance/src/interpret/views.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/synthesize/ (a sourceless-view module)
- cited: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs (unchanged rows)
- inferred: crates/verify/ess-conformance/tests/view_settings.rs (new)
confidence: medium

## Fit review

1. **Need.** The suite checks the value a view answers with no stored row. Requester: a limits or
   capabilities endpoint (issue body, repro 1).
2. **Class: gap** (follows from the construct; without it the construct would be unchecked).
3. **Already expressible?** No (run 1 of the reproduction).
4. **Fit.** Reuses the existing view expectation (literal fields under `contains`, as in the run-4
   suite) and the shared evaluator for `expr:`. Fixed settings need no `configuration:` (Amendment 1,
   First cut).
5. **Second adopter.** The capabilities endpoint of the parent story.
6. **Cost.** New scenarios only for specifications using the construct; no suite format.
7. **Alternatives.** (a) Leave the view unchecked: a red flag of the assessing skill. (b) Check it
   only through a wire target: nothing checks it on the interpreter. (c) Chosen.

## Decisions

**accept, redesigned**, as the parent story (Amendment 1 of ADR
`a-command-guard-reads-a-component-setting`, approved 2026-10-28).

## Source

github:beyond10x/ess#674
