---
title: Format version history
sidebar_position: 3
description: What each ESS format version number means, which release introduced it, and what an older reader does with a newer document.
---

# Format version history

An ESS document declares its own format in its bytes — `ess/6`, `ess-diff/5`,
`ess-conformance/11`. That number is the format's major version and nothing else. It is not the
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

A version number is per family. `ess/6` and `ess-conformance/11` are both current; they count
separately and always have.

The unreleased fixture-values candidate adds `ess/7`, `ess-scenario/3`,
`ess-conformance/12` and `ess-conformance/13`. These versions are not in the published 0.28.0 release.
Command `fixture_inputs` declarations and explicit authored `$fixture` references resolve
typed values from an independent provider before scenario startup. Requests and assertions
use the same copied values; an observed result cannot supply its own expectation.
Suite 12 carries this vocabulary, and suite 13 also carries coverage. Older readers refuse
the new formats; fixture-free suites keep their previous format. Browser replay refuses fixtures.

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
| `infra-observation/` | [0.1.0][r1] | `/2` is a reduced, deliberately partial recovery profile, not a superset of `/1`. |

## Still at version 1

Never revised, and a document that claims a higher number is refused:
`ess-realization/1`, `ess-realization-ir/1`, `ess-composition/1`, `ess-client-plan/1`,
`ess-docs/1`, `ess-build/1`, `ess-build-ir/1`, `ess-component/1`, `ess-component-ir/1`,
`ess-release/1`, `ess-release-bundle/1`, `ess-release-catalog/1`, `ess-runtime/1`,
`ess-runtime-ir/1`, `ess-stack/1`, `ess-stack-lock/1`, `ess-environment/1`, `ess-deployment/1`,
`ess-service-interface/1`, `ess-openapi-import/1`, `ess-browser-catalog/1`,
`ess-conformance-input/1`, `ess-conformance-replay/1`, `infra-ir/1`, `infra-spec/1`,
`infra-graph/1`, `infra-drift/1`, `infra-simulation/1`, `infra-projection/1`.

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
