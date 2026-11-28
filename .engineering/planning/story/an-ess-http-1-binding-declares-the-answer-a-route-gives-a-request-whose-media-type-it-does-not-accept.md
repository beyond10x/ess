---
format: aep.planning-md/3
id: story:an-ess-http-1-binding-declares-the-answer-a-route-gives-a-request-whose-media-type-it-does-not-accept
kind: story
status: draft
title: An ess-http/1 binding declares the answer a route gives a request whose media type it does not accept
summary: 'A sixth closed standard refusal beside too_large: 415 unless declared, enforced by the generated servers where declared'
tags:
- external-request
- gap-reports-2026-11-25
relations:
- serves: vision:O2
- informed_by: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 3
---
## Outcome

An `ess-http/1` binding declares the answer a route gives a request whose media type it does not
accept: a sixth closed standard refusal beside `too_large`, `unsupported_media_type` (proposed
spelling, decision MT-D1), any `4xx` and `415` unless the binding says another, with the body and
headers a standard refusal takes today (`none` or one literal member, a header literal). It is
declared once for the binding and again per route where it differs, as the other refusals are.
The generated Rust and Go servers answer it before any command runs, and the OpenAPI projection
publishes the `415`. A binding that declares nothing keeps today's bytes and today's served
behaviour (decision MT-D2).

## Context

A downstream application, on 0.99.1 (report of 2026-11-25): a served JSON-body route answers `415`
with a fixed JSON body of one `error` member holding a literal text, to a request without a JSON
`Content-Type`, before any command runs. The binding cannot say so: 0.99.0 added
`limits.request_bytes` and `refusals.too_large` (`413`), and `refusals:` has no key for a wrong
media type. Effect at the reporter: 16 rows `status 415` stay exceptions beside its comparison
with the OpenAPI document. The reporter asks for a standard refusal for an unsupported media type
(`415`, body none or a literal) declared beside `too_large`.

What ESS does today (code read at `origin/main` 60679eac1d, 2026-11-25):

- The refusal keys are `malformed`, `not_granted`, `too_large`, `unauthenticated` and `unfinished`
  (`crates/specify/ess-http/src/lib.rs:1102-1149`); a refusal is a status, a body of `none` or one
  literal member, and headers (`lib.rs:1161-1229`). Any other key is `ESS-HTTP-029`
  (`lib.rs:4059`).
- The precedence list reads "not the declared media type" as the input being unreadable, step 6:
  `refusals.malformed`, else `400` (`docs/design/http-presentation-binding.md:847-849`).
  `too_large` is step 1, decided at head time (`:836-841`).
- The generated servers do not do what step 6 says for a JSON body. Rust `json_body`
  (`crates/generate/ess-synth/src/rust/http/bound.rs:488-494`) and Go `jsonBody`
  (`crates/generate/ess-synth/src/go/http/bound.rs:398-408`) never read `Content-Type`: a body of
  any media type that parses as JSON is run, and one that does not parse answers `malformed` (`400`).
  A `form` route checks its media type (Rust `content_type_is`, `bound.rs:457-471`, used at
  `bound.rs:1778`; Go `contentTypeIs`, `go/http/bound.rs:368-376`, used by `formObject`) and a
  `multipart` route checks the boundary (Rust `bound.rs:935-946`); both answer `malformed`
  (`400`). No server answers `415`. `crates/generate/ess-synth/src` has Rust and Go HTTP servers
  and no TypeScript one.
- OpenAPI: `too_large` projects a response at its status, only where a limit or the refusal is
  declared (`crates/generate/ess-gen/src/openapi.rs:1807-1830`, `:2003-2007`; design page
  `:734`: a binding with neither changes no byte). No `415` response is written for any
  operation.

## Acceptance

Red test first. The bodies below are for the picks recommended in `## Decisions`; a different pick
changes the matching line.

1. A binding with `refusals: {unsupported_media_type: {status: 415, body: {error: {literal:
   unsupported_media_type}}}}` validates and compiles; a second compile writes identical bytes; the
   key is also accepted under `routes[i].refusals`. Red today: the key is `ESS-HTTP-029`
   (`lib.rs:4059`).
2. `status: 500` and the key written twice are `ESS-HTTP-029` at the key; a body other than `none`
   or one literal member is `ESS-HTTP-029` at `.body`; the message of an unknown key lists the six
   keys.
3. A generated Rust server and a generated Go server, each built once for the whole test run, answer
   a `POST` to a JSON-body route with `Content-Type: text/plain` and with no `Content-Type` and a
   body with the declared status, headers and body, and no command runs (the model's state is
   unchanged). `application/json` and `Application/JSON; charset=utf-8` run the command.
4. Both servers answer the same bytes for the same request (parity, as the `too_large` story
   holds).
5. A route whose body is not required and empty is not refused for its missing `Content-Type`
   (MT-D4).
6. OpenAPI: each operation that reads a body of a checked encoding gains a `415` response with the
   declared body; a binding that does not declare the key produces byte-identical OpenAPI, Rust
   and Go output (every committed byte table unmoved).
7. With the key declared on a `form` route, a request of another media type answers it and no longer
   `malformed` (MT-D3); with it undeclared the `400` stays.

## Fit review

Covers finding 1 of the downstream application's report of 2026-11-25 (0.99.1).

1. **Need.** A fact about a request before the model sees it cannot be stated: a binding cannot say
   what a route answers to a media type it does not accept. A caller of the OpenAPI document
   cannot learn the `415`, and 16 rows stay exceptions at the reporter.
