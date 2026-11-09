---
format: aep.planning-md/3
id: story:projections-write-a-timestamp-in-its-declared-encoding
kind: story
status: draft
title: Projections write a Timestamp in its declared encoding
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#758
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-timestamp-newtype-declares-its-wire-encoding
scope:
- confidence: inferred
  path: crates/generate/ess-gen/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/timestamp_encoding.rs
revision: 4
---
## Need

Once a `Timestamp` newtype declares `encoding: unix_seconds | unix_millis`
(`story:a-timestamp-newtype-declares-its-wire-encoding`), every projection that writes the type must
write the declared wire form. Today a `Timestamp` and each newtype of it project as
`{"type": "string", "format": "date-time"}` (`crates/generate/ess-gen/src/types.rs:33, 667`), so a
client generated from the OpenAPI document would send RFC 3339 text to a server that reads
seconds.

## Acceptance

- `ess generate --kind openapi` and the JSON Schema projection write a newtype with
  `encoding: unix_seconds` as `{"type": "integer", "description": "… seconds since the Unix
  epoch"}` and with `unix_millis` as the same with milliseconds; the view parameter
  `filters.created.from` of the reproduction view is an integer query parameter.
- `x-ess-encoding: unix_seconds` (or `unix_millis`) is written beside it, so a reader can tell the
  instant from a count.
- Generated documentation (`ess-gen/src/docs.rs`) states the encoding in the type's sentence.
- A model with no encoded `Timestamp` keeps its projection bytes.

## Scope

- cited: crates/generate/ess-gen/src/types.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/src/docs.rs
- inferred: crates/generate/ess-gen/tests/timestamp_encoding.rs
confidence: medium

## Fit review

1. **Need.** The published contract carries the wire form the server reads. Requester's syntax
   (theirs): "carried into the IR and every projection that writes or parses the parameter".
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No (parent story).
4. **Fit.** Projections already write a newtype's declared constraints (`x-ess-alphabet`, bounds:
   `story:generated-decoders-enforce-declared-value-constraints`, Outcome). This adds one more fact
   of the same kind, under the same `x-ess-` prefix.
5. **Second adopter.** The webhook event of the parent story, whose AsyncAPI payload schema must say
   milliseconds.
6. **Cost.** Projection bytes change for models that declare the key; no format of the projections
   moves.
7. **Alternatives.** (a) Keep `date-time` and add only a description: refused, the schema would
   reject the integer the server takes. (b) An integer schema with an `x-ess-encoding` marker:
   chosen.

## Decisions

accept, redesigned, as the parent story.

## Source

github:beyond10x/ess#758
