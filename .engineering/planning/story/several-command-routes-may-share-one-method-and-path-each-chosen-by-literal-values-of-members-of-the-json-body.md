---
format: aep.planning-md/3
id: story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body
kind: story
status: draft
title: Several command routes may share one method and path, each chosen by literal values of members of the JSON body
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected
revision: 5
---
## Need

A service may take several operations on one method and path and tell them apart by values in the
JSON body. Two shapes:

- **Open set.** `POST /hooks` takes `{"kind": "...", "data": {...}}`. `kind: opened` is one
  operation. `kind: closed` is two, split again by the Boolean `data.reopen`. Any other `kind` is a
  fourth operation that answers `204` and changes nothing.
- **Closed set.** `POST /v2/hooks` takes `{"object": "...", "action": "...", ...}`. The pair
  (`object`, `action`) picks one of nine operations. Any other pair answers `400`, and callers rely
  on that `4xx` (a `5xx` would be retried).

`ess-http/1` binds one operation per method and template on a component:

- `ESS-HTTP-010`, `crates/specify/ess-http/src/lib.rs:7522-7533` (message `COLLISION`, `lib.rs:70`:
  "two operations claim the same method and path"). The check is `Clash::Same`
  (`lib.rs:7382-7384`, `clash` at `lib.rs:7404-7425`).
- `ESS-HTTP-005` (`lib.rs:3197`): one operation is routed twice. Two body values that lead to one
  command cannot be written as two routes either.

So each such command stays on a path ESS derives for it, and the projected contract lists paths the
service does not serve.

Minimal reproduction (written fresh; not run): component `desk` accepts `tickets.desk.OpenTicket`,
`tickets.desk.CloseTicket`, `tickets.desk.ReopenTicket` and `tickets.desk.IgnoreHook`.

```yaml
routes:
  - {command: tickets.desk.OpenTicket, method: post, path: /hooks}
  - {command: tickets.desk.CloseTicket, method: post, path: /hooks}
```

```console
$ ess specify http validate --path spec --binding desk.http.yaml
error[ESS-HTTP-010]: routes[1]: two operations claim the same method and path: … are one route on component `desk`, since variable names do not tell two templates apart
```

## Outcome

Under **AE-D2**, option A (the default), a command route with a `json` body may declare
`select:`. A select is a map from a body member path to a literal (`String`, `Boolean` or
`Integer`). It may also be a list of such maps, which are alternatives for one operation, so
`ESS-HTTP-005` keeps its meaning. A member path is a dotted path of JSON object members, the grammar
of `request.members` (`ae1d-a-json-request-reads-an-input-from-a-nested-member`).

```yaml
routes:
  - {command: tickets.desk.OpenTicket, method: post, path: /hooks, select: {kind: opened}}
  - {command: tickets.desk.CloseTicket, method: post, path: /hooks, select: {kind: closed, data.reopen: false}}
  - {command: tickets.desk.ReopenTicket, method: post, path: /hooks, select: {kind: closed, data.reopen: true}}
  - {command: tickets.desk.IgnoreHook, method: post, path: /hooks}          # the catch-all
  - command: tickets.desk.ChangeLabel
    method: post
    path: /v2/hooks
    select: [{object: label, action: added}, {object: label, action: removed}]
```

Routes of one component and method may share a template when their selects tell them apart. The
rule is the template rule of 0.89.0, applied to members:

- A member a select does not name matches any value, including an absent member.
- Two selects meet when one body can match both. They are **the same** when they test the same
  members with the same literals: `ESS-HTTP-010`, as today.
- They are **nested** when one tests every member the other tests, with the same literals, and at
  least one more. The more specific one answers, and nothing is refused.
- They are **crossed** when each tests a member the other leaves open: `ESS-HTTP-011`, naming both
  members and a body that matches both.
- A route on the shared template without `select` is the least specific. It is the catch-all, and
  it makes the set open.
- Without a catch-all the set is closed. A body that no select matches is refused as `malformed`,
  with the binding's or the route's `malformed` answer (a `4xx`), so an unknown pair is never a
  `5xx`. This is fail-closed. No new refusal key.

A tested member may also be an input field (by its wire name at the top level, or through
`request.members`). That input then receives the value the body carries, and its type must admit
the literal. A tested member is never an undeclared member, under `unknown: refuse` or `ignore`.

`ESS-HTTP-010`'s message names `select` as the way out when both routes are declared.

## Acceptance

1. Red today: the reproduction above is refused `ESS-HTTP-010` (a case in
   `crates/specify/ess-http/tests/compile.rs` that asserts the code and the path `routes[1]`).
2. The open-set binding above validates and compiles (`ess specify http compile` exit 0).
   `ess-http-ir/1` writes `select` only where it is declared, maps in member order and lists in the
   order written. Every committed binding fixture compiles to the same bytes
   (`crates/specify/ess-http/tests/fixtures/`, compared in `compile.rs`).
3. A closed set of nine routes (pairs of `object` and `action`, no catch-all) validates.
   `ess specify http compile` prints each route with its select.
4. Each of these is refused, naming the key and both routes where two are involved:
   - two routes with the same tests (`ESS-HTTP-010`);
   - crossed selects (`ESS-HTTP-011`, the message naming the two members and a matching body);
   - two catch-alls on one template (`ESS-HTTP-010`, unchanged);
   - `select` on a view, on a `form` or `query` request, or beside `request.body` (a bare body has
     no members) (`ESS-HTTP-020`);
   - an empty map, an empty list, a literal that is `null`, a list or an object, or a path breaking
     the member grammar (`ESS-HTTP-020`);
   - a literal of a type the input field on that member does not admit (`ESS-HTTP-020`, naming the
     field and its type);
   - one alternative of a list that is the same as, nested in, or crossed with another of the same
     route (`ESS-HTTP-020`; alternatives are disjoint).
