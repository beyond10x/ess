---
format: aep.planning-md/3
id: story:the-openapi-projection-writes-a-route-shared-by-body-selection-as-one-operation-whose-request-body-is-one-of
kind: story
status: draft
title: The OpenAPI projection writes a route shared by body selection as one operation whose request body is one of the bound commands' bodies
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body
- depends_on: story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected
revision: 1
---
## Need

After `ae1a-several-operations-share-one-route-by-body-members`, several commands may sit on one
method and path. An OpenAPI path item holds one operation per method, and
`crates/generate/ess-gen/src/openapi.rs` writes one operation per route. With a shared route that
would write one operation and silently drop the others, or write two operations under one method
key. Either is a wrong document.

## Outcome

Under **AE-D3**, option A (the default), `ess generate --kind openapi --http` writes one operation
for each shared method and path:

- `operationId` is the `operation_id` of the first sharing route in document order, or that route's
  qualified name. Its `x-ess-operations` lists every sharing route in document order:
  `{operationId, command, select}`, with `select` as written and the catch-all marked
  `select: otherwise`.
- `requestBody` is `oneOf` the bound commands' request schemas. In each, the tested members are
  `const` (a nested member nested as in `ae1d-…`) and `required`. When every route tests exactly one
  and the same top-level `String` member and there is no catch-all, OpenAPI's `discriminator`
  (`propertyName`, `mapping`) is written too. The catch-all's schema has no `const`, and its
  description says it takes every body the others do not.
- `responses` merge per status. One command's answer is written as today. Two commands' answers on
  one status are `oneOf` the bodies, and the description names each command and outcome.
- A closed set (no catch-all) states that a body no route selects is answered as `malformed`, under
  the `malformed` status.

A binding without `select` keeps its OpenAPI bytes.

## Acceptance

1. Red today: with `ae1a` landed and this story not, the open-set binding of `ae1a` projects one
   operation for `post /hooks` that names one command. A case in
   `crates/generate/ess-gen/tests/openapi_http_presentation.rs` asserts that `x-ess-operations` lists
   four entries, and it fails.
2. After: the open set projects one operation, `oneOf` four request schemas with `kind` as `const`
   in three and none in the catch-all, `x-ess-operations` in document order, and the `204` and other
   statuses merged.
3. The closed set projects `oneOf` nine schemas, `object` and `action` as `const`, no
   `discriminator` (two members), and the `malformed` status with the description of the
   unselected body.
4. A one-member, closed, `String` set writes `discriminator` with a `mapping` to component schemas.
5. Every committed OpenAPI document and every existing `openapi_*` test keeps its bytes.
6. `docs/design/http-presentation-binding.md` § "What OpenAPI writes" states the shared operation.

## Scope

- cited: `crates/generate/ess-gen/src/openapi.rs`
- inferred: `crates/generate/ess-gen/src/http.rs` (`routes()` grouping by component, method, path)
- cited: `docs/design/http-presentation-binding.md` (§ "What OpenAPI writes (k4)", 761-778)

## Owned files

- `crates/generate/ess-gen/src/openapi.rs`: the path-item writer, the merged operation, `oneOf`
  request and responses, `x-ess-operations`.
- `crates/generate/ess-gen/tests/openapi_http_presentation.rs` (new cases).
- `docs/design/http-presentation-binding.md` (§ k4 only).

## Not yours

`ess-http` validation (`ae1a`), servers (`ae1c`), apps (`ae1f`).

## Tests first

Acceptance 1 (red), then 2 to 4. Each test is pure and runs in milliseconds.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under the 10-minute pool limit.

```console
task fmt-check
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test openapi_http_presentation
cargo test -p ess-gen --locked --test openapi_http
cargo test -p ess-gen --locked --test openapi
cargo test -p ess-gen --locked --test route_order
task projection-check
task test-xtask
```

`task projection-check` holds the committed projections. `task test-xtask` runs because `docs/`
moves.

## Depends on

`ae1a-several-operations-share-one-route-by-body-members`, and
`ae1d-a-json-request-reads-an-input-from-a-nested-member` (it nests the `const` under its member
path).

## Changelog

- OpenAPI with `--http` writes a route shared by body selection as one operation: its request body
  `oneOf` each command's body with the selecting members `const`, and `x-ess-operations` naming every
  command.

## Fit review

1. **Need**: as `ae1a`.
2. **Class**: the projection half of the same gap.
3. **Expressible**: no.
4. **Fit.** OpenAPI 3 holds one operation per method (`paths.<path>.<method>`), so a merged
   operation is the only faithful form. `oneOf` with `const` is the standard way to tell bodies
   apart. `x-ess-operations` follows the existing `x-ess-route-source` extension
   (`http-presentation-binding.md:239`).
5. **Second adopter**: the webhook endpoint of `ae1a`.
6. **Cost.** Bytes move only for bindings using `select`. A third-party generator reads one
   operation with a `oneOf` body. That is honest, and it is the cost written into the document.
7. **Considered.** One operation per command under invented paths (a lie, today's state). Refusing
   `--http` for a shared route (AE-D3 C: no document at all). A new group key for the operationId
   (AE-D3 B).

## Decisions

Accept, redesigned. **AE-D3**, default A.

## Size

M, about 350 lines with tests.
