---
title: Roadmap
sidebar_position: 3
description: The evidence-driven direction for new ESS model kinds and adapters.
status: planned
lede: ESS grows from concrete adapter needs rather than a universal meta-model.
---

# Roadmap

The [maturity outlook](./outlook.md) proposes the reliability, compatibility, naming, and
verification work needed to make the existing toolchain dependable. It describes improvement
criteria rather than a release commitment.

Near-term work extends the existing typed model where a real importer or projector establishes the
semantics: richer service and interface coverage, then repository, organization, team, role, and
ownership structures as their first adapters require them. CLI presentation already has its typed
binding (`ess-cli/1`, `ess specify cli` and `ess generate cli`).

The standing constraints are:

1. no generic facet registry or arbitrary JSON property bag;
2. no `ess-ir/2` without a persisted compatibility reason;
3. no merge of `EssIr` and `InfraIr` without a use case that removes duplication or enables a
   required comparison;
4. no importer guesses, no projector applies, and every adapter declares coverage;
5. every new persisted field is assessed against old-reader behavior.

## Not scheduled

- **Obligations as trackable records.** A synthesis obligation is an entry in the generated plan
  (`PLAN.md`, `plan.json`), not a record a task can own and evidence can close. Nothing blocks it;
  it has not been scheduled.
- **A conformance target in another process.** `ess verify conform run` reaches only the targets
  built into `ess`, and the `ess-conformance` crate is not published. A target speaking a wire
  protocol, or a published runner crate, would let any implementation be held to a suite without
  the generated Go or TypeScript package. It has not been scheduled
  ([A suite at every level of a test pyramid](../concepts/test-pyramid.md#commit-the-generated-runner)).

Accepted designs live in the repository’s `docs/design/` tree. This site documents shipped behavior
rather than publishing proposed work as product fact.
