---
format: aep.planning-md/3
id: story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected
kind: story
status: draft
title: A JSON request may carry an input field in a nested member of its body, validated, compiled and projected
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A webhook body wraps the operation's fields in an envelope: `{"kind": "closed", "sent_at": 17…,
"data": {"ticket_id": "t-1", "reason": "done"}}`. The command's input is flat
(`ticket_id`, `reason`). It must stay flat, because a row is addressed by a top-level input field
(`instance: ticket_id`) and a nested input cannot address one.

`ess-http/1` has two request layouts for `json`, and neither nests:

- the input object, each field a top-level member under its wire name
  (`docs/design/http-presentation-binding.md:322-325`);
- one input field as the whole body, `request.body: input.<field>` (`:337-358`).

So the adopter flattens the wire too, and the projected request schema states a body no caller
sends.

Minimal reproduction (written fresh; not run): `tickets.desk.CloseTicket(ticket_id: String, reason:
Optional<String>)`.

```yaml
routes:
  - command: tickets.desk.CloseTicket
    method: post
    path: /hooks
    request: {encoding: json, members: {ticket_id: data.ticket_id, reason: data.reason}}
```

Today `RequestSpec` (`crates/specify/ess-http/src/lib.rs:274-288`) refuses `members` as an unknown
field. The exact text is not established by reading.

## Outcome

Under **AE-D4**, option A (the default), a `json` request may declare `members: {<input field>:
<member path>}`. A member path is one or more JSON object member names joined by `.`. Each name is
non-empty and holds no `.`. An input field it names is read from that member. Every other input
field the path does not carry stays a top-level member under its wire name, as today.

- A required field under a missing object is `malformed`, as a missing top-level member is today.
- An `Optional` field under a missing object reads as absent.
- The member grammar is the one `select:` reuses (`ae1a-several-operations-share-one-route-by-body-members`).
- OpenAPI writes the nested schema: `data` becomes an object with `ticket_id` required and `reason`
  not required.

## Acceptance

1. Red today: the reproduction is refused for the unknown field `members` (a case in
   `crates/specify/ess-http/tests/request_body.rs` asserting the refusal names `members`).
2. After: it validates and compiles. `ess-http-ir/1` writes `request.members` only where it is
   declared, in field order. Every committed fixture compiles to the same bytes (`compile.rs`).
3. Refused, `ESS-HTTP-020` at `routes[i].request.members.<field>`:
   - a name that is not an input field of the command;
   - a field the path carries;
   - `members` beside `form`, `query` or `request.body`, or on a view;
   - an empty segment;
   - two fields on one member path;
   - one path a prefix of another (`data` beside `data.ticket_id`);
   - a top-level member path equal to the wire name of an unmapped field.
4. `ess generate --kind openapi --http` writes the nested request schema with `required` per level.
   A binding without `members` keeps its OpenAPI bytes (`crates/generate/ess-gen/tests/openapi_http_presentation.rs`).
5. `docs/design/http-presentation-binding.md` § "Request encoding",
   `website/docs/reference/formats.md` (the `ess-http/1` row) and `website/docs/guides/http-routes.md`
   state the key and its refusals.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RequestSpec` 274-288, `request.body` reader 328-340)
- cited: `crates/generate/ess-gen/src/openapi.rs` (request body schema)
- cited: `docs/design/http-presentation-binding.md` (§ "Request encoding", 320-368)

## Owned files

- `crates/specify/ess-http/src/lib.rs`: `RequestSpec.members`, compiled request `members`, the
  member-path type and reader, the `ESS-HTTP-020` cases.
- `crates/generate/ess-gen/src/openapi.rs`: the request schema writer for a nested member.
- Tests: `crates/specify/ess-http/tests/request_body.rs`, `compile.rs`;
  `crates/generate/ess-gen/tests/openapi_http_presentation.rs`.
- `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`,
  `website/docs/guides/http-routes.md`.

## Not yours

The generated servers (`ae1e-generated-servers-read-an-input-from-a-nested-member`) and the
generated apps (`ae1f-…`).

## Tests first

The red case of acceptance 1, one case per refusal of acceptance 3, and one OpenAPI case. All are
pure and run in milliseconds.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under the 10-minute pool limit.

```console
task fmt-check
cargo clippy -p ess-http --all-targets --locked -- -D warnings
cargo test -p ess-http --locked --test request_body
cargo test -p ess-http --locked --test compile
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test openapi_http_presentation
cargo test -p ess-gen --locked --test openapi_http
task test-xtask
task ci-lint
```

## Depends on

Nothing. It shares `request` checks in `lib.rs` with
`story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields` (active): whichever lands
second rebases. `members` composes with `unknown: ignore` (an undeclared member at any level is
ignored).

## Changelog

- `ess-http/1`: a `json` request may read an input field from a nested body member,
  `request.members: {<field>: data.<member>}`; OpenAPI writes the nested request schema.

## Fit review

1. **Need.** An envelope wraps the operation's fields in a member. The model's input must stay flat
   so it can address a row. The report flattened the wire to match. It
   proposed no syntax.
2. **Class: gap.** No nested layout (`http-presentation-binding.md:322-358`).
3. **Expressible today?** No. A struct input holding the envelope cannot address a row
   (`instance:` reads a top-level input field).
4. **Fit.** It mirrors `request.body: input.<field>`: the binding, not the model, owns the wire
   layout of a body. It composes with `select:` (one grammar for member paths), `unknown: ignore`,
   path parameters and refinement (a refinement restates `members`, `ESS-HTTP-012`, like `body`).
   Response side: no report asks for nested response layout. `response.<field>` already answers
   one field bare. Targets: OpenAPI here, servers `ae1e`, apps `ae1f`.
5. **Second adopter.** A chat platform's event callback `{"type": …, "event": {"user": …,
   "channel": …}}`, where the command takes `user` and `channel`.
6. **Cost.** One optional key, so no format version (precedent `request.body`, 0.89.0). No new code.
7. **Considered.** (a) Change nothing. (b) A struct input with nested identity: that opens grid R,
   "instance from a nested input field", and changes every target's addressing (AE-D4 B).
   (c) Chosen: binding-level member mapping.

## Decisions

Accept, redesigned. **AE-D4**, default A.

## Size

M, about 420 lines with tests.
