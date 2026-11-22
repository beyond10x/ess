---
format: aep.planning-md/3
id: story:a-writer-side-conformance-admits-the-narrowing-a-sender-may-do
kind: story
status: draft
title: A writer-side conformance admits the narrowing a sender may do
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/writer-conformance
- confidence: inferred
  path: crates/specify/ess-composition/tests/writer_conformance.rs
- confidence: cited
  path: docs/design/composition-type-conformance.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 12
---
## Outcome

A domain that keeps its own copy of an imported message it sends can assert that copy with a
`conformances:` entry marked `writer: true`, and the comparison asks the question a sender needs
answered: can the local type send a value the imported type's reader refuses? If not, the pair
conforms. A local field `null_when_absent` against an imported `omitted_or_null` field conforms,
because every document it writes is one the imported reader accepts. A local `Optional` field
against an imported required one is drift under `writer: true`, where a plain entry admits it
today. This is the mirror of `reader: true` (`ess-composition/3`); the reader half of the same
request, a local `omitted_or_null` reading an imported `null_when_absent`, is the composition half
of `story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways` and is not repeated here.

## Need

An adopter's domain imports a protocol whose response message declares a field `Optional<…>`,
`presence: omitted_or_null`. The adopter's service sends that message and always writes the key,
so its own type for the message declares `presence: null_when_absent`, which makes the generated
Rust field a plain `Option<…>` that is always serialized. `ess specify compose` refuses the
conformance entry between the two types, and `reader: true` on the entry gives the same refusal.

Reproduced on the released 0.97.3 binary with a generic model: `matrix/` (imported system `wire`,
local system `relay`, one struct per presence form with one field `codec`, every local form
against every imported form, once without and once with `reader: true`; `matrix/run.sh`, output
`matrix/run.txt`):

```text
$ ess specify compose --path composition-exact.yaml --service relay=local --service wire=owner --format json
exit 1
NullWhenAbsent -> OmittedOrNull : drift | field `codec`: presence `null_when_absent` where `wire.session.OpenedOmittedOrNull` has `omitted_or_null`
NullWhenAbsent -> Required : conforms
$ ess specify compose --path composition-reader.yaml ...   (every entry `reader: true`)
NullWhenAbsent -> OmittedOrNull : drift | field `codec`: presence `null_when_absent` where `wire.session.OpenedOmittedOrNull` has `omitted_or_null`
OmittedOrNull -> NullWhenAbsent : drift | field `codec`: presence `omitted_or_null` where `wire.session.OpenedNullWhenAbsent` has `null_when_absent`
```

