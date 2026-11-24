---
format: aep.planning-md/3
id: story:the-generated-rust-and-go-servers-read-an-input-field-from-the-nested-body-member-its-binding-names
kind: story
status: draft
title: The generated Rust and Go servers read an input field from the nested body member its binding names
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected
revision: 1
---
## Need

After `ae1d-a-json-request-reads-an-input-from-a-nested-member`, a binding may say that
`ticket_id` travels as `data.ticket_id`. The generated servers decode the request body as the
command's input object (`crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`;
`docs/design/http-presentation-binding.md:352-354`). They would refuse the real body (an undeclared
`data` member) or, under `unknown: ignore`, run the command with the required field missing.

## Outcome

Both servers read each mapped field from its member path, and every other field from the top level,
as today:

- A required field missing at any level is `malformed`.
- An `Optional` field under a missing object reads as absent.
- A member of the wrong JSON type is `malformed`.
- Under `unknown: refuse`, a member no field and no `select` names is refused at every level. The Go
  server already refuses an undeclared member at any depth
  (`story:the-go-server-refuses-an-undeclared-member-at-any-depth`), and the Rust server follows it
  for the envelope's objects.
- Under `unknown: ignore`, such a member is ignored at every level.

## Acceptance

1. Red today: a fixture binding in `crates/generate/ess-synth/tests/http_binding/` maps
   `ticket_id: data.ticket_id`. The Rust and Go cases post `{"data": {"ticket_id": "t-1"}}` and
   assert that the command ran on `t-1`. This fails.
2. After, both servers alike:
   - the nested body runs the command;
   - `{"data": {}}` is `malformed`;
   - `{}` with an `Optional` mapped field and no `data` reads absent;
   - `{"data": {"ticket_id": 7}}` is `malformed`;
   - an undeclared `data.extra` is refused under `refuse` and ignored under `ignore`.
3. Every existing byte pin and committed tree passes unmodified (`http_binding_rust`,
   `served_codec_names`, `go`, `json_body_closure`, `feasibility`).

## Scope

- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `rust/json.rs`, `go/http/bound.rs`
- cited: `crates/generate/ess-synth/tests/http_binding_rust.rs`, `tests/json_body_closure.rs`

## Owned files

- `crates/generate/ess-synth/src/rust/http/bound.rs`, `rust/json.rs` (the input decoder for a mapped
  field), `go/http/bound.rs`.
- `crates/generate/ess-synth/tests/http_binding/` (one fixture and cases), `tests/http_binding_rust.rs`.

## Not yours

Body selection (`ae1c`, after this, same files). Validation and OpenAPI (`ae1d`).

## Tests first

Acceptance 1 for Rust, then Go. They use the shared fixture build of `http_binding_rust`, built once
per run. Name any case over 5 s.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-synth --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test http_binding_rust
cargo test -p ess-synth --locked --test json_body_closure
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-synth --locked --test go
```

`json_body_closure` needs `task json-body-harnesses` and its variables (`AGENTS.md` § Gate).

## Depends on

`ae1d-a-json-request-reads-an-input-from-a-nested-member`.

## Changelog

- The generated Rust and Go servers read an input field from the nested body member
  `request.members` names.

## Fit review

As `ae1d`. Target note: both servers decode from one parsed JSON value, so `ae1c` adds selection
without parsing twice.

## Decisions

Accept, redesigned. **AE-D4**, default A.

## Size

M, about 350 lines with tests.
