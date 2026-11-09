---
format: aep.planning-md/3
id: story:a-command-route-takes-a-multipart-request-with-file-parts
kind: story
status: draft
title: A command route takes a multipart request with file parts
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#754
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/multipart.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 4
---
## Need

A service takes an uploaded file as a `multipart/form-data` request (RFC 7578): one part per input,
the file part carrying a filename and its own media type, the others plain text parts.
`ess-http/1` has no such encoding. The nearest admitted form, `encoding: form` with the file typed
`Bytes`, publishes a base64 text field the service does not accept, and cannot state the filename
or the part's media type. Minimal reproduction (command `demo.reports.ImportRows(file: Bytes,
file_name: String, format: demo.reports.Format, dry_run: Boolean)` on `post
/reports/rows/import`; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding multipart.http.yaml     # request: {encoding: multipart}
error[ESS-HTTP-020]: routes[0].request.encoding: `multipart` is not an encoding; a request is encoded `json`, `form` or `query`
$ ess generate --path spec --kind openapi --http form.http.yaml --out out-openapi   # request: {encoding: form}
#   requestBody: application/x-www-form-urlencoded, and the file as
#   {type: string, pattern: <base64>, contentEncoding: base64}
```

Design: a fourth request encoding, `multipart`, and a `parts` map for the attributes of a file part:

```yaml
  - command: demo.reports.ImportRows
    method: post
    path: /reports/rows/import
    request:
      encoding: multipart
      query: [input.dry_run]
      parts:
        file:
          filename: input.file_name
          media_types: [text/csv, application/vnd.ms-excel]
```

- Each input field the path, `request.query` and a `parts` attribute do not carry is one part under
  its wire name. A `Bytes` field (or `Optional<Bytes>`) is a file part, its bytes raw; any other
  field with a flat spelling is a text part in the spelling `form` uses; a `List` repeats its part.
  A struct, `Map`, `Json` or nested list is refused, as for `form`.
- `parts.<field>` names a file part's field (a `Bytes` input) and may state `filename:
  input.<field>` and `media_type: input.<field>`, each reading that attribute of the part into a
  `String` input (a newtype of one counts; `Optional` when the part may lack it), and
  `media_types: [...]`, the media types the part may carry; another is `malformed`.
- `unknown: ignore` admits a part no field names, as it admits a form key.

## Acceptance

- `ess specify http validate` accepts the route above, less its `query:` line until
  `story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body` lands (exit 0);
  `ess specify http compile` writes
  `"encoding": "multipart"` and `parts` (fields in name order, `media_types` as written); every
  binding without them compiles to the same bytes.
- Refused, each naming its key: `multipart` on `get` (`ESS-HTTP-020`, as `form`); a part field
  without a flat spelling (`ESS-HTTP-021`); `parts` beside any other encoding, a `parts` key that is
  not a `Bytes` input of the command, a `filename` or `media_type` source that is not `input.<f>`
  naming a `String` input no other place carries, one input read by two attributes, an empty
  `media_types` or an entry that is not an RFC 6838 `type/subtype` (one new code, the next free
  `ESS-HTTP-0NN`).
- A refinement that changes a refined route's `encoding` or `parts` is `ESS-HTTP-012`.
- `docs/design/http-presentation-binding.md` § "Request encoding" gains the `multipart` row and the
  `parts` key, and § "What compiling checks (stage 2)" the new code; `website/docs/guides/http-routes.md`
  shows the route.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/multipart.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
- inferred: website/docs/reference/diagnostics.md
confidence: medium

## Fit review

1. **Need.** A command receives a file, its filename and its media type, beside plain fields, as
   one multipart request; the specification cannot state that request. Reproduction above. The
   requester's proposal (theirs): `request: {encoding: multipart}`, each input one part, `Bytes` a
   file part, and `parts: {file: {filename: input.file_name, media_type: [text/csv, ...]}}`.
2. **Class: gap.** The encoding set is closed at `json`, `form`, `query`
   (`crates/specify/ess-http/src/lib.rs:4729`, `ESS-HTTP-020`), documented as such, not a
   defect.
3. **Expressible today? No.** `form` with `Bytes` validates but publishes a base64 text field a
   multipart service refuses (reproduction), and filename and media type have no place: an idiom
   that misstates the protocol is no answer (`docs/design/generic-language-features.md` § Policy,
   policy 1).
4. **Fit.** The language grids do not apply. Vocabulary: `encoding` gains one member beside `form`,
   with `form`'s flat spelling for text parts and `form`'s `unknown` rule; `filename` and
   `media_type` read into inputs with the `input.<field>` source spelling every binding key uses.
   The requester's `media_type: [list]` is split into `media_types` (the admitted set) and
   `media_type: input.<f>` (the value received), so one word does not mean both a constraint and a
   source; reading both filename and media type is offered, since the same question arises for
   each. Siblings: views take no request (`ESS-HTTP-020`); a response stays JSON or text. `Bytes`
   keeps its base64 JSON spelling everywhere else (`docs/design/generic-language-features.md` § G1
   Types). Targets: OpenAPI and the Rust and Go servers in the dependent stories; UI refuses
   `--http`; the interpreter and runners speak no HTTP.
5. **Second adopter.** A claims service takes `POST /claims/{claim_id}/photos` as multipart with
   one `photo` file part (`image/jpeg` or `image/png`) and a `caption` text part, for
   `example.claims.AttachPhoto(claim_id, photo: Bytes, photo_type: String, caption: String)`.
6. **Cost.** One encoding member and one optional key in `ess-http/1` / `ess-http-ir/1`, omitted
   when absent; no format version (`encoding: text` and `body` were added inside `/1`); one new
   code. An older reader refuses `multipart` by name ("is not an encoding").
7. **Alternatives.** (a) Change nothing: uploads stay unbindable. (b) A `File` primitive in `ess/N`
   holding bytes, name and media type: puts a wire envelope into the model and moves a source
   format for a binding fact. (c) The requester's `parts` with one `media_type` key for the list:
   see 4. Chosen: `multipart` plus `parts` with sources and an admitted set.

## Decisions

**Accept, redesigned.** Kept: `encoding: multipart`, one part per input, `Bytes` as a file part, a
filename bound to an input, a list of admitted media types. Changed: the list is `media_types`, and
`media_type: input.<f>` may read the received one. The query flags beside the body are
`story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body`.

## Source

github:beyond10x/ess#754
