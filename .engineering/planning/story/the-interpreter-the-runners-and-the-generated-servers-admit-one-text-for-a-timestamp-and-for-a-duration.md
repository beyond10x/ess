---
format: aep.planning-md/3
id: story:the-interpreter-the-runners-and-the-generated-servers-admit-one-text-for-a-timestamp-and-for-a-duration
kind: story
status: draft
title: The interpreter, the conformance runners and the generated servers admit one text for a Timestamp and one for a Duration, and one corpus holds them to it
summary: Primitive::admits and the Go, TypeScript and browser lanes admit any text for both types while the generated servers refuse a non-RFC 3339 Timestamp and name ISO 8601 for a Duration; TS-D1 and TS-D2, wire contract
tags:
- external-request
- gap-reports-2026-11-24
- wire-contract
relations:
- serves: vision:O2
- informed_by: story:a-duration-member-is-refused-as-its-format-refuses
- informed_by: story:an-admitted-timestamp-is-an-instant-the-server-can-order
- depends_on: story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table
revision: 1
---
## Summary

ESS answers "is this text a `Timestamp`" and "is this text a `Duration`" differently in its own
readers. For a `Timestamp` the interpreter, the Go and TypeScript conformance runners and the browser
adapter admit any text, while the generated Rust and Go servers, and the published schema's
`format: date-time`, admit only an RFC 3339 `date-time`. For a `Duration` every reader admits any text,
while the published schema says `format: duration` and each generated server's refusal says "an
ISO 8601 duration". Reported on 0.99 by a downstream application whose executor reads a path segment
the way the generated server does and so refuses a `Timestamp` the interpreter admits. A value that
one reader admits and another refuses is a conformance run that passes on one side and fails on the
other, and an authored `example:` that the generated server later refuses.

## What each reader does (tag 0.99.1, read, no run)

Interpreter and runners, one table per lane:

- Rust, `Primitive::admits`: `(String | Timestamp | Duration, Text)` admits any text
  (`crates/specify/ess-domain/src/types.rs:2954-2956`; the method opens at `:2930` and its doc calls
  it "the one table for is this node a value of this primitive", `:2925-2929`). `Uuid` and `Bytes`
  are the two primitives that ask a grammar (`:2946-2953`).
- Go runner: `primitive("timestamp" | "duration")` checks only that the value is a string
  (`crates/verify/ess-conformance/src/go/runtime.go:4355-4358`).
- TypeScript runner: the same, `case 'string': case 'timestamp': case 'duration':` checks
  `typeof value === 'string'` (`crates/verify/ess-conformance/src/ts/runtime.ts:6616-6623`).
- Browser adapter: `case 'string': case 'timestamp': case 'duration': return typeof value === 's...`
  (`crates/verify/ess-conformance/assets/coverage-admission.js:361`).
- The evaluator reads each of the two by a grammar of its own and reads anything else as unknown:
  a `Timestamp` by `Rfc3339Instant::parse_rfc3339` (`crates/specify/ess-primitives/src/expr.rs:791-793`),
  a `Duration` by `parse_elapsed`, `PT[<n>H][<n>M][<n>S]` in whole units, no day, week, month, year,
  fraction or sign (`expr.rs:836-865`; the module doc says so at `expr.rs:37-39`).

Generated servers (path segments and query values reach the same decoder as a body member, as
`Flat::Text`: `crates/generate/ess-gen/src/http.rs:833-840`,
`crates/generate/ess-synth/src/rust/http/bound.rs:236-253`):

- Rust: `Timestamp` through `json::timestamp_at` (RFC 3339 `date-time`; `T` and `Z` may be lower case,
  any number of fraction digits, a leap second on the last minute of a UTC day;
  `crates/generate/ess-synth/src/rust/json.rs:828-847`), refusing with "an RFC 3339 instant"
  (`rust/wire.rs:1054-1057`). `Duration` through `json::text_at`, any string, refusing a non-string
  with "an ISO 8601 duration" (`rust/wire.rs:1058-1061`).
- Go: `timestampAt` and `dateTime` (`crates/generate/ess-synth/src/go/http.rs:2554-2575`, the same
  grammar and the same text), `textAt` for a `Duration` (`go/http.rs:1328-1329`, `:2545-2552`).
- TypeScript: no generated server was found in the tree at 0.99.1. The generated type realization
  writes a `Timestamp` as the TypeScript `string` and checks nothing
  (`crates/generate/schema-contract/src/realize/ts.rs:92`). Not searched exhaustively; named as
  unverified.
- Published schema: `{"type":"string","format":"date-time"}` and `{"type":"string","format":"duration"}`
  with no `pattern`, on purpose (`crates/generate/ess-gen/src/types.rs:40-48`, `:660-672`; the matrix
  in `docs/design/review-primitive-semantics.md:37-38`). The validator the tree builds asserts only
  `date-time` (`crates/generate/schema-contract/src/uniqueness.rs:22`); no `duration` checker exists.

