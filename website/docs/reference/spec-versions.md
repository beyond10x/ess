---
title: Format version history
sidebar_position: 3
description: What each ESS format version number means, which release introduced it, and what an older reader does with a newer document.
---

# Format version history

An ESS document declares its own format in its bytes — `ess/10`, `ess-diff/7`,
`ess-conformance/17`. That number is the format's major version and nothing else. It is not the
release that produced the document, and not the specification version the document describes;
[Formats and digests](./formats.md) separates those three. This page says what each number changed,
which release introduced it, and what happens when an older reader meets a newer document.

## When the number moves

A format version is required when meaning, identity, references, canonicalization, names, or the
persisted envelope changes. A new internal capability that leaves the persisted shape alone does
not move it.

Every family is read by a build that states which versions it implements and refuses the rest. The
refusal is the point: a reader that accepts a shape it does not understand returns a wrong answer
about somebody's system, and blames the document for the age of the tool.

A version number is per family. `ess/10` and `ess-conformance/17` count
separately and always have.

## `ess/` — the authored specification

The format of the system a person writes. Read by `ess specify validate` and everything downstream
of it.

| Version | Released in | What changed | An older reader |
|---|---|---|---|
| `ess/1` | [0.1.0][r1] | The first format: types, commands, events, errors, views, entities. | — |
| `ess/2` | [0.20.0][r20] | Finite `Binary64` fields, distinct from integer and decimal values. Signed zero, subnormals and nearest-even rounding are preserved end to end. | Refuses `Binary64` at every declared type position, including as a map key. |
| `ess/3` | [0.23.0][r23] | `when_subject_state`, which combines a declared held lifecycle state with input guards. Opt-in binding accessors read two or three declared field segments from an event envelope. | Refuses the document. |
| `ess/4` | [0.23.0][r23] | Error wire names declared without merging semantic error identities. Typed command response fields fill emitted event payloads through explicit response mappings, with complete payload ownership. | Refuses the document. |
| `ess/5` | [0.27.0][r27] | An enum variant carries its own `wire`, `display`, `summary` and `code`. A variant is authored as a bare name or as a mapping. | Refuses with `unsupported_format_version` at `types.<type>.variants.<variant>`. |

`ess/6`, introduced in [0.28.0][r28], admits an input eligibility predicate beside an external
cause. The fault remains independently arranged; the guard does not select the observed outcome.
Earlier source formats refuse this combination. It also admits `when_subject` over a
declared enum field of an existing subject and `preserves` for a successful silent no-op.
History is observed independently from lifecycle. The bounded arrangement search preserves
different histories that reach the same state; unknown or unobservable history refuses synthesis.
A preserving outcome cannot assign fields, emit events, or declare an error.

`ess/7`, introduced in [0.29.0][r29], adds command-local `replays`, which returns the originating
success's retained typed result without repeating its effects. The origin supplies
the observable subject identity; the retry cannot invent another selector. It also
admits an effect-free named error as the finite default complement of explicit
subject-state branches sharing one existing subject. Earlier formats refuse these
constructs. A replay response containing Decimal or Binary64, including through
nested declarations, is outside the exact-result observation profile and refuses
synthesis. This does not change existing response-to-event comparisons.

