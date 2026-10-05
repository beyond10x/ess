# Served codec names are allocated, not flattened

beyond10x/ess#415.

## What was missing

A served surface names every codec and handler after its declaration's whole qualified name,
flattened into one identifier: `renewal.input.AB` becomes `decode_command_renewal_input_a_b` in
Rust and `decodeCommandRenewalInputAB` in Go. Flattening drops the separators, so two distinct
declarations can land on one identifier. `renewal.input.AB` and `renewal.input.A_B` do, and so do
`renewal.lease.AB` and `renewal.lease_a.B` in two bounded contexts.

The Rust target refused such a model with `wire-collision`, naming
`decode_command_…`, `encode_command_…` and `encode_outcome_…`. The Go target had no such check: it
emitted both functions under one name, and the server did not compile. An authored
`naming: { code: … }` did not help, because it renames the payload types and seam methods, not
the codecs.

## The rule

`crates/generate/ess-synth/src/codec_names.rs` allocates one **stem** per declaration. Every codec
and served handler of that declaration is its prefix plus the stem. Both layouts hold the
allocation (`rust::layout::Layout::codec`, `go::layout::Layout::codec`). Every emitter and the Rust
feasibility inventory read it from there, so no second derivation of a name exists.

1. **Candidate.** The target's flattening of the whole qualified name: `value_ident(type_fragment)`
   in Rust (`renewal_input_a_b`), `type_fragment` in Go (`RenewalInputAB`). This is the name every
   codec had before.
2. **Family.** Names are allocated within a family whose functions share prefixes:
   - declared types: `encode_`/`decode_`
   - events: `encode_event_`
   - errors: `encode_error_`
   - commands and views together. Their handlers share `serve_`/`run_` in Rust and `serve` in Go,
     beside `encode_command_`, `decode_command_`, `encode_outcome_`, `encode_response_`,
     `encode_view_` and `decode_params_` (Go: `decodeCommand`, `answer`, `encodeResponse`,
     `encodeView`, `decodeParams`, `params`).
3. **No collision, no change.** A candidate that no other declaration of its family flattens to is
   the stem, unchanged. Every model without such a collision keeps every byte it had.
4. **Collision.** Declarations sharing a candidate are ordered by the bytes of their qualified
   names. The first keeps the candidate. Each later one takes the smallest numeric suffix `_2`,
   `_3`, … whose result is not the candidate of any declaration in the family and was not already
   allocated. All candidates are reserved before any suffix is chosen.

The allocation depends only on the set of declarations. Declaration order, file order and the
order of a component's `accepts` list do not change a byte.

| model | Rust stem | Go stem |
|---|---|---|
| `renewal.input.AB` | `renewal_input_a_b` | `RenewalInputAB` |
| `renewal.input.A_B` | `renewal_input_a_b_2` | `RenewalInputAB_2` |
| `renewal.input.AB2` | `renewal_input_a_b2` | `RenewalInputAB2` |

`'B' < '_'` in byte order, so `AB` keeps the candidate. `AB2` flattens to a different candidate
and is untouched. Neither current flattening ever puts `_` before a digit, so today no declaration
occupies a suffix. The occupancy check stays anyway, so the rule does not depend on that holding
for every future flattening.

## What stays refused

A collision between families is outside the rule. One example is a declared type
`command.command.x.Y`, whose `encode_command_command_x_y` is also the input encoder of the command
`command.x.Y`. The Rust target still refuses it with `wire-collision`, the defensive refusal for
anything the allocator does not resolve. Field-level `wire-collision` (two fields mapping to one
JSON member) is unchanged.

## Embedded and direct targets

A model with no `reached_by: network` component has no served surface. Rust inventories no wire
functions for it and emits no codecs, so this rule changes nothing there: the model is admitted
exactly as before. Go refuses a component port when two accepted commands derive one seam method
name and `naming.code` does not separate them. That boundary is untouched and has nothing to do
with codecs.

## Conformance

`crates/generate/ess-synth/tests/served_codec_names.rs` covers:

- both flattenings in both Rust layouts, Web and Go, with each command's names declared once
- an actual served call per command, in both Rust layouts and in Go, answering with that command's
  own outcome and event
- declaration order changing no byte
- the embedded boundary
- the cross-family refusal
