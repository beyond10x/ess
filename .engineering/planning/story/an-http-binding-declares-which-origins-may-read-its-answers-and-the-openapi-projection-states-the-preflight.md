---
format: aep.planning-md/3
id: story:an-http-binding-declares-which-origins-may-read-its-answers-and-the-openapi-projection-states-the-preflight
kind: story
status: draft
title: An HTTP binding declares which origins may read its answers, and the OpenAPI projection states the preflight answer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A service lets browser pages from other origins read its answers (CORS, Fetch standard § 3.2). It
answers every `OPTIONS` preflight `204` with `Access-Control-Allow-*` headers and adds
`Access-Control-Allow-Origin` to its answers. `ess-http/1` cannot state it:

- `options` is in the method set (`docs/design/http-presentation-binding.md:89`), but
  `ESS-HTTP-006` admits a command on `post`, `put`, `patch`, `delete` and `get` only, and a view on
  `get` only (`crates/specify/ess-http/src/lib.rs:3198`). A preflight is no operation anyway.
- An outcome's literal `headers:` could carry `Access-Control-Allow-Origin: *` on every outcome of
  every route (`lib.rs` `ESS-HTTP-026`). That spells a policy as hundreds of copied lines, cannot
  echo an allowed origin, cannot add `Vary: Origin`, and leaves the preflight unstated.

Who may read an answer is a security boundary. Today a binding states none, and the generated servers
send no CORS header, which is the fail-closed default.

Minimal reproduction (written fresh; not run):

```yaml
cors: {origins: any}
routes:
  - {view: tickets.desk.OpenTickets, method: get, path: /tickets}
```

Refused today: `cors` is an unknown field of `HttpSpec` (`lib.rs:1853-1880`).

## Outcome

Under **AE-D5**, option A (the default), `ess-http/1` takes a policy key, not authored headers:

```yaml
cors:
  origins: [https://desk.example.test]   # or `any`
  credentials: false                       # default false
  headers: [Authorization]                 # request headers a page may send; default none
  expose: [X-Response-Time]                # response headers a page may read; default none
  max_age: 600                             # seconds; default absent
```

- Top level: it applies to every route.
- A route may declare `cors: none` (this route is not readable cross-origin) or its own `cors:`
  map, standing in place of the top-level one on that route.
- The allowed methods are derived: on a path, the methods its routes declare.
- `origins: any` beside `credentials: true` is refused, because the Fetch standard forbids `*`
  with credentials.
- An origin is `scheme://host[:port]`, with no path.
- A header name is an RFC 9110 token, once ignoring case.
- `expose` may name only headers some covered route declares.
- Absent `cors`: no CORS answer, as today.

OpenAPI writes, per covered path, an `options` operation answering `204` with the
`Access-Control-Allow-*` headers. Each covered response gets an `Access-Control-Allow-Origin` header
(and `Vary: Origin` when origins are listed). `x-ess-cors` holds the policy on the document.

## Acceptance

1. Red today: the reproduction is refused for the unknown field `cors` (a case in
   `crates/specify/ess-http/tests/headers.rs`).
2. After: it validates and compiles. `ess-http-ir/1` writes `cors` (top level and per route) only
   where it is declared. Every fixture compiles to the same bytes (`compile.rs`).
3. Refused, with a new code `ESS-HTTP-0NN` (numbered when it lands, the next free code), each at its
   key:
   - `origins: any` with `credentials: true`;
   - an origin with a path, query, fragment, userinfo or `*` inside it;
   - a duplicate origin;
   - an empty `origins`;
   - a header name that is not a token, or is given twice ignoring case;
   - an `expose` name no covered route declares;
   - `max_age` outside `0..=86400`;
   - a route `cors` value other than `none` or a map.
4. A refinement restates `cors` (`ESS-HTTP-012`, like `refusals`): a server binding may narrow the
   origins of the protocol binding and may not widen them.
