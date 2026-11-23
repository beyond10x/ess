---
format: aep.planning-md/3
id: story:base64-content-is-declared-as-bytes-and-an-equality-between-two-bytes-timestamp-or-duration-inputs-is-witnessed-both-ways
kind: story
status: draft
title: Base64 content is declared as Bytes, and an equality between two Bytes, Timestamp or Duration inputs is witnessed both ways
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/tests/predicate_reference_page.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/grammar_primitive_witnesses.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: website/docs/guides/specify/fields-and-invariants.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 9
---
## Outcome

An author whose field carries base64 content finds, in the specification guide and the predicate
reference, that the field is declared as `Bytes` (directly or as a newtype of it): validation
refuses an authored value that is not padded base64, synthesis sends only values that decode, and
every projection publishes the encoding. A guard that compares two inputs of a primitive whose
witness ignores the field's path (`Bytes`, `Timestamp`, `Duration`) with `==` or `!=` gets a
scenario for both sides, where one side is refused with `ESS-SYNTH-003` today. No new key, no new
format version.

## Need

Reported by an adopter, on 0.97.1: a text type can take an alphabet and length bounds, but cannot
say that its content is base64 (length a multiple of 4, `=` padding only at the end). A text field
carrying base64 in two commands of a downstream specification is synthesized with placeholder
values like `<field>-1`. The real backend answers HTTP 400, so those scenarios fail on real
behaviour. The adopter's probe: one of the three values synthesis sent decodes.

Reproduced on the released 0.97.3 binary, all models in this scratch directory:

- `a-string/` (`run.txt`): a `String` newtype with the tightest constraints the language has, the
  64 base64 characters plus `=` as `alphabet:` and `value.count >= 4`, `<= 64`. Valid, 5
  scenarios. The values it sends (`decode.sh` checks length and `base64 -d`):
  `attachment len=10 INVALID`, `attachmentt1 len=12 valid`, `attachmentt1048594 len=18 INVALID`,
  `signatureeb64 len=13 INVALID`. One of four decodes, and only because its length happens to be
  12. This matches the adopter's probe.
- `f-string-example/`: `example:` on the String inputs (ess/11) fixes the plain instance only. The
  further instance falls back to the path: `contentt1 len=9 INVALID`, the `<field>-1` symptom
  the adopter reported.
- `b-attempts/` (`run.txt`): the refinements an author would try are refused.
  `value.count % 4 == 0` gives `unparsable_predicate`. `pattern:` and `format:` on the newtype
  give `unknown field ..., expected one of of, alphabet, prefix, invariants`. `{matches: …}` gives
  `unknown operator "matches"`.
- `c-lengths/`: an alphabet without `=` and `any: [value.count == 4, == 8, == 12, == 16]`
  validates. Synthesis only filters by invariants, it does not build values from them, so two
  outcomes are lost:
  `refusal[ESS-SYNTH-003] … attachment = "signaturefb64" meets it, but its declared type … refuses
  that value`. That is not a workaround.

What exists and is not discoverable:

- `d-bytes/` (`run.txt`): the same model with `of: Bytes`. Valid. Every value synthesis sends
  decodes (`AA== valid`, `AQ== valid`, `Eg== valid`). The JSON Schema of the newtype is
  `"pattern": "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$",
  "contentEncoding": "base64"`.
- `e-siblings/Bytes-bad-example.yaml`: `example: "signature-1"` on a Bytes input is refused with
  `ESS-COMMAND-002 … is not a value of shop.order.Payload (Bytes)`.
- `Bytes` is in no page of the specification guide or the reference as a type an author picks.
  `website/docs/guides/specify/fields-and-invariants.md` documents `alphabet:` (:191), `prefix:`
  (:235), `Json` (:298) and `Binary64` (:317), but not `Bytes`. Grid O in
  `docs/design/language-grids.md:63-65` has no `Bytes` column. `predicates.md` "Text shapes
  without patterns" (:1147) lists alphabet, prefix, `.count` and the string operators, and does not
  mention the primitive grammars.

A defect of the idiom, found while proving it (`e-siblings/run.txt`). The same model with the guard
`signature_b64 == attachment`, one model per payload primitive:

```text
Integer    4 scenario(s), 0 refusal(s)
Uuid       4 scenario(s), 0 refusal(s)
Timestamp  refusal[ESS-SYNTH-003] … no candidate of the 1 tried satisfies `none of: signature_b64 == {fact: attachment} as timestamp`
Duration   refusal[ESS-SYNTH-003] … `none of: signature_b64 == {fact: attachment}`
Bytes      refusal[ESS-SYNTH-003] … `none of: signature_b64 == {fact: attachment}`
```

The cause is at tag `0.97.3`. `primitive_value`
(`crates/verify/ess-conformance/src/witness.rs:7659-7685`) gives every `Bytes`, `Timestamp` and
`Duration` leaf of one instance one value (`AA==` for every Bytes leaf, `base64_byte` :7923-7928;
documented at :1193-1197). `alternatives` (:6735) tries each side of a fact comparison at the
other side's base, which is the same value, so it adds nothing. Only an ordering adds a step, a
day for `Timestamp` (:6811-6828). So `!=` between two such inputs has no candidate. With
`example: "c2lnbmF0dXJl"` on one input, the same Bytes model writes 4 scenarios and 0 refusals
(`e-siblings/Bytes-example.yaml`).

