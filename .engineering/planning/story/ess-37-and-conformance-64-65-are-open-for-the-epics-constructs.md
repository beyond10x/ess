---
format: aep.planning-md/3
id: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
kind: story
status: implemented
title: ess/37 and the ess-conformance/64-65 suite pair are open for the epic's constructs
tags:
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- informed_by: review-result:zero-issues-design-round-1
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/epic_formats_open.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/support/coverage_producer.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/count_json.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/suite_pair_64.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesis_seeds.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/one_time_response.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.test.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_generated_docs_pass2.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t1_pair_64_65.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/count_reports.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/direct_response_constraints.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/event_multiplicity.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/field_presence.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/leaf_payloads.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/one_time_reader_cells.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/runtime_parity_typescript_28_35.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/subject_guard_input.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/suite.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/support_seeds/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/system_issued_vocabulary.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/undeclared_fields.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
- confidence: cited
  path: website/docs/start/first-specification.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:44Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:30Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Need

The stories of `epic:zero-open-issues-2026-11` land on one integration branch and reach `main` in
one pull request, so they reach adopters in one release. 28 of them open "the next source format"
and about 55 a new conformance suite format, each in its own words (`ess/37`, "the next open
format", "G2's format", "the next suite pair"), and nothing says which number each acceptance reads
(review-result:zero-issues-design-round-1, first finding).

Decision (coordinator, 2026-11-09, recorded in the run's non-interactive approval-record): every
source-language change of this epic lands in **one** new source format, `ess/37`, and every suite
change in **one** new suite pair, `ess-conformance/64` (ordinary) and `ess-conformance/65`
(coverage). `docs/design/generic-language-features.md` § policy 2 gives one format per design
decision so that formats stop churning (its § "Format churn"); one release carrying G2's and G3's
remainders and the single-request constructs in one format is fewer formats than one per decision,
and no construct gets a format of its own. Below `ess/37` every new spelling stays refused as
`unsupported_format_version` naming `ess/37`, as each story's own acceptance says.

Every new `ess verify diff` change kind of this epic lands in **one** new delta version,
`ess-diff/24`, the same way (review-result:zero-issues-design-round-2).

This story opens the three versions, empty of constructs, so the stories that fill them do not race
to open them.

## Acceptance

- `ess specify formats` lists `ess/37` with no release, and a specification declaring
  `format: ess/37` validates exactly as it does at `ess/36` (`ess specify validate` exit 0 on
  `examples/billing` re-declared at `ess/37`; IR bytes equal apart from the format field).
- `ess-conformance/64` and `ess-conformance/65` are read wherever `/62` and `/63` are read
  (`ess verify conform run` on a suite whose `suite_version` is rewritten to `/64`, resp. a coverage
  suite at `/65`, gives the same verdicts), and no synthesis writes them yet: a suite's major is still
  chosen by the features its specification uses.
- `ess verify diff` reads and writes `ess-diff/24` wherever it reads and writes `ess-diff/23`, and no
  change kind is written at `/24` yet (`cargo test -p ess-diff --locked`, the delta-format tests
  in `crates/verify/ess-diff/src/change.rs`, with a case for `/24`).
- `cargo test -p ess-domain --locked` and the format-history check (`cargo xtask format-history
  --check`, through `task projection-check`) pass with the new row.

## Scope

Delivered in turn 1 of the zero-open-issues plan (merge 4dded413eb, corrections 209583be9d 24d5cb0f6f 55e3856e5b). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/edge/ess-cli/tests/adversary_t1_epic_formats.rs`
- cited, confirmed: `crates/edge/ess-cli/tests/epic_formats_open.rs`
- cited, confirmed: `crates/edge/ess-cli/tests/support/coverage_producer.rs`
- cited, confirmed: `crates/edge/ess-xtask/src/docs.rs`
- cited, confirmed: `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`
- cited, confirmed: `crates/specify/ess-domain/src/system.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/admission.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/count_json.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/coverage.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/go/mod.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/go/runtime.go`
- cited, confirmed: `crates/verify/ess-conformance/src/lib.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/scenario.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/suite_pair_64.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/synthesis_seeds.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/ts/one_time_response.ts`
- cited, confirmed: `crates/verify/ess-conformance/src/ts/predicate.test.ts`
- cited, confirmed: `crates/verify/ess-conformance/src/ts/runtime.test.ts`
- cited, confirmed: `crates/verify/ess-conformance/src/ts/runtime.ts`
- cited, confirmed: `crates/verify/ess-conformance/tests/adversary_generated_docs_pass2.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/adversary_t1_pair_64_65.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/count_reports.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/direct_response_constraints.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/event_multiplicity.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/field_presence.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv`
- cited, confirmed: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`
- cited, confirmed: `crates/verify/ess-conformance/tests/leaf_payloads.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/one_time_reader_cells.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/runtime_parity_typescript_28_35.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/subject_guard_input.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/suite.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/support_seeds/mod.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/system_issued_vocabulary.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/undeclared_fields.rs`
- cited, confirmed: `crates/verify/ess-diff/src/change.rs`
- cited, confirmed: `crates/verify/ess-diff/src/delta.rs`
- cited, confirmed: `models/toolchain/domains/specify.yaml`
- cited, confirmed: `website/docs/reference/formats.md`
- cited, confirmed: `website/docs/reference/spec-versions.md`
- cited, confirmed: `website/docs/start/first-specification.md`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Shared surfaces after this story

The `ess/37` row of `FORMAT_HISTORY` (`crates/specify/ess-domain/src/system.rs`), the generated
`website/docs/reference/spec-versions.md` table and `CHANGELOG.md` are append-only for the stories
of this epic: each implementor proposes its sentence, and the coordinator writes and reconciles it
at merge. They are not a typed scope entry of those stories, so `aep plan artifact waves` does not
serialise every format story on them.
