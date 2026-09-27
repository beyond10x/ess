---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w1-synthesis-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: routed search criterion, 64-candidate witness limit'
relations:
- reviews: story:synthesis-kills-connective-and-source-mutants
revision: 1
---
Adversary pass 2 against story:synthesis-kills-connective-and-source-mutants (beyond10x/ess#154, #155, #132, #160, #161), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE
cases: executed 664→670, red 2
origin: introduced 1, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary2_connective_and_source_mutants.rs`.
Red: on the stored-field path, one literal write no row can change makes the search fall back to a
row that also leaves a changeable write unchanged; a three-input nested guard
`all: [any: [a>10, b>10], c>10]` is refused after 64 witness candidates. Green: nested input-guard
connectives with two inputs, nested stored-field any-over-all, Decimal and Timestamp boundary
mutants, committed generated suites unchanged against fresh synthesis, help-line routing of every
NoWitness reason.

Coordinator routing: finding 1 back to the implementor (final round); finding 2 decided out of
this unit (the witness search limit is untouched by the diff) and filed as its own story; finding 3
accepted (the issue asks to prefer).

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 796, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the routed stored-field search accepts only a row where every write changes, so one unchangeable literal write makes it fall back to a row that also leaves a changeable write unchanged, unlike the plain path's fewer-unchanged criterion"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5855, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "undecided", "message": "a guard over three Integer inputs such as all:[any:[a>10,b>10],c>10] is refused after 64 candidates, so neither the branch nor its connective mutants get a scenario; the limit is in witness search code this diff does not touch"},
 {"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 802, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "when no arrangement can change a written value the search silently keeps a non-distinguishing row and emits no note, which the issue's prefer wording allows but leaves the surviving sets mutant unreported"}]
```