## Fit review

**Decision: accept, redesigned.** The need is real. The proposed shape, a `format: base64` (or
`bytes`) refinement of text, names a concept ESS already spells as the primitive `Bytes`. Two
spellings of one idea is the first red flag of `.agents/skills/assessing-external-requests/SKILL.md`.
So nothing new is added to the language. The story makes the existing spelling discoverable and
fixes the witness defect of its type family.

1. **Need apart from the syntax.** A field whose content is an encoding of binary data must be
   sent, by synthesis, only as values that decode. Validation must refuse authored values that do
   not decode. Minimal reproduction: `a-string/run.txt`. The requester's syntax, theirs:
   `format: base64` or `bytes` on a text type.
2. **Class.** It is a *convenience*: it is expressible today as `Bytes` (`d-bytes/run.txt`). It is
   a *defect* for the equality witness: `witness.rs:6735` says "each side tried at the other's
   base", which promises both sides, and `e-siblings/run.txt` shows one side refused. It is a
   *documentation gap*: `Bytes` is absent from the author guide (Need).
3. **Already expressible.** Yes. `Primitive::Bytes` is admitted only as padded base64
   (`crates/specify/ess-domain/src/types.rs:2952`, `is_padded_base64`; the grammar and its corpus
   are in `docs/design/review-primitive-semantics.md:40, 259-271`). Every target holds it:
   - JSON Schema: `contentEncoding: base64` with the anchored pattern
     (`crates/generate/ess-gen/src/types.rs:36, 672`); OpenAPI and AsyncAPI do the same
     (`openapi.rs:143`, `asyncapi.rs:114`).
   - Rust: `Vec<u8>` (`ess-synth/src/rust/layout.rs:527`), and the decoder refuses anything but
     base64 (`rust/wire.rs:1018`).
   - Go: `[]byte`, `base64.StdEncoding` (`go/layout.rs:1224`, `go/http.rs:1004, 1184`).
   - Entity Runtime: `BindingRequirement::BytesSpelling` (`ess-entity-runtime/src/lib.rs:1374`).
   - Synthesis: one-byte padded values (`witness.rs:7680`).
   - TypeScript runner: I don't know; not read.

   The idiom keeps every checked figure in the specification: the type, the grammar, the schema.
   So it is a valid answer under policy 1 of `docs/design/generic-language-features.md`.
4. **Fit.**
   - Grid O (`language-grids.md:63`) has no `Bytes` column. The witness fix sits in the `==` `!=`
     row for the whole set of path-free grammar columns (`Timestamp`, `Duration`, `Bytes`), not
     for Bytes alone. Integer and Decimal already work (`e-siblings`: Integer 0 refusals).
   - Vocabulary: no new key. "A text with a format" is spelled in ESS as a primitive with its own
     grammar (`Uuid`, `Bytes`, `Timestamp`, `Duration`), each with a validator and a generator ESS
     owns.
   - A second encoding of the same bytes would be `encoding:` on a newtype. That is the key
     `story:a-timestamp-newtype-declares-its-wire-encoding` (active) introduces for `Timestamp`,
     never a text refinement (D3).
5. **Second, unrelated adopter.** A document-upload command whose `thumbnail` is base64 image
   data. A webhook registration that carries a public key as base64. Both backends refuse text
   that does not decode. Both are `Bytes`.
6. **Cost.** Chosen design: a guide section, a reference line, a grid column and one witness rule
   in `ess-conformance`. Synthesized values change only for models that today get the
   `ESS-SYNTH-003` above, and they gain a scenario. No `ess/N`, no suite major, no diff vocabulary.
   The proposal as written would have cost a source format (`ess/38`), a closed format registry
   on text, validation codes, a witness generator per format, diff classifications, and a second
   way to say what `Bytes` says.
