---
format: aep.planning-md/3
id: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
kind: story
status: active
title: ess/37 and the ess-conformance/64-65 suite pair are open for the epic's constructs
tags:
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- informed_by: review-result:zero-issues-design-round-1
- serves: vision:O2
scope:
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
  path: crates/verify/ess-conformance/tests/count_reports.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/direct_response_constraints.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/event_multiplicity.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/field_presence.rs
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
  path: website/docs/start/first-specification.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:44Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
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

Re-derived 2026-11-10 by `story-scoper` on the integration tree, from the reader sites, format rows and byte pins and the two commits that last opened a pair and a format (8a4a93fb7d, 278bd73be9).

- cited: crates/specify/ess-domain/src/system.rs (`FORMAT_HISTORY`, the `ess/37` row; `system.rs:60-61` feeds `cargo xtask format-history`)
- cited: crates/verify/ess-conformance/src/scenario.rs (`SUPPORTED_SUITE_FORMATS` at 559-562 and its test at 3617)
- cited: crates/verify/ess-conformance/src/admission.rs (210-214 and 276-284)
- cited: crates/verify/ess-conformance/src/go/mod.rs (`admitted_major` at 445, the shared admission table, 488-493)
- cited: crates/verify/ess-conformance/src/count_json.rs
- cited: crates/verify/ess-conformance/src/coverage.rs (17-50 lists every coverage major; /65 is added)
- cited: crates/verify/ess-conformance/src/synthesis_seeds.rs (98-115 and 185)
- cited: crates/verify/ess-diff/src/delta.rs (17)
- cited: crates/verify/ess-diff/src/change.rs (the delta-format tests the acceptance names)
- cited: crates/edge/ess-xtask/src/docs.rs (`FORMAT_RELEASES` at 147 needs the `ess-diff/24`, `ess-conformance/64` and `/65` rows; 69-71, 612-617)
- cited: models/toolchain/domains/specify.yaml (42; `model_enums.rs:220` holds it equal to `SUPPORTED_FORMATS`)
- cited: website/docs/start/first-specification.md (24 and 120; `tutorial_page.rs:858-870` requires the newest format)
- cited: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv (the `models/toolchain` rows move with `specify.yaml`, as in 278bd73be9 and ec2cb77cad)
- cited: crates/verify/ess-conformance/tests/one_time_reader_cells.rs (544)
- cited: crates/verify/ess-conformance/tests/system_issued_vocabulary.rs (181-194)
- cited: crates/verify/ess-conformance/tests/field_presence.rs (98)
- cited: crates/verify/ess-conformance/tests/leaf_payloads.rs (333)
- cited: crates/verify/ess-conformance/tests/subject_guard_input.rs (476)
- cited: crates/verify/ess-conformance/tests/suite.rs (765)
- cited: crates/verify/ess-conformance/tests/count_reports.rs (520)
- cited: crates/verify/ess-conformance/tests/undeclared_fields.rs (609-625)
- cited: crates/verify/ess-conformance/tests/support_seeds/mod.rs (607-614)
- cited: crates/verify/ess-conformance/tests/adversary_generated_docs_pass2.rs (160 expects 64 and 65 refused)
- inferred: crates/verify/ess-conformance/src/go/runtime.go (cap at 104)
- inferred: crates/verify/ess-conformance/src/ts/runtime.ts (7079-7189)
- inferred: crates/verify/ess-conformance/src/ts/one_time_response.ts
- inferred: crates/verify/ess-conformance/src/ts/runtime.test.ts
- inferred: crates/verify/ess-conformance/src/ts/predicate.test.ts
- inferred: crates/verify/ess-conformance/tests/direct_response_constraints.rs
- inferred: crates/verify/ess-conformance/tests/event_multiplicity.rs
- inferred: crates/verify/ess-conformance/tests/runtime_parity_typescript_28_35.rs
- inferred: crates/edge/ess-cli/tests/support/coverage_producer.rs
- inferred: crates/verify/ess-conformance/src/lib.rs (a pair module line, as `member_paths.rs:40-49` did)
- inferred: crates/verify/ess-conformance/src/suite_pair_64.rs (new; the name is a placeholder)
- inferred: crates/edge/ess-cli/tests/epic_formats_open.rs (new; the three acceptances)
- inferred: website/docs/reference/formats.md

confidence: medium — the Rust reader sites, format rows and byte pins are read from the tree; whether the Go and TypeScript runtimes move here is a decision (below), and the names of the two new files are guesses (story-scoper, 2026-11-10)

Coordinator decision, 2026-11-10: the Rust reader and the Go and TypeScript emitters share one admission table (`go::admitted_major`, `admission.rs:210`, `go/mod.rs:488-493`, `ts/mod.rs:66-68`), so opening /64 and /65 makes both emitters write runners for /64 suites while their runtimes stop at 63 (`go/runtime.go:104`, `ts/runtime.ts:7079-7189`). This story moves the Go and TypeScript runtime caps in step, as 8a4a93fb7d did; `story:the-go-runtime-checks-a-view-rows-shape` and `story:the-typescript-runtime-checks-a-view-rows-shape` (later turns) build on that.

Corrections to the earlier pass: removed `crates/verify/ess-conformance/src/coverage_build.rs` (only writes the coverage major; the acceptance says no synthesis writes /64 or /65) and `schemas/generated/ess.schema.json` (`FormatVersion` is a pattern; an empty format does not change `RawSpecFile`); `coverage.rs` moves from inferred to cited; the reader sites, the format-bump pins and the runtime caps above are added. `synthesize.rs` is not touched: nothing in it names a suite major.

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`; `website/docs/reference/spec-versions.md` (generated from `FORMAT_HISTORY`; the `ess-diff/24` row beside `:508`); the `models/toolchain` rows of `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` and `synthesis-component-suites-table.tsv`, which move because `specify.yaml` changes the toolchain model's digest.

## Shared surfaces after this story

The `ess/37` row of `FORMAT_HISTORY` (`crates/specify/ess-domain/src/system.rs`), the generated
`website/docs/reference/spec-versions.md` table and `CHANGELOG.md` are append-only for the stories
of this epic: each implementor proposes its sentence, and the coordinator writes and reconciles it
at merge. They are not a typed scope entry of those stories, so `aep plan artifact waves` does not
serialise every format story on them.
