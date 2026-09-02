---
format: aep.planning-md/1
id: decision-blocker:relation-vocabulary
kind: decision-blocker
status: open
title: Nobody has decided the relation vocabulary
summary: Kinds, cardinality form, direction and on_delete are open; default for silence is written in the body.
owner: ess
relations:
- blocks: story:relations-in-the-domain-model
revision: 1
---
# Decision blocker: Nobody has decided the relation vocabulary

## What is undecided

1. Kinds: `owns` and `references` only, or a single kind with an `ownership: true` flag.
2. Cardinality: `one | many` on the target side only, or `min`/`max` on both sides.
3. Direction: declared on the owner, on the child, or on both with a consistency check.
4. Whether a relation may name `on_delete` (`cascade | restrict | detach`) at all, given that ESS describes and the runtime decides.

## Who decides

The ESS owner, on `story:relations-design-page`.

## Default if nobody answers

Two kinds (`owns`, `references`); cardinality `one | many` on the target side; declared on the source entity only, with `owns` targets checked for a single owner; no `on_delete` in v0.1.

## What it withholds

`story:relations-in-the-domain-model` cannot start without it: the parser's shape is the decision.
