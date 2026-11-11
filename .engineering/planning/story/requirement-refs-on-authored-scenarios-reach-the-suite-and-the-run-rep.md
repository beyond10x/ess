---
format: aep.planning-md/3
id: story:requirement-refs-on-authored-scenarios-reach-the-suite-and-the-run-rep
kind: story
status: draft
title: Requirement refs on authored scenarios reach the suite and the run report
relations:
- informed_by: story:requirement-refs-reach-suite-and-report
revision: 1
---
## Outcome

Authored scenarios accept `refs:` and the suite carries them; a synthesized scenario inherits the
refs of the command and outcome its id names (never those of setup outcomes); the run report lists
refs per scenario with an index from each ref to its scenario outcomes. Requested in
https://github.com/beyond10x/ess/issues/503 (part B): a specification traced to a published
standard cannot show which scenario proved which requirement, because refs stop at the compiled IR
(`Refs` fields in `crates/specify/ess-compiler/src/ir.rs:1010,2079,3051,3207`).

Split off from story:requirement-refs-reach-suite-and-report, which keeps the defect half (a
refs-only edit rated `unclassified-changed` by `ess verify diff`). This is a feature request and
sits outside epic:zero-open-issues-2026-11.

## Problem

Reproduced on 0.92.0 with the workbench base specification
(`.engineering/repro/workbench-2026-11-09/examples/00-base/spec`) given `refs: [std:REQ-1]` on
`gap.demo.CreateParent` and `refs: [std:REQ-2]` on its `created` outcome (`ess specify validate`:
valid).

1. An authored scenario with `refs:` is refused:

   ```yaml
   type: ess-scenario/1
   domain: gap.demo
   scenario: create-parent
   summary: A parent is created.
   refs: [std:REQ-1]
   timeline:
     - at: 2026-01-05T09:00:00Z
       command: gap.demo.CreateParent
       input: {id: p-1, name: first}
       outcome: created
   ```

   `ess verify conform synthesize --path spec --scenarios scenarios --out suite.json` (exit 1):
   `refusal[ESS-AUTHOR-001]: ... unknown field `refs`, expected one of `type`, `domain`, `scenario`,
   `summary`, `fixtures`, `arrange`, `timeline`, `assert``.
2. Without `refs:` on the scenario, the synthesized suite (9 scenarios, 1 authored) and the
   `--report-format 2` run report contain neither `REQ-1` nor `REQ-2`.

## Fit review

1. **Need.** Show, per requirement reference, which scenarios exercised it and how they ended.
2. **Class.** Gap: authored scenarios refuse the key (`Document` is `deny_unknown_fields`,
   `crates/verify/ess-conformance/src/authored.rs:377-379`); suite and report carry nothing.
3. **Already expressible?** For synthesized scenarios only by an external join of scenario id and
   IR refs; `ConformanceScenario.source` (`crates/verify/ess-conformance/src/scenario.rs:1442`)
   includes setup outcomes and would over-attribute. No idiom for authored scenarios or the report.
4. **Fit.** Same `ExternalRef` spelling (`crates/specify/ess-domain/src/refs.rs`) and key. The
   report reuses the `Outcomes` id-list shape. Scenario files are not `ess verify diff` inputs, so a
   scenario-refs diff is declined.
5. **Second adopter.** A service tracing outcomes to an internal control catalogue and reporting
   per control.
6. **Cost.** The next free scenario format (`ess-scenario/7`; `/5` is caller values and `/6` member
   paths since this was drafted), refs gated like `/2` to `/6`; the next free ordinary and coverage
   suite pair (current suites reach `/63`), selected only when a scenario has refs, so ref-free
   suites keep their bytes; `ess-conformance-report/3` and `ess-conformance-run/3`
   (`--report-format 3`). No source format. Go and TypeScript runners admit the new suite pair.
7. **Alternatives.** (a) Change nothing. (b) Compute refs at report time from IR and scenario id:
   authored scenarios still have no home. (c) This story.

## Decisions

- Accept, redesigned: a new scenario format rather than `/1`; command and outcome refs merged in
  author order without duplicates; no scenario-file diff. Format numbers are taken at
  implementation time as the next free ones, since this draft is outside the epic that fixed
  `ess-conformance/64` and `/65`.
- Recording a run as AEP evidence is the AEP side's work; ESS has no AEP dependency.

## Acceptance

- `authored_refs_reach_the_suite`: the authored scenario above, under the new scenario format, is
  compiled and its suite entry carries `refs: [std:REQ-1]`; the same document under `/1` to `/6` is
  refused naming the new format.
- `synthesized_scenario_inherits_subject_refs`: `CreateParent/outcome/created` carries
  `std:REQ-1, std:REQ-2`; a scenario that only arranges a parent in setup carries neither.
- `ref_free_suite_keeps_bytes_and_version`: a specification without refs keeps its suite bytes and
  suite version (`synthesis_suite_bytes_table` rows do not move).
- `refs_select_the_new_suite_pair`: a suite with a scenario that carries refs is written at the new
  ordinary suite major, and its coverage suite at the new coverage major.
- `go_and_ts_runners_admit_the_new_suite_pair`: the Go and TypeScript runners run that suite and
  pass the interpreted target's scenarios.
- `run_3_carries_refs_per_scenario`: the `--report-format 3` run lists each scenario's refs.
- `report_3_indexes_refs_to_outcomes`: the report lists, for each ref, the scenario ids and
  statuses that exercised it.
- `report_2_unchanged`: the `--report-format 2` report bytes are unchanged.

## Scope

- cited: crates/verify/ess-conformance/src/authored.rs
- cited: crates/verify/ess-conformance/src/scenario.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/admission.rs
- inferred: crates/verify/ess-conformance/src/go/mod.rs
- inferred: crates/verify/ess-conformance/src/report.rs
- inferred: crates/verify/ess-conformance/src/counts.rs
- inferred: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/edge/ess-cli/src/main.rs

confidence: medium

Units: B1 suite (authored, scenario, synthesize, admission, Go and TypeScript admission, scenario
schema via `cargo xtask schema`); B2 report (report, counts, runner, CLI), after B1.

## Source

github:beyond10x/ess#503 (part B, the feature half)
