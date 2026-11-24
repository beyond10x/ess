---
format: aep.planning-md/3
id: story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares
kind: story
status: draft
title: A generated UI reads each view of a shared route from its own member, and shows a missing member as an error
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their
- derived_from: story:a-generated-ui-reads-the-answers-its-http-binding-declares
- depends_on: story:a-generated-ui-reads-the-answers-its-http-binding-declares
revision: 4
---
## Need

`story:a-generated-ui-reads-the-answers-its-http-binding-declares` (active) carries each view
route's declared body into the UI route table and reads `{<member>: array}` from under the member,
in React and in the terminal. That route table is keyed by view
(`crates/ui/ess-ui/src/binding.rs:267`, `ViewRoute` at `:275-294`). So two views that share a route
after `sl01-member-body` are two entries, each with its own member, and no runtime code is new for
them.

Two things are left that the active story does not cover:

- No test holds a shared route end to end. The binding computation
  (`crates/ui/ess-ui-check/src/model.rs`, `binding_over`) has never been given two views on one
  method and path.
- What a runtime shows when the declared member is missing from the answer, or is not an array.
  Today an object without `rows` becomes one row
  (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:348-351`, `:631-633`;
  `crates/ui/ess-ui-tui/src/http.rs:419-421`). The active story does not say.

## Outcome

Over a binding with a shared route, `ess generate ui --http` writes one route entry per view with
its member, and two sections over the two views each show their own view's rows. Sharing one
request between them is not promised. A missing member, or a member that is not an array, is shown
as the section's error state naming the member, in both runtimes. It is not shown as an empty list
or as one row.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/bound_answers.rs` (the active story's binary): over a
  binding computed from the `sl01-member-body` reproduction, the adapter reading
  `shop.delivery.InTransit` from the answer `{"pending": [{id: 1}], "in_transit": [{id: 2}]}` yields
  the one row `{id: 2}`.
- The terminal yields the same rows (a case in the `crates/ui/ess-ui-tui/src/http.rs` tests).
- A missing member, and a member that is not an array, are each shown as the section's error state
  naming the member, in both runtimes.
- `cargo test -p ess-cli --test generate_ui_http` gains the shared binding: one route entry per
  view, each with its member. A binding without a shared route keeps its bytes.

## Fit review

Second report, G1, unit 3 of 3. The fit review is in `sl01-member-body`. `crates/ui/ess-ui` stays
free of the model: the member is plain data the binding computation copies from `ess-http-ir/1`, as
the active story does.

## Decisions

None beyond SL-D7.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (348-351, 631-633)
- cited: `crates/ui/ess-ui-tui/src/http.rs` (419-421)
- inferred: `crates/ui/ess-ui-check/src/model.rs` (`binding_over`), only if it refuses a shared
  path; tests named above
- Size: S, about 100 changed lines.

## Depends on

- `story:a-generated-ui-reads-the-answers-its-http-binding-declares` (it carries and reads the
  member). This unit is derived from it.
- `sl01-member-body` (the shared route).
- It no longer changes `crates/ui/ess-ui/src/binding.rs`, because the active story adds the member
  there, so it no longer shares that file with `sl05-union-variant`.

## Source

A downstream application, 2026-11-24, second report, G1.
