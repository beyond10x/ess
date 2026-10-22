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
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/newtype_any_prefix.rs
revision: 10
---
## Need

Reported by an adopter on 2026-10-22 (item 5 of the second batch, relayed by the orchestrator; adopter names kept out). A
`Version` String newtype with an alphabet and `not: {value: {starts_with: "."}}` gains "starts with a digit", written as
`any:` of ten `starts_with` branches. Interpreted conformance on the adopter's contract drops from 24/24 passed to 13 passed,
11 unsupported, every unsupported one because the interpreter cannot mint a `Version` the model leaves to the implementation.

Narrowed on 2026-10-22 by the re-probe below: the disjunction is not the cause. The value the interpreter mints for an
entity field or event payload left to the implementation (`crates/verify/ess-conformance/src/interpret/execute.rs:2805`,
`minted`) satisfies no positive text invariant (`starts_with`, `ends_with`, `contains`, or an `any:` of them) its base text
does not already meet. Needed: `minted` produces a value of a String newtype that satisfies its positive text invariants,
as the response path's `text_choice` (`interpret/response.rs:444`) already does since `262c66ca8`.

## Reproduction

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

An entity `Client` with a `version: Version` field, and a `Handshake` command creating it with `sets: {version: {generated:
true}}` and an event payload `version: {generated: true}`.
`ess verify conform synthesize --path spec.yaml --suite-format 5 --out suite.json` then
`ess verify conform run --target interpreted --path spec.yaml --suite suite.json --report-format 2`, on 0.68.0 and 0.67.0:

```text
unsupported demo.server.Handshake/outcome/accepted
    ESS-CF-TARGET: cannot expose invoking `demo.server.Handshake`: the model leaves a value of `demo.server.Version` to the implementation, and its type has no shape the interpreter can mint
1 scenarios: 0 passed, 0 failed, 0 error, 1 unsupported
```

Controls, both versions: the same model without the `any:` passes 1/1; with the alphabet narrowed to `"0123456789."` the
`any:` model passes 1/1 (every base text happens to start with a digit). So the disjunction is the cause, and a wider alphabet
exposes it.

Variant with the value in a response (`response: [{name: version, type: Version}]`, `returns: true`): 0.67.0 gives the same
mint refusal; 0.68.0 gives `ESS-CF-TARGET: cannot expose invoking `demo.server.Handshake`: a bounded actual command response is
not interpreted yet`. Unsupported on both; only the reason changed, through `262c66ca8` (response string-newtype
interpretation, whose candidate search also does not satisfy the `any:`).

## Where it lives

- `crates/verify/ess-conformance/src/interpret/execute.rs:2805` (`minted`): a non-UUID value is drawn from
  `crate::witness::fields` with `Distinction::further(tick)`; any failure there becomes `Undetermined::NoValue` (message at :352).
- `crates/verify/ess-conformance/src/interpret/response.rs:106` (`base`) and `unavailable` (:98) for the response variant.
- The witness does not pick a disjunct of an invariant-level `any:` of text operators when the alphabet admits a non-matching
  first character; `crates/verify/ess-conformance/src/witness.rs` was not traced to the line.
- Related, not the same: `story:response-newtype-witness-lets-prefix-and-suffix-overlap` (prefix and suffix overlap in the
  response path).

## Regression status

Not a 0.68.0 regression: the entity/event fixture gives byte-identical suites and the same refusal on 0.67.0; no commit in
`0.67.0..52bd110dd` touches `interpret/execute.rs` or `witness.rs`. Defect. Does not block 0.68.0.

## Re-probe on 0.68.0 (2026-10-22)

A 12-cell matrix (alphabet x `any:` x entity x response) and five single-invariant controls on the 0.68.0 integration build (runtime identical to 52bd110dd) and on
0.67.0, `ess verify conform run --target interpreted`. Synthesis exits 1 on every cell with ESS-SYNTH-013 (no view publishes
`Version`), which is by design and does not stop the run. "Entity" means the `Client` row and `Handshaken` event carry
`version: {generated: true}`; "response" means `response: [{name: version, type: Version}]` with `returns: true`.

