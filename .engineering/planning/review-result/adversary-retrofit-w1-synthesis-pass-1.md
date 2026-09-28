---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w1-synthesis-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: stored-field any, near-zero count boundary, stored-path literal write and the view hint'
relations:
- reviews: story:synthesis-kills-connective-and-source-mutants
revision: 1
---
Adversary pass 1 against story:synthesis-kills-connective-and-source-mutants (beyond10x/ess#154, #155, #132, #160, #161), aep:adversary, 2026-09-27.

verdict: red
cases: executed 655→664, red 4
origin: introduced 1, pre-existing 0, undecided 3

Cases in `crates/verify/ess-conformance/tests/adversary_connective_and_source_mutants.rs`. Red:
stored-field `any:` witnessed only on a row satisfying every disjunct (#155); `digits.count <= 0`
and `< 0` survive against `<= 1` / `< 1` (#160); a literal write behind a stored-field guard is
arranged over a row already holding the value (#161); the new help line offers an eventual view
where an immediate view is required. Green: #160 on Integer guards with all four operators; #161
with three same-typed inputs and with an enum; #161 on a moves branch; #132 with a published field;
#154 with three states, each source dropped. The two rewritten existing assertions
(`retained_replay.rs`, `string_alphabet_and_length.rs`) still protect what they did.

Coordinator routing: all four defects are inside the unit's issues and go back to the implementor;
origin "undecided" rows are decided as in scope of this unit.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 1122, "category": "coverage", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "a stored-field any: guard is witnessed only on a row satisfying every disjunct, so a stored-field all-to-any connective mutant survives"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5704, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "with_count cannot resize an empty refusing witness, so the accepting side of digits.count <= 0 and < 0 is never sent and the moved boundary survives"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 1843, "category": "coverage", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "the routed stored-field path returns before the literal-write re-arrangement, so a literal write is witnessed over a row already holding that value"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 710, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the new view hint claims an eventual view suffices for every NoWitness reason mentioning a view, including subject-state selection and state-guard preservation that require immediate views"},
 {"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 1480, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "an eventual Contains check after a refusal is satisfied by a lagging projection on its first poll, so published fields are not reliably checked though the PartialObservation note implies they are"}]
```
