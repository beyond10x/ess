---
title: Record a physical realization
description: Bind local, loopback, or hosted entrypoints to one exact ESS without changing EssIr.
---

# Record a physical realization

An ESS says what a system means. A realization says which immutable artifacts implement that exact
ESS and how people or programs enter them. Keeping those documents separate lets a local TUI and a
hosted workbench implement the same semantics without turning interface or deployment choices into
the system model.

Create an `ess-realization/1` document that locks the compiled ESS identity, selects its components
and actors, assigns every selected component to one immutable implementation artifact, and declares
one or more typed entrypoints. Each entrypoint names:

- an interaction shape: `observe`, `invoke`, or `agent_loop`;
- an attachment boundary: `in_process`, `loopback`, or `network`;
- public availability and support posture independently of network reach;
- ESS semantic references resolved by that surface;
- exactly one argv or HTTP(S) URL invocation; and
- typed runtime requirements. Environment and credential requirements contain names and
  descriptions, never secret values.

Validate and compile it against the specification:

```shell-session
$ ess realization validate --path realization.yaml --spec specification/
my-realization — 4 entrypoint(s), valid
$ ess realization compile --path realization.yaml --spec specification/ --out realization.ir.json
my-realization — 4 entrypoint(s), compiled to realization.ir.json
```

Generate the user-facing comparison from the same resolved data:

```shell-session
$ ess realization generate \
    --path realization.yaml \
    --spec specification/ \
    --out docs/running-modes.md
docs/running-modes.md — generated
$ ess realization generate \
    --path realization.yaml \
    --spec specification/ \
    --out docs/running-modes.md \
    --check
docs/running-modes.md — current
```

Compilation refuses stale ESS digests, unresolved semantic references, actors or components outside
the selected realization, incomplete implementation coverage, multiple recommended entrypoints,
undeclared `${NAME}` placeholders, and invocations that appear to contain inline credentials.
