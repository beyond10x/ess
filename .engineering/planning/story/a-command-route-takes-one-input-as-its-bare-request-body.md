---
format: aep.planning-md/3
id: story:a-command-route-takes-one-input-as-its-bare-request-body
kind: story
status: active
title: A command route takes one input as its bare request body
summary: 'ess-http/1 request.body: input.<field>, the rest on the path; OpenAPI, Rust and Go servers and both UIs (beyond10x/ess#748 part 1)'
tags:
- external-request
refs:
- provider: github
  reference: beyond10x/ess#748
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/http.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T10:55:30Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-06T10:55:31Z", actor: "human:timo", revision: 10}
---
## Outcome

An `ess-http/1` command route may take one input field as its whole request body, bare:
`request: {encoding: json, body: input.<field>}`. Every other input field of the command is carried
by the path. The OpenAPI projection, the generated Rust and Go servers and the generated UIs (React
and terminal) all read or send that body as the field's own JSON value.

Closes https://github.com/beyond10x/ess/issues/748 together with
`story:a-literal-path-segment-takes-precedence-over-a-variable-one` and
`story:an-each-write-may-take-repeated-identities-in-list-order`.

## Fit review

1. **Need.** A service takes a JSON array as the whole body of a command, with the owner's identity
   in the path: `PUT /boards/{boardId}/cards/priority` with body `[{"id": "c-1", "priority": 1}]`.
   Minimal reproduction: command `demo.board.SetCardPriorities {board_id: BoardId, priorities:
   List<CardPriority>}` with `CardPriority {id: CardId, priority: Integer}`, bound `put
   /boards/{board_id}/cards/priority` with `parameters: {board_id: input.board_id}`. Today the body
   is published as the object `SetCardPriorities.Input` (`crates/generate/ess-gen/src/openapi.rs:1146-1180`,
   `$ref …Input`). The requester's syntax, theirs: `request: {encoding: json, body: input.priorities}`.
2. **Class: gap.** The request side has no bare form. The response side has one: `body:
   response.<field>` answers one response field bare (`docs/design/http-presentation-binding.md:321`).
   A retrofitted specification of an existing service cannot state the body that service reads.
3. **Already expressible?** No. `request` holds `encoding` and `unknown` only
   (`crates/specify/ess-http/src/lib.rs:267-276`, `deny_unknown_fields`); `encoding: array` is
   `ESS-HTTP-020`; a route-level `body:` is a view key (`ESS-HTTP-024`). The issue lists each refusal
   on 0.87.0.
4. **Fit.** The key mirrors `response.<field>` on the response side: the same `body` word, the same
   `<namespace>.<field>` source grammar as `parameters:` (`input.<field>`). It composes with path
   `parameters` (the path carries the rest) and with `encoding: json` only. Sibling question: a view
   has no request body, so the key is refused on a view route, as `request` already is. Targets:
   OpenAPI writes the field's schema as the request body; Rust and Go servers decode the field from
   the whole body; the React bound adapter and the terminal UI send the field's value as the body;
   `ess ui check` stops refusing such a route. The interpreter and synthesis do not read
   `ess-http/1` and are unchanged.
5. **Second adopter.** A settings service takes `PUT /accounts/{account_id}/settings` with the
   settings object as the bare body; a tagging service takes `POST /items/{item_id}/tags` with a
   bare array of strings.
6. **Cost.** One optional key in `ess-http/1`, no format version: an older reader refuses
   `request.body` as an unknown field by name, the route `ess-cli/1` took for `trailing` in 0.54.0
   (`docs/design/http-presentation-binding.md:186-188`). The compiled request omits the key when
   absent, so every existing binding compiles to the same bytes. Refusals reuse `ESS-HTTP-020`
   (the request's rules).
7. **Considered.** (a) change nothing: the route stays undeclarable; (b) `encoding: array`: refused,
   an encoding names a media type, not a shape; (c) a body of a list input only, as requested;
   (d) a body of any one input field that is not `Optional` (chosen): the same rule the response
   side has for `response.<field>`, with no list-only special case.

## Decisions

Accept, redesigned. The key is the requester's (`request.body: input.<field>`). It is widened from
"a list input" to "one input field that is not `Optional`", as `response.<field>` is. In this cut
every other input field must be carried by the path. `ess-http/1` has no per-field query placement
beside a JSON body, so "the rest in the query" is refused by name (`ESS-HTTP-020`) rather than
invented here.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-http/tests/request_body.rs` (new):
  - `a_command_route_takes_one_list_input_as_its_bare_body`;
  - `a_body_beside_an_input_the_path_does_not_carry_is_refused` (`ESS-HTTP-020`, naming the field);
  - `a_body_on_a_view_route_or_beside_form_or_query_is_refused`;
  - `a_body_naming_no_input_or_an_optional_one_is_refused`;
  - `a_binding_without_body_compiles_to_the_same_bytes`.
- `crates/generate/ess-gen/tests/openapi_http.rs`, `a_bare_list_body_is_an_array_request_schema`:
  `requestBody.content.application/json.schema` is `{type: array, items: {$ref: …CardPriority}}`,
  not `…Input`.
- `crates/generate/ess-synth/tests/http_binding_rust.rs`, `both_servers_decode_a_bare_array_body`
  (Rust and Go): the `put` with a bare array reaches the command with the list and the path's
  `board_id`; an object body is answered as `malformed`.
- `crates/ui/ess-ui-check/tests/checks.rs`, `a_bare_body_route_is_served_to_a_generated_ui`: no
  refusal for the route. `crates/ui/ess-ui-react/tests/` and `crates/ui/ess-ui-tui/` each send the
  field's value as the body (one case each).
- `docs/design/http-presentation-binding.md` § Request encoding and the `ess-http` reference page
  state the key; one CHANGELOG line.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs:262-300` (`RequestSpec`, `Request`), `:4503-4640`
  (`ESS-HTTP-020` checks); `crates/generate/ess-gen/src/openapi.rs:1146-1180` (`request_body`);
  `crates/ui/ess-ui-check/src/model.rs:1705-1714` (the UI's encoding refusal).
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `crates/generate/ess-synth/src/go/http/bound.rs`
  (body decode); `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (bound adapter send);
  `crates/ui/ess-ui-tui/src/http.rs` (terminal send); `website/docs/reference/` ess-http page.

## Plan

- unit: U2 of `wave-2026-11-06-ess` (ESS 0.89.0), first of two stories in one tree, then
  `story:a-literal-path-segment-takes-precedence-over-a-variable-one`.
- adversary: yes (new wire shape across OpenAPI, two servers and two UI clients).
- shared surfaces: `ess-ui-react` `data.ts.tmpl` and `ess-ui-check` `model.rs` with the wave's
  ride-along pull requests, which merge into the integration branch before this unit branches.
