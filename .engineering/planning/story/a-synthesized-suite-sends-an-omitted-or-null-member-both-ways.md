---
format: aep.planning-md/3
id: story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways
kind: story
status: draft
title: A suite holds an omitted_or_null input to reading both a missing member and null, and a reader may widen a field's presence to omitted_or_null
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/adversary_reader_conformance.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/omitted_or_null.rs
- confidence: cited
  path: docs/design/composition-type-conformance.md
- confidence: cited
  path: docs/design/omitted-or-null-presence.md
- confidence: cited
  path: website/docs/guides/specify/wire-names.md
revision: 12
---
## Outcome

A member declared `presence: omitted_or_null` keeps the promise its published schema makes: a
synthesized suite sends it once left out and once as `null` and expects the same answer, so a
target that refuses either spelling fails. A composition reader that declares `omitted_or_null`
conforms to an imported field declaring any presence, because it rejects none of the values that
field sends. The wire-names guide names the "required on the wire, may be null, also read when
missing" case and points to `omitted_or_null`. No new presence mode, no source format.

## Need

A downstream specification describes a wire catalog in which some `Optional` members are
"required, may be null": a protocol's SDK accepts them missing, and its golden vectors send `null`.
It needs one declaration that decodes both spellings as absent. The requester found only
`null_when_absent` (refuses a missing member) and `omitted_when_absent` (refuses `null`); a second
downstream specification imports the first byte for byte and needs the same.

That value exists since 0.69.0: `presence: omitted_or_null` (`format: ess/31`,
`docs/design/omitted-or-null-presence.md:12-28`, `crates/specify/ess-domain/src/types.rs:532-552`).
The requester most likely missed it because the Go and TypeScript conformance runners name only the
two older values (`crates/verify/ess-conformance/src/go/runtime.go:5254`,
`crates/verify/ess-conformance/src/ts/response.ts:199-200`): a suite never carries the third
(`docs/design/omitted-or-null-presence.md:52-59`).

Reproduction on the released 0.97.1 binary, a generic model of my own: struct `demo.catalog.Entry`
with four `Optional<demo.catalog.Detail>` members (`Detail` a struct), one per presence, used as a
command input and response:

```text
$ ess generate --path model --kind schema --out out
  nulled  (null_when_absent)    anyOf [$ref Detail, null]   required
  omitted (omitted_when_absent) $ref Detail                 not required
  either  (omitted_or_null)     anyOf [$ref Detail, null]   not required
  plain   (none)                $ref Detail                 not required
$ ess generate schema validate-bundle --bundle entry.json --root Entry <instance>
  {"id":"a"}                       "nulled" is a required property
  {..,"omitted":null}              null is not of type "object"
  {..,"either" left out}           1 valid
  {..,"either":null}               1 valid
$ ess verify conform synthesize --path model --out suite.yaml
  execute_command input {"entry": {"id": "entry.id"}}   every Optional member left out, none null;
  `either` carries no presence in the suite
$ ess verify conform author ... (ess-scenario/6, one act per spelling)   0 refusal(s) for all 8
$ ess verify conform run --suite <authored> --target interpreted        1 passed for all 8
```

So the declaration, the published schema and the interpreter already read both spellings. What is
missing is the check: the synthesized suite sends an absent `omitted_or_null` input only left out,
so an implementation that refuses `null` there, which the published schema admits, passes. And a
composition `reader: true` entry compares presence exactly (`crates/specify/ess-composition/src/conformance.rs:291-301`),
so a consumer declaring `omitted_or_null` over an imported `null_when_absent` field is reported as
drift although it reads every value that field sends.

## Design

- **Synthesis.** For a command whose input reaches a member declaring `omitted_or_null` (top level,
  or through structs and newtypes, never under an `Optional` parent, the rule the absent-first
  witness already walks, `docs/design/wire-presence-json-prefix.md:58-65`), the suite holds one
  more `execute_command` that sends every such member as `null` where the existing witness leaves
  it out, and expects the same outcome, response and events. It uses vocabulary every suite format
  already carries (a literal `null` in an input, as an authored scenario does today), so the suite
  format does not move; a model without such a member keeps its suite bytes.
- **Composition.** Under `reader: true` only, a local field declaring `omitted_or_null` conforms to
  an imported `Optional` field with any presence or none. The reverse, and every non-reader
  comparison and `implements` input comparison, stay exact.
