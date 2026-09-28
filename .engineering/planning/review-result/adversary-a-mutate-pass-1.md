---
format: aep.planning-md/3
id: review-result:adversary-a-mutate-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit mutate
relations:
- reviews: story:gained-refusals-are-unwitnessed-not-survived
- reviews: story:mutate-scores-a-baseline-with-skipped-scenarios
revision: 1
---
unit: mutate (#203, #210)
verdict: red
cases: executed 58→62, red 3
origin: introduced 3, pre-existing 1, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/mutate/adv1/
needs-coordinator: yes

Adversary pass 1 (`aep:adversary`), 2026-09-28, head fbc8634e4 + `crates/verify/ess-conformance/tests/adversary_mutate_pass1.rs` (1 passed, 3 failed alone). Held: no killed→survived on reference targets; judge keeps Killed above a gained refusal; exit codes; byte-identical reports; manifest /1 and /2 handling; xtask docs and support checks.

```findings
[{"file":"crates/verify/ess-conformance/src/mutate.rs","line":1555,"category":"property","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"added_refusals is a set difference over (code, scenario) keys that repeat in real suites (gatepass SYNTH-011 x2 per scenario), so an extra or traded same-key refusal is not counted as gained and /2 says survived where /1 says unwitnessed"},{"file":"crates/verify/ess-conformance/src/mutate.rs","line":1539,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"with --target interpreted the billing order-flip mutant is reported survived (exit 1) although every scenario that kills it on the reference is in its excluded set, contradicting the documented meaning of a survivor"},{"file":"crates/edge/ess-cli/src/main.rs","line":605,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"exit 0 is documented as the baseline passed but now also occurs with unexecuted baseline scenarios (17 of 32 not scored on billing interpreted)"},{"file":"website/docs/guides/verify-conformance.md","line":327,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"the guide still says replaying mutant suites in an adopter own language is not implemented, though --emit/--collect exist since 0.37.0"}]
```

Coordinator routing: F1 fixed in-unit (per-key counts, subject in the key); F2 decided: a mutant that no executed scenario kills while any of its own scenarios was excluded is `inconclusive`, not survived (new verdict in /2, exit 3, listing the excluded scenarios); F3 and F4 doc fixes in-unit.
