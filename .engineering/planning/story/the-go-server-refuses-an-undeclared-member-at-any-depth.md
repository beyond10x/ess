---
format: aep.planning-md/3
id: story:the-go-server-refuses-an-undeclared-member-at-any-depth
kind: story
status: active
title: The Go server refuses an undeclared member at any depth of a request body
summary: Go typed decoder accepts undeclared nested members that Rust refuses (0.89.0 U2 adversary; from-finding)
tags:
- backlog
- defect
- from-finding
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/shared.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:38Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:39Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Found by the U2 adversary pass of wave-2026-11-06-ess (ESS 0.89.0). Pre-existing, outside that
unit's acceptance; still so on 0.92.0.

The generated Go server refuses an undeclared member only at the first level of a command body
(`closedInput` in `crates/generate/ess-synth/src/go/http.rs`, and `closedMembers` for a bare body
in `crates/generate/ess-synth/src/go/http/bound.rs`). Its struct decoder
(`crates/generate/ess-synth/src/go/http.rs`, `fn type_decoder`, the `ResolvedBody::Struct` arm)
checks that the value is an object and decodes each declared member, and never refuses another
member. The generated Rust struct decoder calls `json::closed_at` for every struct that does not
declare `undeclared_fields: ignored` (`crates/generate/ess-synth/src/rust/wire.rs`), and the served
OpenAPI closes those structs, so the two servers disagree on one request.

Minimal reproduction (`format: ess/33`): struct `Label {text: String}`, struct
`Tag {label: Label}`, command `TagItem {id: String, tag: Tag, labels: List<Label>}`, one
component; `ess generate synthesize --target go`, `go build`, `POST /items/commands/TagItem`:

| body | Go | Rust (decoder) |
|---|---|---|
| `{"id":"a","tag":{"label":{"text":"x"}},"labels":[{"text":"y"}]}` | 202 | admits |
| the same with `"extra":1` at the top | 400 `body.extra: expected no member: no field is named so` | refuses |
| `"labels":[{"text":"y","extra":1}]` | 202 | refuses (`closed_at`) |
| `"tag":{"label":{"text":"x","extra":1}}` | 202 | refuses (`closed_at`) |

## Acceptance

- `the_go_server_refuses_an_undeclared_member_at_any_depth`: for the reproduction, the generated
  Go server answers `400` naming the member's path (`body.labels[0].extra`,
  `body.tag.label.extra`) for a list element and a struct two levels down, in an unbound body, a
  bound wrapped body and a bound bare body; the generated Rust server gives the same status for
  each body (a case in `crates/generate/ess-synth/tests/http_binding/go.rs` against
  `shared::rust_answers()`).
- A struct declaring `undeclared_fields: ignored` keeps admitting extra members in Go, as in Rust.
- The existing Go decoding cases stay green; a model whose structs declare no closedness change
  keeps its bytes apart from the added check.

## Scope

- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/src/go/http/bound.rs
- cited: crates/generate/ess-synth/src/rust/wire.rs
- inferred: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-synth/tests/http_binding/shared.rs
- inferred: CHANGELOG.md

confidence: high

Breaking for a client of a generated Go server that sends an undeclared member below the first
level: it starts receiving `400`, as from the Rust server; listed in the CHANGELOG.
