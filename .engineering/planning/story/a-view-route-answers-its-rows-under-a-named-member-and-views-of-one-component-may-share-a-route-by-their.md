---
format: aep.planning-md/3
id: story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their
kind: story
status: draft
title: Views of one component may share one route, each answering its rows under its own member
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:an-answer-may-be-wrapped-in-one-named-member
- depends_on: story:an-answer-may-be-wrapped-in-one-named-member
revision: 4
---
## Need

A service answers one `GET` with an object whose members each hold the rows of one view, such as
`{"pending": […], "in_transit": […]}`. Each list is a view of the model.

`story:an-answer-may-be-wrapped-in-one-named-member` (active, beyond10x/ess#755) gives a view
route the body `{<member>: array}`: the view's rows, a JSON array, under `<member>`.
`story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member` (active) publishes
and serves it. Both keep one view per route. Two views on one method and path are refused as
`ESS-HTTP-010`, "two operations claim the same method and path"
(`crates/specify/ess-http/src/lib.rs:7459-7533`, the refusal at `:7525`). So the adopter adds
routes to its service so that the binding can describe them.

This story is what the second report's G1 adds beyond those two stories: the shared route. The
member body, its refusals, its OpenAPI object for one view and its server writers are theirs and
are not restated here.

Minimal reproduction (fresh, not run here): views `shop.delivery.Pending` and
`shop.delivery.InTransit` over one entity, with the same parameters, served by one
`reached_by: network` component, and the binding

```yaml
routes:
  - {view: shop.delivery.Pending, method: get, path: /v1/board, body: {pending: array}}
  - {view: shop.delivery.InTransit, method: get, path: /v1/board, body: {in_transit: array}}
```

Once the active member-body story has landed, each route alone validates. Together, the second
route is refused as `ESS-HTTP-010`.

The requester's proposal, restated: a view route names the member its rows sit under, and two
views may read one answer, written `body: {array_at: pending}`. The member half is the active
story's `{<member>: array}`.

## Outcome

Two or more view routes of one component may share one method and path when each declares
`body: {<member>: array}` with a different member. That route answers one object holding every
member, in route declaration order. The views that share a route take the same parameters, bound
the same way. A paged view stays refused with a member body, as the active story refuses it.
`ess-http-ir/1` keeps one route entry per view, as today. A binding with no shared route compiles
to the bytes it compiled to before.

The OpenAPI projection writes one operation for the shared path. Its `200` schema is an object with
one required array member per view, `additionalProperties: false`, each member's items the view's
row schema, and `x-ess-views` mapping each member to its view.

## Acceptance

- Red first, in `crates/specify/ess-http/tests/`: the binding above validates and compiles. Before
  the change the second route is refused as `ESS-HTTP-010`.
- A binding without a shared route compiles to the bytes it compiled to before. An old-reader
  compatibility test records what the IR reader of the previous release does with the IR of a
  shared route: it refuses it by name, not with a parse error.
- Refused, each with `ESS-HTTP-010` or `ESS-HTTP-024` as today and a message naming the key:
  - two routes on one path naming one member;
  - a route on a shared path whose body is not `{<member>: array}` (`rows`, `array`, `single`,
    `{<member>: single}`);
  - shared views whose parameters differ in name, type or binding;
  - two different `operation_id`s on one shared path;
  - views of two components on one path, refused as today;
  - `representations` (`story:a-view-route-answers-a-csv-representation-its-query-value-selects`)
    on a route of a shared path. Whichever of the two stories lands second adds this case.
- `ess generate --kind openapi --http` writes one operation for the shared path, as above. A
  binding without a shared route keeps its OpenAPI bytes.
- `docs/design/http-presentation-binding.md` names the exception in its `ESS-HTTP-010` row.
  `website/docs/guides/http-routes.md` shows the shared route. `website/docs/reference/formats.md`
  names the release that adds it. `CHANGELOG.md` has one line under `### Added`.

## Fit review

Tree `bcfc51555`, read-only. Second report, G1. Revised on 2026-11-24 after a triage of double
work: the member half of G1 was already the active member-body story.

1. **Need.** A service publishes several views in one answer, a common shape for a dashboard's
   state endpoint. Reproduction above.
2. **Class.** *Gap*: one route serves one operation.
3. **Already expressible?** No.
4. **Fit.** The language grids do not cover `ess-http/1`. The spelling is the active story's
   `{<member>: array}`. The form `{<member>: rows}` is not used, because the active story refuses
   it: `rows` is already an object, so wrapping it would nest (`an-answer-may-be-wrapped-in-one-named-member`,
   § Decisions). The requester's `array_at` is a new word for the same idea, so it is not taken.
5. **Second adopter.** A build dashboard's `GET /status` answers `{queued: […], failed: […]}`.
6. **Cost.** No new `ess-http` version: an admitted case within `ess-http/1`. An older `ess`
   refuses the binding. No specification format moves. The generated servers serve it in
   `sl01-member-servers` and refuse it by name until then. The generated UIs read it in
   `sl01-member-client`.
7. **Considered.** (a) Change nothing: the adopter keeps extra routes. (b) The member body only:
   that is the active story, and the adopter's one answer still cannot be bound to both views.
   (c) Chosen: the shared route on top of the active member body.

## Decisions

**Accept, redesigned.** The spelling is the active story's member form.

- **SL-D7 (reworded on 2026-11-24). May views of one component share one route by their members?**
  (A) Yes: this story and its two units. (C) Decline: the application keeps its extra routes. The
  former option B, the member body only, is the active member-body story. **Recommended: A.**

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (collision 7459-7533, body checks 2698-2721 and
  7190-7271)
- cited: `crates/generate/ess-gen/src/openapi.rs` (718, 1552-1562, 2430)
- inferred: `crates/specify/ess-http/tests/` (new cases), `docs/design/http-presentation-binding.md`,
  `website/docs/guides/http-routes.md`, `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: M, about 300 changed lines (unit 1 of 3).

## Depends on

- `story:an-answer-may-be-wrapped-in-one-named-member` (its `{<member>: array}` form and its
  refusals). This story is derived from it.
- It shares `crates/specify/ess-http/src/lib.rs` with the SSE story, the authentication story, the
  header-input story's unit a and the CSV story, so these merge one after another.

## Source

A downstream application, 2026-11-24, second report, G1.
