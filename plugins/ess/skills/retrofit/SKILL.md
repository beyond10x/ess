---
name: retrofit
description: >-
  Derive an ESS specification for a system that already exists and has none — from its OpenAPI contract, its observed Kubernetes deployment, or its code — then validate it and hand it to conformance. Use when the user asks to retrofit, adopt, reverse-engineer or "get a spec out of" an existing service or repository, when a codebase has an OpenAPI document or a cluster but no `system.yaml` or `ess-inputs.yaml`, or when a specification must describe behaviour that is already shipped rather than behaviour still to be built. Not for a domain drafted from nothing, which is the `specify` skill; not for raising coverage of a suite that already exists, which is the `coverage` skill.
---

# Retrofitting a specification onto an existing system

A retrofit describes what the system **does**, not what it should do. Every declaration in the
draft is read from something in the repository — a contract, an observation, a line of code — and
cites it. What you cannot read is an `UNMAPPED:` marker, never a plausible value.

## 1. Inventory the sources, in order of authority

| Source | Command | What it gives you |
|---|---|---|
| an OpenAPI document | `aep plan reverse openapi --domain <domain> <openapi> --out <file>` | a draft `ess/1` domain: entities and fields from the schemas, with the decisions it could not take listed |
| the same OpenAPI document | `ess infra import openapi --path <openapi> --format yaml` | the operation and interface-type semantics ESS's adapter supports, with its coverage gaps |
| a running Kubernetes deployment | `ess infra import kubernetes …` (see `--help`) | a sanitized observation of what is deployed; the credential edge is explicit |
| code only | read it | handlers, persistence types, state enums, validation — each a citation |

Use every source that exists. Where two disagree, the draft records both and marks the field
`UNMAPPED:`; the disagreement is a finding, not something to resolve by choosing.

`aep` is optional. Without it, write the domain by hand from the schemas, in the shape the `specify`
skill shows.

## 2. Draft, one domain at a time

Start from the smallest document that validates (the `specify` skill has it), then add one entity at
a time, running after each:

```console
ess specify validate --path <specification>
```

For each entity, cite where it came from beside it:

```yaml
  - name: billing.invoice.Invoice
    # read from: openapi.yaml components.schemas.Invoice; src/invoice/model.rs:14
```

Retrofit-specific rules:

- **Lifecycles are read, not designed.** Take states from an enum, a status column or the
  transitions the handlers perform. A state the code never enters is not declared. A transition
  whose trigger you cannot find is `UNMAPPED:`.
- **Commands are the operations that exist.** One command per mutating operation or handler. Do not
  add the command the system should have.
- **Relations need an owner decision.** `owns` against `references` is decided by what a delete
  does in the code today. No delete path found: `UNMAPPED:`.
- **Views follow reads.** One view per read operation clients use; its fields are the fields the
  response carries.

## 3. Prove the draft describes the system

A validated draft only says the specification is coherent. It says nothing about the system until a
suite runs against it:

```console
ess verify conform synthesize --path <specification> --out <suite.json>
```

Then follow the `coverage` skill: build or pick a target that drives the real implementation,
run the suite, and break one behaviour to prove a named scenario goes red. A retrofit is done when
the suite runs against the real system, not when `validate` exits 0.

## 4. Report

- the specification path and `ess specify validate` output, verbatim
- every `UNMAPPED:` marker, one line each, with what would settle it
- every place two sources disagreed
- the suite counts (`passed`/`skipped`/`failed`) if a run happened, or that none did yet
