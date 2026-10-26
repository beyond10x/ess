---
format: aep.planning-md/3
id: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
kind: story
status: draft
title: A guard reads a union tag and a union names its content key
summary: 'Adopter report G2: A guard cannot read a union''s tag, and a union cannot name its content key (ess#637).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#637
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

A guard cannot select on which variant a union input holds (ESS-COMMAND-002, `AnyOf` does not admit an aggregate), and a union cannot name its adjacent content key: the projection fixes it as `value` (`content` when the tag is `value`, `crates/generate/ess-gen/src/types.rs:195`), and `content:` is an unknown field. Adopter report G2, ess 0.67.0 and 0.68.0. Depends on the union-encoding design page owed by `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`.

## Acceptance

- `a_guard_selects_a_union_input_by_its_variant`: `when: {kind: {any_of: [web_crawl]}}` over a union input validates at the new format; a name that is not a variant is refused by name.
- `the_finite_prover_covers_branches_over_every_variant`: branches over a partition of the variants are proved exhaustive.
- `a_union_names_its_content_key`: `content: config` beside `tag: type` validates; JSON Schema, OpenAPI and the generated Rust, Go and TypeScript types read and write `{type, config}`.
- `a_content_key_equal_to_the_tag_is_refused`: `content:` equal to `tag:` is refused; a union without `content:` keeps its bytes.
- `synthesis_witnesses_each_variant_branch`: one scenario per guarded variant branch, passing on the interpreted target.
