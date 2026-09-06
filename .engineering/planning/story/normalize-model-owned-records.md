---
format: aep.planning-md/1
id: story:normalize-model-owned-records
kind: story
status: draft
title: Normalize model-owned records without duplicating their schemas
relations:
- derived_from: story:source-pinned-data-normalization
revision: 1
---
## Evidence

Normalization Plan currently selects roots only from qualified schema Bundles.
ESS model projections carry x-ess-provenance, x-ess-name and x-ess-kind, with
additional model wire/nominal/invariant annotations where applicable. Importing
an ESS-generated model schema through generate schema import-document refuses
those keywords before a normalization source can be formed. This was verified
against the current CLI, not inferred from a type declaration.

The structural realization Plan::from_model already understands model-owned
identity. The normalization boundary must reuse that authority rather than
requiring an adopter to hand-author a second schema or strip annotations from a
generated file. Both would separate the conversion contract from its typed home.

## Outcome

Normalize between model-owned records and qualified source contracts with exact
source identity, preserving model names, wire semantics and explicit obligations.

## Acceptance

- Establish the persisted root/source identity and compatibility consequences
  in the binding design before adding a normalization reader or CLI source option.
- Consume a checked model projection through an explicit typed API; do not accept
  unverified x-ess claims as compiler-minted authority or blanket-ignore unknown
  schema annotations.
- Preserve source model identity and selected root closure in canonical recipes,
  standalone target reports and every generated target's retained source inputs.
- Preserve model newtypes, wire mappings and invariant obligations. Structural
  validity alone is not proof that a model invariant was executed.
- Generate and run a source-to-model or model-to-model normalization without a
  hand-maintained duplicate schema, and refuse stale/mismatched source identity.
- Existing bundle-owned normalization recipes and their canonical bytes retain
  their declared compatibility guarantees.

## Relation

This closes the model-owned input/output boundary required by
source-pinned-data-normalization and uses the existing structural model realization
capability. It does not introduce a generic facet registry or merge unrelated IRs.
