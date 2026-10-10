---
format: aep.planning-md/3
id: story:view-asserts-field-set-from-optional-input
kind: story
status: draft
title: A synthesized view expectation asserts a field set through a declared conversion
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
---
## Outcome

A synthesized scenario asserts a view field whose value an outcome sets through a declared
conversion (`Optional<String>` to `String`) whenever the scenario sends a value for it, as it
already does for a field set without a conversion.

## Evidence

An adopter on ess 0.56.0 reported view fields set from Optional inputs going unasserted, so
sets-retarget mutants on them survived (12 in the adopter's model). A minimal reproducer narrows
the trigger to the declared conversion, not the Optional input as such:

- Variant 1: one input `site: Optional<String>`, refusal `when: {site: {exists: false}}`,
  `created` sets `site: input.site` into a String field the view publishes; no conversion is
  needed. `expect_view` asserts `"site": {"kind": "literal", "value": "site"}`.
- Variant 2 (the fixture below): two Optional inputs and a refusal on `any` of their absences;
  validation then requires the declared `Optional<String>` to `String` conversion. The
  `CreateConfig/outcome/created` `expect_view` asserts only `config_id` and `state`; `site` and
  `client_id` are dropped although `execute_command` sends literals for both.

Run: `ess verify conform synthesize --path <dir>`, then read the `CreateConfig/outcome/created`
scenario's `expect_view`.

On `main` (read, not run): `settled`
(`crates/verify/ess-conformance/src/synthesize.rs:9354-9422`) skips a field when
`field.conversion.is_some()` (`:9361`), which matches variant 2.

Not `story:refusal-witness-sends-echoed-optional-input`: there the input is never sent; here it is
sent and not asserted.

## Fixture

`system.yaml`:

```yaml
format: ess/23
system: repro
version: v1
domains:
  - repro.config
```

`components.yaml`:

```yaml
components:
  - component: config-service
    summary: owns the configuration
    owns:
      domains: [repro.config]
    accepts:
      commands: [repro.config.CreateConfig]
    publishes:
      events: [repro.config.ConfigCreated]
    reached_by: network
```

`domains/config.yaml`:

```yaml
domain: repro.config

types:
  - name: repro.config.ConfigId
    kind: newtype
    of: String

entities:
  - name: repro.config.Config
    identity: {name: config_id, type: repro.config.ConfigId}
    fields:
      - {name: site, type: String}
      - {name: client_id, type: String}
    lifecycle: {initial: Configured, states: [Configured], terminal: [Configured]}

actors:
  - name: repro.config.Admin
    may: [repro.config.CreateConfig]

errors:
  - name: repro.config.MissingField
    summary: site is absent
    fields: []

conversions:
  - from: Optional<String>
    to: String
    because: missing-field refuses an absent value before created runs

commands:
  # `site` is Optional on the wire; the first refusal answers when it is absent, so `created` only
  # runs with a value, and stores it into a String field the view publishes.
  - name: repro.config.CreateConfig
    input:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: Optional<String>}
      - {name: client_id, type: Optional<String>}
    outcomes:
      - name: missing-field
        when:
          any: [{site: {exists: false}}, {client_id: {exists: false}}]
        error: repro.config.MissingField
      - name: created
        creates: repro.config.Config
        instance: config_id
        sets: {site: input.site, client_id: input.client_id}
        emits: [repro.config.ConfigCreated]
        payload:
          repro.config.ConfigCreated: {config_id: input.config_id}

events:
  - name: repro.config.ConfigCreated
    fields:
      - {name: config_id, type: repro.config.ConfigId}

views:
  - name: repro.config.Configs
    source: repro.config.Config
    consistency: read_your_writes
    fields:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: String}
      - {name: client_id, type: String}
      - {name: state, type: repro.config.Config.State}
```

## Acceptance

- A synthesis test over the fixture: the `created` scenario's `expect_view` row carries `site`
  and `client_id` equal to the values sent; variant 1 keeps asserting `site`.
- A sets-retarget mutant on `site` is killed by a synthesized scenario.
- A conversion whose result synthesis cannot compute still leaves the field out, with a note
  naming the field and the conversion.
