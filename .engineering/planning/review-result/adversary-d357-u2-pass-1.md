---
format: aep.planning-md/3
id: review-result:adversary-d357-u2-pass-1
kind: review-result
status: active
title: 'D357 U2 adversary pass 1: the startup scan''s reach and the history input case'
relations:
- reviews: story:a-marked-region-is-not-a-scan-of-what-runs
- reviews: story:concurrent-history-records-inputs
revision: 1
---
unit: D357 U2 (story:a-marked-region-is-not-a-scan-of-what-runs, story:concurrent-history-records-inputs), 14979ec9e on unit/ess-d357-u2, reviewed against 700b4faa6
verdict: NEEDS-CHANGE (1 blocker, 4 warnings, 2 notes, 2 out of scope); fixed at 7619c026d
cases: executed 35→43, red 6
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u2/adv-scratch (copies and cargo logs only)
needs-coordinator: none

**Cases.** `crates/edge/ess-cli/tests/adversary_d357_u2_pass1.rs` carried a byte copy of the guard's scan (checked equal to the guard's text) and synthetic sources: a generic method with an `Fn` bound called by `.f(`, free functions named `self::f` and `module::f`, `Display::fmt` run by `format!` and `From::from` run by `?`, a `panic!` inside a same-file `macro_rules!`, `Option::expect(x, ..)` and `.map(Result::unwrap)`, a real fixture case where an upgraded stand-in answers `session.new` with a close frame, and the row-10 history in `ess-history/3` without its input. Six red, two green.

**Findings.**
- F1 (blocker, introduced), `coverage_browser.rs:869`: a method whose generics hold an `Fn(..)` bound was filed as non-method, so its panic went unreported against the doc comment. Fixed at 7619c026d: generics are skipped before the parameter list.
- F2 (warning, introduced), `coverage_browser.rs:998`: a path-named free function was not followed. Fixed: followed when the prefix is not a same-file type.
- F3 (warning, introduced), `coverage_browser.rs:1094`: trait impls run without their name were not followed. Fixed: every method of a same-file `impl Trait for T` counts as reached once the type is named.
- F4 (warning, introduced), `coverage_browser.rs:1094`: a same-file `macro_rules!` body was never read. Decision: refuse. Fixed: a source defining `macro_rules!` is refused naming the macro; the doc comment says so.
- F5 (note, introduced), `coverage_browser.rs:554`: path spellings of the panic forms were missed. Fixed: six path forms added, matched on a name boundary.
- F6 (warning, INFEASIBLE, introduced), `support/browser.rs:735`: a close frame on the session round trip panicked under a `defect` mark whose reason was untrue. Fixed: a startup refusal at the session stage; the case lives in `browser_startup_refusal_boundary.rs` (a test binary including the browser fixture must be listed for serial runs, which the adversary file is not).
- F7 (note, introduced), `support/browser.rs:742`: an error answer to `session.new` was marked `defect`. Fixed: a startup refusal naming the error.

**Out of scope (rows at wave close).** `session.new` is bounded by its own session timeout, not the caller's startup deadline; `std::process::exit` and `abort` are not among the forms (the doc comment now says so).

**Attacked, not broken.** Strings, raw strings, char and byte literals, nested block comments and lifetimes did not cut a function short; drops reached through fields are followed; the real fixture has no reachable site the scan misses; the stage asserts hold; row 10's `Violation` comes from the recorded input, not the format.

**Fix-round run (7619c026d).** coverage_browser 22, browser_startup_refusal_boundary 8, browser_startup_slow_serve_boundary 6, check_history 6, adversary_d357_u2_pass1 1, with Firefox; the red run on 14979ec9e failed 5 and 2. clippy -D warnings and fmt-check clean.

```findings
[
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":869,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a same-file method whose generics hold an Fn(..) bound was filed as non-method, so a panic in it reached by .f( went unreported; fixed at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":998,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a free function named by path (self::f, module::f) was never followed; fixed at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":1094,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"trait impls run implicitly (Display via format!, From via ?) were not followed; fixed at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":1094,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a panic! inside a same-file macro_rules! was never read; such a source is refused at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":554,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"PANIC_FORMS missed path spellings of its own forms; fixed at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/support/browser.rs","line":735,"category":"contract-drift","severity":"warning","verdict":"INFEASIBLE","origin":"introduced","message":"a close frame on session.new panicked under a defect mark instead of the lost-start refusal; a startup refusal at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/support/browser.rs","line":742,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"a well-formed error answer to session.new was marked defect; a startup refusal naming the error at 7619c026d"},
{"file":"crates/edge/ess-cli/tests/support/browser.rs","line":491,"category":"judgement","severity":"note","verdict":null,"origin":"undecided","message":"out of scope - session.new is bounded by its own session timeout, not the caller's startup deadline"},
{"file":"crates/edge/ess-cli/tests/coverage_browser.rs","line":554,"category":"judgement","severity":"note","verdict":null,"origin":"undecided","message":"out of scope - std::process::exit and abort end a start and are not among the forms"}
]
```
