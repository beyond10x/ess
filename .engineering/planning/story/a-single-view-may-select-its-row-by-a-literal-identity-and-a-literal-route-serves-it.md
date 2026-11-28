---
format: aep.planning-md/3
id: story:a-single-view-may-select-its-row-by-a-literal-identity-and-a-literal-route-serves-it
kind: story
status: draft
title: A single view may select its row by a literal identity, and a literal route serves a view told apart by a path value
summary: 'body: single is ESS-HTTP-031 for identity == literal; a literal route beside a variable one serves views with literal filters (shape 7 and the residual of shape 3); HB-D1 and HB-D2'
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-literal-path-segment-takes-precedence-over-a-variable-one
- informed_by: story:a-view-selects-rows-by-a-caller-attribute
scope:
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
revision: 3
---
## Summary

A real service serves several views on one path prefix and tells them apart by a literal segment
(`/settings/group/alpha.one`, `/settings/group/beta.one`), and answers one object where the view holds
one row. A downstream application met two refusals binding that: a mapping of a view parameter to a
literal on a literal path, and a one-row answer for a view that selects by a literal. The first is a
request for a binding key that fixes a parameter; the finding is that the model already says it
without the key, and that the second half, the one-object answer, is refused today. This story makes
the second half work, documents the first, and records the key as decision HB-D1, recommended
declined.

## Facts (tag 0.99.1 read; refusals and the valid case run with `ess` 0.98.0 on a toy model)