5. `ess generate --kind openapi --http` writes the `options` operations and headers above. A binding
   without `cors` keeps its OpenAPI bytes (`crates/generate/ess-gen/tests/openapi_http_presentation.rs`).
6. `docs/design/http-presentation-binding.md` gains § "Cross-origin reads", and § "Still refused" no
   longer implies CORS is out of scope. `website/docs/reference/formats.md`,
   `website/docs/reference/diagnostics.md` (the new code) and `website/docs/guides/http-routes.md`
   state it.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`HttpSpec` 1853-1880, `RouteSpec` 1561-1610, code table 3180-3240)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- cited: `docs/design/http-presentation-binding.md`

## Owned files

- `crates/specify/ess-http/src/lib.rs`: `HttpSpec.cors`, `RouteSpec.cors`, a `Cors` type, its
  compiled form, the new code and its row.
- `crates/generate/ess-gen/src/openapi.rs`: the `options` operation and headers.
- Tests: `crates/specify/ess-http/tests/headers.rs`, `compile.rs`;
  `crates/generate/ess-gen/tests/openapi_http_presentation.rs`.
- Docs: `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`,
  `website/docs/reference/diagnostics.md`, `website/docs/guides/http-routes.md`.

## Not yours

The servers (`ae3b-generated-servers-answer-the-preflight-and-admit-declared-origins`).
Authentication (`story:an-http-binding-declares-how-a-caller-authenticates`): it decides who the
caller is, while CORS decides which page may read the answer. Neither implies the other.

## Tests first

Acceptance 1 (red), 3 (one case each), 5. All pure.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-http --all-targets --locked -- -D warnings
cargo test -p ess-http --locked --test headers
cargo test -p ess-http --locked --test compile
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test openapi_http_presentation
task test-xtask
task ci-lint
```

`task test-xtask` runs because a diagnostic code and `website/` move (the code table pin).

## Depends on

Nothing. It touches `lib.rs` beside the `ae1` units: land after `ae1a`, or rebase.

## Changelog

- `ess-http/1`: `cors:` states which origins may read a binding's answers, with credentials,
  request and exposed headers and the preflight's lifetime; OpenAPI writes the preflight `options`
  operation.

## Fit review

1. **Need.** Which browser origins may read a service's answers, and the preflight answer. The
   report described the wire (allow headers on every route, every preflight answered `204`) and
   proposed no syntax.
2. **Class: gap.** No key. The literal-header idiom moves the policy into copied lines and cannot
   state the preflight, so policy 1 refuses it as an answer.
3. **Expressible today?** Partly, and badly (above).
4. **Fit.** A policy key beside `limits:` and `refusals:`: top level, overridden per route, held by
   refinement. Allowed methods are derived from routes, so they cannot drift. The owner's note puts
   headers in the protocol kind (PR-D1, PR-D5). CORS is a server access policy whose header lines
   follow from the Fetch standard. No author writes them, so nothing moves if request headers move
   to the protocol document. PR-D6 A keeps response headers in `ess-http/1`. Siblings: `ess-transport`
   has no browser reader. A view and a command route are covered alike. The stream route of the SSE
   story is a route and is covered alike.
5. **Second adopter.** A public read API consumed by a single-page app on another origin, with a
   bearer token in `Authorization` (listed origins, `credentials: false`, `headers: [Authorization]`).
6. **Cost.** One optional key and a new refusal code. No format version (precedent: `limits`,
   `servers`). It is a security boundary, so the decision goes to the owner first (AE-D5).
7. **Considered.** (a) Change nothing: the contract misses CORS. (b) Authored headers and an
   `OPTIONS` route (AE-D5 B): needs `options` admitted for operations, and is fragile. (c) Decline
   as gateway policy (AE-D5 C). (d) Chosen: a policy key.

## Decisions

Accept, redesigned. **AE-D5** (SECURITY), default A, fail-closed: absent means no cross-origin
reads.

## Size

M, about 480 lines with tests.
