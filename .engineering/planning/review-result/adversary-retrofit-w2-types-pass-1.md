---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-types-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: presence suites unreadable and never exercised'
relations:
- reviews: story:field-wire-names-and-presence-policy
- reviews: story:json-values-and-text-patterns
revision: 1
---
Adversary pass 1 against story:field-wire-names-and-presence-policy (#142, #139) and story:json-values-and-text-patterns (#146, #138), aep:adversary, 2026-09-27.

verdict: NEEDS-CHANGE (2 blockers)
cases: executed 1695→1714, red 7
origin: introduced 4, pre-existing 0, undecided 0

New cases in `adversary_types_pass1.rs` under ess-conformance, ess-domain, ess-gen and
ess-entity-runtime tests. Red: suite/24 and /25 are refused by the execution reader and the coverage
writer; a correct and a swapped presence implementation cannot be run; no invocation leaves a
policy field absent; a response field declared null_when_absent passes when omitted; a narrowing
prefix chain lowers duplicate rule names. Held: #142 both spellings with presence; #146 prefix
checks, escaping, narrowing chains, the issue repro; #138 literal refusal, map-key refusal,
structural comparison.

Coordinator routing: all six to the implementor; admission.rs, witness.rs and response.rs assigned
to the unit; F5 decided as "refuse omitted_when_absent on Optional<Json>".

```findings
[{"file": "crates/verify/ess-conformance/src/admission.rs", "line": 184, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The execution reader admits only suite majors 1-19, so suites 24 and 25 are refused and no runner can execute a model with presence policies."},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 326, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Optional inputs are omitted only for defined() guards, so no invocation leaves a presence-declared field absent and a swapped implementation passes."},
 {"file": "crates/verify/ess-conformance/src/response.rs", "line": 167, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The response observation ignores presence, so a response field declared null_when_absent passes when the key is omitted."},
 {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 1005, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Each layer of a narrowing prefix chain lowers a rule with the same name."},
 {"file": "crates/verify/ess-conformance/src/scenario.rs", "line": 1601, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "On Optional<Json> under omitted_when_absent a present JSON null is refused as a forbidden absence."},
 {"file": "crates/specify/ess-domain/src/primitive_admission.rs", "line": 77, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The comment and design page list view fields among the presence positions, but views refuse presence."}]
```
