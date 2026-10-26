---
format: aep.planning-md/3
id: story:a-guard-compares-variants-of-an-ordered-enum
kind: story
status: draft
title: A guard compares variants of an ordered enum
summary: A rule over an ordered scale (a risk floor) is spelled today as a variant-by-variant disjunction inside nested list quantifiers; an enum may declare its order and guards compare by it (adopter report, connectors, 2026-10-26)
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

An adopter (connectors, `connectors.tool_contracts`, ESS 0.69.2) keeps three publication rules in Rust because "ESS has no way to compute over a list": the risk floor (the greatest floor over an operation's effects and consequences must not exceed its declared risk), JSON Schema representability of each operation's input and output, and a profile for every operation.

Reproduced on 0.69.2 at `ess/31` and on the 0.70.0 integration head at `ess/32` (probe: one command, `List<Op>` input, `Op.profile: Optional<Profile>`, `Profile.effects: List<Effect>`, `Profile.risk: Risk`):

- A profile for every operation is expressible today: `when: {exists: {in: input.operations, as: o, that: "missing(o.profile)"}}` validates, synthesizes and passes the interpreter.
- The risk floor is expressible today without a reduction, because "the greatest floor exceeds the risk" is "some effect's floor exceeds the risk": a nested `exists` over `input.operations` and `o.profile.effects` whose body lists, per effect, the risk variants below its floor, `{any: [{all: ["e == secret", {any: ["o.profile.risk == none", "o.profile.risk == low"]}]}, …]}`. It validates, synthesizes 3 scenarios and passes. The cost is the spelling: the floor table is written out as disjunctions of variant equalities, because an enum has no order a guard can compare (`o.profile.risk < high`) and a variant cannot carry a value.
- Schema representability reads a JSON Schema document, which no ESS type models; it stays in code.

## Gap

A guard cannot compare enum variants by a declared order, so a rule stated over an ordered scale (risk, severity, tier) is spelled as a variant-by-variant disjunction. Synthesis witnesses one scenario per branch, not each cell of the table.

## Acceptance

- An enum may declare that its variants are ordered (declaration order), and `<`, `<=`, `>`, `>=` between a fact of that enum and a variant or another fact of it validate from a new format; below it they are refused naming the format.
- Synthesis witnesses an ordered comparison on each side of the boundary, as it does an Integer comparison.
- The connectors risk-floor rule is written as `exists: {in: o.profile.effects, as: e, that: …}` with one ordered comparison per effect, and passes the interpreter.
- A document that orders no enum keeps its IR bytes.
