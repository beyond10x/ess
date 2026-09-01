---
title: Roadmap
description: The evidence-driven direction for new ESS model kinds and adapters.
---

# Roadmap

ESS grows from concrete adapter needs rather than a universal meta-model.

Near-term work extends the existing typed model where a real importer or projector establishes the
semantics: richer service and interface coverage, then CLI, repository, organization, team, role,
and ownership structures as their first adapters require them.

The standing constraints are:

1. no generic facet registry or arbitrary JSON property bag;
2. no `ess-ir/2` without a persisted compatibility reason;
3. no merge of `EssIr` and `InfraIr` without a use case that removes duplication or enables a
   required comparison;
4. no importer guesses, no projector applies, and every adapter declares coverage;
5. every new persisted field is assessed against old-reader behavior.

Engineering wave records and accepted designs live in the repository’s `docs/` tree. This site
documents shipped behavior rather than publishing proposed work as product fact.
