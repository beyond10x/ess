---
format: aep.planning-md/3
id: story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body
kind: story
status: draft
title: A command route reads chosen inputs from the query beside its body
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
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: cited
  path: crates/specify/ess-http/tests/request_body.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/request_encoding.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
revision: 5
---
## Need

A service reads some inputs of a `post` (or `put`, `patch`) command from the query string and the
rest from the body: `POST /reports/rows/import?dryRun=true` with the file in the body. `ess-http/1`
places the whole input by one encoding; `query` is admitted on `get` and `delete` only, and no field
can go to the query beside a body (`docs/design/http-presentation-binding.md` § "Request encoding":
"`ess-http/1` has no per-field query placement beside a JSON body"). Minimal reproduction (command
`demo.reports.ImportRows(file: Bytes, file_name: String, format: demo.reports.Format, dry_run:
Boolean)` on `post /reports/rows/import`; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding query-on-post.http.yaml     # request: {encoding: query}
error[ESS-HTTP-020]: routes[0].request.encoding: `query` carries the input in the query string, the encoding of a command on `get` or `delete` only; a command on `post` is encoded `json` or `form`
$ ess specify http validate --path spec --binding query-beside-body.http.yaml # parameters: {dryRun: {query: input.dry_run}}
error: reading query-beside-body.http.yaml as HTTP binding YAML: routes[0].parameters.dryRun: invalid type: map, expected a string at line 7 column 26
```

Design: `request.query`, a list of input fields read from the query string beside the body, each
under its wire name with the flat spelling `query` already uses:

```yaml
  - command: demo.reports.ImportRows
    method: post
    path: /reports/rows/import
    request: {encoding: json, query: [input.dry_run]}
```

## Acceptance

- `ess specify http validate` accepts `request.query` beside `encoding: json`, `form` or
  `multipart` on `post`, `put`, `patch` and `delete` (exit 0), each case a test in
  `cargo test -p ess-http --locked --test request_encoding`; `ess specify http compile` writes the
  list in the order written.
- A binding without `request.query` compiles to the same bytes as before the change
  (`cargo test -p ess-http --locked --test compile` passes with no fixture rewritten).
- `ESS-HTTP-020` refuses, at `routes[i].request.query`, each of: `query` beside `encoding: query`;
  `query` on a `get` command; an entry that is not `input.<field>` naming an input field of the
  command; a field listed twice; a field carried by the path; a field named by `request.body`. One
  test per case in `--test request_encoding`, each asserting the code and the location.
- `ESS-HTTP-021` refuses a listed field without a flat spelling, as for `query` encoding (a test in
  `--test request_encoding` on a struct-typed field).
- The bare-body rule reads "the path or `request.query` carries every other input field":
  `request: {body: input.priorities, query: [input.dry_run]}` compiles (exit 0, a test in
  `--test request_body`).
- The `multipart` route of `story:a-command-route-takes-a-multipart-request-with-file-parts`, with
  its `query: [input.dry_run]` line, validates (exit 0) and compiles with `dry_run` in
  `request.query` and not among the parts (a test in `--test request_encoding`).
- `ess generate --kind openapi --http` writes each listed field as an `in: query` parameter under
  its wire name, with the rules of `encoding: query` (`required` as the field is, a `List` with
  `style: form`, `explode: true`), and leaves it out of the request body schema, for a `json` and a
  `multipart` route (`cargo test -p ess-gen --locked --test openapi_http_presentation`).
- `docs/design/http-presentation-binding.md` § "Request encoding" states the `request.query` key
  and no longer holds the sentence "`ess-http/1` has no per-field query placement beside a JSON
  body"; `website/docs/guides/http-routes.md` shows a route with `request.query`.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- cited: crates/specify/ess-http/tests/request_body.rs
- inferred: crates/specify/ess-http/tests/request_encoding.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence; moves no byte-pin row.

## Fit review

1. **Need.** An operation's inputs arrive in two places of one request, some as query parameters
   and the rest in the body; the specification cannot place them. Reproduction above. The
   requester's proposal (theirs): `parameters: {dryRun: {query: input.dry_run}}`.
2. **Class: gap.** The design page names the absence on purpose
   (`docs/design/http-presentation-binding.md`, "has no per-field query placement beside a JSON
   body ... refused rather than moved to the query string").
3. **Expressible today? No.** `ESS-HTTP-020` for `query` on `post`; `parameters` maps path variables
   only (`ESS-HTTP-008` for "a mapping for no variable"). Moving the flag into the body changes the
   protocol the service serves.
4. **Fit.** The language grids do not apply. Vocabulary: `request` already holds how the input is
   read (`encoding`, `unknown`, `body`); `query` is the word `encoding: query` uses for the same
   place, and the parameter name is the field's wire name, as for every query key
   (§ "Request encoding": "Keys are the fields' wire names"). The requester's `parameters` form was
   not taken: it would give a query key a name of its own, the per-field rename map answer 9
   rejects ("a second mapping of names the model already owns"), and would mix path variables with
   query keys under one map. Siblings: it applies to every body encoding, and `unknown: ignore`
   applies to the query keys of such a route as to its body. Views already read their parameters
   from the query. Targets: OpenAPI here; servers in
   `story:generated-servers-read-chosen-inputs-from-the-query-beside-the-body`; UI refuses
   `--http`.
5. **Second adopter.** A document store takes `PUT /documents/{document_id}?if_revision=7` with the
   document as a JSON body; `if_revision` is an input of `example.docs.ReplaceDocument`.
6. **Cost.** One optional key in `ess-http/1` / `ess-http-ir/1`, omitted when absent; no format
   version (keys were added inside `/1` before, design page § "Operation ids and servers"); no new
   code (`ESS-HTTP-020` and `-021` widen).
7. **Alternatives.** (a) Change nothing. (b) The requester's `parameters: {name: {query: …}}`:
   second naming, see 4. (c) Admit `encoding: query` on `post` with the body empty: cannot carry the
   file. Chosen: `request.query`.

## Decisions

**Accept, redesigned.** Kept: chosen inputs go to the query beside a body. Changed: the key is
`request.query: [input.<field>]` and the query name is the field's wire name; the requester's
camel-case `dryRun` is the wire name the model declares for `dry_run`.

**Owns `query` beside `multipart`.** This story depends on
`story:a-command-route-takes-a-multipart-request-with-file-parts`, so it lands second and owns the
combination: validation, compile and OpenAPI of a `multipart` route with `request.query`; the
multipart story ships its route without the `query:` line.

## Source

github:beyond10x/ess#754
