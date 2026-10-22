---
format: aep.planning-md/3
id: story:an-optional-input-may-select-between-an-operations-routes
kind: story
status: draft
title: An Optional input may be a path variable of one route of an operation whose other route omits it
summary: ESS-HTTP-009 refuses an Optional tenant on a path segment; with additional routes, absence selects the route without it (a2a wave 2, H3, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
- depends_on: story:an-operation-may-have-additional-http-routes
revision: 5
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H3**, a2a wave-2 HTTP binding gaps list, line 10. The tenant of
H2's `/{tenant}/…` routes is an `Optional<String>` input: present, the caller used the tenant route;
absent, the plain one. A path segment refuses an `Optional` source:

> ESS-HTTP-009: "`input.tenant` has type `Optional<String>`, which a path segment cannot carry"

## Where it lives

- Refusal: `crates/specify/ess-http/src/lib.rs:3550-3562`; the admitted types at `lib.rs:4967`
  ("never `Optional`, `List`, …").
- Design: `docs/design/http-presentation-binding.md:141-143`: "It is never `Optional` … because a path
  segment is always present and holds one value." That holds for one route; with H2's route set, an
  absent value selects the route without the segment.
- Depends on H2 (`story:an-operation-may-have-additional-http-routes`): without a second route an
  `Optional` segment has nothing to fall back to.

## Format

Relaxes `ESS-HTTP-009` only for a route set in which some route omits the variable; documents valid
today keep their meaning. Rides on whatever format decision H2 takes. `ess/N` unchanged.

## Fit review

Assessed 2026-10-22 against the integration tree at 0.68.0, with
`.agents/skills/assessing-external-requests/SKILL.md`. This review relaxes the "never `Optional`"
clause of `docs/design/http-presentation-binding.md:141-143` and depends on
`story:an-operation-may-have-additional-http-routes` (H2), which decides how one operation gets more
than one route.

**1. The need, apart from the proposed syntax.** Some protocols serve one operation on two paths, one
of which carries an extra value in a segment, such as a tenant: `GET /tasks/{id}` and
`GET /{tenant}/tasks/{id}`. The operation's input holds that value as `Optional`. It is present when
the caller used the longer path and absent when the caller used the shorter one. The domain fact is
that the choice of route carries the presence of the value. Minimal reproduction: a view with
parameter `tenant: Optional<String>` and the route `{view: GetTask, method: get, path:
'/{tenant}/tasks/{id}', parameters: {tenant: param.tenant, id: param.id}}` → `ESS-HTTP-009`, "has
type `Optional<String>`, which a path segment cannot carry" (`crates/specify/ess-http/src/lib.rs:3552-3562`).
*Requester's proposal (theirs):* admit an `Optional` source on a path segment when another route of
the same operation omits it (a2a wave-2 HTTP binding gaps list, line 10).

**2. Class: gap.** The refusal is on purpose and correct for a single route: "a path segment is
always present and holds one value" (`http-presentation-binding.md:141-143`; `segment_type`,
`lib.rs:4965-4967`). Its premise stops holding only once an operation has a second route that lacks
the segment, which H2 introduces. So this is not a defect.

**3. Can it already be expressed? No.**
- Make the source required: then the shorter route cannot supply it, and absence, which the domain
  distinguishes, is lost.
- Split into one operation per route: that changes the protocol's operations (one `GetTask` becomes
  two). It moves the figure out of the specification, so policy 1 rules it out as an answer
  (`docs/design/generic-language-features.md` § Policy, policy 1).
- Keep the tenant in the query or body: that changes the path the protocol fixes.

**4. Fit with what is there.**
- *Outside grids R and O.* The binding is not part of `ess/N` (`http-presentation-binding.md` §
  Decision). The cell this decision opens is the source-type rule of the path grammar.
- *Presence is derived from the route, not declared.* For one operation, take the `Optional` sources
  that any of its routes maps to a path variable. The operation's route set must hold **exactly one
  route for each combination of presence** of those sources. With one such source that means
  exactly two routes, one with the segment and one without.
  - On a route whose path maps the source, the value is `Some(segment)`.
  - On a route whose path does not, the value is `None`.
  - Two routes with the same presence combination are refused, because a client could not tell
    which to use. A combination with no route is refused, because the operation could not be called
    with it. Both are `ESS-HTTP-009` with a new text naming the operation, the source and the
    missing or doubled combination.
  - A single route mapping an `Optional` source stays refused exactly as today.
- *The source never travels in the body or query.* A source that any route of the operation maps to
  a path variable leaves the body (command) or the query (view) on **every** route of that
  operation. Otherwise a caller could send `tenant` as a body key on the shorter route, and absence
  would no longer follow from the route. That key counts as unknown, which is the rule precedence
  step 6 already states for a path variable's field (`http-presentation-binding.md:523-547`, step 6).
- *Non-optional sources* are untouched by this story. Whether a required source may sit on the path
  of one route and in the body of another is H2's decision.
- *`Some("")` cannot be sent.* The servers' `matched` refuses an empty segment
  (`crates/generate/ess-synth/src/rust/http/bound.rs:178-196`), so the longer route never yields
  `Some("")`, and the shorter one yields `None`. A required `String` path source has the same gap
  today. The binding records it: the HTTP conformance target (stage 3, not built) must classify a
  `Some("")` witness as unreachable over HTTP rather than as a failure. Until it exists, nothing
  executes that witness over HTTP.