1. The reporter's refusals reproduce word for word. A binding of a view with parameter `item_id` on
   the literal path `/settings/group/alpha.one` with `parameters: {item_id: '"alpha.one"'}` is
   refused `ESS-HTTP-008` ("`/settings/group/alpha.one` has no variable `item_id`; a mapping names a
   variable of the path", `crates/specify/ess-http/src/lib.rs:5481`) and `ESS-HTTP-009` ("`"alpha.one"`
   is not `param.<name>`: a view's path variable reads one parameter by name"). A path `parameters:`
   source is `input.<field>` or `param.<name>` and nothing else
   (`docs/design/http-presentation-binding.md:127-176`, the table row `ESS-HTTP-009`).
2. Two views on one variable path are one route, `ESS-HTTP-010` ("variable names do not tell two
   templates apart", `http-presentation-binding.md:172-176`). A literal path beside a variable one is
   not: since 0.89.0 the literal template is the more specific and answers its own paths, the
   generated servers try routes in that order and OpenAPI keeps its path order
   (`story:a-literal-path-segment-takes-precedence-over-a-variable-one`, beyond10x/ess#748,
   `http-presentation-binding.md:140-168`).
3. The model states a literal selection without a parameter. A view filter may compare a field with
   a literal (`filter: note == "x"`, `.engineering/repro/workbench-2026-11-09/examples/20/spec/demo.yaml:164`;
   `filter: 'source == "web"'`, `crates/verify/ess-conformance/tests/adversary_filters_pass2.rs:68`).
   Run: two views `Alpha` (`filter: item_id == "alpha.one"`) and `Beta` (`filter: item_id ==
   "beta.one"`), neither with `params:`, beside the existing `ItemById`, bound on `get
   /settings/group/alpha.one`, `get /settings/group/beta.one` and `get /settings/group/{item_id}`
   (`parameters: {item_id: param.item_id}`): `ess specify http validate` answers `3 route(s), 0 open,
   valid`. No parameter is declared, so none becomes a query parameter the caller never sends.
4. The one-object answer is refused for such a view. `body: single` is admitted only where the
   filter, or a member of a top-level `all`, compares the source entity's identity field with
   `param.<name>` (`selects_identity`, `crates/specify/ess-http/src/lib.rs:8807-8824`: both sides
   must be a fact path). Run: the `Alpha` view with `body: single` and `empty: {status: 404}` is
   refused `ESS-HTTP-031` ("does not select its source entity `shop.items.Item` by its identity field
   `item_id`: neither its filter nor a member of a top-level `all` is `item_id == param.<name>`, so it
   can hold more than one row"). The identity of an entity is unique, so `item_id == "alpha.one"`
   holds at most one row exactly as `item_id == param.item_id` does; the check is narrower than its
   own reason.
5. A view that selects by a literal identity has no parameter, so a suite that reads it reads the one
   row every time, and a suite that reads a view with a free parameter reads any value. A binding key
   that fixed a parameter to a literal would leave the model saying "any value" and the route saying
   "this one": the suite could send a value the route never admits.

## Acceptance

1. Red first, `single` over a literal identity: `a_single_view_selects_its_identity_by_a_literal`
   (a new module of `ess-http`'s integration tests). The `Alpha` view above with `body: single` and
   `empty: {status: 404}` validates and compiles; before the change it is `ESS-HTTP-031`. The compiled
   IR and the OpenAPI schema are those of any `single` route (one object, `404`).
2. The narrowing stays: `a_single_view_that_can_hold_two_rows_is_still_refused` pins `ESS-HTTP-031`
   for a literal compared with a non-identity field, for `!=`, for a literal under `any:`, and for a
   filter with no identity comparison.
3. The generated servers: `the_go_and_rust_servers_answer_one_binding_alike` gains one literal route
   with `body: single`; the Rust and Go servers answer the row and the `empty` status with the same
   bytes, and a request to the sibling variable route still reaches its own view.
4. A literal route beside a variable route is pinned on the modelling form of Fact 3:
   `literal_routes_beside_a_variable_route_serve_views_with_literal_filters` validates, compiles with
   the literal templates ordered before the variable one, and the OpenAPI document lists all three
   paths. Green on reading at 0.99.1; the test holds it.
5. Documentation: `website/docs/guides/http-routes.md` gains a short section "A path value that picks
   the view": the literal route, the literal filter, no parameter, and why a route does not fix a
   parameter (Fact 5). `docs/design/http-presentation-binding.md` § "Path templates" gains one
   sentence on it. The refusal text of `ESS-HTTP-009` for a source that is not `param.<name>` adds, for
   a quoted literal, that a view served at a literal path states the literal in its filter.
6. Each test runs within 60 s and each test step within 120 s; the new cases build no server per test.

## Fit review

1. Need: bind views that a service tells apart by a literal path value, and answer the one object a
   service answers.
2. Class: a modelling answer for the binding the reporter asked for (a literal source for a view
   parameter, decision HB-D1), and a gap in a validation rule (`ESS-HTTP-031` narrower than its reason).
3. Covers: shape 7 of the twentieth report in both of the reporter's forms (a literal on a literal path,
   literal over variable precedence: the latter is released as 0.89.0); the residual of shape 3 (a
   single-object GET for a view that is not selected by a path parameter: a settings object, one fixed
   key); the first step of the "who am I" view of shape 1b
   (`story:a-view-may-answer-the-row-that-describes-the-reading-actor`).
4. Borders:
   `story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their` (draft: several views on one route, one object of their rows, a different answer to the same need);
   `story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body`
   (commands, selected by the body); `story:a-view-selects-rows-by-a-caller-attribute` (identity
   against `caller.<attribute>` is the same widening of `selects_identity`, owned by that story once it
   lands and by HC-D1). A search of the store and the open issues found no story on a literal-keyed
   `single` view and none on a parameter fixed by the route.
5. A second adopter: any service with `/config/{name}`-style routes where a few names have their own
   handlers and one fixed name answers one object.

## Decisions

- HB-D1 (shape 7): may a route fix a view parameter to a literal (a source `literal` beside
  `param.<name>`, the parameter leaving the query).
  - A. Decline: the literal goes in the view filter (Fact 3) and the route carries no mapping.
    The model and the route say one thing; the suite reads exactly what the route serves.
  - B. A route key `fixes: {<parameter>: <literal>}` on a literal path: the parameter leaves the query,
    the suite is given the same literal for it. Cost: a binding that restricts a semantic parameter, so
    synthesis must read the binding to know which values it may send; an optional key in `ess-http/1`
    and `ess-http-ir/1`; a refinement rule; a conformance rule per target.
  - C. Mapping `param.<name>` to a literal segment that must equal a value of the parameter's enum:
    only for enum-typed parameters. Cost: B's, for fewer models.
  - Recommended A. The reporter's wording is evidence of the need (a literal path serving one view),
    which Fact 3 serves without a key. Not security-marked: the owner is not needed unless B is taken.
- HB-D2 (shape 3 residual, Fact 4): may `single` admit a filter that compares the identity field with
  a literal.
  - A. Yes: `selects_identity` accepts a literal on one side; no document, IR or generated byte changes,
    and a released reader refuses a binding it now admits.
  - B. No: a literal-keyed view answers `array` or `rows`, and an adopter that needs one bare object
    wraps it itself.
  - Recommended A: the check is narrower than the reason it states (at most one row), and the case is
    the common one for a settings-style resource.

- HB-D1: A (decline a route key that fixes a parameter, model the literal in the filter), taken by the coordinator on 2026-11-25 (D118); reversible until the release that carries it

- HB-D2: A (`single` admits identity == literal), taken by the coordinator on 2026-11-25 (D118); reversible until the release that carries it

## Scope

- `crates/specify/ess-http/src/lib.rs` (`selects_identity` 8807-8824, the `ESS-HTTP-009` text 5470-5490)
- `crates/specify/ess-http/tests/` (a new module in the existing integration-test binary)
- `crates/generate/ess-synth/tests/http_binding/go.rs` (`the_go_and_rust_servers_answer_one_binding_alike`, 755)
- `docs/design/http-presentation-binding.md`, `website/docs/guides/http-routes.md`, `CHANGELOG.md`
