---
format: aep.planning-md/3
id: review-result:adversary-n-reader-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-next unit reader
relations:
- reviews: story:a-reader-side-conformance-admits-reader-widening
revision: 1
---
unit: story:a-reader-side-conformance-admits-reader-widening
verdict: red
cases: executed 57→69, red 9
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/reader/adv1/{red-alone.log,suite.log,docs.log,cli.log,xtask.log}
needs-coordinator: yes

Adversary pass 1 (`aep:adversary`), 2026-09-28, head a15c2a9cd + `crates/specify/ess-composition/tests/adversary_reader_conformance.rs` (3 passed, 9 failed alone).

- Json read as Map<String, Json> admitted (:258), also inside Map (:268), List (:278), Optional (:287) and via a Json newtype (:296): rejects arrays and scalars.
- Extra local field named/wire-named as an omitted producer field read as another type admitted (:317, :330), also inside a union variant (:341).
- `/2` with `reader: null` compiles (:414); released 0.38.0 refused it via deny_unknown_fields.
- Controls green: plain field subset (:308); /3 without reader equals /2 over 9×10 pairs (:393); /1 refuses reader (:443).

`cargo test -p ess-composition --no-fail-fast`: EXIT=101 (adversary 3/12; others green). cargo xtask docs EXIT=0; ess-xtask EXIT=0; ess-cli composition binaries 46 passed.

Held: Optional producer vs required consumer at every level; enum→String Optional handling and inside List/Map; wire-name superset; newtype→narrower primitive; presence on shared fields; /2 refusing reader: false; FORMAT_RELEASES row.

```findings
[{"file":"crates/specify/ess-composition/src/conformance.rs","line":358,"category":"acceptance","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"Reader widening admits producer Json read as Map<String, Json> (also through Map, List, Optional and a Json newtype), which rejects arrays and scalars the producer sends."},{"file":"crates/specify/ess-composition/src/conformance.rs","line":173,"category":"acceptance","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"An extra optional local field is matched by name only, so one whose wire name equals an omitted producer field wire name is admitted while reading that key as a different type."},{"file":"crates/specify/ess-composition/src/lib.rs","line":1366,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"An ess-composition/2 document carrying reader: null deserializes to None and compiles, where released /2 refused any reader key."},{"file":"crates/generate/ess-gen/src/types.rs","line":110,"category":"judgement","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Decision 6 field subset is admitted although ESS projects every struct closed (additionalProperties false, deny_unknown_fields), so a generated reader type rejects producer values carrying the omitted fields; reach not shown."},{"file":"crates/specify/ess-composition/tests/owner_types.rs","line":546,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The unit test and the /3 docs (formats.md, spec-versions.md, design page) assert the Json-as-map admission and claim nothing that could reject a producer value is admitted."}]
```

Coordinator routing: F1 fix (Map<String, Json> only for an object-shaped producer; bare Json and Json newtypes stay drift); F2 fix; F3 refuse null; F4 decision: keep the subset (it is #191's core) and state in docs and the key's description that `reader: true` asserts a tolerant reader (unknown fields ignored), which ESS-generated closed types are not; F5 follows F1.
