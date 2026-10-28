---
format: aep.planning-md/3
id: specification:composition-caller-is-not-a-role-entry
kind: specification
status: draft
title: A caller of a contract command is not a role entry
summary: Decline of adopter report N12 (ess#666) with the idiom, run on the integration build
tags:
- adopter-report
relations:
- decides: story:role-input-drift-tells-a-caller-from-an-implementation
- informed_by: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Statement

A caller of a contract command is not a role entry. A composition compares a contract command's inputs only for the local command that answers it (`stands: implements`), so a caller that sends a subset of the contract's inputs has no entry to fail. Leave the caller out of `roles`. Where the local command decides before the contract command succeeds, use `stands: gates`, which compares no inputs.

Answers adopter report N12 (ess#666). Decision: **decline, with this idiom**. `role_input_drift` on an `implements` entry stays: the answering command must accept every input the contract's caller sends (`docs/design/composition-contract-roles.md`, check 4; approval-record:d133-role-input-drift-keeps-the-rule).

## Run

ESS 0.74.0, on the `contract-role` fixture systems (`demo` and `wire`, under `crates/specify/ess-composition/tests/fixtures/contract-role/`). The local command `demo.gate.Bare` lacks the contract's `channel` and requires a `topic` the contract never sends. The same command is composed twice: once with no role entry, once with an `implements` entry.

Document without the role (exit 0):

```yaml
format: ess-composition/4
composition: demo-wire
services:
  - {key: demo, system: demo, version: v1, source_digest: "8a89cf4df04ad27e9f5e431b5cc5e470fd1f17d16d813f7af61f7b086142684a", component: relay}
  - {key: wire, system: wire, version: v1, source_digest: "7a15d835f8918513c0a1be77f61ba0de8ccca09e090f9b59b27815cc85a28c5e", component: wire-server}
```

```console
$ ess specify compose --path <no-role document> --service demo=crates/specify/ess-composition/tests/fixtures/contract-role/demo --service wire=crates/specify/ess-composition/tests/fixtures/contract-role/wire
demo-wire — 2 exact component surface(s), 0 semantic reference(s), compiled
$ echo $?
0
```

The same document with this added (exit 1):

```yaml
roles:
  - service: demo
    plays: wire
    commands:
      - {local: demo.gate.Bare, stands: implements, contract: wire.server.Subscribe}
```

```console
$ ess specify compose --path <implements document> --service demo=crates/specify/ess-composition/tests/fixtures/contract-role/demo --service wire=crates/specify/ess-composition/tests/fixtures/contract-role/wire
<implements document> was refused:
[role_input_drift] roles[0].commands[0]: `demo` command `demo.gate.Bare` does not implement `wire` command `wire.server.Subscribe`: field `channel`: `wire.server.Subscribe` sends it and `demo.gate.Bare` does not declare it; field `topic`: `demo.gate.Bare` requires it and `wire.server.Subscribe` never sends it
$ echo $?
1
```

So the idiom works: the caller composes with no entry, and the check runs only where an entry says a command answers the contract command.
