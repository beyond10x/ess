---
format: aep.planning-md/3
id: story:the-published-schema-admits-the-name-aliases-the-parser-reads
kind: story
status: implemented
title: The published schema admits the name aliases the parser reads
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:45Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:45Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-01T15:20:31Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
# Story: the published schema admits the name aliases the parser reads

## Outcome

A document the parser reads is not refused by the published JSON Schema for spelling a name through
an alias: a binding written with `id:` and a component written with `component:` validate against
`schemas/generated/ess.schema.json` as they validate with `ess specify validate`.

## Why

Found by adversary pass 1 on wave correctness-1 unit charset (2026-09-28,
`review-result:adversary-c1-charset-pass-1`, finding 4): the `id` alias for `RawBindingSpec.name`
(`crates/specify/ess-domain/src/binding.rs:206`) and the `component` alias for a component name are
not published, so the schema refuses 21 committed documents that write `id:` for a binding and 29
that write `component:` for a component (e.g.
`crates/generate/ess-synth/tests/fixtures/binding-selection.yaml`), and the name charset the wave
published never applies to that spelling. Pre-existing on `b5c53e8a1`; not a charset defect.

Also reported by the same pass, pre-existing and unrelated to names: 7 committed documents use a
string `Ranking` that the schema types as an object — include it here or split it.

## Acceptance

- Every committed YAML document that `RawSpecFile::parse` reads validates against the published
  schema (the adversary's test `every_committed_document_the_parser_reads_is_admitted_at_the_new_charsets`
  generalised from 5 locations to the whole document), or the exception is named with its reason.
- The alias spellings carry the same charset pattern as the canonical key.
- The case `a_binding_written_with_its_id_alias_is_admitted_by_the_schema` in
  `crates/specify/ess-domain/tests/adversary_charset_pass1.rs`, which asserts today's refusal, is
  flipped to assert admission.

## Scope

- `crates/specify/ess-domain/src/binding.rs` (`RawBindingSpec`), `component.rs` — cited
- `schemas/generated/ess.schema.json` (regenerated) — cited
- `crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json` (re-pin) — inferred

## Delivered

D350, 2026-11-01: unit 4198b2241 (merged 46cef82a4), fix round 813ee8838 with the adversary's cases (merged e979f19b7). `name::aliased_name_schema` publishes a binding's and a component's name as `oneOf` of `name:` and its alias, each with the name charset, `additionalProperties: false`; `ess.schema.json` regenerated and the three `wire:RawSpecFile#/definitions` consumer-coverage rows re-pinned. `published_name_aliases.rs` reads every committed YAML file (`git ls-files`, 612 parsed documents); the only refusals are the named exceptions (the `Ranking` key, now `story:the-published-schema-reads-a-ranking-key-as-a-string`, and one empty file). Adversary pass 1: F1 (corpus walk) fixed, F2 (the `additionalProperties` mutant) guarded by its case 2.
