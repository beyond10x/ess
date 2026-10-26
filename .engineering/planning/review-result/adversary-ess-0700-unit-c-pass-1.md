---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-c-pass-1
kind: review-result
status: active
title: Adversary pass 1, unit C (event decoder, dispatch events)
relations:
- reviews: story:a-published-event-payload-has-a-generated-decoder
- reviews: story:generated-dispatch-and-handle-return-the-emitted-events
revision: 1
---
## Adversary pass 1, unit C (ess-0700 wave)

Worktree base d19ce4314, implementor's changes uncommitted; adversary added
`crates/generate/ess-synth/tests/adversary_c_event_wire.rs` (8 cases: 4 red, 4 green guards).

Suite run: `cargo test -p ess-synth --locked --no-fail-fast --test served_publication --test
http_binding_rust --test adversary_c_event_wire`, exit 101: `adversary_c_event_wire: FAILED. 4 passed;
4 failed` / `http_binding_rust: ok. 48 passed` / `served_publication: ok. 10 passed`.

| location | verdict | what was measured |
|---|---|---|
| crates/generate/ess-synth/src/rust/wire.rs:851 | NEEDS-CHANGE | an optional field with a declared wire name, sent under its model name, decodes as None with no refusal |
| crates/generate/ess-synth/src/rust/http.rs:1434 | NEEDS-CHANGE | events already on the log before a call (after System::redeliver) are returned as that call's events, ahead of the command's own |
| crates/generate/ess-synth/src/rust/wire.rs:358 | CONFIRMED | a field-less event's decoder accepts any payload, a string or null included |
| crates/generate/ess-synth/src/rust/wire.rs:851 | INFEASIBLE | Optional<Json> holding Some(Null) decodes as None; not reachable through the served path |
| crates/generate/ess-synth/src/rust/http.rs:988 | CONFIRMED | docs say the list holds the bindings' reactions to this command, but a held retry from an earlier call lands in a later call's list |

```findings
- file: crates/generate/ess-synth/src/rust/wire.rs
  line: 851
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an optional event field with a declared wire name, sent under its model name, is silently dropped rather than refused as the story's acceptance requires
- file: crates/generate/ess-synth/src/rust/http.rs
  line: 1434
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: events already on the log before a call (e.g. after System::redeliver) are returned as that call's events, ahead of the command's own
- file: crates/generate/ess-synth/src/rust/wire.rs
  line: 358
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a field-less event's decoder accepts any payload, including a string or null, despite decode_system_event documenting a payload-path refusal
- file: crates/generate/ess-synth/src/rust/wire.rs
  line: 851
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: Optional<Json> holding Some(Null) does not round-trip through encode_system_event and decode_system_event
- file: crates/generate/ess-synth/src/rust/http.rs
  line: 988
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the docs say the list holds the bindings' reactions to this command, but a held retry from an earlier call lands in a later call's list
```
