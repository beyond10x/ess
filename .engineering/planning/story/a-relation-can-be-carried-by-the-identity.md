---
format: aep.planning-md/3
id: story:a-relation-can-be-carried-by-the-identity
kind: story
status: active
title: A relation carried by the entity's own identity field is refused (ESS-ENTITY-005)
refs:
- provider: github
  reference: beyond10x/ess#230
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T03:34:15Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T03:34:16Z", actor: "human:timo", revision: 3}
---
## Outcome

A relation whose `via:` names the entity's own identity field validates: an entity keyed by
`user_id` can declare `{name: user, kind: references, target: demo.provisioning.User, cardinality:
one, via: user_id}`, a one-to-one relation keyed by the same id. The identity field is type-checked
against the target's identity type exactly as a declared `via:` field is. Every consumer of relation
`via:` (compiler IR, conformance synthesis ownership arrangement, generated code, OpenAPI, docs,
ess-diff) handles an identity-carried relation; a `references` relation never makes synthesis
arrange the target row as an owner.

## Acceptance

- The issue's model (`via:` naming the identity field, `cardinality: one`) validates with no
  ESS-ENTITY-005 refusal.
- `via:` naming the identity field whose type differs from the target's identity type is refused
  with the same type-mismatch refusal a declared field gets.
- `cardinality: many` via the identity field is refused with a named cause.
- Conformance synthesis over a model with an identity-carried `references` relation does not
  arrange the target row as an owner of the source row.
- Models without the construct keep their bytes: `cargo xtask generate --check` stays clean.
