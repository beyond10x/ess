---
format: aep.planning-md/3
id: story:a-refused-grant-scenario-runs-on-the-interpreter
kind: story
status: implemented
title: A synthesized grant/denied scenario runs on the interpreted target
summary: 'ESS 0.68.0 synthesizes a grant/denied scenario whose expect_view_unchanged step its own interpreted runner rejects when the accepting branch is accepts: nothing.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T12:33:52Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-26T12:33:52Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-26T13:38:00Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Finding

Reported 2026-10-23 by the ess-ui templates session against 0.68.0; not yet checked against 0.69.0.

- `ess verify conform` on the interpreted target answers `error lamp.shed.CheckWiring/grant/denied` with `ESS-CF-SUITE: no consistent view query preceded expect_view_unchanged` (5 scenarios: 4 passed, 1 error).
- The command's accepting branch is `accepts: nothing`. The runner clears its last view after the refused send (`crates/verify/ess-conformance/src/runner.rs` around 1040-1047, per the report) and then rejects the step (around 2922).
- Either synthesis emits a consistent view query before `expect_view_unchanged` in this scenario, or the runner keeps the view across a refused send.

## Acceptance

- The model below synthesizes a suite that passes on the interpreted target, 5 of 5, and a target that changes a view on the refused grant fails `CheckWiring/grant/denied`.

## Repro model

`system.yaml`:

```yaml
format: ess/29
system: lamp
version: v1

domains:
  - lamp.shed
```

`components.yaml`:

```yaml
components:
  - component: shed-service
    summary: Fits bulbs and checks the wiring.
    owns:
      domains:
        - lamp.shed
    accepts:
      commands:
        - lamp.shed.FitBulb
        - lamp.shed.CheckWiring
    publishes:
      events:
        - lamp.shed.BulbFitted
    reached_by: network
```

`domains/shed.yaml`:

```yaml
domain: lamp.shed

summary: Bulbs a keeper fits, and a check that writes nothing.

types:
  - name: lamp.shed.BulbId
    kind: newtype
    of: String

entities:
  - name: lamp.shed.Bulb
    identity:
      name: bulb_id
      type: lamp.shed.BulbId
    fields:
      - name: watts
        type: Integer
    lifecycle:
      initial: Fitted
      states: [Fitted]
      terminal: [Fitted]

actors:
  - name: lamp.shed.Keeper
    may:
      - lamp.shed.FitBulb
      - lamp.shed.CheckWiring

  - name: lamp.shed.Visitor
    may:
      - lamp.shed.FitBulb

errors:
  - name: lamp.shed.WiringUnsafe
    summary: The wiring is not earthed.
    fields: []
  - name: lamp.shed.BulbTaken
    summary: A bulb with this id is already fitted.
    fields: []

events:
  - name: lamp.shed.BulbFitted
    fields:
      - {name: bulb_id, type: lamp.shed.BulbId}

commands:
  - name: lamp.shed.FitBulb
    input:
      - name: bulb_id
        type: lamp.shed.BulbId
      - name: watts
        type: Integer
    outcomes:
      - name: taken
        existing_instance: true
        error: lamp.shed.BulbTaken
      - name: fitted
        creates: lamp.shed.Bulb
        instance: bulb_id
        emits: [lamp.shed.BulbFitted]
        payload:
          lamp.shed.BulbFitted:
            bulb_id: input.bulb_id
        sets:
          watts: input.watts

  - name: lamp.shed.CheckWiring
    input:
      - name: earthed
        type: Boolean
    outcomes:
      - name: checked
        when: earthed == true
        accepts: nothing
      - name: unsafe
        when: earthed == false
        error: lamp.shed.WiringUnsafe

views:
  - name: lamp.shed.Bulbs
    source: lamp.shed.Bulb
    consistency: read_your_writes
    fields:
      - {name: bulb_id, type: lamp.shed.BulbId}
      - {name: watts, type: Integer}
```
