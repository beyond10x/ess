---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-b-pass-2
kind: review-result
status: active
title: Adversary pass 2, unit B (creator predicate guard)
relations:
- reviews: story:a-row-set-branch-arranges-rows-past-the-creators-predicate-guard
revision: 1
---
## Adversary pass 2, unit B (ess-0700 wave)

Worktree ess-0700-b at 8d3b82569 plus the implementor's correction round 1, uncommitted;
adversary added `crates/verify/ess-conformance/tests/adversary_b_creator_predicate_guard_2.rs`
(20 cases, 7 red). Run with `row_sets` (22 ok), pass 1's binary (13 ok) and
`synthesis_suite_bytes_table` (2 ok, all 349 pinned suites unchanged): exit 101.

Held: two Optionals compared in separate or one refusal; an Optional struct field; an Optional
creator input; a field no command writes; plain text literals and `now - 60s` read back;
determinism.

Coordinator disposition (two passes spent): the four introduced findings go to a final correction
round that the coordinator verifies; the struct-member and later-writer findings are deferred to
`story:a-related-row-optional-reached-through-a-member-or-a-later-command-is-arranged`.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3443,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"compared_optional writes an Optional the refusal reads in its refusing polarity, so a witnessable found is ESS-SYNTH-001"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":2975,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"passes judges one refusal unpassable beside its opposite and names it alone for every input, though alone it is witnessed"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":2977,"category":"judgement","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"contradictory refusals are named by one side only"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":2990,"category":"boundary","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"printed predicate drops text quotes; a literal with a comma reads back as a different predicate"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3451,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"an Optional member of a struct compared with the input is never written; witnessable found refused"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3609,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"an Optional written only by a later update command is not arranged; witnessable found refused"}
]
```
