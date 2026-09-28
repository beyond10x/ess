---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w1-names-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: underscore names in schema patterns, browser reader and fuzz'
relations:
- reviews: story:field-names-underscore-and-newtype-map-keys
revision: 1
---
Adversary pass 1 against story:field-names-underscore-and-newtype-map-keys (beyond10x/ess#141, #143), aep:adversary, 2026-09-27.

verdict: red
cases: executed 1265→1280, red 5
origin: introduced 7, pre-existing 0, undecided 0

New cases in `adversary_names_pass1.rs` under ess-domain, ess-primitives, ess-synth and
ess-conformance tests. Red: the published schema still refuses `_`-leading relation names; one
schema pattern keeps the pre-#141 field rule; `FactPath`'s published pattern refuses `_url.host`;
the browser coverage reader refuses a predicate on `_url`; `x == _a.b` is a fact to Rust and a
literal to the browser. Green: Go and Rust output for `_url`/`url` twins, bindings and `_id`
identities compiles; #143 boundary refusals; #143 suite identical to the primitive key.

Coordinator routing: all back to the implementor, with the affected files assigned to the unit.

```findings
[{"file": "crates/specify/ess-domain/src/entity.rs", "line": 698, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "RelationSpec name and via are parsed by deserialize_field_name but publish the pre-#141 pattern, so the committed ess.schema.json refuses names the parser reads."},
 {"file": "crates/verify/ess-conformance/assets/coverage-admission.js", "line": 150, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The browser coverage reader still refuses fact paths starting with an underscore."},
 {"file": "crates/verify/ess-conformance/assets/coverage-admission.js", "line": 228, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "An unquoted operand _a.b is a fact path to Rust, Go and TS and a literal to the browser reader."},
 {"file": "crates/specify/ess-primitives/src/facts.rs", "line": 900, "category": "contract-drift", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "FactPath::PATTERN still refuses _url although FactPath::new admits it."},
 {"file": "fuzz/src/pipeline.rs", "line": 198, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "The fuzz lane parses each document alone, so a cross-file map-key newtype is refused there while the CLI admits it."},
 {"file": "crates/verify/ess-conformance/src/authored.rs", "line": 176, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The doc comment quotes the pre-#141 field pattern."},
 {"file": "crates/generate/ess-synth/src/rust/name.rs", "line": 10, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The module doc states the pre-#141 field pattern."}]
```
