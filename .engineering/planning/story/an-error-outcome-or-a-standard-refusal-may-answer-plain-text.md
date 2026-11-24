---
format: aep.planning-md/3
id: story:an-error-outcome-or-a-standard-refusal-may-answer-plain-text
kind: story
status: draft
title: An error outcome or a standard refusal may answer plain text
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 3
---
## Need

An existing service answers its errors with a plain-text body: one line of text, `text/plain`.
In `ess-http/1` an error outcome answers `{<member>: error}` (a JSON object), `envelope` or `none`.
A standard refusal answers `{<member>: {literal: <text>}}` (a JSON object) or `none`. `encoding:
text` is admitted only beside `response.<field>` of a `String`, and refused under `errors` and on any
other body:

- `crates/specify/ess-http/src/lib.rs:5961-5965`: "`encoding: text` writes the characters of one
  `String` response field, `response.<field>`, and outcome `<name>` answers `<body>`",
  `ESS-HTTP-024`.
- `docs/design/http-presentation-binding.md` § "Response body per outcome" (the `encoding` table,
  and "`text` … under `errors` is refused").
- The generated OpenAPI then states JSON error bodies the service does not send (third report,
  finding 4).

Minimal reproduction (written fresh; not run): command `demo.docs.ReadDocument`, error outcome
`missing` with `error: demo.docs.Missing`, whose error declares `message: String`.

```yaml
    outcomes:
      missing: {status: 404, body: error.message, encoding: text}
```

```console
$ ess specify http validate --path spec --binding docs.http.yaml
error[ESS-HTTP-024]: routes[0].outcomes.missing.body: … `error.message` is not a body …
```

(The exact text of the first refusal is not established by reading: `error.<field>` is not a body
form today, so the refusal may be the unknown-form message rather than the encoding message.)

## Outcome

An outcome with `error`, and `errors:`, may answer `body: error.<field>`: one error field's value
bare, as `response.<field>` answers one response field. It is JSON by default, and with
`encoding: text` on a `String` field it is that string's UTF-8 bytes under `text/plain;
charset=utf-8`. A standard refusal may answer `encoding: text` with `body: {literal: <text>}`. Two
answers on one status that the wire cannot tell apart stay refused (`ESS-HTTP-025`): two text
bodies on one status need distinct statuses or `alike:`.

## Acceptance

Unit a (binding, IR, OpenAPI):

- The reproduction validates and compiles. `ess-http-ir/1` writes the new body and `encoding` only
  when declared, so every existing binding compiles to the same bytes. New cases in
  `crates/specify/ess-http/tests/compile.rs`, starting from the refused document above.
- Refused, each naming the key: `error.<field>` on an outcome without `error`; a field the error does
  not declare, or an `Optional` one; `encoding: text` on a field that is not a `String` (a newtype
  of one counts); a text refusal body that is not a field value's worth of UTF-8 text; two answers on
  one status that `ESS-HTTP-025`'s rule cannot tell apart, the message saying that two text bodies
  are told apart by status or `alike:` only.
- `ess generate --kind openapi --http` writes the field's schema under `text/plain` (or
  `application/json` for the bare JSON form) for that response. A binding without the form keeps its
  bytes.
- § "Response body per outcome" lists `error.<field>`, and the `encoding` table admits `text` under
  `errors` and on refusals.

Unit b (servers):

- The generated Rust and Go servers write the bare field or the literal as text with
  `Content-Type: text/plain; charset=utf-8` (`http_binding_rust`, both alike). Every existing byte
  pin passes unmodified.
- A generated UI that reads the binding's answers
  (`story:a-generated-ui-reads-the-answers-its-http-binding-declares`) reads a text error, or
  refuses the binding by name. Silent misreading is not acceptable.

Amendment (ninth gap report, 2026-11-24): a text body is the field's one line followed by exactly one line feed, OpenAPI says so, and a target that answers without the line feed fails the suite.

Amendment (ninth gap report, 2026-11-24): the `empty:` answer of a `single` view route (`EmptySpec`, `crates/specify/ess-http/src/lib.rs:782-789`, a `4xx` with `none` or one literal member) takes the text form of this story, so a route whose body is one line of text is stated and not left to a comment.

## Fit review

1. **Need.** A service's plain-text error answer cannot be stated, so the projected contract states
   a JSON error body the service never sends. Report: third report, finding 4 ("plain-text error
   bodies").
2. **Class: gap.** The narrower-first answers of stage 2 left it out
   (§ "Decided for stage 2", preamble). It is no security rule.
3. **Already expressible?** No (`lib.rs:5961-5965`).
4. **Fit.** It reuses the bare-field form (`response.<field>`) and its `text` encoding for the error
   side, and the literal of a standard refusal. One spelling per concept. Siblings: `response.<field>`
   already has it, and `envelope` stays JSON by definition.
5. **Second adopter.** A proxy-fronted service whose gateway answers `429 Too many requests` as
   text.
6. **Cost.** New body form and admitted encodings in `ess-http/1`, absent by default, so no
   version. This follows the precedent of `encoding: text` in 0.77.0. No new code: `ESS-HTTP-024` and
   `ESS-HTTP-025` are widened.
7. **Considered.** (a) Change nothing. The OpenAPI document then states the wrong body. (b) Chosen:
   the bare field and literal text. (c) A free `content_type:` on an answer. This was rejected: an
   open set of media types the servers could not write (§ "Still refused").

## Decisions

Accept, redesigned (the reporter asked to state plain-text errors; the form reuses the bare-field
shape). No open question.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (encoding check 5940-5970, body forms)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs` (unit b)
- cited: `docs/design/http-presentation-binding.md`
- Size: unit a about 400 lines, unit b about 350.

## Depends on

Nothing. Unit b after unit a. It shares `lib.rs` body checks with `ac4-success-status-on-refusal`,
so the two land one after the other.

## Source

A downstream application, third report, finding 4 (ESS 0.98.0).
