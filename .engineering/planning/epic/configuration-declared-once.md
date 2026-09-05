---
format: aep.planning-md/1
id: epic:configuration-declared-once
kind: epic
status: draft
title: Configuration is declared once and delivered many ways
revision: 1
---
# Epic: configuration is declared once and delivered many ways

## Outcome

A component states what it needs configured — typed names, and whether the model may hold the
value — and every way a value arrives is derived from that one declaration. The epic is finished
when `ess-runtime/1`'s slots are a projection rather than a hand-authored document, and when adding
a second delivery form for a value costs a projector rather than a parallel set of hand-written
types.

## Why

Configuration is described twice here and typed once, in the wrong layer.

- `ess/1` says **nothing** about configuration. No component, domain or entity construct types a
  setting.
- `ess-realization/1` carries a name and a one-line summary per prerequisite over a closed kind
  vocabulary — `EnvironmentVariable`, `Filesystem`, `Credential` — and states plainly that "values
  and credential material have no field in this shape"
  (`crates/specify/ess-realization/src/lib.rs:298-327`).
- `ess-runtime/1` has the only typed model: `ConfigSlot`, `SecretSlot`, `EndpointSlot`
  (`crates/generate/ess-deployment/src/runtime.rs:18-67`). Every one carries
  `pub environment: String`. A slot *is* an environment variable bound into a container — a
  deployment-plane fact, hand-authored, tied to nothing typed in the specification.

So the layer that types configuration is the one that cannot be reused, and the layer a second
delivery form would read from does not exist.

## What it costs a consumer

`beyond10x/connectors` is the measured case. `crates/connectors-config/` is **3033 lines**
(`personal.rs` 1498, `hosted.rs` 1460) of hand-written configuration types read by `toml = "0.8"`,
with eighteen-plus config structs in `personal.rs` alone. Its own backlog carries
`story:one-connection-config-shape`, filed because two structurally identical config types are
drifting: "a field added to one silently diverges the other". Nothing in ESS could have caught that,
because nothing in ESS knew either type was configuration.

## The line this epic must hold

`secret:` is semantic; `environment:` is not. A secret is a value the model is **forbidden** to
represent, which is a fact about the system of the same kind `reached_by` states. Which variable
carries it, which TOML key, which Vault path — all derived per target, never chosen in the semantic
layer. ESS has declined a transport DSL twice and this epic does not introduce one.

## Not in this epic

- Any adapter. A TOML reader, env, keyring and Vault are what the epic makes cheap; building one is
  separate work behind it.
- Rewriting a consumer's existing configuration types.
- A defaults or templating language. A literal is a literal; anything computed belongs to a
  projection.

## Provenance

Raised by the `connectors` operator on 2026-09-04 while adopting the command-line surface
(`epic:command-line-surface`), as: *would it make sense to model configuration, external values and
value providers, so we could later have adapters like TOML config?* The three-layer reading above
was measured against this repository at `0.14.0`, not inferred.
