---
format: aep.planning-md/3
id: story:openapi-publishes-a-multipart-request-with-file-parts
kind: story
status: draft
title: OpenAPI publishes a multipart request with file parts
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#754
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-command-route-takes-a-multipart-request-with-file-parts
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
revision: 4
---
## Need

A route with `request: {encoding: multipart, parts: …}`
(`story:a-command-route-takes-a-multipart-request-with-file-parts`) must be published so a client
generated from the OpenAPI document sends a file part, not a base64 text field. Today the request
body is written as `application/x-www-form-urlencoded` or `application/json` only
(`crates/generate/ess-gen/src/openapi.rs:1189-1206`), and `Bytes` is always `{type: string,
contentEncoding: base64, pattern: …}` (`openapi.rs:143`), which ess 0.92.0 writes for a `form` route
with a `Bytes` field.

Neutral example: `demo.reports.ImportRows(file: Bytes, file_name, format, dry_run)` on `post
/reports/rows/import`, `request: {encoding: multipart, query: [input.dry_run], parts: {file:
{filename: input.file_name, media_types: [text/csv, application/vnd.ms-excel]}}}`.

## Acceptance

- The operation's `requestBody` content is `multipart/form-data` with an object schema of the parts
  under their wire names: `file` as `{type: string, contentMediaType: application/octet-stream}`
  (OpenAPI 3.1, no `contentEncoding`), `format` as its flat schema; `file_name` and `dry_run` are
  not properties.
- An `encoding` object gives `file` the `contentType` `text/csv, application/vnd.ms-excel`; the
  property's description names the input `filename` (and `media_type`, when bound) is read into.
- A `List<Bytes>` field is an array of such strings; `required` lists the parts whose fields are
  not `Optional`.
- A binding without `multipart` keeps its OpenAPI bytes (existing OpenAPI tests pass unmodified);
  a new test pins the document above.

## Scope

- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
confidence: medium

## Fit review

1. **Need.** The published contract must describe the multipart request, or every generated client
   sends base64 text. Reproduction: the binding story's (`form` route publishes base64).
   Requester's proposal (theirs): `multipart/form-data` with `format: binary` (3.0) or
   `contentMediaType` (3.1) and an `encoding` object per part.
2. **Class: gap**, the projection half of beyond10x/ess#754.
3. **Expressible today? No.**
4. **Fit.** ESS emits OpenAPI 3.1.0 (`openapi.rs:264`), so a raw file part is `contentMediaType`,
   the 3.1 form, consistent with the document's use of 2020-12 keywords over 3.0's (`openapi.rs:143`).
   The `Bytes` schema of `{command}.Input` and of every JSON body is unchanged; only the multipart
   request body writes its own schema.
5. **Second adopter.** The claims photo upload of the binding story.
6. **Cost.** OpenAPI bytes move only for multipart routes. No format.
7. **Alternatives.** (a) `format: binary` (3.0): wrong dialect for a 3.1 document. (b) Reuse
   `{command}.Input`: its `Bytes` member is base64 and it holds the filename field.

## Decisions

**Accept as proposed** for the projection the requester describes (3.1 `contentMediaType` and an
`encoding` object), under the binding design of
`story:a-command-route-takes-a-multipart-request-with-file-parts` (accept, redesigned).

## Source

github:beyond10x/ess#754
