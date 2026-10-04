---
format: aep.planning-md/3
id: review-result:related-guard-behaviour-319-20261004-r1
kind: review-result
status: active
title: 'Generated related guards adversary pass 1: agrees with the interpreter; one test gap kept'
relations:
- reviews: story:related-guard-behaviour
revision: 1
---
unit: W3-1 related-guard-behaviour (#319), pass 1
verdict: CONFIRMED (one warning; generated Rust and Go agreed with the interpreter on every model tried)
cases: executed 54→64, red 0 against the unit's tree (red evidence on a mutated copy)
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (env, logs, emitted copies); build dir and Go cache removed
needs-coordinator: none

Publication copy of the only adversary pass on the #319 unit (worktree `<worktrees>/ess/ess-w3-319-related-guard-behaviour-20261004`, base `d1026d1f0`, uncommitted diff of six files). The reviewer added `crates/generate/ess-synth/tests/adversary_related_guard_behaviour_pass1.rs` (10 cases); no production file edited.

Cases: six admitted models and 21 request sequences compared step by step (outcome, error, carried fields) between generated Rust/Go servers and `interpret::execute::execute` — green; 8 threads and about 1150 requests against the Go entry point under `-race` — green; a related row in another domain owned by one component — green; 91 committed Rust/Go/Web artifacts of billing and gatepass byte-identical — green; `existing_instance:` answering before a missing related row — green; a faulty control where generated code answers `no-shop` before `already-placed` — the reviewer's sequence catches it while the unit's synthesized suite fails 0 scenarios.

Suite: ess-synth lib 16, adversary_go_behaviour_pass2 6, declared_behaviour 32, the new file 10 — all passed. `cargo xtask generate --check`: projections up to date.

Finding (warning, introduced) `crates/generate/ess-synth/tests/declared_behaviour.rs:2008`: no unit test runs generated code against `existing_instance:` answering before a missing related row; a tree answering `no-shop` first passes the synthesized suite with 0 failures. Named fix: keep `adv319_existing_instance_answers_before_a_missing_related_row_*` (adversary file :1584, :1591, :1602, :1615).

Attacked and not broken: both declaration orders with and without wrong_state; Optional input omitted vs null; stored reference absent, dangling, present; held state that also blocks; a branch that sets the reference it read; a first command rewriting the reference; concurrent delete (Go holds one serving mutex from input read to answer; Rust serves one connection at a time); Web and Clap link the Rust behaviour.

```findings
[
  {"file": "crates/generate/ess-synth/tests/declared_behaviour.rs", "line": 2008, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "no test in the unit runs generated code against existing_instance answering before a missing related row; a Rust or Go tree that answers no-shop first passes the synthesized suite with 0 failures, and adversary_related_guard_behaviour_pass1.rs:1602/:1615 catches it"}
]
```
