---
format: aep.planning-md/3
id: story:a-literal-path-segment-takes-precedence-over-a-variable-one
kind: story
status: implemented
title: A literal path segment takes precedence over a variable one
summary: ESS-HTTP-011 admits a literal beside a variable at the first difference; servers dispatch literal first (beyond10x/ess#748 part 2)
tags:
- external-request
refs:
- provider: github
  reference: beyond10x/ess#748
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- supersedes: story:a-literal-path-segment-may-take-precedence-over-a-variable
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T10:55:31Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-06T10:55:32Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-06T20:27:59Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Two route templates of one component and method that a request path can both match, a literal beside
a variable at the first position they differ, are admitted. At that position the literal takes
precedence: `/boards/{board_id}/cards/priority` is served before `/boards/{board_id}/cards/{card_id}`
for the path `/boards/b-1/cards/priority`, and every other path ending in a card id reaches the
variable route. `ESS-HTTP-011` refuses only what this rule cannot order.

Part 2 of https://github.com/beyond10x/ess/issues/748.

## Fit review

1. **Need.** A service serves a fixed sub-resource beside a variable one under the same prefix
   (`/cards/priority` beside `/cards/{card_id}`). Minimal reproduction: the board specification of
   `story:a-command-route-takes-one-input-as-its-bare-request-body` with a second command
   `UpdateCard` bound `put /boards/{board_id}/cards/{card_id}`. On 0.87.0 this is `ESS-HTTP-011`:
   "segment 4 is `{cardId}` in one and `priority` in the other, so one request path matches both,
   and a literal takes no precedence over a variable". Requester's options, theirs: a literal takes
   precedence, or the binding states the literals a variable excludes.
2. **Class: gap.** A deliberate refusal (`docs/design/http-presentation-binding.md:181`), chosen in
   stage 1 because no precedence rule was stated. A real service's route table cannot be described.
3. **Already expressible?** No. Rebinding either route to a non-overlapping path describes a service
   that does not exist.
4. **Fit.** OpenAPI 3.1 states this rule ("When matching URLs, concrete (non-templated) paths would
   be matched before their templated counterparts", Paths Object), as do most HTTP routers, so the
   OpenAPI projection needs no new construct. The rule is per position: compare segment by segment;
   at the first position one template holds a literal and the other a variable, the literal wins.
   Every pair that overlaps under the current definition (`crates/specify/ess-http/src/lib.rs:6575-6600`,
   `Clash::Overlap`) is ordered by it, so no new key and no opt-in. Targets: the Rust and Go servers
   try routes "in path order, tried in turn" (`crates/generate/ess-synth/src/rust/http/bound.rs:603`);
   that order is the template string's order (`crates/generate/ess-gen/src/http.rs:537-541`), in
   which `{` sorts after lower-case letters but before `~`, a valid `pchar`. Dispatch therefore needs
   an explicit precedence order; the OpenAPI document keeps its path order, so its bytes do not
   move. A generated UI fills a template and never matches one, so it is unchanged.
5. **Second adopter.** `GET /users/me` beside `GET /users/{user_id}`; `POST /orders/search` beside
   `POST /orders/{order_id}`.
6. **Cost.** No key and no format version: a document this admits was refused by every earlier
   reader, by name. No existing binding changes meaning, because none could overlap. `ESS-HTTP-011`
   keeps its code for what remains refused, and its row in the design page and the reference changes.
7. **Considered.** (a) change nothing; (b) an `excludes: [priority]` list on the variable, the
   requester's second option: a new key that restates what the literal route already says; (c) the
   literal-first rule (chosen): no surface, the rule OpenAPI and common routers already follow.

## Decisions

Accept, redesigned after the U2 adversary pass (2026-11-06), taking the analysis of the earlier
draft `story:a-literal-path-segment-may-take-precedence-over-a-variable` (A2A report H5), which this
story supersedes:

- **Nested overlaps only.** Two templates of one component and method where, at every position they
  differ, one holds a literal and the other a variable: the first template's paths are a strict
  subset of the second's, and the more specific template answers. `/boards/{board_id}/cards/priority`
  beside `/boards/{board_id}/cards/{card_id}` is nested.
- **Crossed overlaps stay `ESS-HTTP-011`.** Where each template holds the literal at some position the
  other holds a variable (`/{tenant}/tasks` beside `/tasks/{id}`), any order is a domain decision the
  specification would make silently, so it stays refused, naming both operations and the segment.
- **Per method, as today.** Specificity is decided among the request's method's templates; `404` and
  `405` are unchanged.
- **Literals compare after normalisation.** A request segment is compared with a literal after
  decoding percent-encoded unreserved characters (RFC 3986 §6.2.2.2), so `%7Earchived` reaches the
  `~archived` literal route in both servers, as `~archived` does.
- **An older reader fails closed.** Where compiled `ess-http-ir/1` is read back by an `ess` that does
  not order by specificity (`HttpIr::from_json`), a shadowed route must make that reader refuse by
  name rather than serve the wrong route; the unit checks whether any reader takes IR across
  versions and, if one does, records the shadowing paths on the route (`shadowed_by`, written only
  when non-empty, so every IR valid today keeps its bytes).

`ESS-HTTP-010` (the same template twice) is unchanged.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-http/tests/compile.rs`, `a_literal_beside_a_variable_at_the_first_difference_compiles`
  (the board pair) and `the_same_template_twice_is_still_ess_http_010`.
- `crates/generate/ess-gen/tests/` route order: `dispatch_order_puts_a_literal_before_a_variable`,
  including a literal starting with `~`; `openapi_path_order_is_unchanged_for_existing_bindings`.
- `crates/generate/ess-synth/tests/http_binding_rust.rs`, `both_servers_serve_the_literal_route_before_the_variable_one`
  (Rust and Go): `/boards/b-1/cards/priority` reaches `SetCardPriorities`, `/boards/b-1/cards/c-9`
  reaches `UpdateCard` with `card_id = c-9`.
- `docs/design/http-presentation-binding.md` row `ESS-HTTP-011`, the reference row, one CHANGELOG line.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs:6575-6700` (`Clash`, `ESS-HTTP-011`), `:3040`
  (refusal table); `crates/generate/ess-gen/src/http.rs:537-541, 634` (route sort);
  `crates/generate/ess-synth/src/rust/http/bound.rs:179-200, 603` (template match, route loop).
- inferred: `crates/generate/ess-synth/src/go/http/bound.rs` (the Go route loop).

## Plan

- unit: U2 of `wave-2026-11-06-ess` (ESS 0.89.0), second story in the U2 tree, after
  `story:a-command-route-takes-one-input-as-its-bare-request-body`.
- adversary: yes, with the U2 pass.
