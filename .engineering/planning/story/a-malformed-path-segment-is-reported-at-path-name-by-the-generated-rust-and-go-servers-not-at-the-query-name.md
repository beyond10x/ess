---
format: aep.planning-md/3
id: story:a-malformed-path-segment-is-reported-at-path-name-by-the-generated-rust-and-go-servers-not-at-the-query-name
kind: story
status: draft
title: A malformed path segment is reported at `path.<name>` by the generated Rust and Go servers, not at the `query.<name>` or `body.<name>` of the input's decoder
summary: The flat reader names path.<name> only for an empty or non-UTF-8 segment; a Uuid, Decimal, Timestamp or Duration segment of the wrong spelling is refused by the decoder at query.<name> or body.<name>; naming the path moves the generated text of every binding
tags:
- issues-2026-11-24
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:a-uuid-newtype-identity-binds-a-path-segment
revision: 2
---
## Outcome

A path segment the generated Rust and Go servers refuse as `malformed` names the segment where the
caller wrote it, `path.<name>` (the variable of the route's template), whichever input it feeds,
instead of the place the input's decoder reads the value: `query.<name>` on a view route and
`body.<name>` on a command route.

## Need

**Reading.** On branch `issues/all-open-2026-11-24` at `87ca1f6f2` (the commit that built
`story:a-uuid-newtype-identity-binds-a-path-segment`, #755); not on `main` yet.

- `crates/generate/ess-synth/tests/http_binding/uuid_segments.rs:143-155`: for `GET /items/not-a-uuid`
  and `PUT /items/not-a-uuid`, both servers answer `400` with
  `{"refused": "<at>: expected a UUID, found `not-a-uuid`"}` where `<at>` is `query.item_id` for the
  view (`:145`) and `body.item_id` for the command (`:146`). The module comment (`:3-5`) states it:
  "the field where the input's decoder reads it".
- Why: a segment is first read by `path_value` (`rust/http/bound.rs:378`) with the field's flat kind. It
  refuses only what it cannot read as text: a segment that is not percent-encoded UTF-8, or an empty
  one for a field that is not a `String`, and it writes `path.<variable>` for those
  (`rust/http/bound.rs:931`, `go/http/bound.rs:1431`). Any other text becomes a JSON value: a whole
  number for an `Integer`, `true` or `false` for a `Boolean`, a JSON string for every other kind
  (`flat_value`, `:358-370`). A segment that fails there, `not-a-uuid` for a `Uuid` included, is
  refused by the input's decoder further on, which carries its own location.
- So the location depends on whether the flat reader or the decoder refuses, not on the route: `path.<name>`
  for an empty or non-UTF-8 segment, `query.<name>` or `body.<name>` for a value of the wrong spelling.
- The interpreted target is not part of this: `ess-conformance` has no HTTP surface (no `ess-http`
  dependency and no `path.`, `query.` or `body.` location in its source at that commit), so it
  reports no location for a path segment at all.

## Acceptance

At the format step that lands this, `uuid_segments.rs` changes its two expected locations to
`path.item_id` and the Rust and Go servers of the shared model both answer `GET /items/not-a-uuid`
and `PUT /items/not-a-uuid` with `{"refused": "path.item_id: expected a UUID, found `not-a-uuid`"}`,
an empty or non-UTF-8 segment keeps its `path.<name>` answer, and the committed generated servers are
re-synthesized at the new format.

## Decisions

- **It moves the generated text of every binding**, so it does not land as a defect fix. Reading
  every segment through its field's decoder at the path location changes the generated Rust and
  Go text of each bound route that has a path variable of a decoder-read kind, and with it the
  committed generated trees and their byte pins; it lands with a format step (the open source
  format and its suite pair), not before.
- **Decided elsewhere:** `story:a-uuid-newtype-identity-binds-a-path-segment` records that #755 was
  built with the decoder's location (D93); this story is the change it names.

## Scope

- `crates/generate/ess-synth/src/rust/http/bound.rs` (`path_members`, the segment read)
- `crates/generate/ess-synth/src/go/http/bound.rs` (`path_members`, the segment read)
- `crates/generate/ess-synth/tests/http_binding/uuid_segments.rs` (the two expected locations)
- `docs/design/http-presentation-binding.md` (§ "Path templates", the `malformed` sentence)
- `CHANGELOG.md`