- **Documentation.** The design page states the spelling each ESS encoder writes for an absent
  `omitted_or_null` member today: left out by an interpreted event field
  (`crates/verify/ess-conformance/src/interpret/execute.rs:3150-3155`) and the generated Rust wire
  (`crates/generate/ess-synth/src/rust/wire.rs:11`), `null` by an interpreted view row
  (`crates/verify/ess-conformance/src/interpret/views.rs:449-457`). The wire-names guide names the
  catalog case.

Alternatives, one line each:

- (a) A third mode as proposed: a second spelling of `omitted_or_null` (red flag 1 of the skill).
- (b) A separate decode-tolerance key (`decodes: [null, omitted]`) beside the encode form: two keys
  for one wire fact, a new `ess/N`, and every runner learns a vocabulary for a distinction no ESS
  check needs today; revisit only if a byte comparison against an adopter's vectors needs a
  declared encode form.
- (c) Decline with the idiom alone: the idiom works, but the suite would not check the `null`
  decode its schema publishes (policy 1, `docs/design/generic-language-features.md:590-594`).
- (d) A binding-level tolerance (HTTP binding): presence is a property of the field in every
  message, event and composition, not of one route; the `request.unknown` precedent is about
  undeclared members, not declared ones.

## Acceptance

- `cargo test -p ess-conformance --locked --test omitted_or_null`: the suite synthesized from a
  model whose command input `entry` is a struct with `either: Optional<Detail>, presence:
  omitted_or_null` holds two `execute_command` steps for the same outcome, one whose input leaves
  `either` out and one that sends `"either": null`; both pass on `--target interpreted`.
- In the same test file, a `process` test target that answers a decoding refusal when `either` is
  `null` fails the `null` step, and one that refuses it when missing fails the other.
