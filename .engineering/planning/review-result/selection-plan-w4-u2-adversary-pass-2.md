---
format: aep.planning-md/3
id: review-result:selection-plan-w4-u2-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 4 unit U2 (synthesis bytes pin)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: story:synthesis-reads-selection-plan unit 1 (U2 synthesis bytes pin), pass 2, uncommitted tree at base 373df4ecba
verdict: nothing found
cases: executed 4→6, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/ess-selection-plan/w4-u2-scratch/adversary-1/ (2.1G)
needs-coordinator: none
```

Added `synthesis_suite_bytes_table_claim_fixtures.rs`: each of the 12 `fixtures/claim-search/` models equals its in-test builder byte for byte, and every fixture has a builder (green).

Walkers, package-scoped, `--no-fail-fast`: ess-conformance 72 passed; ess-domain 44; ess-compiler 16 (1 ignored, pre-existing); ess-entity-runtime 18; ess-synth 68 (2 ignored dump-only); ess-gen 3; ess-xtask 14; ess-cli 8. ess-ui-react walks only its own generated `src/`.

Component table: `synthesize_for` splits the whole suite (`synthesize.rs:2411-2455`), component order from a `BTreeMap`; two runs identical.

Mutants (18, scratch copy): every one a #464 test catches now turns the table red, including `held_state_claims` without its guard (door-rough moved). Eight stay green under every test run (`related_guard` rfind / no related, three `row_set` order flips, `authored::not_taken`, `reaches_external` without held / without earlier); equivalence is unknown without a full-crate run. Six of them sit on units 4 and 5's code.

```findings
[]
```
