---
format: aep.planning-md/3
id: story:id-newtype-event-field-addresses-a-channel
kind: story
status: draft
title: A channel address parameter may come from an id newtype field
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

A parameterized channel address may take its parameter from an event field whose type is a newtype
over `Uuid` (and possibly `Integer`), rendered as one subject token, so per-entity subjects work when
ids are typed newtypes.

## Evidence

Reported by an adopter on ess 0.59.0 and reproduced on 0.60.0: ESS-TRANSPORT-020 refuses
`event.run_id` ending in a `Uuid` newtype ("an address parameter ends in required primitive
`String`", `crates/specify/ess-transport/src/lib.rs`). This is the current design:
`docs/design/parameterized-event-channel-addresses.md` restricts the last segment to `String` and
lists enum/newtype parameters as out of scope. Accepting more types changes what the transport
format admits, so it needs a design change and a format decision first.

## Fit review

Pending: assess with `.agents/skills/assessing-external-requests/SKILL.md` before this story is
proposed. Open: which newtype bases are admitted, how each renders as a token, and whether this is
`ess-transport/3`.

## Acceptance

- `uuid_newtype_parameter_compiles_to_one_token`
- `integer_newtype_parameter_renders_decimal_token`
- `newtype_over_unbounded_string_still_refused`