5. A derived route never shares a template with a declared one (unchanged), and a binding without
   `select` reports what it reports today, byte for byte (`compile.rs` existing cases unmodified).
6. `docs/design/http-presentation-binding.md` gains § "Several operations on one route", and the
   `ESS-HTTP-010`, `011` and `020` rows of the code table (`lib.rs:3180-3240`) and of the design
   page name `select`. `website/docs/reference/formats.md` (the `ess-http/1` row) and
   `website/docs/guides/http-routes.md` state the key.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RouteSpec` 1561-1610, `Clash` 7382-7400, `clash`
  7404-7425, `surface` 7470-7560, code table 3180-3240)
- inferred: `crates/specify/ess-http/src/lib.rs` (compiled `Route`, `HttpIr::from_json` invariants)
- cited: `docs/design/http-presentation-binding.md`
- cited: `website/docs/reference/formats.md`, `website/docs/guides/http-routes.md`

## Owned files

- `crates/specify/ess-http/src/lib.rs`: `RouteSpec.select`, compiled `Route.select`, a `Select`
  type with its member-path reader (shared with `request.members`), the clash of two selects, the
  `ESS-HTTP-010`/`011`/`020` messages.
- `crates/specify/ess-http/tests/compile.rs`, `tests/request_body.rs` (new cases only).
- `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`,
  `website/docs/guides/http-routes.md`.

## Not yours

OpenAPI (`ae1b-…`), the generated servers (`ae1c-…`), the generated apps (`ae1f-…`), and
`request.members` (`ae1d-…`), whose member-path reader this story reuses.

## Tests first

`compile.rs`: the reproduction (red, `ESS-HTTP-010`), the open set and the closed set (green after),
and one case per refusal of acceptance 4. Each runs in milliseconds. No new test binary.

## Gate

Each command is one job within the 10-minute pool limit. Each test stays under 60 s, and the step
under 120 s.

```console
task fmt-check
cargo clippy -p ess-http --all-targets --locked -- -D warnings
cargo test -p ess-http --locked --test compile
cargo test -p ess-http --locked --test request_body
task test-xtask
task ci-lint
```

`task test-xtask` runs because `website/` and `docs/` move. `task ci-lint` runs because the public
API of `ess-http` changes.

## Depends on

`ae1d-a-json-request-reads-an-input-from-a-nested-member` (the member-path reader; same file,
landed first).

## Changelog

- `ess-http/1`: command routes of one component may share a method and path, each chosen by
  `select:`, literal values of members of the JSON body; a body no select matches is `malformed`
  unless a route without `select` takes it.

## Fit review

1. **Need.** A service takes several operations on one method and path and tells them apart by body
   values, with an open set (a catch-all answers) or a closed set (anything else is a `4xx`). The
   report named the need and proposed no syntax.
2. **Class: gap.** `ESS-HTTP-010` (`lib.rs:7522-7533`) and `ESS-HTTP-005` (`lib.rs:3197`).
3. **Expressible today?** No. The idiom of today keeps derived paths, which moves the figure (the
   real route) out of the specification into a comment. That is no answer under policy 1
   (`docs/design/generic-language-features.md:590-595`).
4. **Fit.** It reuses the nested-or-crossed rule that admits literal-over-variable templates
   (`Clash`, `lib.rs:7382-7400`): the same idea, over body members instead of path segments. The
   linker still never chooses (`docs/design/linker-never-chooses.md`). Every order is a fact of the
   binding: more specific wins, and crossed stays refused. The closed set needs no new refusal,
   because `malformed` already says "this body is not one this route reads". Siblings: a view takes
   no body, so it is refused by name. Query and form selection could follow later from the same
   rule, but no report names it. `ess-transport` subjects already select by address, not by body.
   Targets: OpenAPI `ae1b`, servers `ae1c`, apps `ae1f`. Conformance is semantic, so the
   synthesized suite does not change.
5. **Second adopter.** A payment-provider webhook endpoint: `{"type": "invoice.paid" | …}` on one
   URL, with an unknown type acknowledged `200` and ignored.
6. **Cost.** One optional key in `ess-http/1` and `ess-http-ir/1`, absent by default, so no format
   version. An older reader refuses `select` by name (precedent: stage 2 keys, `request.body`). No
   new code. `010`, `011` and `020` widen.
7. **Considered.** (a) Change nothing: the paths are wrong (AE-D2 C). (b) Chosen: per-route `select`
   with the template rule. (c) One `discriminator:` member per path (AE-D2 B): it does not reach the
   pair, nor the second-level split by a Boolean.

## Decisions

Accept, redesigned (the reporter named the need, and the shape is new). **AE-D2**, default A.
**AE-D1** (the crossed-templates story has no finding behind it).

- S4: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.

- Stricter than drafted (found by the implementor, 2026-11-25): both keys are refused beside
  `multipart` and on `get`; a selected member above or inside a field's member is refused; a
  refinement restates both keys (`ESS-HTTP-012`); a catch-all on a shared path whose body is not a
  `json` input object is `ESS-HTTP-010`.

## Size

M, about 550 lines with tests.

## Delivery

Commits `d4082cd47a` (`request.members`, decision S5 = A) and `29f00aff50` (`select` per route,
decision S4 = A) on the branch `feat/http-nested-members-and-shared-routes`, pushed. The whole
check ran 8 lines, each exit 0. No pull request yet.
