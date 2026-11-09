---
format: aep.planning-md/3
id: story:a-timestamp-newtype-declares-its-wire-encoding
kind: story
status: draft
title: A Timestamp newtype declares its wire encoding (unix_seconds, unix_millis)
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#758
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

An instant travels on the wire as whole seconds (or milliseconds) since the Unix epoch, not as
RFC 3339 text, and is still an instant: compared with stored `Timestamp` fields, ordered, offset by
a Duration. A read API takes its window as `filters.created.from=1730851200`. A `Timestamp` has one
wire form, RFC 3339 text (`crates/generate/ess-gen/src/types.rs:33`, `{"type": "string", "format":
"date-time"}`). An `Integer` keeps the wire form and loses the comparison. A clock-reading newtype
(`reading: {encoding: unix_seconds}`, `docs/design/timestamp-clock-provenance.md`) keeps the wire
form and refuses the comparison by design, because it demands observed process evidence.

Minimal reproduction (0.92.0, `ess/36`): entity `example.calls.Call` (`created_at: Timestamp`) and
a view filtered `created_at >= param.created_from and created_at < param.created_to`, with
`created_from` declared as follows (`repro/run.txt`):

```text
type: Timestamp, wire: filters.created.from                           -> valid
type: example.calls.Instant (newtype of Timestamp)                    -> valid
type: Timestamp, encoding: epoch_seconds                              -> unknown field `encoding`
type: example.calls.EpochSeconds (Integer, reading: unix_seconds)     -> ESS-VIEW-002 `param.created_from` is a clock reading; comparison requires observed source, epoch and formatter evidence
type: Integer                                                         -> ESS-VIEW-002 operator `>=` does not admit `Timestamp` (Text) and `Integer` (Number)
```

This story is the model half: the design, the key, validation, IR and diff.

## Acceptance

- A design page `docs/design/value-wire-encodings.md` states that a newtype of `Timestamp` may
  declare `encoding: unix_seconds | unix_millis` (absent: RFC 3339 text, as today), the value names
  shared with `reading.encoding` (`crates/specify/ess-domain/src/reading.rs`, `ReadingEncoding`); that
  such a value is an instant, whole seconds or whole milliseconds, compared and offset as every
  `Timestamp`; and that the encoding is a wire fact, so every position carrying the type (command
  input, stored field, event field, view field, view parameter) carries it.
- At the new source format, `example.calls.EpochInstant` (`kind: newtype, of: Timestamp, encoding:
  unix_seconds`) validates, and so does the reproduction view with `created_from` and `created_to`
  of that type (`ess specify validate` exit 0).
- `encoding:` on a newtype of anything but `Timestamp` (directly or through a newtype), or beside
  `reading:`, is refused as `type_mismatch` / `conflicting_declaration` naming the key; a value
  outside `unix_seconds` and `unix_millis` is refused naming the two. Below the new format the key
  is refused as `unsupported_format_version` naming it.
- `ess specify compile` writes the encoding on the resolved type, omitted when absent; a model
  without it keeps its IR bytes (`examples/` unchanged); `schemas/generated/ess.schema.json` is
  regenerated.
- `ess verify diff` reports a moved encoding as a breaking change of the type, under the diff kind
  the design page names.

## Scope

- inferred: docs/design/value-wire-encodings.md
- cited: crates/specify/ess-domain/src/types.rs
- cited: crates/specify/ess-domain/src/reading.rs
- cited: crates/specify/ess-domain/src/system.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/specify/ess-domain/tests/timestamp_encoding.rs
- inferred: schemas/generated/ess.schema.json
- inferred: website/docs/reference/spec-versions.md
confidence: medium

## Fit review

1. **Need.** An instant whose wire form is a number of seconds (or milliseconds) since the Unix
   epoch, compared as an instant. Requester's syntax (theirs): `encoding: epoch_seconds |
   epoch_millis | rfc3339` on a `Timestamp` view parameter. Reproduction above.
2. **Class.** Gap: no declaration gives a `Timestamp` another wire form (`types.rs:33` of
   `ess-gen`), and the two near idioms lose the comparison (`run.txt`, `integer` and
   `reading-newtype`).
3. **Already expressible?** No. `Integer` is refused in the comparison. A reading newtype is refused
   too (`run.txt`), by its design: "Generic predicate comparison must not silently erase an attached
   clock contract" (`timestamp-clock-provenance.md`). Leaving the parameter a plain `Timestamp`
   misstates the wire.
4. **Fit.** The wire form of a value is a fact of its type, so it is declared on a newtype, the way
   `alphabet:`, `prefix:` and `reading:` are, rather than on one view parameter. It then holds in
   every position the type appears in. A view parameter, a command input and an event field each
   ask the same question, so a key on view parameters alone would open one cell. The newtype stays
   a `Timestamp` for the checker, because same-representation newtypes compare
   (`docs/design/generic-language-features.md:79-81`; `run.txt`, `timestamp-newtype` valid). Grids
   R and O are unchanged. Value names reuse `ReadingEncoding` (`unix_seconds`), not new words.
   Targets: `story:projections-write-a-timestamp-in-its-declared-encoding`,
   `story:generated-codecs-read-and-write-a-timestamp-in-its-declared-encoding`,
   `story:synthesis-sends-whole-unit-instants-of-an-encoded-timestamp`.
5. **Second adopter.** A webhook event whose `occurred_at` is Unix milliseconds
   (`unix_millis`), compared in a binding condition with a stored instant.
6. **Cost.** The next open source format; one key on newtypes, two values; no new diagnostic code;
   an IR field omitted when absent; a diff kind if the design page needs one (compatibility:
   breaking for every reader of the type's wire). Generated code and projections change only for a
   model declaring it.
7. **Alternatives.** (a) Change nothing: the specification states RFC 3339 where the wire carries
   seconds. (b) The requester's key on a view parameter: refused, one position of several.
   (c) Relax the reading contract so a `unix_seconds` reading compares with a `Timestamp`: refused,
   it would erase the provenance rule the reading design exists for. (d) An encoding on a
   `Timestamp` newtype: chosen. `rfc3339` is the default and is not spelled.

The requester's second ask, `aliases:` (several keys for one parameter, first present wins) and a
filter map read from the first of several prefixes, is not taken. It is one server's lenient
parsing (local policy). A server that also accepts other keys does not contradict a specification
that states one key. The projection documents the declared key, which is the one a client must
send.

## Decisions

accept, redesigned. The wire form moves from a view-parameter key to the `Timestamp` newtype
(`encoding: unix_seconds | unix_millis`), reusing the reading contract's value names. `aliases:`
is declined as local policy.

## Source

github:beyond10x/ess#758
