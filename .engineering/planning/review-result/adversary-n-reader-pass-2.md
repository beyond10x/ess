---
format: aep.planning-md/3
id: review-result:adversary-n-reader-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-next unit reader
relations:
- reviews: story:a-reader-side-conformance-admits-reader-widening
revision: 1
---
unit: story:a-reader-side-conformance-admits-reader-widening
verdict: red
cases: executed 70→85, red 2
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/reader/adv2/red-alone.log, suite.log
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head d6d073d0d + `crates/specify/ess-composition/tests/adversary_reader_conformance_pass2.rs` (13 green controls, 2 red). Pass-1 F1–F5 fixed (pass-1 file 12/12). No case found where reader admits a consumer that rejects a producer value; both reds are over-refusals.

`cargo test -p ess-composition --no-fail-fast`: EXIT=101; adversary_compose_conformance 10, adversary_compose_pass2 9, adversary_reader_conformance 12, adversary_reader_conformance_pass2 13/15, composition 12, owner_types 27.

Held: struct through two newtypes read as Map<String, Json>; Optional struct null_when_absent vs required map drift; enum not object-shaped; wire-name case variants; extra field sharing a wire name compared by type; nested wire name not top-level; non-boolean reader values refused (JSON and YAML); /3 reader false/null/empty = no reader; /2 refuses reader; /3 without reader = /2 on 9×12 pairs.

```findings
[{"file":"crates/specify/ess-composition/src/conformance.rs","line":386,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"object_shaped treats only String-keyed maps as JSON objects, so a Map<Integer, _> producer read as Map<String, Json> is refused although every value it sends is an object the reader accepts."},{"file":"crates/specify/ess-composition/src/conformance.rs","line":160,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Under reader a local field sharing a producer field name is matched by name and refused for its wire name, while extra fields are matched by wire name, so a local field that reads another producer field key with a conforming type is refused."}]
```

Trend: pass 1 → 5, pass 2 → 2 notes, carried 0. Coordinator routing: both deferred to story:reader-conformance-over-refusals (fail-safe over-refusals); the two cases are rewritten to assert today's refusal with a message naming that story; outcome no-op.
