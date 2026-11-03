---
format: aep.planning-md/3
id: story:view-asserts-field-set-from-optional-input
kind: story
status: implemented
title: A synthesized view expectation asserts a field set through a declared conversion
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: inferred
  path: docs/design/binding-mapping-bounded-accessor.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:23Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:23Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:49Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance`, synthesis in `src/synthesize.rs` — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs:12064` (`if field.conversion.is_some() { continue; }` in `settled`) — cited. The story's `:9361` and `:9354-9422` are stale.
- **Symbols:** `settled` (`synthesize.rs:12056`), `shown` (`:12756`), `view_expectations` (`:9677`) — cited
- **Also likely:** `crates/verify/ess-conformance/tests/synthesis.rs`; its abstention guard `OVERWRITTEN_BY_A_LITERAL` (`:3502`) and the test at `:4122` pin today's skip — inferred
- **Documents:** `docs/design/binding-mapping-bounded-accessor.md:259-263` says "Conformance must not assert that an opaque converted value equals its raw source." The Acceptance overrides that rule for the case it names, so the page is amended first — inferred
- **Confidence:** medium — the location is read; landing it depends on a value-preservation decision the spec does not make (`ResolvedConversion` holds only `from`, `to`, `because`, `ir.rs:592-599`) — cited
- **Would collide with:** any unit touching `synthesize.rs` `settled`, `shown` or the view-row claim path, and the conversion abstention in `tests/synthesis.rs` — inferred
- **Safety fact:** `settled` is the one place a `sets:` value becomes a row claim, and it skips conversions (`synthesize.rs:12064`; `minted_row` skips them at `:11934`) — cited
- **Already in the tree:** variant 1 (no conversion) already asserts a plain input set through the `InputField` arm at `synthesize.rs:12085`; variant 2 and the sets-retarget kill are not on main — cited
- **Not established:** whether `Optional<String> -> String` hands the value on unchanged; which `RefusalCause` the note naming the field and the conversion reuses (`synthesize.rs:815`).
- **Size:** medium, about 150 hand-written lines; no format version (newest `FormatVersion::V33`, `system.rs:501`) unless the rule is format-gated — inferred
- **Targeted run:** ess-conformance `--test synthesis`, `synthesis_suite_bytes_table`, `binding_input_through_conversion`, plus one new target — inferred

## Delivered

W2 (D408), 2026-11-03: a view row field set from an `Optional<T>` input through the declared conversion is asserted at the value sent; a conversion synthesis cannot compute leaves the field out and is named in a note. Unit e0236a014f; merged a23b3e9e59.
