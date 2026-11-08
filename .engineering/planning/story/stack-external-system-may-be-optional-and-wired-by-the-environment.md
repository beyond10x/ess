---
format: aep.planning-md/3
id: story:stack-external-system-may-be-optional-and-wired-by-the-environment
kind: story
status: draft
title: A stack's external system may be optional, and its bound configuration reaches the containers that use it
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 1
---

## Outcome

A stack can declare that an external system it uses is optional, an environment may then leave
it unbound, and the containers that would use it get no wiring for it. Configuration an
environment binds on an external system (beyond its endpoint URLs) reaches the containers that
declare they use it, so it is stated once, on the system, rather than repeated per release.

## Evidence

An adopter on 0.90.0 deploying a WebSocket push service could run with or without two supporting
systems (a message bus, and a key-value store reached through a failover coordinator, which needs
several coordinator addresses and a primary name), and hand-wrote the chart and its values to
switch them. On current main:

- `ess-stack/1` names them only as external systems: contract, endpoints, authorities and
  configuration coordinates, and a `managed` flag
  (`crates/generate/ess-deployment/src/stack.rs:39-59`). The deliver guide says external systems
  are used "but not deployed" by the stack (`website/docs/guides/deliver/resolve-a-stack.md:82-84`),
  while the field's own comment reads "whether this stack is expected to deploy the provider"
  (`stack.rs:47-49`); nothing in `ess-deployment` reads `managed`.
- Every external system must be bound by every environment, with every endpoint, authority and
  configuration coordinate (`crates/generate/ess-deployment/src/environment.rs:336-375`); there is
  no optional system.
- Every runtime endpoint slot must resolve (`environment.rs:417-443`); an endpoint slot cannot be
  optional (`crates/generate/ess-deployment/src/runtime.rs:69-82`).
- An external system's bound configuration is checked (`environment.rs:366-375`) but never reaches
  a release: `DeploymentRelease` carries only the release's own `config` and derived endpoints
  (`environment.rs:461-475`, endpoint derivation `environment.rs:513-521`). A container that needs
  the primary name must declare it again as its own config slot, bound again per release.

Minimal reproduction (neutral): a stack that uses a bus only in some environments.

```yaml
# ess-stack/1
external_systems:
  - system: bus
    contract: message-bus
    endpoints: [client]
    config: [subject_prefix]
# ess-environment/1 for an environment without the bus: refused today,
# "external system bus has no environment binding"
external_systems: []
```

## Fit review

1. **The need, apart from the syntax.** Two domain facts: (a) a system can be deployed with or
   without a supporting system, and the deployment must then carry no wiring for it rather than a
   placeholder; (b) configuration that belongs to the supporting system (a subject prefix, a
   primary name, a list of coordinator addresses) is a property of that system's binding, not of
   each release that uses it. The requester (theirs): "optional dependencies of a stack with the
   configuration that wires them".
2. **Class: gap.** (a) cannot be stated (`environment.rs:336-344` refuses an unbound system);
   (b) is a gap in delivery: bound coordinates are checked and dropped (`environment.rs:366-375`,
   `environment.rs:461-475`). The meaning of `managed: true` is undocumented, which is close to a
   defect (`stack.rs:47-49` against `resolve-a-stack.md:82-84`).
3. **Can it already be expressed?** Only falsely: binding the absent system to a placeholder URL
   satisfies `environment.rs:346-354` and states a system that does not exist; repeating the
   coordinate as a release config slot works, at the cost of stating one fact twice, which no check
   holds equal. Neither is an idiom the skill admits.
4. **Fit.** Deployment intent; no cell of grid R, O or G2 (`docs/design/language-grids.md`).
   Vocabulary: `story:runtime-secret-may-be-optional` already proposes `optional: true` on a
   runtime slot, and an endpoint slot already names an external system and one of its endpoints
   (`runtime.rs:69-82`); a config slot naming a system and one of its coordinates is the same shape.
   Siblings: optional applies to secret, endpoint and config slots alike. Targets: the stack
   resolver, `deployment compile`, the Helm projection (an optional slot's environment variable is
   rendered inside a condition), and `deployment reconcile`'s values writer. Which document states
   the optionality, and what `managed` means, is not decided:
   `decision-blocker:how-a-stack-carries-an-optional-or-managed-dependency`.
5. **A second adopter.** A service that uses a cache in production and runs without one in a
   development environment; a service that publishes audit events to a broker only where an
   audit pipeline exists.
6. **Cost.** At least one format change, which one depends on the decision: optional slots and a
   system-sourced config slot move `ess-runtime` (joining `/3` if it is still unreleased); a stack
   `optional:` key moves `ess-stack/1` to `/2`; delivering bound coordinates to releases changes the
   compiled `ess-deployment/1` document unless it travels inside the existing per-release config map.
   New diagnostics: an optional system bound partially; a required slot targeting an optional
   system.
7. **Considered.** See the decision-blocker: (A) change nothing, (B) optionality derived from
   runtime slots, (C) optionality stated on the stack plus optional runtime slots, (D) the stack
   deploys managed providers as chart dependencies.

## Decisions

- **Defer.** The need is real, but where optionality is stated and whether ESS deploys a managed
  provider is a design decision nobody has made. Recorded as
  `decision-blocker:how-a-stack-carries-an-optional-or-managed-dependency`, `blocks:` this story.
- Not decided here and not to be invented by an implementor: a list type for coordinator addresses.
  A coordinate stays a string; how a process splits it is the process's concern.

## Acceptance

Written for the blocker's recommended option (C); revised if another option is chosen.

- A stack external system with `optional: true` may be absent from an environment; an environment
  that binds it binds all of it, or is refused naming the missing part.
- A runtime endpoint, config or secret slot targeting an optional system must itself be optional,
  or `stack resolve` refuses it by name; an optional slot whose system is unbound renders no
  environment variable in the chart and no entry in the reconciled values.
- A config slot naming `system:` and a coordinate receives that system's bound value at
  `deployment compile`; a slot naming a coordinate the system does not declare is refused by name.
- `managed: true` has one documented meaning, or is refused by name, as the blocker decides.
- An unset setting changes no bytes: a stack, runtime and environment without the new keys compile
  to byte-identical locks, deployments and charts.
- Each touched format's consequence is decided and recorded, with an old-reader compatibility test.

## Scope (inferred)

- `crates/generate/ess-deployment/src/stack.rs`, `environment.rs` (`compile_deployment`,
  `derive_endpoint`, `render_container`), `runtime.rs` (slot types).
- `crates/edge/ess-cli/src/recovery/mod.rs` (values document).
- `website/docs/guides/deliver/resolve-a-stack.md`, `website/docs/reference/formats.md`, a design
  page under `docs/design/` for the decision.
- Overlaps `story:runtime-secret-may-be-optional` on the `optional` key.
