---
format: aep.planning-md/3
id: story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares
kind: story
status: draft
title: A generated UI reads a view's rows from the member its route declares
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their
revision: 1
---
## Need

The UI binding's `ViewRoute` carries a view's path, parameters, method and path variables, but not
its body (`crates/ui/ess-ui/src/binding.rs:275-295`). The React data adapter therefore guesses from
the answer's shape: `{rows}` is rows, an array is rows, anything else is one row
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:348-351`, `:631-633`). The terminal's HTTP
adapter does the same (`crates/ui/ess-ui-tui/src/http.rs:419-421`). After `sl01-member-body`, a
member body would be read as one row.

## Outcome

`ViewRoute` carries `member` when the route declares a member body, absent otherwise, so a binding
computed without one keeps its bytes. React and the terminal read the rows from that member. Two
sections over two views that share a route read one answer each. Sharing one request between them
is not promised.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/` (a bound-adapter case beside `live_binding.rs`):
  over a binding computed from the `sl01-member-body` reproduction, the adapter reading
  `shop.delivery.InTransit` from the answer `{"pending": [{id: 1}], "in_transit": [{id: 2}]}` yields the
  one row `{id: 2}`. It fails before the change, yielding the whole object as one row.
- The terminal yields the same rows (`crates/ui/ess-ui-tui/tests/http_adapter.rs`).
- A missing member, or a member that is not an array, is shown as the section's error state, naming
  the member. It is not shown as an empty list.
- A binding JSON without a member body keeps its bytes.

## Fit review

Second report, G1, unit 3 of 3. The fit review is in `sl01-member-body`. `crates/ui/ess-ui` stays
free of the model: the member is plain data the binding computation copies from `ess-http-ir/1`.

## Decisions

None beyond SL-D7.

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`ViewRoute` 275-295)
- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (348-351, 631-633)
- cited: `crates/ui/ess-ui-tui/src/http.rs` (419-421)
- inferred: `crates/ui/ess-ui-check/src/model.rs` (`binding_over` copies the member), tests named above
- Size: S, about 220 changed lines.

## Depends on

`sl01-member-body`. It can be built beside `sl01-member-servers`, because the two share no file.
It shares `binding.rs` with `sl05-union-variant`, so the two are merged one after the other.

## Source

A downstream application, 2026-11-24, second report, G1.
