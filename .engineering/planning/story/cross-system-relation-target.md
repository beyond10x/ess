---
format: aep.planning-md/3
id: story:cross-system-relation-target
kind: story
status: draft
title: A relation may target an entity another system declares
relations:
- serves: vision:O2
revision: 1
---
## Outcome

An entity in one system can declare a relation whose target entity another system declares, so
type packs authored as separate ESS systems (for example engineering and org) can relate their
entities (a merge request's author is a person).

## Acceptance

- The relation's target is written as a reference through an `ess-composition` import (the existing
  cross-system mechanism: service imports and semantic references), not by copying the other
  system's entity.
- Validation refuses a target the imported system does not declare, and a linking field whose type
  differs from the target's identity type, with the existing composition diagnostics or new codes.
- The compiled IR records the target's system and entity, so a downstream generator can emit the
  edge between the two packs.

## Status

Design to settle before implementation: whether the relation lives in the importing system's
entity (with a composition reference) or in a composition document that joins the two systems.
