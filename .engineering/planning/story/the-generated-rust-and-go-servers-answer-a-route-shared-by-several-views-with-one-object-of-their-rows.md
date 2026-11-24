---
format: aep.planning-md/3
id: story:the-generated-rust-and-go-servers-answer-a-route-shared-by-several-views-with-one-object-of-their-rows
kind: story
status: draft
title: The generated Rust and Go servers answer a route shared by several views with one object of their rows
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-view-route-answers-its-rows-under-a-named-member-and-views-of-one-component-may-share-a-route-by-their
revision: 1
---
## Need

After `sl01-member-body`, a binding may put a view's rows under a member and let several views share
one route. The generated servers write a view route's answer from its one view and its body
(`crates/generate/ess-synth/src/rust/http/bound.rs:1383-1395`,
`crates/generate/ess-synth/src/go/http/bound.rs:1912-1935`), so they cannot serve a shared route.
Until this unit lands they refuse it by name.

## Outcome

`ess generate synthesize --http` for Rust and Go serves a route shared by views: one handler reads
the shared parameters once, reads every view, and writes one object with each view's rows under its
member, members in route declaration order. A single view with a member body writes
`{<member>: [rows]}`. Every refusal the route answers (malformed parameters, not granted, too
large) is unchanged.

## Acceptance

- Red first, in `crates/generate/ess-synth/tests/` (the bound-HTTP tests of each target): a
  generated Rust server and a generated Go server over the reproduction of `sl01-member-body`
  answer `GET /v1/board` with `{"pending": […], "in_transit": […]}` holding the rows each view
  selects. It fails before the change with the by-name refusal.
- A member body on one view answers `{"<member>": [...]}`.
- Rust and Go write the same bytes for the same rows, in member declaration order.
- The served route passes the conformance runner's view reads for both views (the runner reads
  each view's rows from its member).
- A binding without the form generates the same server bytes in both targets. The committed
  example trees under `generated/` are unchanged.

## Fit review

Second report, G1, unit 2 of 3. The fit review is in `sl01-member-body`. Every target handles the
form or refuses it by name (skill, question 4).

## Decisions

None beyond SL-D7.

## Scope

- cited: `crates/generate/ess-synth/src/rust/http/bound.rs` (1383-1395)
- cited: `crates/generate/ess-synth/src/go/http/bound.rs` (1912-1935)
- inferred: `crates/generate/ess-gen/src/http.rs` (`bound_routes` 538, groups routes by method and
  path), the conformance runner's view read for a member body, tests under
  `crates/generate/ess-synth/tests/`
- Size: M, about 420 changed lines.

## Depends on

`sl01-member-body`.

## Source

A downstream application, 2026-11-24, second report, G1.
