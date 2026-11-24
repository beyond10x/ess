---
format: aep.planning-md/3
id: story:the-generated-rust-and-go-servers-send-a-request-on-a-shared-route-to-the-command-its-body-members-select-and
kind: story
status: draft
title: The generated Rust and Go servers send a request on a shared route to the command its body members select, and refuse a body no route selects
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body
- depends_on: story:the-generated-rust-and-go-servers-read-an-input-field-from-the-nested-body-member-its-binding-names
revision: 1
---
## Need

After `ae1a-several-operations-share-one-route-by-body-members`, a binding may put several commands
on one method and path. The generated servers dispatch on method and path alone, in the order of
`ess_gen::http::dispatch_order` (`crates/generate/ess-gen/src/http.rs:577`). A shared route would
reach one command and never the others. A generator that silently serves part of a binding is a red
flag of the fit-review skill.

## Outcome

Both generated servers (Rust: `crates/generate/ess-synth/src/rust/http.rs`, `rust/http/bound.rs`;
Go: `crates/generate/ess-synth/src/go/http/bound.rs`) dispatch a shared route in two steps:

1. Method and path, as today.
2. Then the body. The body is read once, within the route's `limits` (the body cap applies first,
   unchanged), and parsed as JSON once. Each sharing route's select is tested most specific first.
   The validation of `ae1a` makes that order total within each matching chain. The first match
   decodes its command's input from the same parsed value (with `request.members`, `ae1e`). The
   catch-all takes what no select matches.
3. With no catch-all, an unmatched body is answered with the route's `malformed` answer. A body
   that is not JSON, or not a JSON object, is `malformed` before any select is tested.

A tested member's JSON type must equal the literal's. `"true"` is not `true`, so it matches no
select and falls to the catch-all or to `malformed`.

## Acceptance

1. Red today: a new fixture binding in `crates/generate/ess-synth/tests/http_binding/` (the open set
   and the closed set of `ae1a`, on a minimal model of the same shape) is generated. The Rust and Go
   cases of `http_binding_rust` post `{"kind": "closed", "data": {"reopen": true}}` and assert that
   `ReopenTicket` answered. This fails while every request reaches one command.
2. After, both servers alike:
   - each selected body reaches its command;
   - a `kind` no route names reaches the catch-all, which answers `204`;
   - an unknown pair on the closed set answers the `malformed` status (`400`) with its declared
     body;
   - `"reopen": "true"` (a string) is not `true`;
   - a non-object body is `malformed`;
   - a body over the cap is `too_large` before any select.
3. Every committed generated server tree and every existing byte pin passes unmodified:
   `http_binding_rust`, `feasibility`, `adversary_related_guard_behaviour_pass1`, `served_codec_names`,
   `go`. A binding without `select` generates the same bytes.

## Scope

- cited: `crates/generate/ess-gen/src/http.rs` (`dispatch_order` 577)
- inferred: `crates/generate/ess-synth/src/rust/http.rs`, `rust/http/bound.rs`, `go/http/bound.rs`
- cited: `crates/generate/ess-synth/tests/http_binding_rust.rs`, `tests/http_binding/`

## Owned files

- `crates/generate/ess-gen/src/http.rs`: the per-route select table handed to the generators, in
  most-specific order.
- `crates/generate/ess-synth/src/rust/http.rs`, `rust/http/bound.rs`, `go/http/bound.rs`: body
  selection.
- `crates/generate/ess-synth/tests/http_binding/` (one fixture and cases in `go.rs`, `shared.rs`),
  `tests/http_binding_rust.rs`.

## Not yours

Nested-member decoding (`ae1e-generated-servers-read-an-input-from-a-nested-member`, landed first,
same files). OpenAPI (`ae1b`). Apps (`ae1f`).

## Tests first

The red case of acceptance 1 for Rust, then for Go. The fixture's generated servers are built once
per test run by the existing shared harness of `http_binding_rust`, never once per case (the test
budget rule). Time the step before and after, and name any case over 5 s in the report.

## Gate

Each test stays under 60 s and the step under 120 s. A job over 10 minutes is killed by the pool:
split it, do not raise it.

```console
task fmt-check
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test route_order
cargo clippy -p ess-synth --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test http_binding_rust
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-synth --locked --test go
cargo test -p ess-synth --locked --test feasibility
```

## Depends on

`ae1a-several-operations-share-one-route-by-body-members`, and
`ae1e-generated-servers-read-an-input-from-a-nested-member` (same server files, landed first).

## Changelog

- The generated Rust and Go servers dispatch a route shared by body selection to the command the body
  selects; a body no route selects is answered `malformed`, or by the catch-all route.

## Fit review

As `ae1a` for questions 1 to 3, 5 and 7. 4: one dispatch table both servers already share
(`dispatch_order`). The selection is a second key on it, so the Rust and Go servers cannot disagree.
6: generated code changes only for bindings with `select`. Security: an unmatched body never reaches
a command, so this is fail-closed.

## Decisions

Accept, redesigned. **AE-D2** (default A) decides the rule, and this story implements it.

## Size

M, about 500 lines with tests.
