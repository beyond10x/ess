---
format: aep.planning-md/3
id: review-result:parameterized-transport-391-20261004-r1
kind: review-result
status: active
title: 'Parameterized transport adversary pass 1: publishers block on acknowledgements'
relations:
- reviews: story:feature-request-391
revision: 1
---
unit: U3 #391 parameterized NATS event addresses, pass 1
verdict: NEEDS-CHANGE
cases: executed 35→45, red 5 (plus 4 red cases inside the generated Rust/Go packages these tests build)
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (env, logs, probes); build dir and Go cache removed
needs-coordinator: (a) the F1 fix breaks two implementor tests that assert the blocking; (b) whether the format registry and reference rows (F4) belong to this unit

Publication copy of the only adversary pass on the #391 unit (worktree `<worktrees>/ess/ess-parameterized-transport-20261003`, HEAD `da72325cb` with a staged merge of `a3bf31579` and uncommitted fixes). The reviewer added `crates/generate/ess-publisher/tests/adversary_391_pass1.rs` and `crates/specify/ess-transport/tests/adversary_391_pass1.rs`; no production file edited. No containers started.

Suite: ess-transport and ess-publisher with the new files 40 executed, 5 failed; ess-gen asyncapi_transport 10 passed, 1 ignored. Each publisher behaviour runs against a released `/1` package (green) and a `/2` package (red).

F1 (blocker) `crates/generate/ess-publisher/src/rust_dynamic_support.rs.txt:155` (Go `go_dynamic_support.go.txt:106`): one worker serves commands and performs transport I/O; `request` holds the commands lock across the reply, the worker sends inline, so every array publish blocks on another message's broker acknowledgement, violating the released buffer-and-return contract (`docs/design/event-publishers.md`) that the /1 control still meets. Reached by every dynamic package with a real transport.

F2 (warning) `crates/generate/ess-publisher/tests/generated.rs:869` and `:1084`: the implementor's tests assert that a publish stays blocked during an in-flight drain, encoding F1 instead of FIFO only.

F3 (warning) `rust_dynamic_support.rs.txt:131` (Go `:88`, `:101`): `on_error` runs on the operation worker, so a publish from the callback deadlocks the operation permanently (Go also holds `b.mu`, hanging Close).

F4 (warning) `website/docs/reference/formats.md:162`: `ess-transport/2`, `ess-transport-ir/2`, `ess-client-report/2` and `ESS-TRANSPORT-017`–`020` appear in no reference page or `FORMAT_RELEASES` row (`ess-xtask/src/docs.rs:263-265`); formats.md still says refusals 001–016.

F5 (warning) `crates/specify/ess-transport/src/lib.rs:153`: a parameter key repeated with two sources in JSON or YAML compiles with the last source silently winning (`BTreeMap` overwrite) instead of being refused; /1 refuses duplicate struct keys.

F6 (note) `generated.rs:1495`, `asyncapi_transport.rs:316`: the actual-NATS and AsyncAPI-validator controls are `#[ignore]`d; CI runs both explicitly and `ci_lanes.rs` guards they are not skipped.

Attacked and not broken: template admission, event-path typing at every depth, wire renames and RFC 6901 escaping, 6 correlated stream-overlap vectors, IR2 mapping-order independence, AsyncAPI CLI acceptance of repeated and single-envelope addresses, Rust/Go token-rule parity, generated accessors, Go concurrent publish/flush/close under -race.

```findings
[
  {"file": "crates/generate/ess-publisher/src/rust_dynamic_support.rs.txt", "line": 155, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Dynamic Rust and Go array publishers block every publish call on another message's broker acknowledgement (worker does I/O inline; go_dynamic_support.go.txt:106), violating the released buffer-and-return contract that the /1 control still meets"},
  {"file": "crates/generate/ess-publisher/tests/generated.rs", "line": 869, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Rust test (and Go at :1084) asserts that a publish stays blocked during an in-flight drain, encoding the F1 regression instead of FIFO only"},
  {"file": "crates/generate/ess-publisher/src/rust_dynamic_support.rs.txt", "line": 131, "category": "concurrency", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "on_error runs on the operation worker, so a publish from the callback deadlocks that operation permanently in Rust and Go (go_dynamic_support.go.txt:88), where /1 accepts it"},
  {"file": "website/docs/reference/formats.md", "line": 162, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "ess-transport/2, ess-transport-ir/2, ess-client-report/2 and ESS-TRANSPORT-017..020 appear in no reference page or FORMAT_RELEASES row, and formats.md still states refusals 001-016"},
  {"file": "crates/specify/ess-transport/src/lib.rs", "line": 153, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A parameter key repeated with two different sources in JSON or YAML compiles with the last source silently winning instead of being refused"},
  {"file": "crates/generate/ess-publisher/tests/generated.rs", "line": 1495, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The actual-NATS and AsyncAPI-validator controls are #[ignore]d (also asyncapi_transport.rs:316) against the wave rule, mitigated only by explicit CI invocation"}
]
```
