---
format: aep.planning-md/3
id: story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes
kind: story
status: draft
title: The interpreter mints entity-field and event-payload values of a String newtype that satisfy its positive text invariants
summary: 'A value minted for an entity field or event payload misses any starts_with, contains or any: of prefixes its base text does not already meet, so the scenario turns Unsupported (adopter item 5); 0.68.0 fixed the response path only (262c66ca8); same on 0.67.0.'
tags:
- adopter-report
- defect
- ess-0.70.0
- ess-0.70.0-first
- severity-medium
- zero-open-issues
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
revision: 13
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

## Scope

- cited: crates/verify/ess-conformance/src/interpret/values.rs
- cited: crates/verify/ess-conformance/src/interpret/execute.rs
- cited: crates/verify/ess-conformance/src/interpret/response.rs
- inferred: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/tests/newtype_any_prefix.rs

confidence: high