Conformance suites: synthesis writes only spellings both sides admit, a `Timestamp` as
`<month>-<day>T00:00:00Z` or `...T<hh>:<mm>:00Z` and a `Duration` as `PT<n>S`
(`crates/verify/ess-conformance/src/witness.rs:8008-8013`, `:8076-8091`, and `Elapsed::fmt`,
`crates/verify/ess-conformance/src/scenario.rs:1805-1810`). So a synthesized suite never sends a value
on which the readers disagree. An authored act, a setup value, a candidate or an `example:` can: the
interpreter admits `"soon"` for a `Timestamp` and the generated servers answer 400 for it.

## Which side the specification text supports

- `Timestamp`: the text is not silent. The published schema says `date-time`
  (`ess-gen/src/types.rs:669`), the guide says a `Timestamp` "is ordered by the RFC 3339 instant it
  names" (`website/docs/guides/specify/fields-and-invariants.md:197`), the instant grammar is fixed
  for Rust, Go and TypeScript over one vector file (`docs/design/expression-family-source22.md:739`,
  decision 14), and `story:an-admitted-timestamp-is-an-instant-the-server-can-order` already decided
  that the decoders refuse what that grammar cannot read. The generated servers are right; the
  interpreter, the Go and TypeScript runners and the browser adapter are the side to move.
- `Duration`: the text contradicts itself. The schema says `format: duration` (RFC 3339 Appendix A:
  `P30D`, `P1W`, `PT1H`, `P1Y2M3DT4H5M6S`); the generated `Duration` documents `P30D` as its example
  (`crates/generate/ess-synth/src/rust/mod.rs:467`, `go/mod.rs:754`); the evaluator reads only
  `PT...` and calls everything else unknown (`expr.rs:37-39`, `:836-838`). No reader checks any of it.
  `story:a-duration-member-is-refused-as-its-format-refuses` (active) already holds the decision for
  the admission grammar (RFC 3339 Appendix A). This story does not re-decide it; TS-D2 asks only
  whether the evaluator moves with it.

## Acceptance

Two red cases, one per type, each failing on the tree at 0.99.1, and one agreement check over both.
One corpus file holds the texts and every lane reads it: the vector file
`crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` gains `timestamp` rows beside its
`duration` rows. No test builds a server per case: the decoders run in the harness a task builds once
(`task json-body-harnesses`, the pattern `adversary_w1_gen1_security` uses), under the 60 s per test
and 2 minute step bounds of `AGENTS.md`.

1. `timestamp_text_is_admitted_alike_by_every_reader` (red on 0.99.1): for each corpus row, `Primitive::admits`
   (Rust), the Go runner's `primitive("timestamp", ...)`, the TypeScript runner's `primitive('timestamp', ...)`
   and the browser adapter's `primitiveAdmits('timestamp', ...)` answer as the row states. Refused rows:
   `"soon"`, `""`, `"2020-01-01"`, `"2020-01-01T00:00:00"` (no offset), `"2020-13-01T00:00:00Z"`,
   `"2020-01-01T00:00:60Z"` (the instant grammar has no leap second, TS-D1 A). Admitted rows:
   `"2020-01-01T00:00:00Z"`, `"2020-01-01T00:00:00+01:00"`, `"2020-01-01T00:00:00.123456789Z"`. Today every
   refused row is admitted by all four.
2. `duration_text_is_admitted_alike_by_every_reader_and_the_server` (red on 0.99.1): the same four lanes and
   the generated Rust and Go decoders, over the rows the active duration story names (refused `"abc"`,
   `"P"`, `"PT"`, `"P1DT"`, `"PT1.5S"`, `"-PT1S"`, `"pt1s"`; admitted `"PT1S"`, `"P30D"`, `"P1W"`,
   `"P1Y2M3DT4H5M6S"`). Today nothing refuses a refused row, and the servers' refusal text names a grammar
   they do not check.
3. `a_flat_value_is_read_as_the_interpreter_reads_it` (red on 0.99.1, `Timestamp` rows): a view parameter
   of each of the two types, bound to a path variable and to a query value, is answered by the generated
   Rust and Go servers as `Primitive::admits` answers the same text: a refused text answers 400 naming the
   parameter, an admitted one answers the view. Today `"soon"` as a `Timestamp` path segment is 400 on both
   servers and admitted by `admits`.
4. The error text of every reader that refuses names the grammar it checks, and the three target spellings
   agree: the Rust text in `wire.rs:1054-1061`, the Go text in `go/http.rs:1328-1329` and the runners' text in
   `runtime.go` and `runtime.ts` are one sentence per type (the sentence moves with TS-D2).
5. A mutation of any one lane's grammar (the Go runner admitting any string again) turns case 1 or 2 red, shown
   by the story's own mutation run.
