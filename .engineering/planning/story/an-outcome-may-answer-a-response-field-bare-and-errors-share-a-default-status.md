---
format: aep.planning-md/3
id: story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status
kind: story
status: implemented
title: An ess-http/1 outcome may answer one response field bare, and error outcomes may share a default status
summary: 'body: response keeps the response object wrapper; upstream answers the bare message and puts errors under default (a2a wave 2, H6, 2026-10-22).'
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T05:42:26Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T05:42:27Z", actor: "human:timo", revision: 6, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:22Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H6**, a2a wave-2 HTTP binding gaps list, line 13. Upstream answers
the bare response message (a `v1Task`); ESS answers `{outcome, published, response}`, and errors
cannot be put under one `default` status. No refusal: the adopter did not try, because
`outcomes: {<o>: {body: response}}` still keeps the `{task: …}` wrapper (the command's response
object, whose one field is the task).

## Where it lives

- Body forms: `crates/specify/ess-http/src/lib.rs:484-497`: `Envelope` (the default) and `Response`,
  "the command's declared response object, bare". The gap is one level further: answering the value
  of the response's single field, not the response object.
- Status per outcome: `ESS-HTTP-023` and `-025` (`lib.rs:2509`, `lib.rs:2511`) hold each outcome to
  its own status; there is no `default` status for every error outcome, as OpenAPI's `default`
  response and the upstream document use.
- Overlaps `story:response-field-declares-its-value-source` (a2a wave 1, M7), which concerns the
  response's fields, not its wire body; checked, not a duplicate.

## Format

A new body form (for example `response.<field>`) and a default status are new keys or words in
`ess-http/1`; documents valid today keep their bytes, so additive within `/1` under the stage-2
precedent. `ess/N` unchanged. Fit review decides.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), reproduced on 0.68.0.

- Count: 39 generated error sites in the adopter's server answer with ESS's envelope at status 502 where the protocol declares another status.
- Reproduction (minimal spec, a command with an `external:` refusal and no `outcomes:` in its binding): the generated `--layout crate --http` server answers that refusal with status 502 and the envelope body. With `outcomes: {unsupported: {status: 400, body: {error: error}}}` declared, the same server answers 400 with the error object. So a declared per-outcome status works; the 39 sites are refusals the binding lists no outcome for.
- Where it lives: an `external:` refusal derives `UPSTREAM` = `"502"`, `crates/specify/ess-http/src/status.rs:28` and `:74` (traced through the generated output). The missing piece is the default this story names: one status and body shared by every error outcome a route does not list, so 39 sites need not be listed one by one.

## Fit review

Assessed with `.agents/skills/assessing-external-requests/SKILL.md` against the integration branch
`integrate/ess-batch-0690` at `d3f407440a`. Two needs, reviewed together because both sit on one
route's answer; each gets its own verdict.

### 1. The need, apart from the proposed syntax

- **(a) A bare value of one response field.** A protocol answers a command's success with one
  message, bare. ESS can answer the command's declared response object bare (`body: response`,
  `crates/specify/ess-http/src/lib.rs:486-501`, design page "Response body per outcome",
  `docs/design/http-presentation-binding.md:315-329`), but never the value of one of its fields: a
  command whose response is `{task: Task}` answers `{"task": {…}}` where the protocol answers
  `{…}`. Minimal reproduction: `crates/specify/ess-http/tests/fixtures/orders.yaml`, command
  `catalog.orders.PlaceOrder` (response `{order_id}`), route
  `outcomes: {placed: {status: 200, body: response}}` compiles to `body: response`, and both
  servers write `http::bare(response)` (`crates/generate/ess-synth/src/rust/http/bound.rs:985-987`,
  `crates/generate/ess-synth/src/go/http/bound.rs:1383-1385`): the body is `{"order_id":"o1"}`, never
  `"o1"`. No body word spells one field (`lib.rs:4031-4053`, the `"response"` arm is the only one
  that reads the response).
