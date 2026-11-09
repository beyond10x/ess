---
format: aep.planning-md/3
id: story:an-http-binding-declares-how-a-caller-authenticates
kind: story
status: draft
title: An HTTP binding declares how a caller authenticates
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#681
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

An HTTP surface states how a caller proves who it is (a bearer token, an API key, a client
certificate), and an `ess-http/1` binding has no key to say it. The model states who may do what
(`may:`, `caller`); `architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`
(accepted 2026-10-23, option B) puts the scheme on the binding, and every artifact read from the
binding (OpenAPI, the Rust and Go servers) follows from that one declaration.

Reproduced on 0.92.0 (model: one command `example.order.PlaceOrder` granted to
`example.order.Clerk`, component `reached_by: network`; binding: one route `post /orders`):

```console
$ ess specify http validate --path model --binding http-auth.yaml
error: reading http-auth.yaml as HTTP binding YAML: unknown field `authentication`, expected one of `type`, `specification`, `refines`, `refusals`, `servers`, `routes` at line 7 column 1
```

## Acceptance

- `docs/design/http-presentation-binding.md` gains a stage-3 section "Authentication" and its
  answer 8 is revised to point at it: the key shape below, its validation, the `unauthenticated`
  standard refusal, the OpenAPI and server consequences, and the format consequence (`ess-http/1`
  and `ess-http-ir/1` gain optional keys, absent when not declared).
- `ess specify http validate` accepts:

  ```yaml
  authentication:
    schemes:
      bearer_auth: {kind: bearer}
      key_auth: {kind: api_key, in: header, name: X-Api-Key}
    required: [bearer_auth]
  routes:
    - command: example.order.PlaceOrder
      method: post
      path: /orders
      authentication: [bearer_auth, key_auth]
  ```

  `kind` is one of `bearer` (RFC 6750), `basic` (RFC 7617, with `realm`), `api_key` (`in: header |
  query | cookie`, `name`) and `mutual_tls`; a route's list (alternatives, any one suffices)
  replaces the top-level `required`, and `[]` makes the route open.
- Refused as `ESS-HTTP-034`, each naming the key: an unknown kind (`oauth2` and `open_id_connect`
  named as not yet admitted), a scheme name that is not an RFC 9110 token, a list naming an
  undeclared scheme, a declared scheme no route requires, `authentication` over a model that
  declares no actor, and a route whose required schemes carry no challenge (`api_key`,
  `mutual_tls` only) without a `refusals.unauthenticated` override.
- `refusals:` (top-level and per route) admits `unauthenticated` with a `4xx` status; a `401`
  there declares a `WWW-Authenticate` challenge (`ESS-HTTP-027`, `ESS-HTTP-029` widened).
- `ess specify http compile` writes `authentication` (schemes by name) after `servers` and each
  route's `authentication` after its `operation_id`, both absent when not declared, so every
  existing binding compiles to the same bytes; a refinement restates a refined route's
  `authentication`.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/reference/formats.md
- inferred: website/docs/reference/diagnostics.md
- inferred: crates/specify/ess-http/tests/authentication.rs
- inferred: crates/edge/ess-cli/tests/http_presentation.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A surface's authentication requirement is a fact the binding cannot state; the
   projected contract and the generated servers are then silent about it (`run.txt` § 2, § 3).
   The issue's proposal (theirs, from the issue body): emit `components.securitySchemes` and a
   per-operation `security:` in the projected OpenAPI.
2. **Class.** Gap: no key in `ess-http/1` (`run.txt` § 3; the closed key list in
   `website/docs/reference/formats.md:208`), and the omission is documented as waiting for a
   construct (`docs/design/http-presentation-binding.md:831-837`,
   `crates/generate/ess-gen/src/openapi.rs:164-191`).
3. **Already expressible?** No. A declared `401` with a `WWW-Authenticate` challenge on one
   outcome (`ESS-HTTP-027`, design answer 18) states an answer, not a requirement; the only
   workaround is merging `securitySchemes` into the projected document by hand, which takes the
   figure out of the specification (policy 1, `docs/design/generic-language-features.md` § Policy).
4. **Fit.** Not a source-language cell (no grid R or grid O cell, `docs/design/language-grids.md`);
   location decided by the accepted ADR (binding, not `ess/N`). Reuses the binding's existing
   shapes: a top-level default with a per-route override (`refusals:`, design line 504 and
   formats.md:208), the challenge vocabulary of `ESS-HTTP-027`, the `operation_id`/`servers`
   pattern of optional IR keys absent when undeclared. The servers already take the realization's
   `authenticate` hook (`crates/generate/ess-synth/src/rust/http.rs:733-758`,
   `go/http.rs:1302-1314`); this gives it a declared answer. Sibling bindings: `ess-cli/2` is
   local (no caller to authenticate); the event transport's broker credentials are a different
   question (connection, not request) and are not opened here.
5. **Second adopter.** A ticketing API whose `example.ticket.*` commands require an API key in
   `X-Api-Key` and whose health view is open (`authentication: []`).
6. **Cost.** No `ess/N` change; `ess-http/1` and `ess-http-ir/1` gain optional keys, absent when
   undeclared, as `operation_id` and `servers` were added in 0.76.0 (`formats.md:208-209`). One
   new diagnostic code (`ESS-HTTP-034`), two widened (`027`, `029`). No migration.
7. **Alternatives.** (A) Scheme on the model's actors, a new `ess/N`: rejected by the ADR (puts
   transport mechanics into the transport-free model). (C) Change nothing and document the merge
   step: every adopter writes the same unchecked code. (P) A projection-only OpenAPI key: the
   servers would ignore it (a red flag of the assessing skill) and it would pick B's location
   without B's server behaviour. Chosen: B as the ADR records it.

## Decisions

Accept, redesigned. Need accepted; the request's shape (two OpenAPI members) becomes one binding
declaration that the OpenAPI projection, the Rust server and the Go server all read, per
`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`. Not in this
decision: OAuth 2.0 and OpenID Connect schemes (refused by name until a need names a flow), and the
ADR's open question of a scheme taken from a composed contract (additive later; nothing here
precludes it).

## Source

github:beyond10x/ess#681
