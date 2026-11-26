---
format: aep.planning-md/3
id: story:layers-from-the-domain-reference-graph
kind: story
status: draft
title: ess reports a system's domain reference graph, its cycles and its layers
summary: One command derives the domain dependency graph from declared relations and stored identifiers, names each cycle with its fields, and orders the domains into layers.
relations:
- decomposes: epic:adopter-gaps-domain-language
- serves: vision:O2
revision: 1
---
## Outcome

`ess` reports the layers of a system from the references between its domains: one command reads a
validated specification and prints the domain dependency graph, its cycles with the entity fields
that close each one, and an order of layers in which every domain names only domains of a lower
layer. With a declared layer assignment it refuses a reference that points upward.

## Evidence

A downstream report from the `weft` specification (a self-deployed compute cloud), 2026-11-26.
Thirteen domains were written in parallel by separate agents, each referring to the others only
through identifiers in a shared domain. To find the system's layers, the operator validated all
fifteen files as one system (`weft v1 — 15 file(s), valid`, 0.24 s) and reconstructed the graph by
hand from the `ESS-ENTITY-019` (`implied_relation`) warnings: 133 warnings, each one an edge
`<domain>.<Entity>.<field> -> <domain>.<Entity>`. Grouped by domain they gave:

- one domain with no outgoing reference (`cluster`), the base;
- five edges against the direction, each closing a cycle, for example
  `accounts.Export.endpoint_id -> formations.Endpoint` while `formations` names `accounts`;
- two domains whose entities fall on two sides of a cut (`resources`: `ResourceClass` and `Device`
  name only `cluster`; `Volume`, `Bucket` and `Claim` name `accounts`);
- nine layers once the five edges are removed.

The parsing was a `sed` over warning text. Nothing in `ess` states this graph:
`ess specify graph` renders the interaction graph (commands, events, emissions), not the
references between domains, and the warnings stop once the relations are declared, so the graph
disappears exactly when the model is complete.

## What the command owes

1. A domain graph from every reference between domains: declared `relations:`, and stored
   identifier fields whose type identifies an entity of another domain (what `ESS-ENTITY-019`
   detects today). Each edge carries its count and the entity fields behind it.
2. The strongly connected components of that graph; for each cycle, the edges in it with their
   entity fields, so an author can see which field to move.
3. A layer order: the condensation of the graph sorted topologically, each domain at the length of
   its longest path to a domain with no outgoing edge.
4. Optionally, an entity-level view that shows a domain whose entities fall into two layers, the
   signal that the domain should split.
5. Output as text, JSON and DOT/Mermaid, deterministic bytes for one input.
6. Later: a declared layer per domain (or per directory of domain files), and validation that
   refuses an upward reference by name, naming the field.

## Acceptance

On a fixture of three domains `a`, `b`, `c` where `b` stores an `a` identifier, `c` stores a `b`
identifier and `a` stores a `c` identifier, the command prints one cycle `a -> c -> b -> a` with
the three fields; after the field in `a` is removed it prints layers `a`, `b`, `c` in that order.
