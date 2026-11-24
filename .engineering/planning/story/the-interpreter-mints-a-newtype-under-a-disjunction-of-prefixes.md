---
format: aep.planning-md/3
id: story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes
kind: story
status: active
title: The interpreter mints entity-field and event-payload values of a String newtype that satisfy its positive text invariants and its length invariants, distinct per row.
summary: 'A value minted for an entity field or event payload misses any starts_with, contains or any: of prefixes its base text does not already meet, so the scenario turns Unsupported (adopter item 5); 0.68.0 fixed the response path only (262c66ca8); same on 0.67.0.'
tags:
- adopter-report
- defect
- ess-0.70.0
- ess-0.70.0-first
- issues-2026-11-24
- severity-medium
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/newtype_any_prefix.rs
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:47Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:48Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Reported by an adopter on 2026-10-22 (item 5 of the second batch). A `Version` String newtype with an alphabet and
`not: {value: {starts_with: "."}}` gains "starts with a digit", written as `any:` of ten `starts_with` branches.
Interpreted conformance on the adopter's contract drops from 24/24 passed to 13 passed, 11 unsupported, every
unsupported one because the interpreter cannot mint a `Version` the model leaves to the implementation.

Narrowed by the 0.68.0 re-probe and re-confirmed on 0.92.0: the disjunction is not the cause. The value the
interpreter mints for an entity field or event payload left to the implementation satisfies no positive text
invariant (`starts_with`, `ends_with`, `contains`, or an `any:` of them) its base text does not already meet. Needed:
the mint produces a value of a String newtype that satisfies its positive text invariants, as the response path's
`text_choice` (`crates/verify/ess-conformance/src/interpret/response.rs:444`) already does since `262c66ca8`.

## Reproduction (0.92.0)

Brand-free fixture, `format: ess/29`, domain `demo.server`:

```yaml
- name: demo.server.Version
  kind: newtype
  of: String
  alphabet: "0123456789.abcdefghijklmnopqrstuvwxyz-"
  invariants:
    - not: {value: {starts_with: "."}}
    - not: {value: {ends_with: "."}}
    - not: {value: {contains: ".."}}
    - any: [{value: {starts_with: "0"}}, ..., {value: {starts_with: "9"}}]   # ten branches
```

An entity `Client` with a `version: Version` field, a `Handshake` command creating it with
`sets: {version: {generated: true}}` and an event payload `version: {generated: true}`.
`ess verify conform synthesize --path any --out any/suite.json` (exits 1 with ESS-SYNTH-013, no view publishes
`Version`, by design) then `ess verify conform run --target interpreted --path any --suite any/suite.json
--report-format 2`:

```text
unsupported demo.server.Handshake/outcome/accepted
    ESS-CF-TARGET: cannot expose invoking `demo.server.Handshake`: the model leaves a value of `demo.server.Version` to the implementation, and its type has no shape the interpreter can mint
1 scenarios: 0 passed, 0 failed, 0 error, 1 unsupported
```

Same result with `starts_with: "1"` alone (wide alphabet) and with `contains: "."` alone (alphabet `0123456789.`).
Control: the three `not:` invariants without any positive one pass 1/1.

0.68.0 re-probe matrix (alphabet x `any:` x entity x response), kept for reference: suites byte-identical between
0.67.0 and 0.68.0; the response-only cell passes since `262c66ca8`; every cell where the value is minted for an entity
field or event payload and the type carries a positive text invariant its base text misses is unsupported; a narrow
alphabet passed earlier only because the base text happened to start with a digit.

## Where it lives

- `crates/verify/ess-conformance/src/interpret/values.rs:60` (`pub fn mint`, public since `6e46a5abb7`): a non-UUID
  value is drawn from `crate::witness::fields` with `Distinction::further(tick)`; any failure becomes
  `Undetermined::NoValue`.
- `crates/verify/ess-conformance/src/interpret/execute.rs:2828` (`minted`) and `:2809` (`mint`) delegate to it.
- `crates/verify/ess-conformance/src/interpret/response.rs:444` (`text_choice`) and `:353` (`TextRules::candidates`):
  the response path's candidate search that already satisfies the positive invariants.
- `crates/verify/ess-conformance/src/witness.rs`: the witness does not pick a disjunct of an invariant-level `any:` of
  text operators when the alphabet admits a non-matching first character.

## Decisions

2026-10-22: the story is the narrowed need above. The fix site is the interpreter's mint (now `interpret/values.rs:60`,
reached from `interpret/execute.rs:2828`), drawing from the witness (`crates/verify/ess-conformance/src/witness.rs`) or
reusing the response path's candidate search (`interpret/response.rs:444`, `text_choice`); the implementor picks and
cites which. Synthesis does not change: suites stay byte-identical. `values::mint` is public and documented as the
interpreter's own choice that may change, so its answers for such types may change; the CHANGELOG says so.

