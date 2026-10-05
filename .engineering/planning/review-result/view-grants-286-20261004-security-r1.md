---
format: aep.planning-md/3
id: review-result:view-grants-286-20261004-security-r1
kind: review-result
status: active
title: 'View grants security review 1: runner verdict parity, no-actor read, ui-check reads'
relations:
- reviews: story:feature-request-286
revision: 1
---
unit: ess-w3-286-view-grants (#286 view read grants), security review pass 1
verdict: NEEDS-CHANGE
cases: executed 25→35, red 8
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: review scratch logs; build dir and Go cache removed
needs-coordinator: yes — finding 1 needs one verdict for both runtimes; finding 5 is pre-existing

Publication copy of the only (security) review pass on the #286 unit (worktree `<worktrees>/ess/ess-w3-286-view-grants-20261004`, base `3c040623c`, uncommitted diff of 31 files). The reviewer added `crates/verify/ess-conformance/tests/view_grant_security.rs`, `crates/ui/ess-ui-check/tests/view_grant_security.rs` and `crates/generate/ess-synth/tests/view_grant_security.rs`; no production file edited.

F1 (blocker, introduced) `crates/verify/ess-conformance/src/runner.rs:977`: an unexpected read refusal is `error` in the Rust runner and `failed` in Go (`go/runtime.go:2583`, `:2693`), so the runners disagree on the admitted scenario for a target that refuses a granted reader.

F2 (warning, introduced) `crates/verify/ess-conformance/src/synthesize/view_grant.rs:184`: no synthesized scenario reads a read-granted view as no actor (`ExpectNotGranted` requires an actor), so a Rust or Go server whose read check lets no actor through passes every read-grant scenario; a view every actor may read gets no denied scenario.

F3 (warning, introduced) `crates/ui/ess-ui-check/src/model.rs:381`: an `actor: anonymous` document reading a read-granted view checks clean, though the served surface refuses every request with no actor (comment at :378 still says ESS has no read grants).

F4 (warning, introduced) `model.rs:457`: a page whose actor may not read a read-granted view it reads checks clean; `page_actor_grants` checks commands only.

F5 (warning, pre-existing) `crates/generate/ess-synth/src/rust/entry.rs:61` and `go/entry.rs:176`: demonstration actor-header authentication reads only the first `Authorization` header, so Clerk followed by Watcher is served the read-granted view.

Reviewed and not faulted: 403 with no row for no, malformed, unknown, wrong-case and foreign-scheme actors in Rust and Go; every method gives the same refusal; grant checked before reading or decoding parameters on both paths; one route per view; OpenAPI and server agree; models without view grants unchanged; ess/22 gate enforced; a runtime that cannot send an actor fails the denied scenario.

```findings
[
  {"file": "crates/verify/ess-conformance/src/runner.rs", "line": 977, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "An unexpected read refusal is 'error' in the Rust runner and 'failed' in Go (runtime.go:2583), so the runners disagree on the admitted scenario for a target that refuses a granted reader."},
  {"file": "crates/verify/ess-conformance/src/synthesize/view_grant.rs", "line": 184, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "No synthesized scenario reads a read-granted view as no actor, so a Rust or Go server whose read check lets None through passes every read-grant scenario."},
  {"file": "crates/ui/ess-ui-check/src/model.rs", "line": 381, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "An actor: anonymous document reading a read-granted view checks clean, though the served surface refuses every request with no actor."},
  {"file": "crates/ui/ess-ui-check/src/model.rs", "line": 457, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A page whose actor's grant does not name a read-granted view it reads checks clean; page_actor_grants checks commands only."},
  {"file": "crates/generate/ess-synth/src/rust/entry.rs", "line": 61, "category": "integrity", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "Demonstration actor-header authentication in Rust and Go (go/entry.rs:176) reads only the first Authorization header, so Clerk followed by Watcher is served the read-granted Board."}
]
```