| `Version` invariants | alphabet | entity | response | 0.67.0 | 0.68.0 |
|---|---|---|---|---|---|
| not `.`-start/end, no `..`, `any:` of ten digit prefixes | wide | no | yes | unsupported: bounded response not interpreted | **passed** |
| same | wide | yes | yes | unsupported: bounded response not interpreted | unsupported: cannot mint `Version` |
| same | wide | yes | no | unsupported: cannot mint | unsupported: cannot mint |
| `starts_with: "1"` alone | wide | yes | no | unsupported: cannot mint | unsupported: cannot mint |
| `contains: "."` alone | `0123456789.` | yes | no | unsupported: cannot mint | unsupported: cannot mint |
| not `.`-start/end, no `..`, `contains: "."` (with or without `any:`) | `0123456789.` | yes | no | unsupported: cannot mint | unsupported: cannot mint |
| any cell without a positive text invariant | either | either | either | passed | passed |
| not `.`-start/end, no `..`, `any:` of digit prefixes | `0123456789.` | either | either | passed | passed |

Suites are byte-identical between the versions on every cell; only the interpreter differs.

What 0.68.0 fixed: the response path. `262c66ca8` (response string-newtype interpretation, `response.rs:444` `text_choice`)
searches candidate texts until the declared type admits one, and it satisfies the ten-branch `any:`.

What remains, and is wider than first written: the value the interpreter mints for an entity field or event payload left to the
implementation (`execute.rs:2805` `minted`, through `crate::witness::fields` with `Distinction::further`) satisfies no positive
text invariant its base text misses. A single `starts_with: "1"` or `contains: "."` fails the same way; the disjunction is not
the cause, and the narrow-alphabet pass earlier was a base text that happened to start with a digit. When the value is both
minted and returned, 0.68.0 now reports the mint refusal instead of the response one, because the response step no longer
refuses first.

Narrowed need: `minted` produces a value of a String newtype that satisfies its positive text invariants (`starts_with`,
`ends_with`, `contains`, an `any:` of them), as the response path's `text_choice` already does. Inferred, not traced: reuse
`TextRules::candidates` / `text_choice` in `minted`. No commit in `0.67.0..52bd110dd` touches `interpret/execute.rs` or the
witness module; not a regression.

Fixtures: the matrix generator and every cell's suite and report are kept outside the repository with the batch's scratch files.

## Decisions

2026-10-22: the story is the narrowed need above, and its title says so. The fix site is `minted`
(`interpret/execute.rs:2805`), drawing from the witness (`crates/verify/ess-conformance/src/witness.rs:413-447`, `:938`,
`:4652`) or reusing the response path's candidate search (`interpret/response.rs:444`, `text_choice`); the implementor picks
and cites which. Synthesis does not change: suites stay byte-identical.

## Out of scope

- The response path: fixed in `262c66ca8`; its cells in the re-probe table pass on 0.68.0 and are not touched.
- `story:response-newtype-witness-lets-prefix-and-suffix-overlap` (prefix and suffix overlap in `TextRules::candidates`).
  If this story reuses `text_choice`, it inherits that limit; it does not fix it.

## Acceptance

Fixtures: the re-probe's entity/event cells (`Client.version` and the `Handshaken` payload `version: {generated: true}`),
written by the test.

- **An `any:` of prefixes is minted.** Transition: the interpreted target mints `Version` for `Handshake/outcome/accepted`
  with the wide alphabet and the ten-branch `any:`. Before: unsupported, `ESS-CF-TARGET: ... its type has no shape the
  interpreter can mint`. After: passed. Test: `crates/verify/ess-conformance/tests/newtype_any_prefix.rs` (new), case
  `a_minted_entity_value_satisfies_an_any_of_prefixes`. Failing before: the case is unsupported on the current tree.
- **A single positive invariant is minted.** Transition: the same mint under `starts_with: "1"` alone (wide alphabet) and
  under `contains: "."` alone (alphabet `0123456789.`). Before: unsupported on both. After: passed on both. Same file, cases
  `a_minted_entity_value_satisfies_starts_with` and `a_minted_entity_value_satisfies_contains`.
- **No wrong verdict.** Transition: a `Version` whose positive invariants no text over the alphabet satisfies. After: still
  unsupported with the existing message, never passed or failed. Same file, case
  `an_unsatisfiable_text_rule_stays_unsupported`. Mutation check: a `minted` that returns its base text unchecked turns this
  case into a failed or passed verdict, so it is red against that mutant.
- **Nothing else moves.** Cells without a positive text invariant keep their reports, and every fixture's suite is
  byte-identical; existing cases in `tests/witness.rs` and `tests/interpreted_typed_identities.rs` keep their verdicts.

## Schedule

Release 0.70.0 (defect, wave A unit A4). The release tag on the artifact marks it.
