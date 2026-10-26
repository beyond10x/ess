---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-c-pass-2
kind: review-result
status: active
title: Adversary pass 2, unit C (event decoder, dispatch events)
relations:
- reviews: story:a-published-event-payload-has-a-generated-decoder
- reviews: story:generated-dispatch-and-handle-return-the-emitted-events
revision: 1
---
## Adversary pass 2, unit C (ess-0700 wave)

Worktree base d19ce4314 plus the implementor's correction round 1, uncommitted; adversary added
`crates/generate/ess-synth/tests/adversary_c_event_wire_2.rs` (8 cases: 2 red, 6 green guards).

Suite run: `cargo test -p ess-synth --locked --no-fail-fast --test served_publication --test
http_binding_rust --test adversary_c_event_wire --test adversary_c_event_wire_2`, exit 101:
`adversary_c_event_wire: ok. 8 passed` / `adversary_c_event_wire_2: FAILED. 6 passed; 2 failed` /
`http_binding_rust: ok. 48 passed` / `served_publication: ok. 10 passed`.

Attacked and held: a model name equal to another field's wire name, a wire name equal to its own
model name, a case-only rename, nesting through list, map, union content and `Optional<struct>`,
null under the model name, both spellings at once, view parameters (unchanged), an off-by-one in
the start-of-call count with a pumped-but-untaken event and a held retry, a refused call with an
entry already on the log.

| location | verdict | what was measured |
|---|---|---|
| crates/generate/ess-synth/src/rust/http.rs:1393 | CONFIRMED | an event published through a component's port before the call, still in its outbox, is listed by `dispatch_with_events` and `handle_with_events` as the call's own |
| crates/generate/ess-synth/src/rust/wire.rs:511 | CONFIRMED | a field-less command input decoder reads a string, null or array as the input |

Coordinator disposition (budget of two passes spent):

- http.rs:1393 documented, not changed. `collect` is private to the system crate and the served
  crate cannot reach it; a per-outbox count is wrong because `collect` interleaves components.
  The generated docs and the synthesize guide now say an event still in an outbox is listed as
  the call's and tell the caller to pump after a direct port call; the red case became a guard
  pinning that (`adversary_c2_an_event_still_in_an_outbox_is_listed_as_the_calls`).
- wire.rs:511 deferred to `story:a-field-less-command-input-refuses-a-value-that-is-no-object`:
  the decoder shape predates this unit and command-input refusals are outside its stories. The
  red case moved to that story.

```findings
- file: crates/generate/ess-synth/src/rust/http.rs
  line: 1393
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: an event published through a component's port before the call, still in its outbox, is listed by dispatch_with_events and handle_with_events as the call's own, ahead of the command's events
- file: crates/generate/ess-synth/src/rust/wire.rs
  line: 511
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: a field-less command input decoder still reads a string, null or array as the input, though correction 3 made the field-less event decoder beside it refuse one
```
