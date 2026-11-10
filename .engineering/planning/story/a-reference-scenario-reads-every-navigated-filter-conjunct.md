---
format: aep.planning-md/3
id: story:a-reference-scenario-reads-every-navigated-filter-conjunct
kind: story
status: active
title: A reference scenario reads every navigated conjunct of a view filter
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/view_reference.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/
- confidence: cited
  path: crates/verify/ess-conformance/tests/row_queries_navigate_a_reference.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:13Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:14Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A view's `<view>/reference/<field>` scenario (the one-hop navigation of `ess/28`) must pass against
ESS's own interpreted target. With a filter of two conjuncts through the same reference, the
scenario expects a row to be hidden after its referenced row moves into the filtered state, where
the referenced row satisfies both conjuncts and the interpreter, correctly, shows it.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/a-one-hop` of the triage of
beyond10x/ess#564; `Window.incident_id` references `Incident`, which carries `approved: Boolean` and
moves `Open → Resolved`):

```yaml
  - name: demo.ops.CountedWindows
    source: demo.ops.Window
    filter: [incident_id.state == Resolved, incident_id.approved == true]
    fields:
      - {name: window_id, type: demo.ops.WindowId}
      - {name: incident_state, type: Optional<demo.ops.Incident.State>, field: {related: {via: incident_id, field: state}}}
```

```text
$ ess verify conform run --path a-one-hop --target interpreted --report-format 2
  failed demo.ops.CountedWindows/reference/incident_id
  10 scenarios: 9 passed, 1 failed, 0 error, 0 unsupported
exit 1
      expected:
      - demo.ops.CountedWindows holds no row where window_id = "00000000-0000-4000-8000-000000000005"
      observed:
      - '{incident_ended_at = "2020-01-15T00:00:00Z", incident_state = "Resolved", window_id = "00000000-0000-4000-8000-000000000005"}'
```

The arranged incident was opened with `approved: true` and then resolved, so both conjuncts hold.
With the filter `[incident_id.state == Resolved]` alone the same model passes 10/10
(`repro/a2-one-conjunct`).

## Acceptance

- `ess verify conform run --path a-one-hop --target interpreted --report-format 2` reports
  `demo.ops.CountedWindows/reference/incident_id` `passed`: after the incident with
  `approved: true` is resolved, the scenario expects the window's row shown with
  `incident_state: Resolved`; the decoy with `approved: false` stays hidden.
- `ess verify conform run --path a2-one-conjunct ...` keeps 10/10 and its suite bytes.

Pinned by `crates/verify/ess-conformance/tests/row_queries_navigate_a_reference.rs`
(`a_reference_scenario_reads_every_navigated_conjunct_of_the_filter`), whose fixture restates the
minimal specification.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/view_reference.rs (`expect`, `shows`, `held`: the referenced row's values read into the filter)
- cited: crates/verify/ess-conformance/tests/row_queries_navigate_a_reference.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/
- inferred: CHANGELOG.md
confidence: high

## Fit review

1. **Need.** A synthesized suite must hold ESS's reference implementation; a navigated filter of
   two conjuncts gets a scenario that fails it. Reproduction above (run.txt § A). No syntax
   proposed: found while triaging the issue's own first construct (window counts while its
   incident is resolved and approved).
2. **Class.** Defect: the design states each navigated view is witnessed by a
   `<view>/reference/<field>` scenario that reads the far row as it is when read
   (`docs/design/generic-language-features.md:252-255`); the scenario's expectation contradicts it.
3. **Already expressible?** The model is valid and the interpreter answers it correctly; nothing
   for an adopter to change.
4. **Fit.** No language change, no grid cell moves. The expectation is computed by `shows`
   (`synthesize/view_reference.rs`), which reads the far row's held values into the filter; the fix
   is there. Targets: synthesis only.
5. **Second adopter.** Any view filtered on two fields of one referenced row: orders whose customer
   is active and verified; tickets whose desk is open and staffed.
6. **Cost.** No format, no key, no diagnostic code; suites of models with such a filter change
   their expectation bytes.
7. **Alternatives.** (a) Change nothing: the suite fails every correct target. (b) Refuse a filter
   with two navigated conjuncts: takes back an admitted spelling. (c) Chosen: compute the
   expectation over every conjunct.

## Decisions

**Accept as proposed** (a defect; no requester design involved). No format change.

## Source

github:beyond10x/ess#564
