---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-b-pass-1
kind: review-result
status: active
title: Adversary pass 1, unit B (creator predicate guard)
relations:
- reviews: story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard
revision: 1
---
## Adversary pass 1, unit B (ess-0700 wave)

Worktree ess-0700-b at 8d3b82569 plus the implementor's uncommitted diff; adversary added
`crates/verify/ess-conformance/tests/adversary_b_creator_predicate_guard.rs` (13 cases, 3 red).

Run: `cargo test -p ess-conformance --locked --no-fail-fast --test row_sets --test
adversary_b_creator_predicate_guard`, exit 101: adversary `FAILED. 10 passed; 3 failed`, row_sets
`ok. 21 passed`. `--test synthesis_suite_bytes_table` compare mode `ok. 2 passed`.

Held: enum guard; several passable guards; stored guard on an Optional field; related row behind
its own `exists: false`; related row moved to another state; two targets ignoring the guard are
caught; hint wording.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3543,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"related_row fills the row only for refusals that read it alone; an Optional tenant compared with the input stays unwritten and the found branch stays ESS-SYNTH-001, though the interpreter witnesses it"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":2967,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the refusal names a passable guard that compares the row with the input beside the guard that actually refused"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":2980,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"refusal wording: verb does not agree with two guards, the predicate prints in non-authored syntax, and it says every row when one was arranged"}
]
```
