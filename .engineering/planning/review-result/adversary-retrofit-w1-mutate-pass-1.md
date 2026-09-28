---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w1-mutate-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: mutate --emit/--collect holds under 15 attacks, three notes'
relations:
- reviews: story:mutate-drives-an-external-target
revision: 1
---
Adversary pass 1 against story:mutate-drives-an-external-target (beyond10x/ess#153), aep:adversary, 2026-09-27.

verdict: nothing found
cases: executed 668→683, red 0
origin: introduced 0, pre-existing 0, undecided 0

15 new cases in `crates/verify/ess-conformance/tests/mutation_external_adversary.rs` and
`crates/edge/ess-cli/tests/mutate_external_adversary.rs`, all passing on first run: another
implementation's report, a failing report/2 baseline (ESS-MUTATE-001), a report naming an unknown
scenario, a mutant suite replaced by the baseline's, Go `skipped` failures, 13 path-escape spellings
in manifest `dir`, an unknown report format, `--collect` and `--emit` exit codes and outputs.
`cargo test -p ess-conformance --locked` exit 0; `cargo test -p ess-cli --locked --test mutate_external
--test mutate_cli --test mutate_external_adversary` exit 0; 683 passed across 68 binaries.

Notes (no failing test): the collected report spells the implementation `<name> <version>` where
`--target` writes `<name>`; `--collect` does not bind a mutant entry's `dir` to its id or its
`mutant.json`; `manifest.json` is written mid-emission, so an interrupted emit leaves a manifest
naming unwritten suites (safe outcome: inconclusive, exit 3; next emit refuses the non-empty dir).

```findings
[{"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 1836, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a collected ess-mutation-report/1 spells implementation as <name> <version> while --target writes <name>, so the same audit differs in that field by mode"},
 {"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 1785, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "collect does not bind a mutant entry's dir to its id or to the mutant.json beside it, so a hand-edited manifest can score one mutant from another's suite"},
 {"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 1633, "category": "concurrency", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "manifest.json is written in the middle of the emission rather than last, so an interrupted emit leaves a manifest naming unwritten suites and a directory the next emit refuses"}]
```
