---
title: "0.70 — Named rows beside a row set, state wire spellings, and replays that finish"
description: >
  0.70.0 adds source format `ess/32`: one command may guard on rows its input names beside a
  row-set guard, a struct may declare no fields, and a lifecycle state may declare its wire
  spelling.
slug: named-rows-beside-a-row-set
tags: [release, ess]
date: 2026-10-26T20:00:00+01:00
release_tag: "0.70.0"
---

0.70.0 is a feature release. It adds the source format `ess/32` and the suite formats
`ess-conformance/58` and `/59`. A model below `ess/32` keeps its meaning, its IR bytes and its
compiled digest. The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists
every change since 0.63.0.

{/* truncate */}

## Rows the input names, beside a row set

From `ess/32` one command may guard on rows its input names, `when_related: {via:
input.<field>, …}`, beside a row-set guard, `when_related: {entity, where, …}`. "Connect two ports
that exist and face the right way, unless a wire already leaves the source" is one command. A
missing named row answers first, then every refusal over a present named row or a row set in
declaration order, then the accepting branches
([#554](https://github.com/beyond10x/ess/issues/554)).

## More in `ess/32`

- A struct may declare `fields: []`, open or closed
  ([#649](https://github.com/beyond10x/ess/issues/649)).
- A lifecycle state may declare its wire spelling, `states: [{name: Indexing, wire: indexing}]`,
  and two variants of one enum spelled alike on the wire are refused
  ([#639](https://github.com/beyond10x/ess/issues/639)).

## Conformance and generated code

- `ess verify conform synthesize` no longer stops with `InvalidReplay` at `ess/26` and later.
- A view may publish a `Decimal`, compared by value; such a suite is `ess-conformance/58` or `/59`.
- A create-or-update command beside a related-row refusal gets a scenario for every branch
  ([#641](https://github.com/beyond10x/ess/issues/641)).
- The synthesized Rust server decodes published events and returns the events a call published.
- A variant's `code:` names its generated identifier, and a dotted label such as `call.started`
  synthesizes ([#662](https://github.com/beyond10x/ess/issues/662)).
- `ess generate types --check` compares what it would write with `--out`
  ([#654](https://github.com/beyond10x/ess/issues/654)).