- *Every target handles it or refuses it by name.*
  - `ess-http` compile: `ESS-HTTP-009` (`lib.rs:3550-3562`) consults the operation's route set.
  - The Rust and Go servers: each bound row already knows its parameters. A row whose path maps an
    `Optional` source wraps the value in `Some`, and a row that does not writes `None` without
    reading the body or query for it (`rust/http/bound.rs`, `go/http/bound.rs:136`).
  - OpenAPI: two path items. The tenant is a required `in: path` parameter on the longer one, and
    absent from the shorter one's parameters and request body.
  - Refinement (`ESS-HTTP-012`) compares route sets once H2 lands. This story adds nothing to it.
- *Overlaps with other operations.* `/{tenant}/tasks/{id}` and `/tasks/{id}` have different segment
  counts and never overlap each other. The adopter's four tenant `GET` routes do overlap routes of
  other operations, and those overlaps are crossed. They wait on the decision-blocker of
  `story:a-literal-path-segment-may-take-precedence-over-a-variable`. This story removes the type
  refusal; it does not admit those four routes on its own.

**5. A second, unrelated adopter.** A regional commerce API: `GET /v1/orders` and
`GET /v1/{region}/orders`, where `region: Optional<Region>` selects a regional shard when present and
the caller's home region when absent. Another is a content site with `GET /articles/{slug}` beside
`GET /{lang}/articles/{slug}`, with `lang` an `Optional` enum.

**6. Cost.**
- Format: no new key, and no version of its own. The decision rides on H2's format decision.
  - Authored: an older compiler refuses the document as `ESS-HTTP-005` (operation routed twice) or
    `ESS-HTTP-009`, so it fails closed.
  - IR: an older `HttpIr::from_json` refuses one operation routed twice
    (`lib.rs:1905-1910`, the `routed` check in `invariants` at `lib.rs:1964` onward), so an older
    generator never serves the route set with the wrong presence.
  - `ess/N` unchanged.
- Code: `ESS-HTTP-009`'s check and text, the body and query exclusion across an operation's routes,
  the Rust and Go row wrapping, and the OpenAPI parameters. Run `task test-xtask` for the changed
  diagnostic text.
- Docs: `http-presentation-binding.md:141-143` is amended by a dated decision, and the `ESS-HTTP-009`
  table row changes.

**7. Alternatives.**
- (a) *Change nothing.* Rejected: the protocol's tenant routes cannot be bound with the domain's
  type.
- (b) *An optional segment in one template* (`/{tenant?}/tasks/{id}`). Rejected: it needs a grammar
  for an elided segment and makes one route match two segment counts, which every collision rule
  would have to learn. Two routes say the same thing with the grammar that exists.
- (c) *An explicit per-route key* naming the absent sources (`absent: [input.tenant]`). Rejected for
  now: it states what the paths already show, and a mismatch between the key and the paths would
  need its own refusal. It can be added later without changing admitted documents.
- (d) *Chosen:* presence derived from the route, one route per presence combination, and the source
  confined to the path on every route of the operation.

## Decisions

**Accept, redesigned** (verdict: redesigned). Depends on H2.

- An `Optional` scalar source (`input.` or `param.`) may be mapped to a path variable when its
  operation's routes hold exactly one route for each combination of presence of such sources. A
  route whose path maps the source supplies `Some(value)`, and a route whose path does not supplies
  `None`.
- Such a source is read from the path only, on every route of the operation. A body or query key
  for it counts as unknown (precedence step 6).
- A single route mapping an `Optional` source, two routes with the same presence, or a missing
  combination stays `ESS-HTTP-009`, with text naming the operation, the source and the combination.
- What changed from the request:
  - "another route omits it" is tightened to one route per presence combination;
  - the source is excluded from the body and query on every route;
  - `Some("")` is recorded as not sendable over HTTP.
- The four crossed tenant `GET` overlaps are not admitted by this story. They wait on the
  decision-blocker recorded by the H5 story.
- Format: rides on H2. No key and no version of its own. `ess/N` unchanged.

Acceptance, each test written to fail first, as a module of an existing test binary. The stage-3
HTTP conformance target is not built, so acceptance runs through the compiler and the generated
servers:

- `crates/specify/ess-http/tests/compile.rs`:
  - `an_optional_path_source_compiles_with_one_route_per_presence`;
  - `an_optional_path_source_on_a_single_route_stays_ess_http_009`;
  - `two_routes_with_one_presence_are_ess_http_009`;
  - `a_missing_presence_combination_is_ess_http_009`;
  - `an_optional_path_source_leaves_the_body_on_every_route`.
- `crates/generate/ess-synth/tests/http_binding/shared.rs`, the Rust server:
  - `GET /acme/tasks/1` answers with `tenant == Some("acme")`;
  - `GET /tasks/1` answers with `tenant == None`;
  - `POST /items` with body key `tenant` on the shorter route of a command is
    `refusals.malformed` (unknown key).
- `crates/generate/ess-synth/tests/http_binding/go.rs`:
  `the_go_and_rust_servers_answer_one_binding_alike` extended with these requests.
- `crates/generate/ess-gen/tests/openapi_http.rs`: the longer path has a required `in: path`
  `tenant`, and the shorter path has no `tenant` parameter or body property.
- `crates/edge/ess-cli/tests/http.rs`: `ess specify http validate` accepts the route pair and
  refuses the single-route form with `ESS-HTTP-009`.

The requester is told: accepted, redesigned, after H2. The tenant stays `Optional<String>`, and the
tenant route and the plain route select its presence. The four tenant `GET` routes that cross other
operations' routes still wait on the H5 decision-blocker.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22): "TaskData has no tenant".

- Not an ESS gap on its own. The adopter's task entity declares no `tenant` field, while its push-notification configuration entity stores `tenant: input.tenant`; the same `sets:` would store it on the task. What ESS lacks is what this story names: an optional input such as `tenant` choosing between an operation's routes (`/{tenant}/tasks` and `/tasks`).
- No count; no reproduction needed.