- **(b) One answer for every error outcome a route does not list.** A protocol answers every error
  with one status and one body shape. ESS answers an unlisted outcome at its derived status with the
  envelope (`lib.rs:2509-2511`; design page `:333`; `crates/generate/ess-gen/src/openapi.rs:1274-1278`).
  An `external:` refusal derives `UPSTREAM` = `502` (`crates/specify/ess-http/src/status.rs:28`,
  `:74`). Minimal reproduction (a new minimal specification, not the adopter's): a command with one
  `external:` error outcome and a route with no `outcomes:` answers that refusal `502` with the
  envelope; listing it, `outcomes: {unsupported: {status: 400, body: {error: error}}}`, answers
  `400` with the error object. So a declared per-outcome answer works, and the only way to give 39
  refusals the protocol's answer is 39 listings.
- **Requester's proposal, labelled as theirs:** "put errors under one `default` status", by analogy
  with OpenAPI's `default` response and the upstream document. For (a) no syntax was proposed; the
  story suggests `response.<field>` "for example".

### 2. Class

- (a) **gap.** No authored form produces the bare field value; the response object is the smallest
  body ESS can answer (`lib.rs:4040-4053`).
- (b) **convenience at the authoring level, gap at the checking level.** Every unlisted error can be
  listed one by one today, so it is expressible, but the figure the reader checks — "every error of
  this route answers `400` with `{error: <wire>}`" — is then 39 separate figures, and a new error
  outcome added to the command later silently takes its derived status and the envelope on that
  route (`lib.rs:4708-4713`, the unlisted branch of `distinguishable`). The domain fact "this route
  answers all its errors one way" cannot be stated. Treated as a gap.

### 3. Can it already be expressed?

- (a) No. Searched: `body` words (`lib.rs:4031-4053`), design page `:315-329`, the guide
  `website/docs/guides/http-routes.md:124-157`. Changing the specification's response to the
  inner message's fields would make the command's response lie about the domain (an idiom that
  moves the checked figure into the binding's reader); refused as an idiom.
- (b) Only by listing each error outcome (above). Not a valid idiom by the skill's condition: the
  shared figure is not checked as one, and a later error outcome escapes it.

### 4. Fit with what is there

Grid triage: `docs/design/language-grids.md` covers the `ess/N` source language (grid R rows such as
`binding cause where:` at `:37`); `ess-http/1` is a presentation binding over a compiled `ess/N`
model and opens no grid cell. The rows it does open are this format's own:

- **Vocabulary.** `response.<field>` is already the spelling of "one field of the command's
  response" in this format: a header `{source: response.<field>}` (design page `:362`,
  `lib.rs:1033-1059`, `AnswerSource::Response`) and a redirect `location: response.<field>`
  (design page `:481`). A body reusing that text keeps one spelling for one idea.
  `default` is not reused: in OpenAPI it means "any status not listed", while the generated
  servers answer exactly one status; and an outcome may be named `default`, so `outcomes.default`
  would collide with a listed outcome. The concept is "the error outcomes not listed", so the key is
  `errors`.
- **Composition.** `errors` holds exactly what a listed outcome holds (`OutcomeSpec`,
  `lib.rs:599-612`: `status`, `body`, `headers`, `redirect`) and means exactly "this outcome
  listed under every error outcome of the command the route does not list". Every per-outcome
  check therefore applies per covered outcome unchanged: status class (`ESS-HTTP-023`), body
  (`-024`), distinguishability on one status (`-025`, `lib.rs:4671`), headers and challenges
  (`-026`, `-027`), redirects (`-028`). `refusals:` overrides keep their own rule against the
  answers on their status (`overridden`, `lib.rs:4780`).
- **Siblings.** A body from a source: of the four source forms (`lib.rs:1033-1059`) only
  `response.<field>` is opened. `error.<field>` has its own body form already
  (`{<member>: error}`, the whole error object), `input.<field>` echoing input is no answer to a
  domain fact, and `error.wire` alone is a literal. Each is refused by name under `ESS-HTTP-024`.
  A default for success outcomes is not opened: an unlisted success keeps the envelope, whose
  `outcome` member names the branch; a shared bare success body would erase which branch ran.
  A binding-wide `errors:` beside `refusals:` is not opened (see 7).
- **Targets.** Each target reads the IR through one lookup, so none can ignore the new forms:
  the OpenAPI projection (`openapi.rs:1274`, `:2096`), the Rust and Go servers
  (`ess-gen/src/http.rs:780-784` `listed`, used by `rust/http/bound.rs:908` and
  `go/http/bound.rs:1311`), and the server's "does this route declare answers" switch
  (`rust/http/bound.rs:863`, `go/http/bound.rs:1263`). Interpreter, conformance runners, the web
  bridge and `ess verify diff` do not speak HTTP (design page `:431-442`).

### 5. A second, unrelated adopter

- (a) A document-store API answers `GET`-like commands with the stored document bare and puts its
  version in an `ETag` header: response `{document: Document, version: String}`, binding
  `body: response.document`, `headers: {ETag: {source: response.version}}`. Hence any top-level
  field, not only a response's single field.
- (b) A payments API that answers every refusal `400` with `{"type": "<wire name>", …fields}` (the
  common problem-object shape), whatever decided the refusal: `errors: {status: 400, body: {type: error}}`.

### 6. Cost

- **Format.** Additive within `ess-http/1` and `ess-http-ir/1`, the stage-2 precedent
  (design page `:444-449`): one new body word form `response.<field>` and one new optional route key
  `errors`, each omitted from IR when absent. A document valid today compiles to the same IR bytes,
  and an OpenAPI document from it keeps its bytes. `ess/N` unchanged. No ADR: no new format, no
  change to the IR's architecture or to a published contract.
- **Diagnostics.** No new code. `ESS-HTTP-022` widens to `errors` on a view and `errors` covering
  no outcome; `-023` to `-028` are reported at `routes[i].errors…` per covered outcome;
  `ESS-HTTP-012` covers `errors`; `-024` gains `response.<field>` clauses (field named, field not
  `Optional`, on an outcome that may answer `response`).
- **Generated code.** The servers gain one body writer (a member of the response, bare); the
  OpenAPI projection gains the field's schema as a response body.
- **Migration, diff classes, API.** None for documents; `ess_gen::http::listed` takes the outcome
  rather than its name (a crate-internal generator API, two callers).

### 7. What else was considered

| Design | Verdict |
|---|---|
| Change nothing: list every error outcome; reshape the response | refused: the shared figure is not checked as one, a later error escapes it, and reshaping the response falsifies the domain (3) |
| Requester's `default` status (OpenAPI `default` response, `outcomes.default`) | changed: `default` is a projection term that claims any status, and collides with an outcome name; the domain term is "error outcomes not listed" |
| `errors:` with its own narrower shape (status and body only) | refused: a second shape for one concept; reusing `OutcomeSpec` makes the key mean "listed under each", so every existing check and writer applies unchanged |
| Binding-wide `errors:` beside `refusals:` | deferred, additive later: the route is where `outcomes:` lives, and how many routes carry the 39 sites is not known (I don't know); per-route `errors:` is one line per route |
| Body `response.<field>` only for single-field responses | refused: the second adopter (5) answers one field and heads another |
| Body `{source: response.<field>}` (the header value form) | refused: a body map is already `{<member>: error}` / `{<member>: {literal}}`, so `{source: …}` would be read as a member named `source`; the bare source text is the redirect `location` form |
| Admit an `Optional` field | refused for now: an absent value would answer a `null` body; additive to admit later |

## Decisions

- **(a) accept, redesigned.** `body: response.<field>`: the value of one top-level field of the
  command's declared response, bare, under its type's JSON. Admitted wherever `body: response` is
  (an outcome with `returns: true` and no error); the field must be declared and not `Optional`;
  any other source text as a body is refused (`ESS-HTTP-024`). Changed from the request: the
  requester asked for the bare inner message; the spelling reuses the format's existing source text,
  and any field is admitted rather than only a single field.
- **(b) accept, redesigned.** Route key `errors: {status, body, headers, redirect}`, the shape of a
  listed outcome, meaning "listed under every error outcome of the command this route does not list
  under `outcomes:`". Every per-outcome check runs for each covered outcome, reported at
  `routes[i].errors…` naming the outcome; `errors` on a view, or covering no outcome, is
  `ESS-HTTP-022`. OpenAPI answers each covered outcome at the declared status (not an OpenAPI
  `default` response), named in `x-ess-outcomes`. Changed from the request: not `default` (a
  projection term, an outcome-name collision), and per route rather than binding-wide (deferred).
- **Format.** Both additive within `ess-http/1` and `ess-http-ir/1`; old documents keep their IR
  and OpenAPI bytes (held by a compatibility test). No new format version, no ADR, no new code.
