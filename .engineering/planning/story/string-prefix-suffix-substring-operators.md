---
format: aep.planning-md/3
id: story:string-prefix-suffix-substring-operators
kind: story
status: implemented
title: String guards can test a prefix, a suffix or a substring
refs:
- provider: github
  reference: beyond10x/ess#95
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:41:01Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:41:27Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:47Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A routing rule such as "caller number starts with +44" is a checked ESS guard, not `UNMAPPED:`.

## Why

GitHub issue beyond10x/ess#95. `starts_with`, `ends_with` and `contains` on `String` are refused
today (`unknown operator "starts_with"`). entity-core 0.23.0 has `contains` over strings but no
`starts_with`/`ends_with` (`entity-runtime` `crates/entity-core/src/definition.rs:1170`), so the
`ess-entity-runtime` lowering needs an entity-runtime change first.

## Acceptance

- A design page `docs/design/string-predicate-operators.md` (the repository's rule: a construct is a
  design page before it is code) fixing byte-wise, case-sensitive semantics and the witness table
  from the issue.
- Map form only on `String` and newtypes of `String`: `{starts_with: L}`, `{ends_with: L}`,
  `{contains: L}`, negated via `not:`. Refused on other types with a stable code.
- Type checking, finite coverage (open domain), the three evaluator lanes and browser admission,
  Go/Rust selection emitters, entity-runtime lowering, and a suite-format version gate.
- Witness per the issue's table; red-first cases for each operator's satisfying and refuting
  candidate, in every lane.
- `ess` pins the entity-core revision that adds the two operators by `rev`.

## Out of Scope

Regular expressions, SQL `LIKE`, case-insensitive matching.
