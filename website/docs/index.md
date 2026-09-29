---
slug: /
title: Executable System Specification
sidebar_position: 1
description: Model system intent as validated typed data, then derive deterministic artifacts and conformance checks from it.
---

# Executable System Specification

ESS is a standalone Rust toolchain for describing systems as typed, validated data. A specification
can be compiled into deterministic intermediate representation, inspected as a graph, compared
semantically across revisions, projected into supported concrete formats, and used to generate the
conformance scenarios an implementation must satisfy.

The command is `ess`. Its first level is four areas:

```shell-session
$ ess specify validate --path spec                                      # is the specification consistent?
$ ess generate --path spec --kind openapi --out generated               # derive a contract from it
$ ess verify conform synthesize --path spec --target typescript --out conformance   # the tests it obliges
$ ess infra import openapi --path api.yaml --out interface.json         # read an existing source
```

## What ESS owns

- typed system, domain, entity, command, event, view, component, binding, and topology semantics;
- deterministic compilation, inspection, graphing, semantic diff, and impact analysis;
- documentation generation, as Markdown (`docs`) or as a browsable HTML site (`site`), and JSON
  Schema, OpenAPI, and AsyncAPI generation;
- structural synthesis with explicit implementation obligations;
- conformance-suite generation, runners in Go and TypeScript, and standalone reports;
- OpenAPI and Kubernetes adapters with declared coverage;
- sanitized infrastructure observation, analysis, simulation, drift, and projection.

## What it refuses to claim

An importer never fills a semantic gap by guessing. A projector never applies infrastructure or
mutates a live system. Unsupported constructs remain visible as coverage gaps, unresolved
references, obligations, or refusals. Round-trip guarantees apply only to the subset an adapter
declares.

ESS generates documentation *from* a typed specification. It does not turn prose or Markdown into a
specification, and it does not host the site it renders.

## Where to go next

| You want to | Read |
|---|---|
| Install `ess`, pin it for a project, and get from an empty directory to a passing conformance run | [Getting started](./getting-started.md) |
| Understand what a specification declares and what is derived from it | [The model](./concepts/ess.md) |
| See one command's source next to every contract generated from it | [The billing example](https://github.com/beyond10x/ess/tree/main/examples/billing) |
| Look up a command, a format version or a predicate | [CLI](./reference/cli.md), [format versions](./reference/spec-versions.md), [predicates](./reference/predicates.md) |
| Know what is supported today and what is not | [Where this stands](./status/where-this-stands.md) |
