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
- derived_from: story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member
- depends_on: story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member
revision: 4
---
## Need

After `sl01-member-body`, several views of one component may share one route, each answering its
rows under its own member. `story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member`
(active) makes the generated Rust and Go servers write one view's rows under its member
(`{"items": [...]}`). Those servers still write a view route's answer from its one view
(`crates/generate/ess-synth/src/rust/http/bound.rs:1383-1395`,
`crates/generate/ess-synth/src/go/http/bound.rs:1912-1935`), and `bound_routes` groups routes by
method and path (`crates/generate/ess-gen/src/http.rs:538`). A shared route has several views on
one method and path, so they cannot serve it. Until this unit lands they refuse it by name.

This unit is the server half that G1 adds beyond the active story. A single view with a member body
is the active story's and is not restated here.

## Outcome

`ess generate synthesize --http` for Rust and Go serves a shared route. One handler reads the shared
parameters once, reads every view, and writes one object with each view's rows under its member,
members in route declaration order, each member written by the active story's writer for that view.
The caller must be granted every view of the route; otherwise the route answers its not-granted
refusal. Every other refusal (malformed parameters, too large) is answered once, as for a route of
one view. A route that is not shared keeps the active story's writer and bytes.

Decision on a partly granted caller (DD-X3, default A, SECURITY: waits for the owner): the route's
not-granted refusal, as written above. The alternatives are the object with only the granted
members, and a validation refusal of a shared route whose views have different grants.

## Acceptance

- Red first, in `crates/generate/ess-synth/tests/` (the bound-HTTP tests of each target): a
  generated Rust server and a generated Go server over the reproduction of `sl01-member-body`
  answer `GET /v1/board` with `{"pending": […], "in_transit": […]}` holding the rows each view
  selects. Before the change both refuse the model by name.
- A caller granted one of the two views and not the other is answered with the route's not-granted
  refusal, in both servers.
- The Go server answers each request with the Rust server's status, headers and body bytes
  (`the_go_and_rust_servers_answer_one_binding_alike` gains the cases).
- A binding without a shared route generates the same server bytes in both targets. The committed
  example trees under `generated/` are unchanged (`ess-synth`'s whole test set, per the
  package-gate table in `AGENTS.md`).
- No conformance claim is made: no HTTP conformance target exists yet
  (`docs/design/http-presentation-binding.md` § "Stage 3").

## Fit review

Second report, G1, unit 2 of 3. The fit review is in `sl01-member-body`. Every target serves the
shared route or refuses it by name.

## Decisions

None beyond SL-D7.

## Scope

- cited: `crates/generate/ess-synth/src/rust/http/bound.rs` (1383-1395)
- cited: `crates/generate/ess-synth/src/go/http/bound.rs` (1912-1935)
- inferred: `crates/generate/ess-gen/src/http.rs` (`bound_routes` 538, groups routes by method and
  path), `crates/generate/ess-synth/tests/http_binding/go.rs`,
  `crates/generate/ess-synth/tests/http_binding/answer_forms.rs`
- Size: M, about 300 changed lines.

## Depends on

- `sl01-member-body` (the shared route).
- `story:openapi-and-generated-servers-write-an-answer-wrapped-in-a-named-member` (its member
  writer, reused per view). This unit is derived from it, and it changes the same two `bound.rs`
  files, so it lands after it.

## Source

A downstream application, 2026-11-24, second report, G1.
