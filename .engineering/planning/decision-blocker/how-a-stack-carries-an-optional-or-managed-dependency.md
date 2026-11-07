---
format: aep.planning-md/3
id: decision-blocker:how-a-stack-carries-an-optional-or-managed-dependency
kind: decision-blocker
status: cleared
title: Where a stack states that a supporting system is optional, and whether ESS deploys a managed one
relations:
- blocks: story:stack-external-system-may-be-optional-and-wired-by-the-environment
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-11-07T23:12:14Z", actor: "human:timo", revision: 3}
---

# Where a stack states an optional dependency, and what `managed` means

An adopter on 0.90.0 runs a WebSocket push service that can be deployed with or without a message
bus and a key-value store reached through a failover coordinator. `ess-stack/1` can name both only
as external systems (`crates/generate/ess-deployment/src/stack.rs:39-59`), every environment must
bind every one of them in full (`crates/generate/ess-deployment/src/environment.rs:336-375`), and
the configuration bound on them never reaches a release (`environment.rs:461-475`). The field
`managed` reads "whether this stack is expected to deploy the provider" (`stack.rs:47-49`), the
deliver guide says the stack does not deploy external systems
(`website/docs/guides/deliver/resolve-a-stack.md:82-84`), and no code reads the field.

Two questions, one decision: **which document states that a supporting system is optional**, and
**does `managed: true` mean ESS projects the provider's deployment**.

## Options

| option | what it does | cost |
|---|---|---|
| A | Change nothing. Adopters bind an absent system to a placeholder and repeat its configuration as release slots. | No format change. The deployment states a system that does not exist, and one fact is written twice with nothing holding the copies equal; the skill does not admit this as an idiom. |
| B | Optionality derived: runtime slots (secret, endpoint, config) take `optional: true`; an external system every one of whose targeting slots is optional may go unbound. Config slots may name `system:` and a coordinate, filled from the external binding. `managed` documented as declarative only. | `ess-runtime/3` keys (shared with the open `/3` stories if unreleased). `ess-stack/1` unchanged in bytes but changed in meaning: a system it lists as required may go unbound, which the format rule treats as a meaning change without a version. Size M. |
| C | B, plus the stack states it: `optional: true` on an external system in `ess-stack/2`. An optional system is bound in full or not at all; a required slot targeting an optional system is refused at resolve. `managed: true` is refused by name in `/2` (and documented as advisory in `/1`) until a provider projection exists. | `ess-stack/2` beside `/1`, `ess-runtime/3` keys, stack-lock bytes unchanged for `/1` stacks. New diagnostics at resolve and compile. Size M. |
| D | C, plus `managed: true` makes the stack project the provider: a pinned chart dependency (name, version, digest) in the generated chart or a sibling release in the deployment, with its values wired from the binding. | A new provider vocabulary, pinning and digest checks for third-party charts, values ESS cannot validate (it would guess at a foreign chart's schema, against "imports never guess" in `AGENTS.md` § Boundaries), recovery-profile admission of foreign kinds (`crates/edge/ess-cli/src/recovery/model.rs:517-546`). Size L, open-ended. |

## Recommendation

C. The stack is where a reader looks for which systems a deployment needs, so the optionality is
stated there rather than inferred from slots spread across runtimes; that keeps resolve able to
refuse a required slot pointing at an optional system. D is declined for now: deploying third-party
providers is a different product surface, and the need reported (run with or without the
dependency, wired from one binding) is met without it. `managed` gets a stated meaning either way,
because today it says the opposite of the guide.

## What waits

`story:stack-external-system-may-be-optional-and-wired-by-the-environment`. Nothing else: the
`ess-runtime/3` stories (named ports, shutdown, rollout and disruption budget) proceed without it,
and the `optional` key of `story:runtime-secret-may-be-optional` should be spelled the same way
whichever option is chosen.

Decider: operator

## Decision

2026-11-07: option C. Runtime slots may be optional; an external system takes `optional: true`
in a new `ess-stack/2` beside `/1` and is bound in full or not at all; a required slot targeting
an optional system is refused at resolve; `managed: true` is refused by name in `/2`. Option D
(ESS deploys managed providers) is declined for now.