- The Go and TypeScript runners send the `null` step's member as `null` (a recording test target
  sees `"either": null`); shared with the test target of
  `story:conformance-holds-missing-keeps-and-null-clears` where that one lands first.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` and
  `--test synthesis_suite_bytes_table_cli_read` pass with no row moved; a model declaring no
  `omitted_or_null` input keeps its suite format and bytes.
- `cargo test -p ess-composition --locked`: a `reader: true` conformance whose local struct field
  declares `omitted_or_null` composes against imported fields declaring `null_when_absent`,
  `omitted_when_absent` and no presence; a local `null_when_absent` or `omitted_when_absent` reader
  field against an imported `omitted_or_null` field is still drift naming the presence; the same
  pair without `reader: true` is still drift.
- `docs/design/omitted-or-null-presence.md` gains the encoded-as column (per encoder, cited),
  `docs/design/composition-type-conformance.md`'s reader table gains the widening, and
  `website/docs/guides/specify/wire-names.md` shows a "required on the wire, may be null, read when
  missing" member written as `omitted_or_null`; `task test-xtask` passes.

## Fit review

1. **Need, apart from the syntax.** A member whose absence travels as either a missing key or
   `null`, decoded alike, published as such, and checked as such (reproduction above, `run.txt` of
   the triage scratch: `either` valid both ways, the suite sends it left out only). The requester's
   proposal (theirs): a presence value `accepts_null_or_omitted` that "encodes absence one declared
   way and decodes both".
2. **Class: gap, small.** The declaration exists (`ess/31`,
   `website/docs/guides/specify/wire-names.md:163-167`, `website/docs/reference/spec-versions.md:436-441`),
   so the authored surface is not a gap. The gap is verification: the published schema admits
   `null` (`crates/generate/ess-gen/src/types.rs:765-770, 847-862`) and no synthesized step sends
   it; and the composition reader rule contradicts its own stated principle, "admits what a
   consumer that only reads the imported type can also afford, because it rejects no value the
   imported type allows" (`crates/specify/ess-composition/src/conformance.rs:29-31`), by comparing
   presence exactly (`:291-301`; `docs/design/composition-type-conformance.md:105-107`).
3. **Can it already be expressed? Yes, the declaration:** `presence: omitted_or_null` under
   `format: ess/31` (run on 0.97.1 above; below `ess/31` it is `ESS-TYPE-009` naming `ess/31`).
   `Optional<Optional<T>>` validates but projects `$ref T`, not nullable, so it is no idiom. An
   adopter can prove both decodes against their implementation today with two authored acts
   (`ess-scenario/6`, input `{..., either: null}` and the member left out: 0 refusals, both pass on
   the interpreted target). The idiom alone fails policy 1 for the synthesized suite (above).
4. **Fit.** The language grids do not apply: presence is a wire attribute of a field, not an
   expression position, root or operation (`docs/design/language-grids.md` names neither presence
   nor `null`). Vocabulary: no new key, the existing value. Siblings: `presence:` positions are
   already uniform (struct, entity, command input and response, event, error fields,
   `docs/design/omitted-or-null-presence.md:30-32`); the synthesis change covers command input
   members, the only position a suite sends (responses and events are observed, and the suite
   already admits both spellings there, `docs/design/omitted-or-null-presence.md:52-59`); the
   reader widening is the composition sibling of the same promise. Targets: JSON Schema, OpenAPI
   and AsyncAPI unchanged (`ess-gen` `object_annotated`); the interpreter reads both
   (reproduced); the generated Rust server decodes both
   (`crates/generate/ess-synth/src/rust/wire.rs:932-933`); Go and TypeScript runners only send the
   input as written; `ess verify diff` unchanged (a presence change stays `unclassified-changed`,
   reproduced; `docs/design/omitted-or-null-presence.md:94`); Entity Runtime does not read
   presence (`BoundPresence` is required/optional only,
   `crates/generate/ess-entity-runtime/src/lib.rs:183-187`).
5. **Second adopter.** A webhook receiver whose sender documents a `refund` member as always
   present and `null` when there is none, while older sender versions leave it out; the receiver's
   specification declares `refund: Optional<Refund>, presence: omitted_or_null` and wants its
   suite to prove the receiver takes both.
6. **Cost.** No `ess/N` (the value and its meaning are `ess/31`'s); no suite format (no new step
   vocabulary); no new diagnostic; no `ess-composition/N` (the widening admits pairs the format
   refused as drift and refuses nothing it admitted; an older `ess` still reports drift, as for any
   relaxation). Suite bytes move only for models with an `omitted_or_null` input member: one more
   step per such command. Size: `ess-conformance` (synthesis, one test file, runner tests),
   `ess-composition` (`field_naming`, tests), three documentation pages and `CHANGELOG.md`.
7. **Alternatives.** Change nothing; (a) the requested third mode; (b) a decode-tolerance key; (c)
   decline with the idiom; (d) binding-level tolerance; each refused in Design above. Chosen: keep
   the one declaration and make the suite and the composition reader honour it.

## Decisions

**Accept, redesigned.** Decided by the planning session on 2026-11-22. The need is real; the
proposed shape is refused because the requested mode already exists as `presence: omitted_or_null`
(`ess/31`, 0.69.0), and a second spelling would be permanent surface for one idea. What changed
from the request: no new mode and no declared encode form; instead the synthesized suite sends an
`omitted_or_null` input both ways, and a composition reader may widen a presence to it. Reply to
the requester: write `presence: omitted_or_null` under `format: ess/31` or later; the schema
property becomes `anyOf [T, null]` and not required, and both spellings decode as absent. This
draft orders no wave and no release.

## Out of scope

- A declared encode form for `omitted_or_null` (alternative b), and making the interpreted view row
  leave the member out instead of writing `null`: both admitted by every check, and changing the
  view row moves interpreted bytes.
- `ess verify diff` classes for a changed presence (direction-dependent: widening an input to
  `omitted_or_null` is compatible for callers, an output may break a reader that expects the key).
- The keep-on-missing meaning of the two spellings (`story:an-input-keeps-missing-and-null-apart`).
- A field with no presence publishing a non-nullable property while the model and suite admit
  `null` (`plain` above).
- `implements` input comparisons (`compare_inputs`, exact by design).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/tests/omitted_or_null.rs
- inferred: crates/verify/ess-conformance/src/go/runtime.go
- inferred: crates/verify/ess-conformance/src/ts/runtime.ts
- cited: crates/specify/ess-composition/src/conformance.rs
- inferred: crates/specify/ess-composition/tests/adversary_reader_conformance.rs
- cited: docs/design/omitted-or-null-presence.md
- cited: docs/design/composition-type-conformance.md
- cited: website/docs/guides/specify/wire-names.md
- inferred: CHANGELOG.md
