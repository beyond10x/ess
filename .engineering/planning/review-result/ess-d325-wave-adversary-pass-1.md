---
format: aep.planning-md/3
id: review-result:ess-d325-wave-adversary-pass-1
kind: review-result
status: active
title: The D325 wave adversary pass 1
summary: 'NEEDS-CHANGE then fixed: F1-F5 fixed in 61cbb58d2; J1 and J2 accepted'
relations:
- reviews: story:a-structured-input-may-carry-a-fixed-example
- reviews: story:a-fixture-may-be-a-row-the-deployment-already-holds
- reviews: story:a-transition-and-a-state-may-carry-a-summary
revision: 1
---
## Pass

Adversary pass 1 over `integration/ess-d325` at c196f5ce8 (U1 d36fcb59f, U2 83614b8dc, U3 93dd27e9c merged), base cc7345d24. Verdict NEEDS-CHANGE: 1 blocker, 3 warnings, 1 note, 2 judgement notes. Cases executed 85 to 95, 5 red, all introduced by the wave. Tests: `crates/specify/ess-domain/tests/adversary_d325_pass1.rs`, `crates/verify/ess-conformance/tests/adversary_d325_pass1.rs`, `adversary_d325_runtimes.rs` and its Go and JavaScript fixtures.

## Attacked and not broken

An example breaking its struct's invariant is refused; a closed struct nested in an open example refuses undeclared members; list, map, `Json` and newtype members of a struct example are sent as written; an example on a provided-guard identity yields to the fixture; a provided row in the state a guard needs passes beside lifecycle summaries; a subject command on a provided entity is refused by name; adding or removing `provided` diffs as `unclassified-changed`.

## Outcome

Fixed in 61cbb58d2 on `integration/ess-d325`: F1, F2, F3, F4 and F5. F4's adversary assertion expected the example sent as written; a guard over an absent `Optional` is undecided, so it now expects the example's `version` with a `note` that is not `urgent`, its message naming the story. J1 accepted: the behaviour is covered by `provided_rows*.rs` and the ess-cli case named in the finding. J2 accepted: no caller validates a command against a registry with no recorded format.

```findings
[
{"file":"crates/specify/ess-domain/src/command/fixture_inputs.rs","line":86,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a fixture_inputs binding over an input named like a related row's or subject's stored field read by a row, row-set or subject predicate is refused at every format, though no guard reads the input"},
{"file":"crates/verify/ess-conformance/src/interpret/provided.rs","line":58,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the interpreted target mints a provided row's fields without regard to entity invariants and reports unsupported for a valid model"},
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":4251,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"two via-input guards over one provided entity refuse every scenario of the command, not by name, where a created entity synthesizes clean"},
{"file":"crates/verify/ess-conformance/src/witness.rs","line":5818,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"an Optional member a struct example leaves out stays absent even where a guard needs it, so the command's whole suite is refused"},
{"file":"crates/verify/ess-conformance/src/go/fixtures.go","line":62,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"Go and TypeScript admit an entity fixture with empty entity and state at suite/60 that the Rust reader refuses"},
{"file":"crates/verify/ess-conformance/tests/provided_rows.rs","line":null,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"four acceptance-named U2 tests do not exist under their stated names; equivalents exist under other names"},
{"file":"crates/specify/ess-domain/src/command.rs","line":6197,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"a registry with no recorded format admits structured examples as if at ess/33"}
]
```
