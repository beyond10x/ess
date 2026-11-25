---
format: aep.planning-md/3
id: story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error
kind: story
status: draft
title: An error outcome may answer its error's fields as the whole JSON body, with no member naming the error
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text
revision: 4
---
## Need

Services answer errors with a top-level JSON object. Three shapes reached ESS in one report.
Two are expressible today. The third is not:

- `{"error": "server_error", "message": "Try again later", "success": false}`: **expressible**.
  `{error: error}` writes the error's wire name under `error`, beside the error's own fields under
  their wire names (`crates/specify/ess-http/src/lib.rs:544-549`, `Body::Error`;
  `docs/design/http-presentation-binding.md:377`). Declare the error with `naming: {wire:
  server_error}` and fields `message: String`, `success: Boolean`, and fill `success` with the
  literal `false` in the outcome's error `payload:` (ess/19). Read, not run.
- `{"status": "unavailable"}`: **expressible**. An error with wire name `unavailable` and no fields,
  answered `{status: error}`.
- `{"healthy": false, "checks": ["…"]}`: **not expressible**. No member carries the error's name. A
  `Boolean` member cannot hold a wire name, and every error body form writes one
  (`Body` at `lib.rs:533-550`). `error` alone is no body word (`lib.rs:588-597`, "compiled IR writes
  `response`, `response.<field>`, `none` or `{<member>: error}`").

Minimal reproduction (written fresh; not run): `ops.health.Health`, outcome `unhealthy` with
`error: ops.health.Unhealthy(healthy: Boolean, checks: List<String>)`.

```yaml
    outcomes:
      unhealthy: {status: 503, body: error}
```

`ESS-HTTP-024`: "`error` is not a body …" (a word no body form spells; exact text not established).

## Outcome

Under **AE-D6**, option A (the default), `body: error` answers the error's own fields as one JSON
object under their wire names, with no member naming the error. It mirrors `response` (the
response object, bare), as `error.<field>` mirrors `response.<field>` in
`story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text`.

- Two outcomes on one status are told apart only by the field rule `ESS-HTTP-025` already applies
  (a field one error always writes and the other never writes). Two bare errors with the same
  fields on one status stay refused, unless they are `alike:`.
- OpenAPI writes the error's schema as the response body.
- The servers write the error's fields as the object.

## Acceptance

1. Red today: the reproduction is refused `ESS-HTTP-024` (a case in
   `crates/specify/ess-http/tests/bare_field_and_errors.rs`).
2. After: it validates and compiles. `ess-http-ir/1` writes `error` as the body word only where it
   is declared. Every fixture compiles to the same bytes (`compile.rs`).
3. Refused, naming the key:
   - `body: error` on an outcome without an error (`ESS-HTTP-024`);
   - under `errors:` covering two errors on one status whose fields the wire cannot tell apart
     (`ESS-HTTP-025`, the message naming both and suggesting `alike:` or distinct statuses);
   - beside `encoding: text` (`ESS-HTTP-024`; text is `error.<field>`'s).
4. `ess generate --kind openapi --http` writes the error's object schema under the status. A binding
   without the form keeps its bytes (`crates/generate/ess-gen/tests/openapi_http_presentation.rs`).
5. The generated Rust and Go servers answer `503 {"healthy": false, "checks": ["db"]}` for the
   fixture (`http_binding_rust`, both alike). Every byte pin passes unmodified.
6. The two idioms above are added to `website/docs/guides/http-routes.md` § "Encoding, statuses,
   headers and redirects", and `docs/design/http-presentation-binding.md` § "Response body per
   outcome" lists `error`.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`Body` 533-550, deserializer 584-600, body checks)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`
- cited: `docs/design/http-presentation-binding.md` (370-398)

## Owned files

- `crates/specify/ess-http/src/lib.rs`: `Body::ErrorObject` (name chosen at implementation), its
  checks in `ESS-HTTP-024`/`025`.
- `crates/generate/ess-gen/src/openapi.rs`: the response schema for it.
- `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`: writing it.
- Tests: `crates/specify/ess-http/tests/bare_field_and_errors.rs`, `compile.rs`;
  `crates/generate/ess-gen/tests/openapi_http_presentation.rs`; one fixture in
  `crates/generate/ess-synth/tests/http_binding/`.
- `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`,
  `website/docs/guides/http-routes.md`.

## Not yours

`error.<field>` and text bodies (`story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text`,
draft, landed first).

## Tests first

Acceptance 1, 3, 4, then 5 on the shared fixture build of `http_binding_rust`.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-http --all-targets --locked -- -D warnings
cargo test -p ess-http --locked --test bare_field_and_errors
cargo test -p ess-http --locked --test compile
cargo test -p ess-http --locked --test errors_of_one_wire_name
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test openapi_http_presentation
cargo test -p ess-gen --locked --test openapi_errors_of_one_wire_name
cargo clippy -p ess-synth --all-targets --locked -- -D warnings
cargo test -p ess-synth --locked --test http_binding_rust
cargo test -p ess-synth --locked --test go
task test-xtask
```

## Depends on

`story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text` (unit a; same body checks in
`lib.rs`), which lands after
`story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so`.

## Changelog

- `ess-http/1`: `body: error` answers an error's fields as the whole JSON object, with no member
  naming the error.

## Fit review

1. **Need.** A service's error body is the error's own object, with no member carrying the error's
   name. The report read every ESS error body as nested under one member. Two of
   its three shapes are the idioms above, because `{<member>: error}` writes the error's fields at
   the top level already.
2. **Class.** Two conveniences, answered with the idiom. One gap.
3. **Expressible today?** Two of three, as above. The idioms keep every figure in the binding, so
   policy 1 holds.
4. **Fit.** The error side mirrors the response side: `response` ↔ `error`, and `response.<field>` ↔
   `error.<field>` (the plain-text story). That is one spelling per concept. Distinguishability
   reuses `ESS-HTTP-025`'s field rule. Standard refusals keep `{<member>: {literal}}`, because they
   have no fields.
5. **Second adopter.** A rate limiter answering `429 {"retry_after": 30, "limit": 100}`, no code member.
6. **Cost.** One body word, absent by default, so no format version. No new code.
7. **Considered.** (a) Change nothing: the third shape stays unstated. (b) `{<member>: error}` with
   `member: none` (AE-D6 B): a second spelling for the same thing. (c) Chosen: `body: error`.

## Decisions

Accept, redesigned (two shapes declined with the idiom). **AE-D6** (SECURITY: what generated servers
answer), default A.

- S7: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.

## Size

M, about 400 lines with tests.

## Related (2026-11-25)

A second adopter reports two error bodies of this class on 0.99.0 (twentieth gap report, shape 2): a
service answers `{error_message, error_detail}` or `{success: false, message, errors}` with the right
status, and the binding gives the status but answers ESS's own error object. Both are this story's
third case: an object of the error's own fields with no member that holds the error's wire name.

- `{<member>: error}` writes the wire name under `<member>` beside the error's fields
  (`docs/design/http-presentation-binding.md`, "Response body per outcome", the row `{<member>: error}`).
  For `{success: false, message, errors}` no member can hold the name: `success` is a boolean. For
  `{error_message, error_detail}` the idiom writes `error_message` as the error's fixed wire name, which
  fits a message that is a fixed text per error class and not one the service composes.
- The literal `success: false` is an error field filled with a literal in the outcome's error
  `payload:`, as the story states for `success` beside `{error: error}`. So with `body: error` the
  fixtures read: an error with fields `success: Boolean`, `message: String`, `errors: List<…>`, and an
  error with fields `error_message: String`, `error_detail: String`; each answered at its status.
- The success side of the same services, `{success: true, item}` and `{success, items, total}`, is
  `story:a-wrapped-answer-carries-literal-members-and-paging-facts`.

Two lines join the acceptance: (7) the fixture `http_binding_rust` carries the two error shapes above,
and the Rust and Go servers answer the same bytes for both; (8) the compiled IR of a binding that
names both keeps every existing byte. The decision AE-D6 is unchanged, and the report is a second
adopter of its option A.
