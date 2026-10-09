---
format: aep.planning-md/3
id: story:requirement-refs-reach-suite-and-report
kind: story
status: draft
title: Requirement refs on authored scenarios reach the suite and the run report
refs:
- provider: github
  reference: beyond10x/ess#503
relations:
- serves: vision:O2
revision: 3
---
## Outcome

Authored scenarios accept `refs:` and the suite carries them; a synthesized scenario inherits the
refs of the command and outcome its id names (never those of setup outcomes); the run report lists
refs per scenario with an index from each ref to its scenario outcomes; and `ess verify diff` rates
a change to `refs` alone as documentation. Requested in https://github.com/beyond10x/ess/issues/503
(part B): a specification traced to a published standard cannot show which scenario proved which
requirement, because refs stop at the compiled IR (`ess-compiler/src/ir.rs:869,1693,2444,2625`).

## Fit review

1. **Need.** Show, per requirement reference, which scenarios exercised it and how they ended.
2. **Class.** Gap: authored scenarios refuse the key (`authored.rs:352-375`, `deny_unknown_fields`);
   suite and report carry nothing. The diff part is a defect against its own documentation-only
   rule: a refs-only edit is `unclassified-changed` (`ess-diff/src/diff.rs:2664,2716`), where it
   belongs with `summary` changes (`compatibility.rs:652-700`).
3. **Already expressible?** For synthesized scenarios only by an external join of scenario id and
   IR refs; `ConformanceScenario.source` (`scenario.rs:1237`) includes setup outcomes and would
   over-attribute. No idiom for authored scenarios or the report.
4. **Fit.** Same `ExternalRef` spelling (`refs.rs:33-89`) and key. Report reuses the `Outcomes`
   id-list shape (`counts.rs:118-125`). Scenario files are not `ess verify diff` inputs, so a
   scenario-refs diff is declined.
5. **Second adopter.** A service tracing outcomes to an internal control catalogue and reporting
   per control.
6. **Cost.** `ess-scenario/5` (refs gated like `/2`-`/4`, `authored.rs:2181-2215`); suite pair
   `ess-conformance/48`/`49` selected only when a scenario has refs, so ref-free suites keep their
   bytes (the suite reader does not deny unknown fields, `scenario.rs:1212`, so an old reader would
   drop refs silently without the bump); `ess-conformance-report/3` and `ess-conformance-run/3`
   (`report.rs:510` denies unknown fields), `--report-format 3`; `ess-diff/17`. No source format.
   Go and TypeScript runners admit the new suite pair (`go/mod.rs:370`, `admission.rs:209,266`).
7. **Alternatives.** (a) Change nothing. (b) Compute refs at report time from IR and scenario id:
   authored scenarios still have no home. (c) This story.

## Decisions

- Accept, redesigned: scenario format `/5` rather than `/1`; command and outcome refs merged in
  author order without duplicates; no scenario-file diff.
- Recording a run as AEP evidence is the AEP side's work; ESS has no AEP dependency.
- Part C (a requirements register) is `story:requirement-completeness-without-prose-in-the-model`.

## Acceptance

- `authored_refs_reach_the_suite`; `/1`-`/4` with `refs:` refused naming `/5`.
- `synthesized_scenario_inherits_subject_refs` (setup outcomes contribute nothing).
- `ref_free_suite_keeps_bytes_and_version`; `refs_select_suite_48_and_coverage_49`;
  `go_and_ts_runners_admit_48_49`.
- `run_3_carries_refs_per_scenario`; `report_3_indexes_refs_to_outcomes`; `report_2_unchanged`.
- `refs_only_change_is_documentation` at `ess-diff/17`; `refs_change_below_17_stays_unclassified`.

## Scope

| unit | files | after |
|---|---|---|
| B1 suite | `ess-conformance/src/{authored,scenario,synthesize,admission}.rs`, `go/mod.rs`, TS admission, new `requirement_refs.rs`, scenario schema (`cargo xtask schema`) | none |
| B2 report | `ess-conformance/src/{report,counts,runner,lib}.rs`, `ess-cli/src/main.rs` | B1 |
| B3 diff | `ess-diff/src/{diff,change,compatibility,delta}.rs` | none |
