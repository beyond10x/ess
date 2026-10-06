---
format: aep.planning-md/3
id: story:feature-request-430
kind: story
status: implemented
title: 'Caller-swapped run reuses a struct-typed identity input (follow-up to #275)'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#430
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-428
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_fresh_identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/caller_struct_identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/caller-struct-identity.yaml
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:23Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:23Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:47:53Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#430: Caller-swapped run reuses a struct-typed identity input (follow-up to #275).

## Origin
beyond10x/ess#430, filed 2026-10-05; a downstream specification (ess/20, ess 0.52.0) whose struct-addressed records gained a caller attribute. 14 swapped runs errored as already stored and 2 failed on a view row left by the first run.

## Fit review
1. Need: the caller-swapped second run must create its own record. So every caller-supplied identity it sends has to be a fresh value, struct identities included. Fresh reproduction `<fit-review scratch>/probe-430/` on installed ess 0.52.0. Struct identity `slot: {shelf, label}`, a caller attribute, `existing_instance:` beside the creation. The two `execute_command` steps of `demo.box.OpenBox/outcome/opened` send callers `subject-262144`/`subject-262145` and the same `slot` literal `{label: slot.label-1048592, shelf: slot.shelf-1048592}`. No syntax is proposed.
2. Class: defect against #275's documented contract. `Identities` says "Each such literal is replaced, throughout the swapped run, by a value no scenario sends" (`crates/verify/ess-conformance/src/synthesize/caller.rs:1055-1065`). `drawn` draws a fresh struct and keys it by its whole serialization (:1190-1233, `key` :1355-1357). But `redraw_under` only replaces String/Number/Bool leaves. For an object it only recurses into members (:1383-1405), so the drawn struct is never written back. The 0.53.0 head changed this function (typed Number/Bool, `git diff 0.52.0..HEAD`) but not the object case, so it is not fixed in 0.53.0.
3. Existing idiom: a scalar identity (`String`/newtype) gets a fresh value (#275, `tests/caller_fresh_identity.rs`). A struct identity has no workaround short of changing the identity's type, which is a domain change.
4. Fit: no surface. In `redraw_under`, a typed object whose serialization is a key in `drawn` is replaced whole, before recursing. Other cases stay untouched: the `{kind: literal, value: …}` wrapper passes `typed` to `value` (:1386-1388), so only the wrapped struct matches. Two checks are owed. First, `key(Node)` and `serde_json::Value::to_string` must serialize a struct identically; the bot's comment flags this as unverified, and a test pins it. Second, a struct copied by member, e.g. an event field or (after #428) a view parameter bound to `slot.shelf`, must be redrawn too. So the fresh struct differs in every member, and each member's old→new leaf is recorded under the member names derived from the identity (`derived`, :1291-1352). The issue's "at least one member different" is too weak once members are copied independently. `keeps_branch` (:1266-1268) still guards that every send keeps its branch.
5. Second adopter: a warehouse bin addressed by `{aisle, shelf}` and opened by an authenticated picker. The swapped run must open a different bin.
6. Cost: no format. Suite bytes change only for models with a struct identity and a caller attribute; measure across the repository's models as #275 did. No new note or diagnostic. `Note::UnswappedCallers` still covers exhaustion.
7. Alternatives: (a) change nothing and let the scenario keep its first run only. That drops the caller-swap witness #168 exists for. (b) Redraw only one member, the issue's minimum. That breaks a member-level copy. (c) Chosen: whole-struct replacement plus member-level mapping.

## Decisions
accept as proposed, strengthened: every member of a swapped struct identity is fresh, and member-level copies follow it. No format bump. Depends on #428 (edge recorded), which introduces member-level parameter copies.

## Acceptance
- swapped_run_draws_fresh_struct_identity: for the committed fixture `crates/verify/ess-conformance/tests/fixtures/caller-struct-identity.yaml` (the fit-review model: `OpenBox` keyed by the struct `slot`), the second `OpenBox` send carries a `slot` differing from the first in every member, and the interpreted target passes both runs.
- swapped_struct_copy_in_payload_follows: the swapped run's `BoxOpened.slot` expectation equals its own fresh struct.
- swapped_struct_member_copy_follows: a view parameter or event field copying `slot.shelf` is redrawn with the fresh member.
- struct_key_serialization_matches: `key(Node::Map)` equals `serde_json::Value::to_string` of the same struct (pinned).
- scalar_identity_bytes_unchanged: `tests/caller_fresh_identity.rs` fixtures keep their bytes.
- reused_identity_mutant_fails: a target that ignores the sent `slot` and answers the swapped run from the first run's record fails the swapped run's `BoxOpened.slot` expectation; an honest target, storing one row per identity, passes both runs.

## Scope
- crates/verify/ess-conformance/src/synthesize/caller.rs  cited — `redraw_under` :1377-1406, `Identities::drawn` :1190-1233, `derived` :1291-1352
- crates/verify/ess-conformance/tests/caller_fresh_identity.rs  cited — #275 regression stays green
- crates/verify/ess-conformance/tests/caller_struct_identity.rs  inferred — new regression test
- crates/verify/ess-conformance/tests/fixtures/caller-struct-identity.yaml  inferred — the fit-review model, committed
