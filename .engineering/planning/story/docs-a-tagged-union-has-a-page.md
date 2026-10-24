---
format: aep.planning-md/3
id: story:docs-a-tagged-union-has-a-page
kind: story
status: active
title: The tagged union type has a documentation page
summary: 'kind: union with tag: and variants: is not shown on any website/docs page; an adopter modelled a sum type as a struct of optional fields.'
tags:
- docs
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T00:38:01Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T00:38:01Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

The tagged union type (`kind: union` with `tag:` and `variants:`, `crates/specify/ess-domain/src/types.rs:1178`) has no page in `website/docs` that shows its declaration. The pages mention unions only in passing (`guides/specify/fields-and-invariants.md` names "one `Union` variant", `guides/generate-artifacts.md` names union semantics of the projections); a grep of `website/docs` for `kind: union` finds nothing. A downstream adopter modelled a sum type as a struct with optional fields because of that, which loses the one-variant-at-a-time rule and the variant's typed payload.

## Where it lives

- Declaration: `crates/specify/ess-domain/src/types.rs:1178` (`TypeBody::Union { tag, variants }`); a unit variant (`None` payload, ess/22) is the tag alone on the wire (`docs/design/union-unit-variants.md`).
- Example in a test: `crates/generate/ess-synth/tests/json_web.rs:39-43` (`demo.docs.Either`, `tag: kind`, variants `raw: Json`, `text: String`).
- The page goes under `website/docs/guides/specify/`, beside `fields-and-invariants.md`, and is linked from it.

## Acceptance (provisional)

- A `website/docs` page shows a `kind: union` declaration with `tag:` and `variants:`, including a unit variant, in a snippet that `ess specify validate` accepts (checked by the docs snippet test where one covers that directory).
- The page states the always-tagged rule and its reason: an untagged union does not round-trip, because two variants of the same shape cannot be told apart.
- The page shows the generated wire shape of each variant (where the tag and the payload sit, and a unit variant as the tag alone), taken from the generated JSON Schema of the example, not written by hand.
- The page says when to choose a union over a struct of optional fields.
