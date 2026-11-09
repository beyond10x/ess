---
format: aep.planning-md/3
id: story:the-generated-rust-server-reads-a-multipart-request
kind: story
status: draft
title: The generated Rust server reads a multipart request
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#754
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-command-route-takes-a-multipart-request-with-file-parts
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/multipart.rs
revision: 4
---
## Need

The Rust server `ess generate synthesize --target rust --http` writes must read a
`request: {encoding: multipart, parts: …}` route
(`story:a-command-route-takes-a-multipart-request-with-file-parts`). Today it decodes `json`, `form`
and `query` only (`crates/generate/ess-synth/src/rust/http/bound.rs:883-925`); the generated server
carries its own readers and no multipart one.

Neutral example: `demo.reports.ImportRows(file: Bytes, file_name: String, format, dry_run)` on
`post /reports/rows/import`, `request: {encoding: multipart, parts: {file:
{filename: input.file_name, media_types: [text/csv]}}}`.

## Acceptance

- A `multipart/form-data` request with a `file` part (`filename="rows.csv"`, `Content-Type:
  text/csv`, bytes `a;b\r\n`), a `format` part `csv` and a `dry_run` part `true` runs the command
  with `file` the raw bytes, `dry_run = true`,
  `file_name = "rows.csv"` and `format = csv`.
- `malformed` (`400`, or the binding's `refusals.malformed`), naming the part: a `Content-Type` that
  is not `multipart/form-data` or has no `boundary`; a body not closed by the final boundary; a file
  part whose media type is not in `media_types`; a required part missing; a scalar part given twice;
  a part no field names under `unknown: refuse` (ignored under `ignore`); a missing filename bound
  to a required input.
- A route without `multipart` generates byte-identical server source; the existing
  `crates/generate/ess-synth/tests/http_binding` tests pass unmodified.

## Scope

- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/tests/http_binding/multipart.rs
- inferred: crates/generate/ess-synth/tests/http_binding.rs
confidence: medium

## Fit review

1. **Need.** A generated server must accept the request its binding states. Reproduction: the
   binding story's; the decoders are cited above.
2. **Class: gap**, the Rust server half of beyond10x/ess#754.
3. **Expressible today? No.**
4. **Fit.** A hand-written RFC 7578 reader beside the form and JSON readers, as the generated server
   carries its own JSON reader (`docs/design/http-presentation-binding.md` § "What the servers
   write (k5, k6)"); text parts go through the same flat decoding as `form` (`bound.rs:288-345`),
   and file bytes enter the input as the base64 `Bytes` value the shared decoder reads.
5. **Second adopter.** The claims photo upload of the binding story.
6. **Cost.** Generated source grows only for multipart routes.
7. **Alternatives.** (a) A multipart crate dependency in generated code: the generated server has
   none for its JSON or form readers. (b) No server support: a target silently ignoring a construct.

## Decisions

**Accept, redesigned**, as decided in
`story:a-command-route-takes-a-multipart-request-with-file-parts`.

## Source

github:beyond10x/ess#754
