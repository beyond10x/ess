---
format: aep.planning-md/3
id: story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their
kind: story
status: draft
title: A view route answers its rows under a named member, and views of one component may share a route by their members
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A service answers one `GET` with an object whose members each hold a list of rows, such as
`{"pending": […], "in_transit": […]}`. Each list is a view of the model. An `ess-http/1` binding
cannot say so. A view route's `body` is `rows` (`{rows: […]}`), `single` (one row, bare) or `array`
(the rows, bare) (`crates/specify/ess-http/src/lib.rs:619-646`; design
`docs/design/http-presentation-binding.md:380-382`, `:626`). Two views on one method and path are
refused as `ESS-HTTP-010`, "two operations claim the same method and path"
(`lib.rs:7509-7533`). A UI client bound to such a route reads the whole object as one row
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:348-351`: an object without `rows` becomes
`[body]`). The adopter adds routes to its service so that the binding can describe them.

Minimal reproduction (fresh, not run here): views `shop.delivery.Pending` and `shop.delivery.InTransit`
over one entity, served by one `reached_by: network` component, and the binding

```yaml
routes:
  - {view: shop.delivery.Pending, method: get, path: /v1/board, body: {pending: rows}}
  - {view: shop.delivery.InTransit, method: get, path: /v1/board, body: {in_transit: rows}}
```

`ess specify http validate` refuses each `body` as not a view's body (`ESS-HTTP-024`), and the
second route as `ESS-HTTP-010`.

The requester's proposal, restated: a view route names the member its rows sit under, and two
views may read one answer, written `body: {array_at: pending}`.

## Outcome

A view route may declare `body: {<member>: rows}`: the answer is an object, and the view's rows, a
JSON array, are its `<member>`. Two or more view routes of one component may then share one
method and path when each names a different member. That route answers one object holding every
member. The views sharing a route must take the same parameters, bound the same way, and none may
be paged. `ess-http-ir/1` and the OpenAPI projection carry it: one operation whose `200` schema is
an object with one required array member per view, `x-ess-views` naming the view of each member.

## Acceptance

- Red first, in `crates/specify/ess-http/tests/`: the binding above compiles. It fails before the
  change with `ESS-HTTP-024` and `ESS-HTTP-010`.
- `ess specify http compile` writes each route's body as `{member: <name>}` in `ess-http-ir/1`. A
  binding without the form compiles to the same bytes. An old-reader compatibility test shows the
  IR reader of this release refusing nothing it read before.
- Refused, each with its existing code and a message naming the key:
  - two routes on one path naming one member;
  - a route on a shared path without a member body;
  - shared views whose parameters differ in name, type or binding;
  - a member body on a paged view;
  - an empty member, or `rows` as the member (`ESS-HTTP-024`).
- `ess generate --kind openapi --http` writes one operation for the shared path, its `200` an
  object with `required` naming each member, each member's schema the view's row array.
- `docs/design/http-presentation-binding.md` gains the form in its body table and the shared route
  in its `ESS-HTTP-010` row. `website/docs/reference/formats.md` names the release that adds it.

## Fit review

Tree `bcfc51555`, read-only. Second report, G1.

1. **Need.** A service publishes several views in one answer, a common shape for a dashboard's
   state endpoint. Reproduction above.
2. **Class.** *Gap*: the binding cannot state where the rows sit.
3. **Already expressible?** No. The three bodies are fixed shapes, and one route serves one
   operation.
4. **Fit.** The language grids do not cover `ess-http/1`. The spelling reuses the body vocabulary's
   member form: an outcome body `{<member>: error}` is "an object whose `<member>` is …"
   (`http-presentation-binding.md:377`). `{<member>: rows}` reads the same way. The requester's
   `array_at` is a new word for the same idea, so it is not taken. The shared route is the second
   half of the need.
5. **Second adopter.** A build dashboard's `GET /status` answers `{queued: […], failed: […]}`.
6. **Cost.** No new `ess-http` version: one body form added within `ess-http/1`, as `array` was in
   0.77.0 (`http-presentation-binding.md:626`). An older `ess` refuses the form. No specification
   format moves. Generated servers and clients follow in `sl01-member-servers` and
   `sl01-member-client`, so this unit makes the generated servers refuse the form by name until
   they serve it.
7. **Considered.** (a) Change nothing: the adopter keeps extra routes. (b) The member form with one
   view per route only: half the need, because the adopter's single answer still cannot be bound
   to both views. (c) Chosen: the member form and the shared route.

## Decisions

**Accept, redesigned.** The spelling is changed to the existing member form.

- **SL-D7. How much does `ess-http/1` take?** (A) The member body and a route shared by the views
  that name distinct members. (B) The member body only, one view per route. (C) Decline: the
  application keeps its extra routes. **Recommended: A.**

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`ViewBody` 619-646, body checks 2698-2721 and
  7190-7271, collision 7509-7533)
- cited: `crates/generate/ess-gen/src/openapi.rs` (718, 1552-1562, 2430)
- inferred: `crates/specify/ess-http/tests/` (new cases), `docs/design/http-presentation-binding.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: M, about 480 changed lines (unit 1 of 3).

## Depends on

Nothing. Units 2 and 3 depend on it.

## Source

A downstream application, 2026-11-24, second report, G1.