`ess/8`, unreleased, admits the string operators `starts_with`, `ends_with` and `contains` in
every predicate position: command guards, invariants, view filters and binding selections. They
are map form only and apply to `String` and newtypes of it. Earlier source formats refuse them with
`unsupported_format_version` at the position that uses one. A model that uses none keeps its bytes
and its compiled digest. See [string operators](./predicates.md#string-operators).

`ess/9`, not yet released, admits `when_subject: {predicate: …}`: a predicate over the declared
stored fields of the subject a command addresses, read immediately before selection and
conjunctive with an ordinary `when:`. It reads the entity's fields and nothing else — not the
input, not `state`. A refusal may carry it without naming a subject of its own; it reads the one its
sibling branches name. Closed stored-field domains enter the branch partition beside the input, and
an open one, such as `weight_kg > 20`, needs a genuine default. Conformance arranges a row to the
guard through the arranging commands' `sets:` mappings and observes it before the command runs. An
older build refuses the header; this build refuses the predicate form under an earlier header with
`unsupported_format_version`. `{field, equals}` keeps `ess/6` and its bytes.

`ess/10`, not yet released, admits aggregate views: a view field may declare `aggregate:` — one of
`count`, `count_distinct`, `sum`, `min`, `max` and `avg` — and a view may declare `group_by:` over
its other fields. The filter runs per source row, the admitted rows are grouped, and a group with no
admitted row is absent; a view without `group_by` returns exactly one row. Each aggregate field
declares its result type exactly: `Integer` for the counts and for `sum` of an `Integer`, `Optional<T>`
for `min` and `max`, and `Optional<Decimal>` for `avg`, rounded to 6 places half-even. An older build
refuses the header, and this build refuses the construct under an earlier header with
`unsupported_format_version`. A model without it keeps its bytes and its compiled digest. See
[aggregate views](../guides/write-a-specification.md#aggregate-views).

`ess/3` and `ess/4` both arrived in 0.23.0. There was never a release that implemented `3` and not
`4`, and there is no missing release between them.

A bare variant list is admitted by every version, and a variant that declares no naming serializes
back as a bare name, so a specification written before `ess/5` keeps its exact bytes.

## `ess-diff/` — what moved between two revisions

| Version | Released in | What changed | An older reader |
|---|---|---|---|
| `ess-diff/1` | [0.1.0][r1] | The first delta format. | — |
| `ess-diff/2` | [0.19.0][r19] | Previously omitted constructs, served views and reusable row shapes are accounted for. Sliced provenance moves to the explicit `slice-sha256/2:` digest profile. | Regenerate sliced artifacts; a bare legacy slice digest is refused. |
| `ess-diff/3` | [0.23.0][r23] | Cause, selection-plan and reading-contract deltas. | Refuses the delta. |
| `ess-diff/4` | [0.23.0][r23] | Error and response deltas. | Refuses the delta. |
| `ess-diff/5` | [0.27.0][r27] | `VariantWireNameChanged`, `VariantDisplayNameChanged` and `VariantSummaryChanged` on `TypeChange`. | Refuses a delta carrying any of the three. |
| `ess-diff/6` | [0.29.0][r29] | Typed deltas retain the before/after originating replay relation and complete refusal-observation requirement. | Refuses the new vocabulary; existing changes retain their earlier format. |
| `ess-diff/7` | unreleased | `GroupingChanged` and `FieldAggregateChanged` on `ViewChange`: an aggregate view's group keys and what one field computes. | Refuses a delta carrying either. |

`ess-diff/5` exists because a variant's own name does not move when its wire spelling does. Before
it, the variant set and the variant order both said nothing, and the comparison returned an empty
delta for a change that breaks a deployed consumer.

## `ess-conformance/` — the suite

The family that has moved most, because its step and expectation vocabulary is exactly what the
persisted document's shape is.

| Version | Released in | What changed |
|---|---|---|
| `ess-conformance/1` | [0.1.0][r1] | The canonical scenario IR. |
| `ess-conformance/2` | [0.7.0][r7] | The expectation vocabulary grew. |
| `ess-conformance/3` | [0.16.0][r16] | The step vocabulary grew. |
| `ess-conformance/4` | [0.18.0][r18] | The step vocabulary grew again. Written as 0.17.0 and released in 0.18.0; the CHANGELOG section says so. |
| `ess-conformance/5` | [0.21.0][r21] | Selected generated and authored coverage, omitted scenarios, source identities and every refusal occurrence. |
| `ess-conformance/6` | [0.23.0][r23] | Authored `ess-scenario/2` entity setup, as an ordinary suite. |
| `ess-conformance/7` | [0.23.0][r23] | The same, with declared coverage. |
| `ess-conformance/8` | [0.23.0][r23] | Corrected structured text comparison; response-mapping value comparison. |
| `ess-conformance/9` | [0.23.0][r23] | The same, with declared coverage. |

`ess-conformance/10` and `ess-conformance/11` arrive in [0.28.0][r28]. Version 10 adds
`expect_no_error`, `snapshot_subject`, and `expect_subject_unchanged`; version 11
carries the same steps with coverage. Snapshots capture exactly one actual row by
identity before a command and compare every returned field afterward. Synthesis
requires immediate views covering all subject fields, including generated values.
Missing or duplicate rows fail. Legacy suite formats refuse these steps.

`ess-conformance/12` and `/13`, introduced in [0.29.0][r29], add exact retained-result capture and
comparison, plus an explicit empty-direct-event assertion. Version 12 is ordinary;
13 carries the same declared coverage and exact-parent rules as earlier coverage
formats. Rust and Go execute these steps with report/2; TypeScript/browser readers
refuse these envelopes before target callbacks. Older envelopes refuse the new
steps even if the rest of their document is well shaped.

A write-once snapshot binds the actual original response, command/outcome, subject
identity, input and actor. Retry comparison requires the admitted original values,
no error and no direct events, including unknown event names. Paired subject
snapshots compare the complete original subject independently. Integers compare
exactly, text retains its admitted spelling, collections compare recursively, and
an absent optional field differs from explicit null. Decimal and Binary64 are not
admitted in this response profile. This immediate witness cannot distinguish a
current-head result while it still equals the original; adopters must separately
test later-head and restart retries through their real handlers.

`ess-conformance/14` and `/15`, unreleased, carry a string operator where a suite carries a
predicate: a `satisfies` expectation or an observed selection plan. Version 14 is ordinary and 15
carries declared coverage; each implies every major below it. Rust and Go admit and evaluate them,
and refuse an operand that is not a JSON string. Older envelopes refuse the operators, and the
TypeScript and browser readers refuse these envelopes by their version. A string guard over
command input is decided at synthesis and never reaches the suite, so such a suite keeps its
earlier format.

`ess-conformance/16` and `/17`, unreleased, carry the `<view>/aggregate` scenario: rows created
through the declared creating outcome with values only that scenario uses, and one read asserting
every group's exact aggregates and the absence of every group whose rows the filter refuses.
Version 16 is ordinary and 17 carries declared coverage; each implies every major below it.
Coverage 17 also carries the refusals `ESS-SYNTH-016` (no group key or parameter scopes the
view's rows) and `ESS-SYNTH-017` (the rows cannot be arranged). Rust and Go admit and run them;
older envelopes refuse an aggregate scenario or refusal, and the TypeScript and browser readers
refuse these envelopes by their version.

For `ess/7`, generated held-state refusals include ordinary `wrong_state` outcomes:
they compare the complete subject before and after the call and refuse every
direct event, including undeclared names. Incomplete subject views cause a named
synthesis refusal. Earlier source formats retain their existing witness bytes.

Integer exactness starts at the actual target adapter: it must preserve each
handler value without a floating-point conversion before supplying the typed
response. Native adapters can construct numbers directly from integers. A JSON
adapter must decode declared Integer values losslessly or report the observation
Unsupported; generic JSON-to-Node conversion does not provide this guarantee.
Typed response parity does not claim equivalent parsing of arbitrary JSON number
spellings. Existing numeric serialization remains unchanged.

Versions `6` through `9` all arrived in 0.23.0. They are two capabilities crossed with the
ordinary/coverage distinction, not four separate releases.

Two readers fail differently at the same document, which is why the number exists at all. An old
Rust reader parses a closed tagged enum and fails with `unknown variant` — a message that blames
the document. An old Go runner abandons a scenario whose first word it does not know and reports it
skipped, which is the right answer reached by accident. Neither is a verdict anyone should act on.

## `ess-normalization/` — normalization recipes

| Version | Released in | What changed |
|---|---|---|
| `ess-normalization/1` | [0.19.0][r19] | Strict recipes, explicit external dispatch, ordered input/output schema boundaries, separate missing/null behavior. |
| `ess-normalization/2` | [0.20.0][r20] | Ordered string concatenation and joining, exact integer rendering, list concatenation, original collection indices, filtered mapping, first-match selection. |
| `ess-normalization/3` | [0.20.0][r20] | Model-owned stage roots, pinned to complete compiler provenance and explicit type selections. |
| `ess-normalization/4` | [0.20.0][r20] | Selected JSON field, array-item or root tokens captured as canonical standard base64 before first-stage validation. |
| `ess-normalization/5` | [0.20.0][r20] | Explicit input paths; token-preserving `binary64_literal` constants and finite `binary64` conversion steps. |
| `ess-normalization/6` | [0.20.0][r20] | Fixed string array preparation and checked `position` reads. |

Generated maps for the earlier formats stay byte-identical at the same generator version.

## Envelopes revised once

| Format | Revised in | What the later version carries |
|---|---|---|
| `ess-conformance-report/` | [0.19.0][r19] | `/2` separates passed, failed, error, unsupported and skipped counts, bound to the exact executed suite bytes. |
| `ess-schema-bundle/` | [0.19.0][r19] | `/2` is a document-root import with explicit, replay-checked root identity. Component-bundle `/1` bytes are unchanged. |
| `ess-impact/` | [0.19.0][r19] | `/3` versions the corrected dependency vocabulary and the embedded delta. |
| `ess-conformance-run/` | [0.20.0][r20] | `/2` is the checked detailed run output. |
| `ess-target-failure/` | [0.20.0][r20], [0.23.0][r23] | `/2`, then `/3` with the `accessor-resource` cause. |
| `ess-scenario/` | [0.23.0][r23] | `/2` authored setup establishes typed, isolated backend entity rows. |
| `infra-observation/` | [0.1.0][r1], next release | `/2` is a reduced, deliberately partial recovery profile, not a superset of `/1`. `/3` is the full scan with each Secret value recorded as `{"present": true}`: the key name, no digest, no length. `/1` wrote each value's unsalted SHA-256 and byte length, which confirm a guessed low-entropy secret to anyone holding the file. Same fields, new meaning, so a `/1` reader must reject `/3`; this build still reads `/1` and discards its digests. |
| `infra-ir/` | next release | `/3` records each Secret key as present and nothing derived from its value, and is what every full observation with a Secret key compiles to, `/1` included. An IR without a Secret key keeps `/1` and its bytes. A persisted `/1` still reads, returned as `/3` with its Secret digests dropped and a different model digest, so nothing derived from it chains to the `/1` file's own digest; so drift reports a Secret's added and removed keys and never a changed value. An older reader refuses `/3`. |
| `infra-drift/` | next release | `/2` is the namespace topology profile. `/3` is the full-scan comparison with one meaning changed: a Secret's `changed_keys` is always empty, so an empty list means the value is unknown, where under `/1` it meant not rotated. Serialize-only; `/1` documents already written keep their meaning. |
| `ess-observed-bindings-report/` | next release | `/2` adds `OBS-BIND-008`: a container or native sidecar in a bound workload that no binding names is a violation. Same fields, new semantics; a document satisfied under `/1` can be violated under `/2`, so a `/1` reader must reject `/2`. The authored `ess-observed-bindings/1` input keeps its version and fields; it now claims the bound workload runs nothing else. |

## Still at version 1

Never revised, and a document that claims a higher number is refused:
`ess-realization/1`, `ess-realization-ir/1`, `ess-composition/1`, `ess-client-plan/1`,
`ess-docs/1`, `ess-build/1`, `ess-build-ir/1`, `ess-component/1`, `ess-component-ir/1`,
`ess-release/1`, `ess-release-bundle/1`, `ess-release-catalog/1`, `ess-runtime/1`,
`ess-runtime-ir/1`, `ess-stack/1`, `ess-stack-lock/1`, `ess-environment/1`, `ess-deployment/1`,
`ess-service-interface/1`, `ess-openapi-import/1`, `ess-browser-catalog/1`,
`ess-conformance-input/1`, `ess-conformance-replay/1`, `infra-spec/1`,
`infra-graph/1`, `infra-simulation/1`, `infra-projection/1`.

## Release numbers this page cites

Five versions appear in the [CHANGELOG][changelog] with no GitHub Release behind them.

| Version | Why | Where its code shipped |
|---|---|---|
| 0.3.0 | tagged in one clone, never pushed | 0.4.0 |
| 0.5.0 | tagged in one clone, never pushed | 0.5.1 |
| 0.15.0 | tagged in one clone, never pushed | 0.16.0 |
| 0.21.0 | tag pushed; the tree does not pass the current gate, so no release can be cut from it | the tag itself |
| 0.26.1 | never tagged | 0.27.0 |

`ess-conformance/5` is the one format version introduced by any of them, and the 0.21.0 tag
carries it. Everything else in the tables above sits on a version with a published release.

The tag `v0.3.0` is a fifth artifact of the same period, under the naming convention that
preceded bare versions. The release workflow triggers on bare versions only, so it never asked
for a release and promises none.

[changelog]: https://github.com/beyond10x/ess/blob/main/CHANGELOG.md
[r1]: https://github.com/beyond10x/ess/releases/tag/0.1.0
[r7]: https://github.com/beyond10x/ess/releases/tag/0.7.0
[r16]: https://github.com/beyond10x/ess/releases/tag/0.16.0
[r18]: https://github.com/beyond10x/ess/releases/tag/0.18.0
[r19]: https://github.com/beyond10x/ess/releases/tag/0.19.0
[r20]: https://github.com/beyond10x/ess/releases/tag/0.20.0
[r21]: https://github.com/beyond10x/ess/releases/tag/0.21.0
[r23]: https://github.com/beyond10x/ess/releases/tag/0.23.0
[r27]: https://github.com/beyond10x/ess/releases/tag/0.27.0

[r28]: https://github.com/beyond10x/ess/releases/tag/0.28.0

[r29]: https://github.com/beyond10x/ess/releases/tag/0.29.0
