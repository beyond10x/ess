---
format: aep.planning-md/3
id: review-result:adversary-5w-absent-pass-1
kind: review-result
status: active
title: Adversary pass 1, absent-command-input-outcome (the-5-waves)
relations:
- reviews: story:absent-command-input-outcome
revision: 1
---
Adversary pass 1 against story:absent-command-input-outcome (#170), aep:adversary, 2026-09-28, the-5-waves wave 3.

verdict: NEEDS-CHANGE
cases: added 2, red 2 (ess-domain 802→804)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `crates/specify/ess-domain/tests/adversary_absent_guard.rs`. Red: a negated conjunction containing defined(text) and a disjunction with a missing(text) arm over a required input are refused as never holding. Disk reached 146MB free during the run, so ess-synth and ess-conformance attacks were read, not run.

Coordinator routing: all three to the implementor, with the ess/16 gate decision.

```findings
[{"file":"crates/specify/ess-domain/src/command/absent_input.rs","line":179,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"negated_presence ignores De Morgan, so a satisfiable guard over a required input is refused as never holding."},
 {"file":"crates/specify/ess-domain/tests/absent_input.rs","line":214,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The test and design note endorse refusing any: [text == x, missing(text)] as never holding, which is false for the guard as a whole."},
 {"file":"crates/generate/ess-synth/src/rust/mod.rs","line":103,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The public rust/web/clap workspace entry points refuse Json but not an input_absent branch."}]
```
