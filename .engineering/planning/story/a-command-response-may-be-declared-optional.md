---
format: aep.planning-md/3
id: story:a-command-response-may-be-declared-optional
kind: story
status: draft
title: A command outcome may declare that its response can be absent
summary: 'returns: is a Boolean, so a response the protocol lets the server omit cannot be declared; returns: optional is a reader error (adopter item 8, 2026-10-22); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 8 of the second batch, relayed by the orchestrator; adopter names kept out). A
protocol lets the server omit an acknowledgement: the command may answer with its response or with none. `returns:` takes only
a Boolean, so the contract must claim the response is always there (and fail an implementation that omits it) or never there.

Needed: a response that may be absent, declared on the outcome, with conformance accepting both shapes (present and conforming
to the declared fields, or absent).

## Reproduction

Brand-free fixture, `format: ess/29`:

```yaml
domain: demo.server
commands:
  - name: demo.server.Connect
    response: [{name: successful, type: Boolean}]
    outcomes:
      - {name: acknowledged, returns: optional}
```

`ess specify validate --path spec.yaml`, exit 1 on 0.68.0 and 0.67.0, identical:

```text
spec.yaml was refused:
  - spec.yaml: invalid type: string "optional", expected a boolean
```

The refusal is a reader error with no code and no hint naming a supported spelling.

## Where it lives

- `crates/specify/ess-domain/src/command.rs:2560` and :6867: `returns` is a `bool` on the outcome. Synthesis and the response
  check were not traced.
- Related, not the same: `story:response-field-presence-conditioned-on-another-field` (presence of one field, not of the whole
  response).

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0. Gap. Does not block 0.68.0.
