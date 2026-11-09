---
format: aep.planning-md/3
id: story:an-http-binding-projects-its-security-schemes
kind: story
status: draft
title: An HTTP binding projects its security schemes
summary: OpenAPI securitySchemes follow from the authentication port
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#681
relations:
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: architecture-decision-record:a-binding-authenticates-the-caller-through-a-port
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-http-binding-declares-how-a-caller-authenticates
revision: 4
---
## Need

The OpenAPI document projected for an `ess-http/1` binding carries no `components.securitySchemes`
and no per-operation `security`, so a client generated from it cannot know that a route needs a
credential, or which. Once the binding declares its schemes
(`story:an-http-binding-declares-how-a-caller-authenticates`, per
`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`), the projection
states them.

Reproduced on 0.92.0 (one route `post /orders` for a command granted to `example.order.Clerk`):

```console
$ ess generate --path model --kind openapi --http http.yaml --out gen
openapi/order-service.yaml — 8840 byte(s)
$ grep -nE 'security|securitySchemes' gen/openapi/order-service.yaml
(only the prose of info.description and x-ess-may-invoke; no security member)
```

The omission is by design until the binding has a construct
(`crates/generate/ess-gen/src/openapi.rs:164-191`; `docs/design/http-presentation-binding.md`
answer 8, lines 831-837).

## Acceptance

- `ess generate --kind openapi --http` over the binding of
  `story:an-http-binding-declares-how-a-caller-authenticates` writes
  `components.securitySchemes` with `bearer_auth: {type: http, scheme: bearer}` and
  `key_auth: {type: apiKey, in: header, name: X-Api-Key}` (`basic` as `{type: http, scheme:
  basic}`, `mutual_tls` as `{type: mutualTLS}`), and on each operation `security` with one
  requirement object per scheme in the route's list (`[{bearer_auth: []}, {key_auth: []}]`), `[]`
  on an open route.
- Each operation whose route requires a scheme lists the `unauthenticated` answer (`401` with its
  `WWW-Authenticate` header, or the declared override) among its responses.
- A binding without `authentication` projects byte for byte the document it projects today (an
  existing `openapi_http_presentation` test compares unchanged), and the module table in
  `openapi.rs` no longer lists `security`, `securitySchemes` as omitted for a binding that declares
  them.

## Scope

- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/src/http.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- inferred: website/docs/reference/formats.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** The projected contract must say which routes require which credential (issue #681).
   Proposal (theirs): project `securitySchemes` and per-operation `security`.
2. **Class.** Gap: the projection omits it because no declaration exists
   (`openapi.rs:164-191`), not because ESS contradicts a documented semantics.
3. **Already expressible?** No (`run.txt` § 2). Merging the members in a build step is outside the
   specification (policy 1, `docs/design/generic-language-features.md` § Policy).
4. **Fit.** A pure projection of the binding key designed in
   `story:an-http-binding-declares-how-a-caller-authenticates`; the OpenAPI member names are
   OpenAPI 3.1's own. `x-ess-may-invoke` stays (authorization), `security` is added
   (authentication), the split the module doc argues (`openapi.rs:178-191`).
5. **Second adopter.** Any service whose clients are generated from the projected document, e.g.
   an `example.ticket` API requiring an API key.
6. **Cost.** No format: the OpenAPI document is generated output; a binding without the key keeps
   its bytes.
7. **Alternatives.** Derive a scheme from each declared `401` challenge: rejected by design answer
   8 (claims a mechanism the model does not declare). A projection-only key: rejected (servers
   would ignore it). Chosen: project the binding's declaration.

## Decisions

Accept, redesigned: the projection follows from the binding's `authentication` key (ADR option B),
not from a projection-only key. Issue #681 closes when this story merges, after
`story:an-http-binding-declares-how-a-caller-authenticates` and the two server stories.

## Source

github:beyond10x/ess#681
