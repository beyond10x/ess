---
format: aep.planning-md/3
id: story:value-derived-from-a-path-or-url
kind: story
status: draft
title: A value is declared as derived from a path or URL input
tags:
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A specification declares a value derived from a path or URL input (the file name of a path, the
last segment of a URL, the Git common directory of a checkout) instead of modelling it as an
observed field plus ordered outcomes.

## Evidence

An adopter retrofitting a store-management CLI on ess 0.56.0 had no construct for these values and
modelled each as an observed field with outcomes ordered to cover the cases. The archived
`story:a-field-may-be-derived-rather-than-stored` covered derivations over a row's own fields
(set membership, relation count, a hash) and was not built.

## Acceptance

- A closed list of derivations over a `String` path or URL input (file name, extension, parent,
  last URL segment) validates and is type-checked; anything outside the list is refused by name.
- Values that need the environment (a Git common directory) are declared as observations with a
  named source, not as pure derivations; the design page says which is which.
- Synthesis witnesses each derivation with inputs whose derived values differ; the interpreted
  target computes them.
- A design page under `docs/design/` precedes code; it decides whether this revives the archived
  derived-field story or stays separate.
