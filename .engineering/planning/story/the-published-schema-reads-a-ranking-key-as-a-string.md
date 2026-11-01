---
format: aep.planning-md/3
id: story:the-published-schema-reads-a-ranking-key-as-a-string
kind: story
status: draft
title: The published schema reads a ranking key as the string the parser reads
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by U4 of wave D350 (out of scope there): the `Ranking` view key carries a struct-level
`#[schemars(with = "String")]` at `crates/specify/ess-domain/src/view.rs:238`, which
schemars_derive 0.8.22 ignores, so the published document schema describes the key as an object
while the parser reads a string. The alias corpus test of D350 names it as one of its two
exceptions (18 documents).

## Acceptance

- The published schema (`schemas/generated/ess.schema.json`) admits every committed document that
  writes a ranking key as the parser reads it, and the `Ranking` exception is removed from
  `crates/specify/ess-domain/tests/published_name_aliases.rs`.
- A red case first shows a committed ranking document the schema refuses today.