7. **Considered.**
   - (a) Change nothing: the next adopter finds `alphabet:` first, as this one did, and ships
     values that do not decode.
   - (b) A closed set of named text formats on String (`format: base64 | email | uri | …`).
     Rejected. For base64 it duplicates `Bytes`. For the others the repository has no evidence:
     no story, issue or example asks synthesis for a valid e-mail, URI or date-only text. The
     examples' `Email` newtypes are plain `of: String`, and the backends behind them were not
     reported refusing the witness.
   - (c) A pattern language (`pattern:`, `matches:`). Declined three times, for the witness
     automaton and the dialects of three runners:
     - `docs/design/string-predicate-operators.md:529`
     - `docs/design/string-alphabet-and-length.md:551`
     - `docs/design/generic-language-features.md:718`
     - `story:feature-request-449`, declined with the "Text shapes without patterns" idiom
     - `story:json-values-and-text-patterns` (#146), which built `prefix:` and not `pattern:`

     Base64 is not even a regular language with bounds that alphabet and length can state (c-lengths).
   - (d) A lint that tells an author whose String alphabet equals the base64 alphabet to use
     `Bytes`. Rejected: a heuristic on spelling. The guide is the answer.
   - (e) Chosen: document `Bytes` as the base64 type, add the grid column, and witness `!=`
     between two path-free grammar inputs.

## Design

**Documentation.** `fields-and-invariants.md` gains a section after "Say what a text starts with"
(:235), titled "Carry binary content as base64". It says:

- `Bytes` and a newtype of it hold padded base64 (RFC 4648 § 4), checked by validation on every
  authored value (`example:`, literals);
- synthesis sends one-byte values that decode;
- the projections publish `contentEncoding: base64` with the anchored pattern;
- Rust holds `Vec<u8>` and Go `[]byte`, and their decoders refuse text that does not decode;
- `alphabet:`, `prefix:` and `.count` do not apply to `Bytes` (with their refusals);
- a field moved from `String` to `Bytes` changes the generated Rust and Go field type, and
  `ess verify diff` reports it as a type change: `g-diff/run.txt`, `type shop.order.Payload:
  wraps Bytes instead of String`.

The section carries a checked block that validates and synthesizes. `predicates.md` "Text shapes
without patterns" gains one bullet: "content that is an encoding: `Bytes` (base64), `Uuid`,
`Timestamp`, `Duration`, each with its own grammar". `language-grids.md` grid O gains the `Bytes`
column: `==` `!=` yes, ordering no, `in` yes, length no.

**Witness.** In `alternatives` (or its caller, where `fact_comparisons` is known), for an `==`
or `!=` between two leaves whose primitive is `Timestamp`, `Duration` or `Bytes` and whose bases
are equal, one side is also tried at its `apart` value (`witness.rs:7738`, which exists for
optional leaves and already never lands on a required leaf's value). The other side keeps its base.
The new candidate is appended after the existing ones, so a branch that already has a witness
keeps it and its suite bytes.

**Validation codes.** None. **Refusals.** None added. One `ESS-SYNTH-003` disappears per such
branch.

**Format consequence.** None:

- `ess/N`: no new key, no new meaning, no change to the persisted envelope (`AGENTS.md` §
  Determinism and formats).
- IR: unchanged.
- `ess-conformance/`: no new vocabulary, so `select_fresh_format` is unchanged. Only the values of
  newly witnessed scenarios are new.
- `ess-diff/`: unchanged.

Readers of old formats see nothing new. A model that compared two such inputs gains a scenario. Its
row in `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` moves, if
it has one, with the cause named beside the pin.

## Acceptance

- `bytes_newtype_answers_base64_content`: a model with a `Bytes` newtype in two command inputs
  validates. Every input literal of its synthesized suite is padded base64 by `is_padded_base64`,
  and its JSON Schema carries `contentEncoding: base64` and the pattern. New case in
  `ess-conformance`'s tests.
- `two_grammar_inputs_compared_for_equality_are_witnessed_both_ways`: for each of `Bytes`,
  `Timestamp` and `Duration`, the `e-siblings` model shape (`signature_b64 == attachment` beside a
  default branch) synthesizes 4 scenarios and 0 refusals, and the `!=` scenario sends two
  different values. Integer and Uuid keep their suite bytes. Same test file.
- `synthesis_suite_bytes_table` (`ess-conformance`) passes. A moved row names this story.
- `fields_and_invariants` page check: the new section's checked block validates and synthesizes
  under the page test that runs the guide's `ess-check` blocks. If the guide has none, the block
  goes on `predicates.md` and `crates/edge/ess-cli/tests/predicate_reference_page.rs` runs it.
- `predicate_reference_page`: "Text shapes without patterns" contains "`Bytes`" and "base64".
- `task test-xtask` and `task site-build` pass.

## Decisions

- **D1, the verdict.** (A) Accept, redesigned, as above. (B) Decline with the idiom only: a
  `specification` artifact naming `Bytes`, with no guide section and no witness fix.
  **Recommended: A.** The witness defect is real and reachable by any model that compares two such
  inputs, and the documentation gap is how this request arose.
- **D2, the Bytes base value.** (A) Keep `AA==` for every Bytes leaf and add only the
  comparison candidate. (B) Derive the base from the path as `Uuid` does: the base64 of the path's
  UTF-8 bytes (`attachment` → `YXR0YWNobWVudA==`). That keeps two Bytes fields of one input apart,
  so a swapped binding is detectable, and it decodes to the field's name in a failing diagnostic.
  But it moves the suite bytes of every model with a Bytes leaf (`examples/billing`,
  `examples/gatepass`). **Recommended: A.** No reported fault needs B. B is the follow-up when a
  swapped Bytes mapping is reported.
- **D3, other encodings and a decoded length.** (A) Not now. A base64url or hex field, or a
  minimum decoded size, is reopened as `encoding:` on a Bytes newtype (sharing the key and design
  page of `story:a-timestamp-newtype-declares-its-wire-encoding`) and a byte-length read on
  `Bytes`, each when an adopter shows a backend that refuses on it. (B) Add both now in `ess/38`.
  **Recommended: A.** The evidence here is padded base64 only, which `Bytes` already is.
