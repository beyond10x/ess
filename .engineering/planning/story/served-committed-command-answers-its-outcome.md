---
format: aep.planning-md/3
id: story:served-committed-command-answers-its-outcome
kind: story
status: active
title: A committed command is answered with its outcome, not 501
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/bridge.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/served_unfinished_committed.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/transport_free_entry.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:07Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:08Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A command whose effect was committed is answered with its own declared outcome, and a failed delivery is reported beside it; 501 means only that the port did not run (review S24, worst five #2).

## Problem

Re-checked on 0.92.0. A served command whose effect and events were committed, and whose delivery
of what it published to a binding then failed, is answered `501`
`{"refused": "delivering what the command published: …", "committed": true}`. The contract declares
it so for every command:

```
$ ess generate --path spec.yaml --kind openapi --out oa     # any component with reached_by: network
$ sed -n 42,58p oa/openapi/task-service.yaml
        '501':
          description: 'No declared outcome: the realization is unfinished. Either a port this command runs reported an unmet obligation, and nothing was written: `committed` is `false`. Or the command''s effect was committed and delivering what this command published to a binding failed: `committed` is `true`, … Do not retry: …'
          … properties: {committed: boolean, refused: string}, additionalProperties: false
$ grep -c undelivered oa/openapi/task-service.yaml
0
```

Where it lives:

- Rust server: `settle` returns `Err(entry::Refused::Undelivered(…))` when the pump fails
  (`crates/generate/ess-synth/src/rust/http.rs:1436-1460`), and `Undelivered` maps to 501
  (`rust/http.rs:2292`).
- Go server: the same `501` body with `Committed bool` (`crates/generate/ess-synth/src/go/http.rs:2718`).
- Web bridge: `BridgeError::Undelivered` (`crates/generate/ess-synth/src/web/bridge.rs:176-222`).
- Contract: `crates/generate/ess-gen/src/openapi.rs:1302` (the 501 description).

A client sees an error for a command that took effect, and the answer carries no `published` list,
so a plane adapter that follows published events never sees the committed ones.

## Fit review

From `docs/design/review-external-requests-2026-09.md` row S24 and § worst five #2; assessed with
`.agents/skills/assessing-external-requests/SKILL.md`.

1. **Need.** A client must be able to tell a command that took effect from one that did not, and see
   what it published, without parsing an error. Requested twice: the `committed` member (0.47.0/0.48.0,
   adopted) and a downstream need on 0.48.0 for the `published` list of a committed command.
2. **Class: defect.** A success is answered as an error status (review row S24).
3. **Already expressible?** No: `committed: true` says the effect stands but keeps the error status
   and drops the outcome's body and `published`.
4. **Fit.** No authored surface. The served answer of a committed command is its outcome's declared
   status and body, plus `undelivered: [{binding, event}]` declared in the contract; 501 keeps one
   meaning, the port did not run. Rust, Go and the web bridge change together.
5. **Second adopter.** Any service whose notification binding can fail after an order is placed:
   the order exists and the client must not retry.
6. **Cost.** Breaking for a Rust shell matching `Refused::Undelivered`, a web client matching
   `BridgeError::Undelivered`, and an OpenAPI client reading `committed`. No source format moves.
7. **Alternatives.** (a) Change nothing: 501 with `committed` stays. (b) Add `published` to the
   committed 501 body: keeps the error status for a success. (c) Chosen: answer the outcome plus
   `undelivered`.

## Decisions

Review 2026-09: **accept, redesigned** (revise; both requester alternatives absorbed). The
committed case answers its outcome with `undelivered: [{binding, event}]`; no separate change to the
501 body is made. The breaking change is batched with any other breaking change of the same
release. Drafted, not scheduled.

- `Refused::Undelivered` and `BridgeError::Undelivered` are removed, not kept: once the committed
  case answers its outcome neither has a case left to carry, and a kept variant would be a second
  way to report one fact.

## Acceptance

Every bullet but the OpenAPI one is asserted by `cargo test -p ess-synth --locked --test
served_unfinished_committed`, on its fixture with a binding whose delivery fails.

- The Rust and the Go server answer the committed command whose delivery failed with that
  outcome's declared status and body plus `undelivered: [{binding, event}]` naming the failed
  binding and event; neither answers `501` for it.
- The Rust and the Go server answer a command whose port reported an unmet obligation with `501`
  and a body `{refused}` that holds no `committed` member.
- The web bridge answers the same committed command with the same outcome and `undelivered` list.
- The synthesized Rust entry module and web bridge source hold no `Undelivered` variant (no line
  matching `Undelivered` in either); the generated Rust entry point returns the committed command's
  outcome with its `undelivered` list in place of
  `the_rust_entry_point_refuses_the_committed_case_as_undelivered`.
- `ess generate --path <fixture> --kind openapi`: the `501` response describes only an unmet
  obligation with nothing written and its schema has no `committed` property; each command
  outcome's response schema declares an optional `undelivered` array of `{binding, event}`
  (`grep -c undelivered` on the document is greater than 0).

## Scope

- cited: crates/generate/ess-synth/src/rust/http.rs
- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/src/web/bridge.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- cited: crates/generate/ess-synth/tests/served_unfinished_committed.rs
- inferred: crates/generate/ess-synth/src/rust/http/bound.rs
- inferred: crates/generate/ess-synth/tests/transport_free_entry.rs

confidence: medium

Proposes its `CHANGELOG.md` sentence (breaking for a Rust shell, a web client and an OpenAPI
client) and re-pins any byte-pinned generated server rows the `501` and outcome changes move.

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (served-committed-command-answers-its-outcome). Drafted, not scheduled; a fit review precedes dispatch.
