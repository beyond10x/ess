---
format: aep.planning-md/3
id: story:compiled-transport-ir-is-refused-by-name
kind: story
status: draft
title: Transport consumers refuse the compiled transport IR by name
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/transport.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/client.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/transport.rs
- confidence: cited
  path: docs/design/event-transport-binding.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 4
---
## Outcome

`ess generate --kind asyncapi --transport` and `ess generate client --transport`, given a compiled
`ess-transport-ir/*` document, refuse it by name and point at the authored transport document they
read. The format reference and the transport design no longer say that these commands read the
compiled IR.

## Problem

The compiled IR is write-only by design (`docs/design/parameterized-event-channel-addresses.md`).
Both commands read the authored `type: ess-transport/N` document (`crates/edge/ess-cli/src/transport.rs`).
Handed the IR, they fail with a raw serde error. Meanwhile `website/docs/reference/formats.md:205` says
`ess-transport-ir/1` is "Read by `ess generate --kind asyncapi --transport`", and
`docs/design/event-transport-binding.md:109` says "Generated clients, which read `ess-transport-ir/1`".
An adopter reported this on 0.59.0. It still reproduces on 0.92.0.

Reproduction (`model/system.yaml`: `format: ess/20`, one event `metering.items.UsageRecorded`, one
component `producer` publishing it; `transport.yaml`: `type: ess-transport/1`, one NATS broker, one
channel, one stream):

```text
$ ess specify transport compile --path transport.yaml --spec model --out transport.json
transport.yaml — 1 channel(s), 1 stream(s), compiled to transport.json        # "format": "ess-transport-ir/1"
$ ess generate --path model --kind asyncapi --transport transport.json --out out
error: reading transport.json as transport JSON: unknown field `format`, expected one of `type`, `specification`, `brokers`, `channels`, `streams` at line 2 column 10   # exit 1
$ ess generate client --path model --component producer --transport transport.json --target go --package usage --out out
error: reading transport.json as transport JSON: unknown field `format`, expected one of ...   # exit 1
$ ess generate --path model --kind asyncapi --transport transport.yaml --out out
1 artifact(s), written to out                                                   # exit 0
```

## Acceptance

- `ess generate --kind asyncapi --transport <ir.json>` exits 1. stderr names `ess-transport-ir/1`
  (or `/2`) as compiled output that this command does not read, and names the authored
  `ess-transport/N` document as its input. No serde field list appears. Test:
  `asyncapi_given_transport_ir_refuses_naming_authored_format` in `crates/edge/ess-cli/tests/transport.rs`.
- `ess generate client --transport <ir.json>` does the same. Test:
  `client_given_transport_ir_refuses_naming_authored_format` in `crates/edge/ess-cli/tests/client.rs`.
- The authored document keeps working: both commands on `transport.yaml` produce byte-identical output.
- `formats.md` names no reader for `ess-transport-ir/1` and `/2`, and `event-transport-binding.md`
  no longer says clients read the IR. Test: `formats_reference_names_no_ir_reader` (a docs check
  in the existing docs test or `cargo xtask docs`).

## Scope

- cited: `crates/edge/ess-cli/src/transport.rs`
- cited: `website/docs/reference/formats.md`
- cited: `docs/design/event-transport-binding.md`
- cited: `crates/edge/ess-cli/tests/transport.rs`
- cited: `crates/edge/ess-cli/tests/client.rs`

confidence: high

## Related

`story:a-foreign-scenario-type-is-refused-by-format-before-its-keys` (implemented) fixed the same
shape for scenario documents and can be the model for this fix.