## Out of scope

- The response path: fixed in `262c66ca8`.
- `story:response-newtype-witness-lets-prefix-and-suffix-overlap` (prefix and suffix overlap in
  `TextRules::candidates`). If this story reuses `text_choice`, it inherits that limit; it does not fix it.

## Acceptance

Fixtures: the entity/event model above, written by the test.

- **An `any:` of prefixes is minted.** The interpreted target mints `Version` for `Handshake/outcome/accepted` with
  the wide alphabet and the ten-branch `any:`. Before: unsupported, `ESS-CF-TARGET: ... its type has no shape the
  interpreter can mint`. After: passed. Test: `crates/verify/ess-conformance/tests/newtype_any_prefix.rs` (new), case
  `a_minted_entity_value_satisfies_an_any_of_prefixes`.
- **A single positive invariant is minted.** The same mint under `starts_with: "1"` alone (wide alphabet) and under
  `contains: "."` alone (alphabet `0123456789.`). Before: unsupported on both. After: passed on both. Same file, cases
  `a_minted_entity_value_satisfies_starts_with` and `a_minted_entity_value_satisfies_contains`.
- **No wrong verdict.** A `Version` whose positive invariants no text over the alphabet satisfies stays unsupported
  with the existing message, never passed or failed. Same file, case `an_unsatisfiable_text_rule_stays_unsupported`;
  red against a mint that returns its base text unchecked.
- **Nothing else moves.** Cells without a positive text invariant keep their reports, every fixture's suite is
  byte-identical, and existing cases in `tests/witness.rs` and `tests/interpreted_typed_identities.rs` keep their
  verdicts.

Fixture: the reproduction model, written by the test.
- **A fixed length is minted.** Red today: the suite of the reproduction passes on the interpreted
  target, 2 scenarios, 0 unsupported. Case `a_generated_fixed_length_identity_is_minted_and_passes`.
- **Two mints differ.** Two `WriteNote` in one scenario mint two different `note_id`s, each of 52
  characters; and with `value.count == 3` two mints differ or the scenario stays unsupported with
  the existing message: no two ticks mint one value. Cases
  `a_generated_fixed_length_identity_is_minted_and_passes` (52) and
  `a_short_fixed_length_never_mints_one_value_twice` (3).
- **The alphabet holds.** With `alphabet: "0123456789abcdef"` beside the invariant the mint uses
  only characters of the alphabet. Case `a_generated_fixed_length_value_keeps_its_alphabet`.
