---
title: Record a physical realization
description: Bind local, loopback, or hosted entrypoints to one exact ESS without changing EssIr.
---

# Record a physical realization

An ESS says what a system means. A realization says which immutable artifacts implement that exact
ESS and describes how people or programs enter them. Physical invocation, attachment, availability
and support metadata belong in this separate document. Semantic reach (`reached_by`) and CLI
grammar (`cli`) already belong to the authored/compiled ESS model and affect its identity; runtime
workload mapping belongs to deployment. See
[Logical, interface and delivery owners](../concepts/ess.md#logical-interface-and-delivery-owners)
for those boundaries.

Create an `ess-realization/1` document that locks the compiled ESS identity, selects its components
and actors, assigns every selected component to one immutable implementation artifact, and declares
one or more typed entrypoints. Each entrypoint names:

- an interaction shape: `observe`, `invoke`, or `agent_loop`;
- an attachment boundary: `in_process`, `loopback`, or `network`;
- public availability and support posture independently of network reach;
- ESS semantic references resolved by that surface;
- exactly one argv or HTTP(S) URL invocation; and
- typed entrypoint prerequisites (`RuntimeRequirement`). Environment and credential requirements contain names and
  descriptions, never secret values.

A realization requires unique entrypoint IDs and exactly one primary. Multiple entrypoints may
name the same implementation, but these records do not establish simultaneous semantic CLI/HTTP
reach or synthesis for one component. Compilation resolves references and validates invocation
syntax; it does not check attachment/invocation against semantic reach, establish actor
authorization, verify prerequisite availability, or launch an interface.

The current `realization_digest` covers specification identity, synthesis identity and
implementations, excluding entrypoints. An entrypoint-only edit can leave it unchanged. This is
distinct from the compiled-model digest, which includes nondefault reach and present CLI layout;
neither digest proves execution or deployment.

Validate and compile it against the specification:

```shell-session
$ ess specify realization validate --path realization.yaml --spec specification/
my-realization — 4 entrypoint(s), valid
$ ess specify realization compile --path realization.yaml --spec specification/ --out realization.ir.json
my-realization — 4 entrypoint(s), compiled to realization.ir.json
```

Generate the user-facing comparison from the same resolved data:

```shell-session
$ ess specify realization generate \
    --path realization.yaml \
    --spec specification/ \
    --out docs/running-modes.md
docs/running-modes.md — generated
$ ess specify realization generate \
    --path realization.yaml \
    --spec specification/ \
    --out docs/running-modes.md \
    --check
docs/running-modes.md — current
```

Compilation refuses stale ESS digests, unresolved semantic references, actors or components outside
the selected realization, incomplete implementation coverage, multiple recommended entrypoints,
undeclared `${NAME}` placeholders, and invocations that appear to contain inline credentials.
