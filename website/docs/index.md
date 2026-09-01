---
slug: /
title: Executable System Specification
description: Model system intent as validated typed data, then derive deterministic artifacts and conformance checks from it.
---

# Executable System Specification

ESS is a standalone Rust toolchain for describing systems as typed, validated data. A specification
can be compiled into deterministic intermediate representation, inspected as a graph, compared
semantically across revisions, projected into supported concrete formats, and used to generate the
conformance scenarios an implementation must satisfy.

The canonical command is `ess`.

```shell-session
$ ess validate --path examples/billing
$ ess compile --path examples/billing --out target/billing.ir.json
$ ess generate --path examples/billing --kind openapi --out target/openapi
$ ess conform synthesize --path examples/billing --out target/suite
```

## What ESS owns

- typed system, domain, entity, command, event, view, component, binding, and topology semantics;
- deterministic compilation, inspection, graphing, semantic diff, and impact analysis;
- documentation, schema, OpenAPI, and AsyncAPI generation;
- structural synthesis with explicit implementation obligations;
- standalone conformance-suite generation, execution, and reporting;
- OpenAPI and Kubernetes adapters with declared coverage;
- sanitized infrastructure observation, analysis, simulation, drift, and projection.

## What it refuses to claim

An importer never fills a semantic gap by guessing. A projector never applies infrastructure or
mutates a live system. Unsupported constructs remain visible as coverage gaps, unresolved
references, obligations, or refusals. Round-trip guarantees apply only to the subset an adapter
declares.

Start with [the walkthrough](./getting-started.md), read [the model](./concepts/ess.md), or follow
the complete [specification-to-contract example](./examples/specification-to-contracts.md).
