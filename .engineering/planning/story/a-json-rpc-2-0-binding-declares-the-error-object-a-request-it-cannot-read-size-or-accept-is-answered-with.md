---
format: aep.planning-md/3
id: story:a-json-rpc-2-0-binding-declares-the-error-object-a-request-it-cannot-read-size-or-accept-is-answered-with
kind: story
status: draft
title: A JSON-RPC 2.0 binding declares the error object a request it cannot read, size or accept is answered with
summary: Protocol-level answers (-32700, -32600, id null, 413, 415) of the sibling binding; the part of the envelope story and of the literal-only refusal story left open
tags:
- external-request
- gap-reports-2026-11-25
relations:
- serves: vision:O2
- depends_on: story:a-command-binds-to-json-rpc-and-grpc-beside-http
- informed_by: story:an-ess-http-1-binding-declares-the-answer-a-route-gives-a-request-whose-media-type-it-does-not-accept
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 1
---
## Outcome

A JSON-RPC 2.0 binding states the answer a server gives a request that fails before any method is
known: a body that is not JSON (`-32700`), a body that is JSON and no request object (`-32600`),
a body larger than the endpoint reads, and a body of an unsupported media type. Each answer is an
error object with fixed members (`jsonrpc`, `error.code`, `error.message`, `id: null`), and over HTTP
it carries the declared status. The generated server and the projection answer and publish them
from the binding. One status of the endpoint then has one body shape. The binding that holds this is
the sibling binding of `story:a-command-binds-to-json-rpc-and-grpc-beside-http` (decision JR-D1).

## Context

A downstream application, on 0.99.1 (report of 2026-11-25, second case): one route that speaks
JSON-RPC 2.0 answers a body it cannot read as a JSON-RPC error object with `id: null` under `400`
(codes `-32600` and `-32700`), under `413` and under `415`. The route already declares a `400` for a
command outcome, with a closed body of one `error` member, so the served `400` has two bodies. 7
rows stay exceptions beside the reporter's comparison with the OpenAPI document. The reporter asks
whether the stories of issue 650 (an envelope) plus issue 673 (a refusal body that is a literal
only) cover the case, or whether it needs a story of its own.

Answer, from the store (searched 2026-11-25; both issues are in it):

