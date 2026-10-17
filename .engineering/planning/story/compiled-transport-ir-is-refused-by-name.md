---
format: aep.planning-md/3
id: story:compiled-transport-ir-is-refused-by-name
kind: story
status: draft
title: Transport consumers refuse the compiled transport IR by name
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

`ess generate --kind asyncapi --transport` and `ess generate client --transport`, handed a compiled
`ess-transport-ir/*` document, refuse by name and point at the authored transport document they
read; the format reference and the transport design no longer say the compiled IR is read by them.

## Evidence

Reported by an adopter on ess 0.59.0 and reproduced on 0.60.0: both commands exit 1 with a raw
serde error (`unknown field `format`, expected one of `type`, ...`). The compiled IR is write-only
by design (`docs/design/parameterized-event-channel-addresses.md`), but
`website/docs/reference/formats.md` says it is "Read by `ess generate --kind asyncapi --transport`"
and `docs/design/event-transport-binding.md` says clients read `ess-transport-ir/1`. The reader is
`crates/edge/ess-cli/src/transport.rs`.

## Acceptance

- `asyncapi_given_transport_ir_refuses_naming_authored_format`
- `client_given_transport_ir_refuses_naming_authored_format`
- `formats_reference_names_no_ir_reader`
