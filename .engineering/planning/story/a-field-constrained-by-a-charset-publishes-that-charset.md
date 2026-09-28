---
format: aep.planning-md/3
id: story:a-field-constrained-by-a-charset-publishes-that-charset
kind: story
status: implemented
title: A field constrained by a charset publishes that charset
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/selection.rs
- confidence: cited
  path: crates/specify/ess-domain/src/topology.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/adversary_charset_pass1.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/adversary_charset_pass2.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/component_settings.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/published_charsets.rs
- confidence: cited
  path: docs/design/cli-schema-metadata-accounting.md
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T10:14:57Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-09-28T10:14:57Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-09-28T15:58:12Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"test_result":1,"review_outcome":4,"verification":1}}}
---
# A field constrained by a charset publishes that charset

`schemas/generated/ess.schema.json` is a projection of `RawSpecFile` and is what a schema-aware
editor validates an authored document against. Three raw fields share the `CliName` charset
`^[a-z][a-z0-9]*(-[a-z0-9]+)*$`, and only one publishes it:

| field | published pattern |
|---|---|
| `RawComponentSetting.name` | `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` |
| `RawCommandLineSurface.binary` | none — bare `type: string` |
| `RawCommandGroup.name` | none — bare `type: string` |

So an editor accepts a `binary` or a command-group `name` that the parser then refuses. The author
learns at `ess validate` what the schema could have told them while they typed.

Confirmed pre-existing by the wave-25 unit-3 adversary: both bare fields are `pub … : String` at
`e5a97603` (`git show e5a97603:crates/specify/ess-domain/src/component.rs:236,253`), untouched by
the settings work.

## The rule, stated as a class

*A raw field constrained by a charset publishes that charset, or the schema accepts what the parser
refuses.*

`crates/specify/ess-domain/src/types.rs:1448` already does this for `Field::PATTERN`, and wave 25
did it for `RawComponentSetting.name` with a case asserting the published literal and the constant
agree. Two members of the class are left.

**The class is not bounded beyond `component.rs`.** Nobody has audited the other raw types, so the
first thing this story needs is the enumeration — every `pub … : String` in a `Raw*` struct whose
parser applies a charset — rather than fixing the two the adversary happened to name.

## Why it was not done in wave 25

Tightening a published schema is a contract change: a document an older editor accepted becomes one
it refuses. That is a decision with a version story attached, and unit 3 was not asked to make it.
Whoever takes this says whether it is a `ess/N` change or an additive tightening that no existing
valid document notices.

## Acceptance

Every raw field whose parser applies a charset publishes that charset in
`schemas/generated/ess.schema.json`, with one case per field asserting the published literal and the
parser's constant agree. The enumeration of the class is in the source, not in this story.

## Scope

Re-derived 2026-09-28 by `story-scoper`; corrected at close of wave correctness-1 from the implementor's reports (merge `505758a89`).

- **Landed in:** `crates/specify/ess-domain/src/component.rs`, `binding.rs`, `topology.rs`, `entity.rs` (`Transition::NAME_PATTERN`), `selection.rs`; `schemas/generated/ess.schema.json` (regenerated) — cited
- **Also:** `crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json` (3 shape hashes re-pinned) and `docs/design/cli-schema-metadata-accounting.md` (dated review) — cited; the scoper had these as out of scope, the first implementor report wrongly said no pin changed
- **Tests:** `published_charsets.rs` (9), `adversary_charset_pass1.rs` (5), `adversary_charset_pass2.rs` (3) — cited
- **Correction:** the inferred `command.rs` / `outcome_group.rs` members were already typed newtypes publishing their pattern (was: inferred); `Transition.name` and selection names were missing from the scope and are in the class
- **Left out:** `RawRelated` (a pattern refuses nothing below `ess/16`); name aliases → `story:the-published-schema-admits-the-name-aliases-the-parser-reads`
