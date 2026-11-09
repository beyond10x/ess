---
format: aep.planning-md/3
id: story:projections-use-a-unions-declared-content-key
kind: story
status: draft
title: Projections use a union's declared content key
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#637
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
revision: 2
---
## Need

A union's declared `content:` key, admitted by
`story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`, must reach the IR and every
projection. Today the content key is computed from the tag alone
(`crates/generate/ess-gen/src/types.rs:1083-1090`) and re-exported as
`ess_gen::schema::union_content_key(tag)` (`crates/generate/ess-gen/src/schema.rs:76-78`). Its
callers take only the tag.

Minimal model: `example.g.Kind`, `tag: type`, `content: config`,
`variants: {text: , web_crawl: example.g.CrawlConfig}`, as an input of one command.

## Acceptance

- `ess specify compile` writes `content: config` on the union in the IR. A union without
  `content:` keeps its IR bytes.
- `ess generate` JSON Schema and OpenAPI for the minimal model require `type` and `config` on the
  `web_crawl` branch and refuse `value` (`additionalProperties: false`). The `text` branch requires
  `type` only.
- `union_content_key` takes the resolved union, not the tag. Every caller in `ess-gen`, `ess-synth`
  and `ess-conformance` compiles against the new signature, with no behaviour change for an
  undeclared key.
- `ess verify diff` reports a changed or added `content:` as a breaking change of the union, for
  callers and readers.

## Scope

- cited: crates/generate/ess-gen/src/types.rs
- cited: crates/generate/ess-gen/src/schema.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-diff/src/delta.rs
- inferred: crates/generate/ess-gen/tests/union_content_key.rs
confidence: medium

## Fit review

1. **Need.** A wire shape `{type, config}` must be projected as declared. Reproduction: triage
   run, `content-key`, exit 1 (unknown field).
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No (parent story, question 3).
4. **Fit.** Extends the single place that decides the content key. It is the one decision the
   schema module documents as one, so a projection never spells it twice
   (`crates/generate/ess-gen/src/schema.rs:80-86`).
5. **Second adopter.** A webhook envelope `{event, data}`.
6. **Cost.** An IR field, left out when absent. A new `ess verify diff` kind for the content key.
   No change for models without the key.
7. **Alternatives.** (a) Read the key from the source in each projection: rejected, because two
   spellings of one decision would drift. (b) Carry it on the resolved union and pass the union to
   the one function: chosen.

## Decisions

accept as proposed, as the parent story. This story carries the IR, the projections and the diff.

## Source

github:beyond10x/ess#637
