---
format: aep.planning-md/3
id: review-result:binding-arrangement-266-267-20261004-r1
kind: review-result
status: active
title: 'Binding arrangement adversary pass 1: fan-out of two bindings on one row'
relations:
- reviews: story:feature-request-266
- reviews: story:feature-request-267
revision: 1
---
unit: W3-3 #266+#267 bindings in arrangement, flow, delivery and drop, pass 1
verdict: CONFIRMED (one judgement note); both red cases INFEASIBLE (constructed spec shape, no model found that has it)
cases: executed 18→29, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (cargo.sh, logs, dumps); build dir and Go cache deleted
needs-coordinator: none

Publication copy of the only adversary pass on the #266/#267 unit (worktree `<worktrees>/ess/ess-w3-266-267-binding-arrangement-20261004`, base `d1026d1f0`, uncommitted diff of 9 tracked files). The reviewer added `crates/verify/ess-conformance/tests/adversary_w3_266_267_pass1.rs`, `adversary_w3_266_267_go.rs` and `adversary_w3_266_267_dump.rs`; no production file edited. Byte drift over 84 models against base: identical except the new fixture and `models/toolchain` (2 scenarios gained, 5 lost, 6 refusals gained, as its README declares).

F1 (warning, infeasible) `crates/verify/ess-conformance/src/synthesize/binding_effects.rs:1044` (also `trigger_changes` :859): drop asserts the destination row unchanged while another binding on the same trigger event legitimately changes it, so an honest target fails `kicked-starts/binding/on-failure` natively and in Go.

F2 (warning, infeasible) `binding_effects.rs:961` (also :1043, `rest_after` :1056): `settled_row` follows only the reached branch's own chain, so flow and delivery assert an eventual state (`New`) that a second binding on the same row at the same time contradicts (`Started`).

F3 (note) `models/toolchain/README.md:136` (`BoundAway`, `synthesize.rs:1766`): says no release rests in `Tagged`, yet `RunReleaseChecks`' external refusal with escalate leaves it there, so the six `BoundAway` refusals hide scenarios a forced-refusal arrangement could witness.

Attacked and not broken: an eventual dispatcher passes every scenario for the fixture, a two-link chain, a chain ending in a field change, a trigger naming another job and an updating trigger; zero attempts fails delivery and drop; a swallowed force fails drop; a disabled binding fails flow; correlation leakage impossible (`begin_scenario` resets state).

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/binding_effects.rs", "line": 1044, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "drop asserts the destination row unchanged while another binding on the same trigger event legitimately changes it, so an honest target fails kicked-starts/binding/on-failure natively and in Go"},
  {"file": "crates/verify/ess-conformance/src/synthesize/binding_effects.rs", "line": 961, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "rest_after settles only the reached branch's own chain, so flow/delivery assert an eventual state that a second binding on the same row at the same time contradicts (also :1043)"},
  {"file": "models/toolchain/README.md", "line": 136, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "says no release rests in Tagged, yet RunReleaseChecks' external refusal with escalate leaves it there, so the six BoundAway refusals hide scenarios a forced-refusal arrangement could witness"}
]
```
