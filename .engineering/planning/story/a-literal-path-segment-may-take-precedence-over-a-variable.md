---
format: aep.planning-md/3
id: story:a-literal-path-segment-may-take-precedence-over-a-variable
kind: story
status: draft
title: A literal path segment may take precedence over a variable, so /tasks/views/x and /tasks/{taskId}/x coexist
summary: ESS-HTTP-011 refuses a literal beside a variable (stage-1 decision 3); four upstream A2A GET routes are refused (a2a wave 2, H5, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 4
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H5**, a2a wave-2 HTTP binding gaps list, line 12. A literal segment
takes no precedence over a variable, so `get /tasks/views/x` collides with `get /tasks/{taskId}/x`,
and four upstream `/{tenant}/…` GET routes would be refused:

> ESS-HTTP-011 (no text quoted in the report)

## Where it lives

- Refusal: `crates/specify/ess-http/src/lib.rs:5133-5142` ("a literal takes no precedence over a
  variable"); rule table `lib.rs:2503`.
- Decided on purpose: `docs/design/http-presentation-binding.md:633`, stage 1 decision 3, "A literal
  segment does not take precedence over a variable: `ESS-HTTP-011` refuses the overlap." The fit
  review reopens that decision.
- If precedence is admitted, every generated server and the conformance runner must match literal
  before variable; which tables those are is not read here.

## Format

Admits documents refused today. But the routing meaning becomes a rule every reader of
`ess-http-ir/1` must apply (literal first), which a reader built for today's IR would not; the fit
review decides whether that is a format version. `ess/N` unchanged.

## Fit review

Assessed 2026-10-22 against the integration tree at 0.68.0, with
`.agents/skills/assessing-external-requests/SKILL.md`. This review reopens stage-1 decision 3 of
`docs/design/http-presentation-binding.md:633`.

**1. The need, apart from the proposed syntax.** A published HTTP surface serves a fixed path beside
a templated one on the same method, such as `GET /users/me` beside `GET /users/{id}`, or a collection
view under `/tasks/views/…` beside `/tasks/{task_id}/…`. A request that matches both is answered by
the fixed one. `ess-http/1` cannot bind such a surface: `ESS-HTTP-011` refuses the overlap
(`crates/specify/ess-http/src/lib.rs:5133-5142`). Minimal reproduction: one binding with
`get /items/current` (a view) and `get /items/{item_id}` (another view) on one component →
`ESS-HTTP-011`, "a literal takes no precedence over a variable".
*Requester's proposal (theirs):* "a literal segment takes precedence over a variable" (a2a wave-2
HTTP binding gaps list, line 12). It cites `/tasks/views/x` beside `/tasks/{taskId}/x`, and four
upstream `/{tenant}/…` GET routes.

**2. Class: gap.** ESS refuses on purpose (stage-1 decision 3, `http-presentation-binding.md:633`),
so it is not a defect. Without it, a protocol that fixes such paths cannot be bound.

**3. Can it already be expressed?** Only by moving one route off its path, as the adopter did:
`ess/http-unmapped.tsv` moves a view to `/views/tasks/pushNotificationConfigs`. That changes the
surface the protocol fixes, so the binding no longer states the figure the protocol's reader sees.
Policy 1 rules it out as an answer (`docs/design/generic-language-features.md` § Policy, policy 1).

**4. Fit with what is there.**
- *Outside grids R and O.* An `ess-http/1` binding is not part of `ess/N`
  (`http-presentation-binding.md` § Decision). The grid this decision opens is the collision table
  of `ESS-HTTP-010`/`-011` (`http-presentation-binding.md:174-175`).
- *Two overlap shapes exist, and only one has an answer that needs no convention.*
  - **Nested overlap.** At every position where the two templates differ, one holds a literal and
    the other a variable. The first template's paths are then a strict subset of the second's, so
    "the more specific answers" is one answer whichever position is read first. Examples:
    `/users/me` beside `/users/{id}`; `/tasks/views/x` beside `/tasks/{taskId}/x`; the adopter's
    moved view.
  - **Crossed overlap.** Each template holds the literal at some position where the other holds a
    variable. The paths in common belong to neither more than to the other. All four upstream
    tenant routes the request names are crossed. `/{tenant}/tasks` beside `/tasks/{id}` both match
    `/tasks/tasks`, and `/{tenant}/tasks/{id}` beside `/tasks/{taskId}/pushNotificationConfigs` both
    match `/tasks/tasks/pushNotificationConfigs`. A rule that picks one of them, say the leftmost
    literal, decides that a tenant named `tasks` cannot reach its tenant routes. That is a domain
    decision the specification would make silently.
- *Consistent with OpenAPI.* OpenAPI's path-matching note puts concrete paths before templated ones
  and leaves ambiguous cases to tooling. The nested rule says the same, and refuses where OpenAPI
  says nothing.
- *Per method, as today.* `ESS-HTTP-011` already compares templates of one method only, and the
  servers try every row and answer `405` only when no row of the request's method matches
  (`crates/generate/ess-synth/src/rust/http/bound.rs:602-660`). Specificity is decided among the
  request's method's templates. No `404` or `405` changes.
- *Every target handles it or refuses it by name.*
  - Today the generated Rust and Go servers try rows "in path order"
    (`rust/http/bound.rs:602-607`; `go/http/bound.rs:1013-1014`). Path-string order is not
    specificity order. `/{tenant}/x/{id}` sorts before `/{t}/x/current`, because `e` sorts before
    `}`, so the variable route is tried first. A literal starting with `~` also sorts after `{`.
    Both servers must sort by specificity.
  - `HttpIr::from_json` does not check collisions (lib.rs:1936-1939). An older ess reading new IR
    would therefore generate a server that answers some nested pairs with the wrong route. The IR
    must make that reader refuse by name (below).
- *Composes with H1.* A suffixed variable `{id}:subscribe` is more specific than `{id}` at the same
  position: its paths are a strict subset. That is the nested case, which is what lets
  `/tasks/{id}:subscribe` and `/tasks/{id}` share `get`
  (`story:a-path-segment-may-be-a-variable-with-a-verb-suffix`).

**5. A second, unrelated adopter.** Any API with a "self" or "current" alias, such as
`GET /accounts/me` beside `GET /accounts/{account_id}`, or `GET /releases/latest` beside
`GET /releases/{tag}`. Both are nested.

**6. Cost.**
- Authored format: `ess-http/1` unchanged, with no new key. An older compiler refuses such a document
  as `ESS-HTTP-011`, so it fails closed.
- IR: `ess-http-ir/1` stays. Each route that a more specific route of its method and component
  shadows records the shadowing paths under a new route key (working name `shadowed_by`, a sorted
  list), written only when non-empty. Every IR valid today keeps its bytes. An older reader refuses
  the key as an unknown field, naming it (`#[serde(deny_unknown_fields)]` on `Route`, lib.rs:1766-1767).
  This is the stage-2 precedent: a stage-1 reader refuses every stage-2 key by name
  (`http-presentation-binding.md:180-182`). The IR already records derived data so a reader sees the
  whole surface (`:223-224`). AGENTS.md requires an old-reader compatibility test before adding the
  field.
- Code:
  - `clash` (lib.rs:5029-5047) distinguishes nested from crossed overlaps;
  - `ESS-HTTP-011` refuses crossed overlaps only, its text naming a common path;
  - the Rust and Go dispatch sort by specificity;
  - OpenAPI is unchanged (it already admits both paths).
- Docs: stage-1 decision 3 is superseded by a dated entry, and the `ESS-HTTP-011` table row changes.
- No `ess/N` change, no new code (011's meaning narrows), and `task test-xtask` for the changed
  message.

**7. Alternatives.**
- (a) *Change nothing.* Rejected: it refuses common published surfaces, and the only workaround moves
  a fixed path.
- (b) *Leftmost literal wins*, which is the request as written. Rejected for now: it also admits
  crossed overlaps, where it silently makes some identifier values unreachable on one route.
  Nested-only is the narrower rule. Leftmost-literal gives the same answer on every pair the nested
  rule admits: the matching templates form a chain, and its most specific element holds the literal
  at the first position where they differ. So admitting crossed overlaps later is an addition that
  changes no admitted document's meaning (the stage-2 practice of taking the narrower option,
  `http-presentation-binding.md:642-644`).
- (c) *Specificity by literal count or first-declared wins.* Rejected: both depend on authoring order
  or counting conventions that a client cannot read off the paths.
- (d) *Chosen:* nested overlaps are admitted and the most specific template answers. Crossed overlaps
  stay refused.

## Decisions

**Accept, redesigned** (verdict: redesigned).

- Two templates of one component and one method may overlap where one is more specific at every
  position where they differ: a literal against a variable, or a suffixed variable against a plain
  one. A request is answered by the most specific template of its method that matches.
- `ESS-HTTP-011` keeps refusing crossed overlaps and names one path both match. `ESS-HTTP-010` is
  unchanged.
- Stage-1 decision 3 is superseded by a dated decision on `docs/design/http-presentation-binding.md`.
- What changed from the request: a literal takes precedence only where that alone decides every
  shared path. The leftmost-literal convention for crossed overlaps is not adopted.
- Deferred, as a `decision-blocker` with `blocks:` to the tenant half of
  `story:an-optional-input-may-select-between-an-operations-routes`: may a binding admit a crossed
  overlap, and if so by which rule (leftmost literal, or an explicit per-binding acknowledgement
  naming the shadowed values)? Four upstream tenant GET routes wait on it.
- Format: `ess-http/1` and `ess-http-ir/1` unchanged in version. One optional IR route key, refused
  by name by older readers. `ess/N` unchanged.

Acceptance, each test written to fail first, as a module of an existing test binary. The HTTP
conformance target is stage 3 and not built, so acceptance runs through the compiler and the
generated servers:

- `crates/specify/ess-http/tests/compile.rs`:
  - `a_nested_overlap_compiles_and_records_the_shadowing_route` (`/items/current` beside
    `/items/{item_id}`);
  - `a_crossed_overlap_stays_ess_http_011_naming_a_common_path` (`/{a}/x` beside `/y/{b}`);
  - `a_nested_overlap_beside_a_derived_route_compiles`;
  - `ir_without_shadowing_keeps_its_bytes`;
  - `a_reader_without_the_key_refuses_ir_that_carries_it`, the old-reader compatibility test.
- `crates/generate/ess-synth/tests/http_binding/shared.rs`, the Rust server:
  - `get /items/current` answers the literal route's view;
  - `get /items/42` answers the variable route's view;
  - the same holds where path order puts the variable route first: `/{tenant}/x/{id}` beside
    `/{t}/x/current`, and `/v/{id}` beside `/v/~self`.
- `crates/generate/ess-synth/tests/http_binding/go.rs`:
  `the_go_and_rust_servers_answer_one_binding_alike` extended with the nested pair.
- `crates/generate/ess-gen/tests/openapi_http.rs`: both paths present, bytes otherwise unchanged.
- `crates/edge/ess-cli/tests/http.rs`: `ess specify http validate` accepts the nested binding and
  refuses the crossed one with `ESS-HTTP-011`.

The requester is told: accepted, redesigned. `/tasks/views/x` beside `/tasks/{taskId}/x` and the
moved view can return to their own paths. The four tenant GET routes are crossed overlaps and wait
on the decision-blocker.
