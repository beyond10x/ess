---
format: aep.planning-md/3
id: architecture-decision-record:a-command-guard-reads-a-component-setting
kind: architecture-decision-record
status: accepted
title: A command guard reads a component setting, and the suite is synthesized once per configuration
summary: Proposes a setting.<name> guard root compared with literals and one synthesized suite per declared configuration; secret settings stay unreadable (row 43).
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:13Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: accepted 2026-10-23 by the operator, after reviewing all 15 proposed ADRs in full: "I accept all of them" (see this record's transitions).

## Context

A component declares its start-time settings, `settings:` on a component (`ess/1` key, released in
0.24.0; `story:component-declares-its-settings`, `crates/specify/ess-domain/src/component.rs:160`,
`:174-200`). The only reader is the deployment projection, which turns each setting into a runtime
slot (`crates/generate/ess-deployment/src/runtime.rs:910`). Nothing in validation, synthesis, the
interpreter or the generated behaviours reads a setting.

A guard cannot name one. Guard roots resolve to a quantifier binder, a reference hop, `input`,
`caller` (since `ess/16`, `docs/design/caller-values.md`) and the step's own fields; any other root
is refused as `unobservable_fact` (`crates/specify/ess-domain/src/expression.rs:917-994`, refusal
at `:978-985`).

Adopter repro (adopter B, tracker row 43, ESS 0.68.0): a demo component `demo.conn.gateway`
declares `settings: [{name: open_mode, type: Boolean, required: true}]`. Its `Connect` command
refuses an unauthenticated connection unless the deployment started with `open_mode: true`. The
adopter cannot write `when: {all: [cred == None, setting.open_mode == false]}`, so it keeps one
specification per deployment configuration, each with its own suite, and the two copies agree by
inspection.

Related, undecided: `story:outcome-decided-by-environment` (draft, design-first) asks how a refusal
decided by environmental fact is stated, and names "deployment allow lists, rate limits and feature
flags" as the same shape. A start-time setting is the part of that question whose value is known
before the first request, so it can be decided here without settling per-request facts such as a
caller's network address.

## Options

| | what it is | cost |
|---|---|---|
| A | **A `setting.<name>` guard root, with one suite per configuration.** `ess/N` lets a command outcome's guard compare `setting.<name>` with a literal, where `<name>` is a non-secret setting of the component whose `accepts:` names the command (`component.rs:133`) and is of Boolean, enum or Integer type. A suite carries a `configuration:` (the settings the target must be started with). Synthesis partitions on the literals the guards compare each setting with, as it does for `caller` attributes, and writes one configuration per partition. | New `ess/N` (a guard root; below it the read stays `unobservable_fact` naming `ess/N`); new suite format (`configuration:`, refused by older readers); runners start or restart the target once per configuration; generated Rust and Go pass the settings into behaviour. |
| B | **The setting becomes input bound from configuration.** The command gains an input field, and the binding fills it from a declared setting rather than from the request (option (b) of `story:outcome-decided-by-environment`). | Binding format key; the model reads an input that no caller can send, so the generated API, OpenAPI and the request decoder each need a "not on the wire" rule; synthesis still has to start the target per value, so the suite needs a configuration anyway. |
| C | **Keep it out.** Document one specification per configuration. | Nothing to build; the adopter's duplicated specifications and suites stay, and nothing checks that they differ only in the guarded branches. |

## Recommendation

A. A setting is already a typed, named fact of the component, fixed for the lifetime of a process.
Reading it in a guard is the `caller.` precedent with a different source. One suite per
configuration states, in data, which configuration a scenario was meant for, and that is what the
adopter's duplicated specifications express by hand. B puts the same burden on the runner and also
gives the model an input that is not input.

Boundaries: a `secret: true` setting stays unreadable (`conflicting_declaration`). Only comparisons
with literals are allowed, so synthesis has finitely many partitions. A command accepted by no
component, or by components that declare the setting at different types, is refused by name. A
guard over a setting never selects among per-request facts. That question stays with
`story:outcome-decided-by-environment`.

## Effect

- **Format:** next `ess/N` (`ess/31` if this is accepted before the release carrying `ess/31` ships, since that format is still open): the
  `setting.` root in `when:`, `when_subject:` and `when_related:` predicates. A model without it
  keeps its IR bytes. A field or input named `setting` keeps its meaning, as `caller` does
  (`expression.rs:969-975`). The suite format gains `configuration:` per suite or per scenario
  group, and an older reader refuses it.
- **Synthesis:** for each setting a guard reads, the literals partition its domain; synthesis
  enumerates the partitions the guards distinguish (a product bounded as `caller` assignments are)
  and writes each scenario under the configuration that selects its branch. An unreachable branch
  under every configuration is reported as today.
- **Generated code:** Rust and Go behaviours take a typed `Settings` struct derived from the
  component's `settings:`, built once at start-up; the deployment projection is unchanged; Entity
  Runtime refuses the root by name until it carries settings; runners receive the configuration
  and start the target with it.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