The two matrices are identical: `reader: true` changes no presence verdict, because
`Comparison::field_naming` compares the presence of two `Optional` fields by equality whatever the
mode (`crates/specify/ess-composition/src/conformance.rs:291-300`), as the design page says
(`docs/design/composition-type-conformance.md`, "the `presence` of fields that are optional at
both ends ... compared as without `reader`").

What the matrix shows for a sender:

- `reader: true` is the wrong direction for it and refuses correctly: a reader that requires the
  key cannot read a document that omits it. No defect in that half.
- A plain entry is a reader assertion with one tolerance, "a local value may be `Optional` where
  the imported one is required" (`conformance.rs:739-740`). For a sender that tolerance is unsound:
  `NullWhenAbsent -> Required`, `OmittedWhenAbsent -> Required`, `OmittedOrNull -> Required` and
  `Unstated -> Required` all conform, although such a sender may write an absent value the imported
  reader refuses. And it refuses what a sender may safely do: `Required -> <any Optional>` is drift
  (`field `codec`: `String` where the imported type has `Optional<String>``).
- Nothing in a composition says which end writes. The only direction marker is `reader:`
  (`crates/specify/ess-composition/src/lib.rs:644-656`); `roles: implements` compares a command's
  input, the reader side, and no response (`conformance.rs:168-217`).

The adopter's workaround on 0.97.3 is to declare the contract's `omitted_or_null` locally. It
composes (`OmittedOrNull -> OmittedOrNull : conforms`) but turns the generated field into
`EssPresence<Option<String>>` that is skipped when absent (`matrix/run.txt`, types excerpt), so the
sending code changes for a check, not for the protocol.

## Fit review

1. **Need, apart from the syntax.** A sender's copy of an imported message may state fewer wire
   spellings than the message admits (always write `null`, never omit; always send a value), and
   the copy should still be checked against the import. Mirror: a reader's copy may accept more
   spellings than the message is sent with. The requester's proposal (theirs): "let a domain
   declare a stricter presence for a message it emits, and the mirror for a receiver accepting
   more".
2. **Class: gap**, with an unsound corner. Direction cannot be declared for a writer at all, so a
   sender's narrowing is refused and a sender's widening (`Optional` against required) passes. The
   reader half is a defect against the reader rule's own principle, "admits what a consumer that
   only reads the imported type can also afford" (`conformance.rs:29-31`), already drafted in
   `story:a-synthesized-suite-sends-an-omitted-or-null-member-both-ways` (its Design,
   "Composition").
3. **Can it already be expressed?** No. Matching the contract's presence composes but changes the
   generated type (above); dropping the entry loses the check of the whole message; `reader: true`
   is the opposite direction (matrix). No other key on an entry exists.
4. **Fit.** The language grids do not apply (composition, not an expression position). Vocabulary:
   `writer: true` mirrors the released `reader: true` key on the same entry, with the same shape
   (a boolean whose authored presence is kept apart from its value). Siblings: it applies to every
   end kind a conformance takes under `/5` (`type`, `event`, `error`, `command` with `part`), as
   `reader` does; an event the local component publishes or a command response it returns is the
   typical writer end. `roles: implements` compares no response today; a response comparison there
   would use this rule and is out of scope. Targets: no generator, runtime, runner, the Entity
   Runtime or `ess verify diff` reads a composition (`docs/design/composition-type-conformance.md`
   § Compatibility); the client plan does not move.
5. **Second adopter.** A webhook sender mirrors a receiver's published payload type, where `refund`
   is optional and may be left out or `null`. The sender always sets `refund` (required in its own
   model) and wants compose to prove the copy, and to fail if a later change made `refund` optional
   on its side while the receiver required it.
6. **Cost.** One new entry key, so a new composition major (below). No `ess/N`, no IR, no suite
   format, no new diagnostic code: drift stays `type_conformance_drift`, its catalogue text widened
   to name the writer rule. Size: `ess-composition` (`TypeConformance`, `Comparison` gains a
   writer mode beside `reader`), one test file, the design page, `formats.md`, `spec-versions.md`,
   `review-format-catalog.md`, `diagnostics.md`, `CHANGELOG.md`.
7. **Alternatives.** (a) Change nothing: the adopter keeps a type its code does not want, or no
   check. (b) The requested shape read literally, "a stricter presence for a message it emits"
   declared on the field in `ess/N` (an `emits`-side presence): puts a composition fact into the
   source format, a second presence key for one field, and every target would have to read it.
   (c) Relax presence in the plain entry both ways: unsound in one direction whichever way is
   chosen. (d) Infer direction from the surface (the local component publishes the event): works
   for event ends only; a `type` end, the adopter's case, carries no direction. Chosen: an explicit
   `writer: true` on the entry, the dual of `reader: true`, each with one question.

## Design

`writer: true` on a `conformances` entry, admitted from the composition format that carries it
(Decision 5). The test for every rule is one question: can the local type send a value the
imported type's reader refuses? If it cannot, the difference is admitted; if it can, it stays
`type_conformance_drift`.

Presence, as the set of absent spellings a field writes or reads: required ∅, `null_when_absent`
{null}, `omitted_when_absent` {omitted}, `omitted_or_null` {omitted, null}, unstated {omitted, null}
in the model and suite, {omitted} in the published schema (property `T`, not nullable,
`docs/design/omitted-or-null-presence.md` § 1 table). A writer conforms when its set is a subset of
the imported set; equal forms always conform. Local row, imported column:

| local writes \ imported reads | required | `null_when_absent` | `omitted_when_absent` | `omitted_or_null` | unstated |
|---|---|---|---|---|---|
| required | yes | yes | yes | yes | yes |
| `null_when_absent` | drift | yes | drift | yes | drift (Decision 3) |
| `omitted_when_absent` | drift | drift | yes | yes | yes |
| `omitted_or_null` | drift | drift | drift | yes | drift (Decision 3) |
| unstated | drift | drift | drift | yes | yes |

The reader table, for the record (the reader half lands with the draft story named above):
a reader conforms when the imported set is a subset of its own. Today only equal forms and an
`Optional` reader of a required field conform; `omitted_or_null` reading `null_when_absent`,
`omitted_when_absent` or unstated is the over-refusal that story removes.

The rest of the writer mode, each the dual of a released reader widening (Decision 4):

| narrowing | admitted under `writer: true` | still drift |
|---|---|---|
| optionality | a local required value where the imported one is `Optional`, at any position | a local `Optional` where the imported value is required (the plain entry's tolerance does not apply) |
| newtype | a local newtype, through any chain, over what the imported reference is | a different primitive |
| enum as `String` | a local enum where the imported reference is `String` | a local `String` where the imported one is an enum |
| variants | local wire names a subset of the imported wire names | a local wire name the imported enum lacks |
| union | the same tag, local variants a subset of the imported ones by tag value, each conforming as a writer | a local variant the imported union lacks |
| object as a map | a local struct or `String`-keyed map where the imported reference is `Map<String, Json>` | anything that is not always an object |
| field subset | omitting an imported field that is `Optional` and whose presence admits omission (`omitted_when_absent`, `omitted_or_null`, unstated) | omitting a required or `null_when_absent` imported field |
| extra fields | a local field the imported struct lacks, when the imported struct declares `undeclared_fields: ignored` (`EssIr::undeclared_fields`, `ess/24`) | the same against a closed imported struct |
| value constraints (`/5` rules) | a local alphabet that is a subset, a prefix that starts with the imported prefix, a superset of the imported invariants | a wider alphabet, a shorter or other prefix, a missing imported invariant |

Enum variant attributes stay compared exactly: they are facts about a variant, not values on the
wire. Wire names, map keys and lists compare as without `writer`. A writer extra field is decided
from the imported model, so `writer: true` needs no tolerance assertion of the kind `reader: true`
makes.

Refusals: `writer: true` together with `reader: true` on one entry is refused when the document is
read (Decision 2). Under an earlier composition format a `writer` key, whatever its value, `null`
included, is refused as `unsupported_format` naming the new format, as `/2` refuses `reader`.

**Format consequence** (`AGENTS.md` § Determinism and formats): the authored and compiled
composition is a persisted envelope with a closed DTO (`TypeConformance` denies unknown fields), and
the entry gains a key with meaning, so a new composition major is required. `TypeConformance`
keeps `writer` as authored, `Option` apart from its value, and an entry without it serialises
without it, so every `/1`-`/5` document keeps its authored and compiled bytes. Old-reader
compatibility test: an earlier-format document carrying `writer` (true, false, null) is refused;
the same document without it compiles to unchanged bytes. No `ess/N`, IR, client-plan or suite
format moves.

## Acceptance

- `cargo test -p ess-composition --locked --test writer_conformance` (new): every cell of the
  presence table above is one assertion on a two-system fixture (local `relay`, imported `wire`,
  one struct per presence form), under `writer: true`; the drift cells name the field and both
  presences.
- In the same target, the adopter's shape: local `Optional<…>`, `null_when_absent` against an
  imported `omitted_or_null` field composes with no diagnostic under `writer: true`, and stays
  `type_conformance_drift` without it and with `reader: true`.
- In the same target, one admitted and one drift case per row of the narrowing table, including a
  local `Optional` against an imported required field under `writer: true` being drift, and an
  extra local field admitted against an `undeclared_fields: ignored` imported struct and drift
  against a closed one.
- In the same target, `writer: true` on an `event` end and on a `command` end with
  `part: response` applies the same rules.
- In the same target, `writer: true` and `reader: true` on one entry is refused when read naming
  both keys; `writer` under `ess-composition/5` (true, false, null) is refused as
  `unsupported_format` naming the new format.
- `cargo test -p ess-composition --locked` passes with no existing `/2`-`/5` assertion changed;
  a `/5` fixture's compiled bytes are unchanged.
- `docs/design/composition-type-conformance.md` gains a "Writer assertions" section with both
  tables; `website/docs/reference/formats.md`, `spec-versions.md`, `review-format-catalog.md` and
  the `type_conformance_drift` row of `diagnostics.md` describe the key; `task test-xtask` passes.

## Decisions

1. Spelling of the direction. (A) `writer: true` on the entry, the mirror of `reader: true`;
   (B) `direction: writes | reads` replacing `reader` from the new format, two spellings of the
   reader assertion across formats; (C) infer from the end's surface, event ends only.
   Recommended: A.
2. Both keys on one entry. (A) Refused when read; (B) means "holds in both directions", an exact
   comparison without the plain entry's reader tolerance. Recommended: A; B can be added later
   without a format change of meaning, since A refuses it today.
3. An imported field with no presence, under `writer: true`. (A) Read as the published schema
   reads it, {omitted}: a `null_when_absent` or `omitted_or_null` writer against it is drift;
   (B) read as the model and suite read it, {omitted, null}: both conform. Recommended: A, sound
   against an implementation that validates with the published schema.
4. Scope of the writer mode. (A) The full dual of the reader widenings (narrowing table above);
   (B) presence and optionality only, the rest a later story. Recommended: A; B leaves a sender to
   meet the next refusal (an enum subset, a newtype) on the same entry, and splits one rule across
   two releases.
5. Format number. (A) Share `ess-composition/6` with `story:reader-true-refused-for-closed-readers`
   (active, opens `/6` for `tolerant:`) when both land in one release; (B) a `/7` of its own.
   Recommended: A (skill red flag: no format bump a nearby unreleased one could carry).

Recommended decision under the skill: **accept, redesigned**. The need is real; the proposed
"stricter presence for a message it emits" becomes a direction on the conformance entry, not a
second presence key on the field, and it covers every narrowing a sender may do, not presence
alone.