- **No wrong verdict (the story's case, widened to lengths).** A type whose length invariants no
  text meets (`value.count == 52` and `value.count == 53`) stays unsupported with the existing
  message. Case `an_unsatisfiable_length_rule_stays_unsupported`.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/values.rs
- cited: crates/verify/ess-conformance/src/interpret/execute.rs
- cited: crates/verify/ess-conformance/src/interpret/response.rs
- inferred: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/tests/newtype_any_prefix.rs

confidence: high

## Amendment of 2026-11-24: a text newtype whose invariant fixes its length (beyond10x/ess#816, item 4)

Folded, not a unit of its own (#816 item 4 only; the other items of #816 belong elsewhere, see the
triage). The active story owns the same path, `interpret/values.rs::mint` →
`witness::fields`, for the same value kinds (an entity field or event payload left to the
implementation), with the same refusal ("its type has no shape the interpreter can mint") and the
same "an unsatisfiable rule stays unsupported" acceptance, and its Decisions leave the fix site to
the implementor (the witness, or the response path's `text_choice`). Two units would edit
`witness::fields` to one end, so there is one builder: the story's implementor, who picks the site
once and covers the positive text invariants and the length invariants there. The amendment adds
four acceptance statements, four test cases, one scope line, the gate additions and one changelog
reference to the story; nothing else in the story changes.

Size added to the story: S (about 120 changed lines: 25 source, 95 test).
Fixed suite bytes of committed models: none expected, and the story's "Nothing else moves"
(every fixture's suite byte-identical) holds for the amendment too. If the chosen site is
`witness::fields`, its synthesis callers (event payload and delivery-context bases) can only change
where `fields` refuses today, which no committed suite can hold; the byte pins and the stamp under
Gate decide.
Patch-release candidate: no (it changes what the interpreted target answers).

### Need

beyond10x/ess#816, item 4. A `String` newtype with an invariant fixing its length
(`value.count == 52`) validates, but the interpreted target cannot mint a value of it for
`{generated: true}`: every scenario that creates such a row ends `unsupported` (40 of 123 in the
reporter's suite). The reporter moved the length into a comment to get the scenarios back.

### Reproduction

Reproduced on 0.98.0. Model `repro/816-4/` (`format: ess/37`): `NoteId` newtype of `String`,
`invariants: [value.count == 52]`, the identity of `Note`; `WriteNote` creates a `Note` with
`note_id: {generated: true}` in its event.

```console
$ ess verify conform synthesize --path repro/816-4 --out suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to suite.json
$ ess verify conform run --path repro/816-4 --suite suite.json --target interpreted --report-format 2 --report-out report.json
  unsupported depot.notes.WriteNote/outcome/written
      ESS-CF-TARGET: cannot expose invoking `depot.notes.WriteNote`: the model leaves a value of `depot.notes.NoteId` to the implementation, and its type has no shape the interpreter can mint
  2 scenarios: 0 passed, 0 failed, 0 error, 2 unsupported
```

### Cause (read in the source, not run)

Lines at `41dd4aa9e`.

- `crates/verify/ess-conformance/src/interpret/values.rs`, `mint` (60-93): for anything but a bare
  `Uuid` it calls `crate::witness::fields(…, Distinction::further(tick))` and maps its error to
  `Undetermined::NoValue`.
- `crates/verify/ess-conformance/src/witness.rs`, `fields` (1203-1229): `builder.member` builds the
  path-text base and refuses the field at 1219-1224 ("has no base value its declared invariants
  admit"); no invariant ladder is consulted. `invariant_ladders` (5282) and `ground_invariants`
  (5344) are reached only from the command search.
- Distinctness: the base of tick n is `"<path>-<n>"` (`primitive_value`, 7966-8000), and
  `text_length_ladder` (5115) resizes with `resize` (5152), which keeps the first `length`
  characters. A fixed length shorter than the path cuts the suffix off, so every tick would mint the
  same value: two rows with one identity, a wrong verdict instead of `unsupported`.

### Scope (added)

- The mint site the story picks also grounds a refused text base on the field's length invariants
  (`value.count`, `value.utf8_bytes`, `==`, `<=`, `>=`), reusing `text_length_ladder` and the ladder
  `invariant_ladders` builds, and applies the tick's distinction after resizing (for example on the
  last characters, inside the alphabet), so a short fixed length keeps ticks apart. If the site is
  `witness::fields` (1203-1229), it consults that ladder before refusing at 1219 and refuses only
  if no admitted value is left.

### Owned files (added)

- `crates/verify/ess-conformance/src/witness.rs` (already in the story's scope; `fields`)
- `crates/verify/ess-conformance/tests/newtype_any_prefix.rs` (the story's new file; the four cases
  above)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md`

### Not yours

- `invariant_ladders` itself: `d809-a-guard-read-text-input-gets-a-candidate-its-invariant-admits`
  (hotfix wave, lands first); reuse what it leaves.
- Generated Rust and Go minting (`ess-synth`): not reported; if they mint such values wrongly, a
  story of its own.
- #816 items 1-3 and 5-7 (triage: #554, #796's neighbour, D7, a design, #564).

### Tests first (added)

In `crates/verify/ess-conformance/tests/newtype_any_prefix.rs` (the story's new target), the four
cases above. Each synthesizes and runs one small model on the interpreted target: under 5 s
expected (bound 60 s per test, 120 s per step). Time them; one over 5 s is named in the report.

### Gate (added to the story's)

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test newtype_any_prefix`, then
  `--test wrapped_value_invariants`, `--test string_alphabet_and_length`, `--test witness`,
  `--test interpreted_typed_identities`
- `cargo test -p ess-cli --locked`
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`
- `task test-xtask` (the unit edits `CHANGELOG.md`)

### Depends on

The story it amends (one builder, one site); `d809` lands before it (same file, the ladder it
reuses).

### Changelog (added to the story's bullet, or its own)

Under `Fixed`:

- The interpreted target mints a `{generated: true}` value of a text newtype whose invariant fixes
  its length, distinct per row, where every scenario creating such a value ended `unsupported`
  (#816).

### Fit review

See the feature. Class: defect; the reporter's workaround (moving the length into a comment) drops
a checked figure. Cost: interpreter answers for a shape that answers nothing today. Considered: a
unit of its own beside the story (rejected: two builders of `witness::fields` to one end, C7);
depend on the story and land after it (rejected: the story leaves the site open, so the second
builder would redo the first's choice); fold (chosen).

### Decisions

- Decision: accept as proposed, folded into
  story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes.