6. `docs/design/review-primitive-semantics.md` (the matrix rows at `:37-38`) states the one admitted text
   per type and the page that fixes it; `website/docs/reference/` holds the same sentence for authors of an
   `example:`.

## Fit review

1. Need: one text per primitive in every reader. Class: a defect of consistency between readers, not a missing
   feature; the specification text supports the generated side for `Timestamp` and contradicts itself for
   `Duration`.
2. Covers: the reporter's finding that `Primitive::admits` accepts any text for both types, that the server
   requires RFC 3339 for `Timestamp` and any text for `Duration` under a message that names ISO 8601, and that
   an executor following the server for path segments disagrees with the interpreter.
3. Borders, amended in place and not duplicated:
   `story:a-duration-member-is-refused-as-its-format-refuses` (active; holds the `Duration` admission grammar
   and the Rust and Go decoder and `Primitive::admits` changes; gains the TypeScript runner and the
   flat-value case from this story) and `story:an-admitted-timestamp-is-an-instant-the-server-can-order`
   (active; holds the decoders' instant grammar; this story carries the same grammar to the readers it did not
   name). `story:a-malformed-path-segment-is-reported-at-path-name-by-the-generated-rust-and-go-servers-not-at-the-query-name`
   (draft; the path of the refusal, not its grammar). `story:a-uuid-newtype-identity-binds-a-path-segment`
   (active; the flat reader).
4. Not covered by an existing story: `Primitive::admits(Timestamp)` and the Go, TypeScript and browser lanes for
   `Timestamp`; the TypeScript runner for `Duration`; a case that reads the interpreter and the server over one
   corpus. A search of the store and the open-issue list found no story or issue on the text of a `Timestamp`
   or `Duration` beyond the two active stories above (the open issue on an epoch-seconds wire form is another
   question: the encoding of a parameter, not the text of the type).
5. A second adopter: any executor or client that validates a value before it sends it and reads the interpreter's
   admission as its oracle.
6. Wire contract: the change moves what a released reader admits (TS-D1, TS-D2). It is ordered after the release
   that carries the typed decoders story
   (`story:a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-takes-a-package-name-and-leaves-out-its-workspace-table`)
   and after 0.99.2.

## Decisions

- TS-D1, wire contract, recommended A: what `Primitive::admits(Timestamp)` and the runners admit. A: the instant
  grammar of `Rfc3339Instant::parse_rfc3339` (RFC 3339 `date-time`, `T` or `t`, offset `Z` or `+-HH:MM`, up to nine
  fraction digits, seconds 00 to 59), the grammar the decoders take under the active timestamp story. Cost: an
  authored `example:`, setup value or candidate holding another text for a `Timestamp` is refused after the
  change, as the generated servers already refuse it. B: leave the interpreter and the runners wide and narrow
  only the servers (today). Cost: a suite can send a value the servers refuse and the interpreter admits.
  C: A, gated on the specification's `format: ess/N`, older documents keeping any text. Cost: two readers.
- TS-D2, wire contract, recommended A: which `Duration` grammar, and whether the evaluator moves. A: the grammar
  the active duration story picked (RFC 3339 Appendix A, upper-case designators, whole amounts) for admission,
  the evaluator unchanged, so an admitted `P30D` is read as unknown by an expression. Cost: admitted is not
  always evaluable, stated on the reference page. B: the evaluator's `PT[H][M][S]` only. Cost: refuses `P30D`,
  the example the generated primitive documents, and the published `format: duration` admits what the readers
  refuse. C: A, and the evaluator reads the spellings that name a fixed number of seconds (a day as 86,400 s, a
  week as seven days; month and year stay unknown). Cost: a change of expression semantics in three lanes and
  in `ess-primitives` vectors.
- TS-D3 (no decision): the refusal sentence follows the grammar picked; the bytes of generated servers move with
  it and are re-pinned where the change claims them (`served_codec_names`, `go`).

## Scope

- `crates/specify/ess-domain/src/types.rs` (`Primitive::admits`, 2930-2958), `crates/specify/ess-primitives/src/facts.rs`
  (where `is_canonical_uuid` lives; the two new grammars sit beside it), `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`
- `crates/verify/ess-conformance/src/go/runtime.go` (4345-4380), `crates/verify/ess-conformance/src/ts/runtime.ts`
  (6605-6640), `crates/verify/ess-conformance/assets/coverage-admission.js` (361)
- `crates/generate/ess-synth/src/rust/wire.rs` (1054-1061), `src/rust/json.rs`, `src/go/http.rs` (1328-1329,
  2545-2575), their tests under `crates/generate/ess-synth/tests/`
- `docs/design/review-primitive-semantics.md`, `website/docs/reference/`, `CHANGELOG.md`
- Moves the byte pins of `ess-synth` `served_codec_names` and `go` and the `string_alphabet_and_length` row of
  `ess-compiler` where its examples are Timestamp or Duration; the change names the cause beside each.