2. **Class.** Gap. Not local policy: refusing a body of another media type is a property of every
   HTTP surface that reads a body, and the format already declares the three neighbours of it
   (`too_large`, `malformed`, `unauthenticated`).
3. **Idiom.** Every one fails the skill's question 3. `refusals.malformed: {status: 415}` re-statuses
   every unreadable input as `415`, and the generated JSON path never checks the media type, so it
   would not even answer for the case. A model-level guard cannot see a request header. An outcome
   at `415` runs the command first, which is the opposite of the reporter's "before any command
   runs".
4. **Fit.** Same shape as `too_large`: a closed key of `refusals:` at both levels (binding default,
   route override standing in place), the refusal vocabulary (`status`, `body`, `headers`,
   `ESS-HTTP-029`), the precedence list gaining one step (MT-D6). It composes with refinement
   (a refining document restates it, `ESS-HTTP-012`) and with `ESS-HTTP-025` (a `415` is
   told apart from an outcome by status; an outcome that answers `415` on the same route follows
   the existing rule). The web bridge and the interpreter hold no route and are untouched.
5. **Scope of the story: `415` only**, not "the remaining standard conditions". Evidence: of the
   neighbours, `too_large`,
   `unauthenticated` and `not_granted` are declarable already; an unreadable body (`400`) is
   `malformed`, which exists, and what the second case asks of it is a body shape and a second
   body on one status, which is `story:a-json-rpc-2-0-binding-declares-the-error-object-a-request-it-cannot-read-size-or-accept-is-answered-with`;
   the rest of the family (`411`, `431`, a time limit) is fixed in the generated servers and no
   adopter has asked for it, and the `too_large` story already ruled that each such key comes by its
   own story when a projector needs it, never by a registry. So this story is one key, the one
   asked for.
6. **Second adopter.** A service behind a gateway that answers `415` to a body of another media
   type, or a service whose upload route answers `415` to a part of the wrong type (the `multipart`
   `media_types` check is the same fact one level down). Inferred, not reported.
7. **Cost.** One optional key in `ess-http/1`, written to `ess-http-ir/1` only when present,
   so a binding without it keeps its bytes; no new diagnostic code (`ESS-HTTP-029` widened);
   Rust and Go servers gain one check; OpenAPI gains one response where declared.
8. **Considered.** (a) Change nothing: the OpenAPI document keeps stating no `415` and the
   reporter's rows stay exceptions. (b) The reporter's ask, a key beside `too_large`: chosen,
   spelling proposed by this review, the reporter named none. (c) A `request.media_types` list
   per route: an open set of media types the servers cannot write (`## Still refused` of the design
   page); not chosen. (d) Enforcing `415` on every JSON route with no declaration: MT-D2 B.

## Decisions

All open, the coordinator's. Recommended option first.

- **MT-D1. The key.**
  - A (recommended): `refusals.unsupported_media_type`, any `4xx`, `415` unless declared, at
    binding level and `routes[i].refusals`.
  - B: fold into `malformed` with a per-cause status: needs a way to name the cause, a second key
    in effect, and re-statuses every unreadable input where it is set.
  - C: a `request.media_types` list per route: an open set, see Fit review 8 (c).
- **MT-D2. Do the generated servers start to enforce the media type?** Today they do not for a
  JSON body, so this changes served behaviour.
  - A (recommended): they check a JSON route's `Content-Type` only where the route's effective
    binding declares the refusal; a binding that declares nothing keeps today's behaviour and
    bytes. Fails closed for a binding that declares it, unchanged for the rest.
  - B: every JSON route is checked and answers a fixed `415` with `{"refused": ...}` where
    nothing is declared (as the fixed `413` does): fails closed everywhere, changes the served
    behaviour of every existing binding and the bytes of every committed server and OpenAPI table.
  - C: declaration only, servers unchanged: the OpenAPI document states a `415` the server never
    sends; fails open.
- **MT-D3. Which encodings.**
  - A (recommended): every encoding that names one media type (`json`, `form`, `multipart`): where
    declared, a wrong media type answers this refusal instead of `malformed`; undeclared, `malformed`
    stays.
  - B: `json` only; `form` and `multipart` keep `malformed` (`400`).
- **MT-D4. What is accepted for `json`.**
  - A (recommended): the essence `application/json`, parameters and case ignored (as `content_type_is`
    does for `form`); a route whose body is not required and empty is not checked.
  - B: also any `+json` suffix.
  - C: also a missing `Content-Type` with a non-empty body.
- **MT-D5. OpenAPI.**
  - A (recommended): a `415` response on each operation reading a checked encoding, only where the
    binding declares the refusal; other bindings keep their bytes.
  - B: always.
- **MT-D6. Precedence position.**
  - A (recommended): a new step between 5 (`input_absent`) and 6 (the input cannot be read), after
    the grant check, so a caller no grant admits learns nothing of the route's media type.
  - B: with step 1, at head time before the grant check.

Would add to `ess-http/1`: `refusals.unsupported_media_type` and
`routes[i].refusals.unsupported_media_type`, and the same key in the compiled refusals of
`ess-http-ir/1` (waits for the owner's read before a release). A release that carries the key
names its picks of MT-D1 to MT-D6 in its pull-request line. No architecture change is decided here.

## Source

A downstream application, report of 2026-11-25, finding 1 (ESS 0.99.1).