- Issue 650: `story:a-command-binds-to-json-rpc-and-grpc-beside-http` (active) and
  `story:json-rpc-binds-an-exchange` (draft) cover the envelope: a method per command or view,
  by-name params, the request `id` and notifications, one code per ESS error, batch refused
  (`docs/design/rpc-bindings.md:26-49` and `:88-93`; the story's `ESS-JSONRPC-001` to `-008`). Not
  covered: the answers before a method is known. No acceptance line, refusal code or design
  sentence names `-32700`, `-32600`, `id: null`, a body too large or a media type, and the
  binding is not built (no `ess-jsonrpc` crate at `origin/main` 60679eac1d).
- Issue 673: `story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views`
  (implemented) covers a refusal per route and a refusal body of `none` or one literal member
  (`crates/specify/ess-http/src/lib.rs:1176-1229`; `ESS-HTTP-029` at `:4059`). Not covered: a
  refusal body that is an object with fixed members, or one condition answered by two bodies.
- Left, held by no story: (1) the protocol-level answers of the envelope, with the codes the
  protocol fixes; (2) a refusal body shaped as the protocol's error object; (3) one refusal
  applying to `400`, `413` and `415` at once with a different code and text per cause. Part (2)
  makes the "two bodies on one status" of the report: `ESS-HTTP-025` (`lib.rs:4055`) admits a
  refusal beside an outcome on one status only as `{member: literal}` beside `{member: error}` on
  one member, so an object-shaped refusal does not exist to be told apart.
- The `415` and `413` parts of the report are the declaration of `story:an-ess-http-1-binding-declares-the-answer-a-route-gives-a-request-whose-media-type-it-does-not-accept`
  and of the implemented `too_large` refusal, if the route stays under `ess-http/1`.

What the format does about codes: the design page leaves a binding's own codes to
`-32099..-32000` and refuses the rest of `-32768..-32000` (`rpc-bindings.md:88-93`), so an author
cannot spell `-32700` or `-32600`; they belong to the protocol.

## Acceptance

Red test first. The lines are for the picks recommended in `## Decisions`.

1. `docs/design/rpc-bindings.md` states the four conditions, their fixed codes (`-32700`,
   `-32600`), `id: null`, and what a document may set (message; over HTTP the status).
2. A binding document that sets them validates and compiles to identical bytes twice; a code the
   protocol fixes, spelled by the author, is refused by name; a condition declared twice is refused
   by name.
3. A generated server answers a body that is not JSON with `error.code -32700` and `id: null`, a
   body that is JSON and no request object with `-32600` and `id: null`, a body over the cap with the
   declared status and an error object, and a body of another media type likewise, and no method
   runs.
4. A method's own error and a protocol-level answer on one HTTP status both have the envelope's
   shape: the endpoint's answers on one status have one body shape.
5. The projection of the binding lists the four answers; a binding that declares none keeps the
   bytes the binding story pins.

## Fit review

Covers the second case of the downstream application's report of 2026-11-25 (0.99.1).

1. **Need.** A server that speaks the protocol must answer a request it cannot read in the
   protocol's own error object; the binding cannot state it, so the projected contract and the
   served bytes disagree on 7 rows.
2. **Class.** Gap in a binding that is designed and not built; the answer belongs to its design
   before its code. Not local policy: every server of the protocol owes these answers.
3. **Idiom.** None. A refusal under `ess-http/1` holds one literal member, which is not the
   protocol's object, and the requester's route cannot move the envelope into `ess-http/1`: the
   envelope story refuses an `envelope` key there.
4. **Fit.** The `-32700` and `-32600` rows are the protocol's reserved codes
   (`rpc-bindings.md:88-93`), so the binding writes them and the author states only what the
   protocol leaves open. It reuses the refusal vocabulary of `ess-http/1` where a condition is
   transport (size, media type), so the two bindings read alike.
5. **Second adopter.** Any server of the protocol: the codes and `id: null` are the specification
   of the protocol.
6. **Cost.** Keys in `ess-jsonrpc/1`, which is not released, so no released format moves under
   JR-D1 A; under JR-D1 B a key of `ess-http/1` (below).
7. **Considered.** (a) Change nothing: the seven rows stay exceptions. (b) A nested literal body
   for a refusal in `ess-http/1`, so the route stays there: JR-D1 B. (c) The reporter's reading,
   issues 650 and 673 together: they leave the three parts above open.

## Decisions

Open, the coordinator's. Recommended first.

- **JR-D1. Where the answers are declared.**
  - A (recommended): in `ess-jsonrpc/1`, the sibling binding. The protocol's codes are written by
    the generators, the author sets message and, over HTTP, status. `ess-http/1` is untouched. The
    route moves to `ess-jsonrpc/1` when that binding has a server generator.
  - B: `ess-http/1` gains a refusal body of an object with fixed nested members, so the route
    stays under `ess-http/1`. Conflicts with the decision of
    `story:a-command-binds-to-json-rpc-and-grpc-beside-http` (the envelope is the sibling binding;
    the `envelope` key is refused in `ess-http/1`) and with the literal-only body of the
    implemented `ess-http-admits-text-bodies-...` story. Would add to `ess-http/1`: an object body
    of a refusal (waits for the owner's read before a release).
  - C: decline: the server's code answers them and the rows stay exceptions; the declared contract
    keeps stating no protocol error.
- **JR-D2. HTTP status of a condition.**
  - A (recommended): the binding states a status per condition for the HTTP carriage, defaults
    `400`, `400`, `413`, `415`.
  - B: left to the transport binding (`ess-transport/1`). Architecture: the owner's.

Nothing here adds a key to a released format under JR-D1 A. No architecture change is decided here.

## Depends on

`story:a-command-binds-to-json-rpc-and-grpc-beside-http` (the binding it extends; not built).
Informed by the `415` story named above for the media-type condition.

## Source

A downstream application, report of 2026-11-25, second case (ESS 0.99.1).
