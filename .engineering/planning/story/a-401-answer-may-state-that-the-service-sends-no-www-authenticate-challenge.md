---
format: aep.planning-md/3
id: story:a-401-answer-may-state-that-the-service-sends-no-www-authenticate-challenge
kind: story
status: draft
title: A 401 answer may state that the service sends no WWW-Authenticate challenge
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:an-http-binding-declares-how-a-caller-authenticates
- depends_on: story:an-http-binding-declares-how-a-caller-authenticates
revision: 3
---
## Need

An existing service answers an unauthenticated call with `401` and no `WWW-Authenticate` header.
`ess-http/1` refuses every declared `401`, an outcome or a standard refusal, that has no
challenge:

- `crates/specify/ess-http/src/lib.rs:3023-3028`: "`<at>` answers `401` without a
  `WWW-Authenticate` challenge, which compiled IR writes on every `401` (RFC 9110 §15.5.2)",
  `ESS-HTTP-027`.
- Deliberate: `docs/design/http-presentation-binding.md` § "Decided for stage 2", answer 18,
  follows RFC 9110 §15.5.2, under which the server MUST send one. The reporter read it as "ESS
  requires a `Bearer` challenge". It requires a challenge of some scheme, and `Bearer` is the
  example's scheme.

A specification of the real service therefore either states a header the service does not send,
and the stage-3 HTTP target and the generated OpenAPI then claim it, or it leaves the route
derived. Fourth report (a service contract), finding 3, second half.

Minimal reproduction (written fresh; not run): command `demo.docs.ReadDocument` with an outcome
`not-authenticated` (`error: demo.docs.NotAuthenticated`).

```yaml
    outcomes:
      not-authenticated:
        status: 401
        body: none
```

```console
$ ess specify http validate --path spec --binding docs.http.yaml
error[ESS-HTTP-027]: routes[0].outcomes.not-authenticated.headers: `routes[0].outcomes.not-authenticated` answers `401` without a `WWW-Authenticate` challenge, which compiled IR writes on every `401` (RFC 9110 §15.5.2)
```

## Outcome

Under **AC-D4**, option A, a `401` answer (an outcome, `errors:`, or a standard refusal, top-level
or per route) may state `headers: {WWW-Authenticate: none}`. The binding then says the service
sends no challenge. The IR records it. OpenAPI writes no `WWW-Authenticate` header for that response
and adds a description sentence saying the service omits the challenge RFC 9110 §15.5.2
requires. The generated servers send none. A `401` that states neither a challenge nor `none` stays
refused, so the RFC default holds unless a binding opts out by name.

## Acceptance

- The reproduction with `headers: {WWW-Authenticate: none}` validates, and `ess specify http
  compile` writes it. A binding without it compiles to the same bytes.
  `crates/specify/ess-http/tests/compile.rs` gains the case, starting from the refusal above.
- Refused, `ESS-HTTP-027` widened: `none` under any header but `WWW-Authenticate`; `none` on a
  status other than `401`; `none` beside a challenge. The refusal of a bare `401` names `none` as
  the explicit way out.
- The `refusals.unauthenticated` standard refusal of
  `story:an-http-binding-declares-how-a-caller-authenticates` takes the same `none`, top-level and
  per route. A route whose required schemes carry no challenge (`api_key`, `mutual_tls`) may then
  answer its unauthenticated refusal `401` with `none`, and that story's refusal of such a route
  without an override names `none` beside a challenge as the way out.
- `ess generate --kind openapi --http` writes no `WWW-Authenticate` header object for that response,
  and adds the description sentence. A binding without `none` keeps its bytes.
- The generated Rust and Go servers answer that `401` with no `WWW-Authenticate` header
  (`http_binding_rust`, both servers alike). Every existing byte pin passes unmodified.
- `docs/design/http-presentation-binding.md` answer 18 states the exception and its reason. "Still
  refused" no longer implies a challenge on every `401`.

## Fit review

1. **Need.** A service's `401` without a challenge cannot be stated truthfully. Report: third
   report, finding 3. The reporter's wording: "on `401` ESS requires a `Bearer` challenge the real
   service does not send".
2. **Class: gap under a deliberate rule.** The rule is answer 18 of the design page, which follows
   RFC 9110. It is no defect: ESS does what its page says.
3. **Already expressible?** No. Any `401` without a challenge is refused (`lib.rs:3023-3028`).
4. **Fit.** It reuses the `headers:` map of an answer and the word `none` that bodies already use for
   "nothing is written" (§ "Response body per outcome"). Siblings: `Proxy-Authenticate` is not
   admitted at all (answer 18), so it needs nothing.
5. **Second adopter.** A retrofit of a legacy JSON API whose gateway strips `WWW-Authenticate`.
6. **Cost.** One admitted value in `ess-http/1`, absent by default, so no format version.
   Security: omitting a challenge grants no access. The `401` still refuses. Clients that negotiate
   a scheme from the challenge cannot, which is what the service does today.
7. **Considered.** See AC-D4.

## Decisions

**AC-D4**, default A (admit the declared exception). Options and costs are in the gap page's open decisions (`.engineering/waves/gap-reports-2026-11-24.md`, AC-D4).

- S2: stays the owner's.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (challenge check 3023-3028, header values)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs` (header writing reads the IR)
- cited: `docs/design/http-presentation-binding.md`
- Size: S, about 250 lines with tests.

## Depends on

`story:an-http-binding-declares-how-a-caller-authenticates` (active, beyond10x/ess#681). That story
owns `ESS-HTTP-027`'s challenge rule for `refusals.unauthenticated` and widens the same check in
`crates/specify/ess-http/src/lib.rs`; this story adds the one exception to it and lands after it.
It is derived from that story: the authentication surface is that story's, and this one only
admits a declared absence of the challenge.

## Source

A downstream application, fourth report (a service contract), finding 3 (ESS 0.98.0).
