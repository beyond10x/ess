---
format: aep.planning-md/3
id: story:the-generated-rust-server-refuses-what-its-openapi-refuses
kind: story
status: implemented
title: The generated Rust server refuses what its OpenAPI refuses
summary: The Rust server admits any string as a Timestamp and ignores undeclared JSON members that its served OpenAPI refuses (#707)
tags:
- ess-batch-4
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#707
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/json.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding_rust.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:49Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:49Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-03T02:29:22Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
<!-- Starting point for a `story` artifact, seeded by `aep artifact new story <name>`.
     No frontmatter here on purpose: the `---` block is written by the CLI from the id, kind, status
     and relations you gave it, and a second copy in this file would be the one that went stale.
     Delete the italic guidance as you fill each section. -->

# Story: <name>

## Outcome

*What is true for whom once this has shipped, in one sentence. If it names a component rather than a
person, it is a task — say what changes for someone.*

## Context

*Why this is worth doing now, and what it depends on. Link the epic or specification it comes from
rather than restating it; the `derived_from` relation already carries the edge.*

## Acceptance

- Over the toy specification of #707, the generated Rust server answers 400 at decode, before any
  store write, for each of the following. The answer names the path of the offending member.
  - A `Timestamp` member that is not an RFC 3339 date-time, on every command that takes one, nested
    `Optional`, `List` and struct positions included.
  - A member that a JSON object's schema does not declare, at the top level of a command input and
    inside a nested struct or list element.
- Every body the served OpenAPI admits is still admitted. A test sends the issue's valid bodies and
  their stored rows read back unchanged.
- The answer for each rejected body is the same as what the served OpenAPI predicts. A test checks
  the generated server against its own `openapi.json` for these bodies, so the two cannot drift
  apart again.
- The adopter replay at the release shows whether any adopter's generated server or suite sends such
  a body.
  - If none does, the refusal is the default and CHANGELOG lists it under Changed.
  - Otherwise, the unit ships the refusal behind a binding opt-in (D135) and records the adopter as
    an open question.

## Out of Scope

*What a reasonable reader would expect to be included and is not — the boundary that stops this
story quietly becoming an epic.*

## Ambiguities

*Each gap this story found and did not close, classified. `inferable` — the answer is already
written down, so give the `path:line` or the artifact id that settles it.
`requires-stakeholder-input` — nobody here can decide it, so name who does, and raise that entry as
a `decision-blocker` with a `blocks` edge to this story, or it is a sentence somebody improvises
later.*

## Open Questions

*Anything still undecided belongs in `## Ambiguities` above, classified and with its citation or its
decider. Keep this section for a question that is neither — an unowned question is a story that
stalls without anybody noticing.*

## Problem

beyond10x/ess#707, a defect reported 2026-10-29 by a downstream session on ESS 0.77.0. Re-checked with
`ess 0.77.0` on the toy specification in the issue: `ess generate synthesize --target rust --layout
crate --http`, built and run, with requests sent through curl.

The generated Rust server decodes JSON request bodies more loosely than the OpenAPI document it serves:

- (a) A `Timestamp` (OpenAPI `format: date-time`) is read with `json::text_at` only. Any string is
  admitted, `""` and `"yesterday at noon"` included: 202, the row is stored, and the view answers the
  text back.
- (b) An object the OpenAPI declares `additionalProperties: false` ignores members it does not
  declare. This holds in a list element (`unti` beside `unit`: stored with `unit` absent) and at the
  top level of a command input. `request: {encoding: json, unknown: refuse}` on the route changes
  nothing. The only unknown-key check is `http::flat_object`, which serves `form` and `query`.

A misspelt required member is refused, but only because the required member is then missing.
Where a bad timestamp sorts in an ordered view was not observed: `order_by` over a `Timestamp` is an
obligation in 0.77.0.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-synth/src/rust/` (the Rust target emitter) — cited.
- **Files:** `crates/generate/ess-synth/src/rust/wire.rs`: the Timestamp arm reads `json::text_at` (`wire.rs:973`); the struct, command, event and record decoders (`wire.rs:242`, `:376`, `:519`, `:891`) read declared members only and never refuse an undeclared one — cited.
- **Also likely:** `crates/generate/ess-synth/src/rust/json.rs` (a validating RFC 3339 reader and a closed-members reader beside `text_at`, `json.rs:717`); `crates/generate/ess-synth/src/rust/http/bound.rs` (the JSON body path `json_body`, `bound.rs:354`; `with_path` merges path fields after parse, so a top-level closure check must know the carried names); `crates/generate/ess-synth/tests/http_binding_rust.rs` for the tests — inferred.
- **Symbols:** `json::text_at` (`json.rs:717`, `wire.rs:973`, `web/bridge.rs:719`); `http::flat_object` (`http/bound.rs:263`, form and query only); `ResolvedCommand.undeclared_fields` (`crates/specify/ess-compiler/src/ir.rs:2059`); `undeclared_fields_ignored` (`ir.rs:3287`) — cited.
- **Documents:** none; a `CHANGELOG.md` Changed line — inferred.
- **Generated:** `crates/generate/ess-synth/tests/served_codec_names.rs:927` pins FNV digests per fixture, target and layout, so any emitted-byte change moves that table — cited.
- **Confidence:** medium — the defect sites are cited; the JSON top-level closure rule is inferred and nothing was run.
- **Would collide with:** any unit editing the Rust decoders in `ess-synth/src/rust/wire.rs` or `json.rs` (also read by `web/bridge.rs`); `story:a-json-command-body-refuses-an-unknown-member` is the same surface and the same unit (GEN1) — cited.
- **Safety fact:** the new refusals add nothing the served OpenAPI does not already state: `Timestamp` is `format: date-time` (`crates/generate/ess-gen/src/types.rs:667`) and a struct is `additionalProperties: false` unless `undeclared_fields: ignored` (`types.rs:43`) — cited.
- **Not established:** whether a JSON route with `unknown: ignore` must admit extra members (the served JSON body is a `$ref` to the command input, `crates/generate/ess-gen/src/openapi.rs:1129-1138`; the route's `unknown:` is consulted only for form bodies); whether the Go server shares the Timestamp gap (`textAt`, `crates/generate/ess-synth/src/go/http.rs:2285`) — inferred.
- **Size:** medium, about 200 emitter lines and 150 test lines; no spec format change (`SUPPORTED_FORMATS` ends at 33, `crates/specify/ess-domain/src/system.rs:53-56`) — cited.

## Delivered

W1 (D394), 2026-11-02: generated Rust codecs refuse what the served OpenAPI refuses, which covers the beyond10x/ess#707 classes (a malformed Timestamp, an undeclared member). Unit 56b6441ef, with review round 1 in 93c9d4155 and the review cases in 9471e396a; merged 12ca705bc and e1aa6bc04. Known limit: an Optional member sent as null is still taken as absent, tracked as backlog draft story:an-optional-member-sent-as-null-is-answered-as-the-openapi-answers-it.
