---
format: aep.planning-md/3
id: task:open-ess-37-and-conformance-64-65
kind: task
status: archived
title: Open ess/37 and the ess-conformance/64-65 suite pair for the epic's constructs
tags:
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- informed_by: review-result:zero-issues-design-round-1
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:28:45Z", actor: "human:timo", revision: 2}
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

This task opens the two formats, empty of constructs, so the stories that fill them do not race to
open them.

## Acceptance

- `ess specify formats` lists `ess/37` with no release, and a specification declaring
  `format: ess/37` validates exactly as it does at `ess/36` (`ess specify validate` exit 0 on
  `examples/billing` re-declared at `ess/37`; IR bytes equal apart from the format field).
- `ess-conformance/64` and `ess-conformance/65` are read wherever `/62` and `/63` are read
  (`ess verify conform run` on a suite whose `suite_version` is rewritten to `/64`, resp. a coverage
  suite at `/65`, gives the same verdicts), and no synthesis writes them yet: a suite's major is still
  chosen by the features its specification uses.
- `cargo test -p ess-domain --locked` and the format-history check (`cargo xtask format-history
  --check`, through `task projection-check`) pass with the new row.

## Scope

- cited: crates/specify/ess-domain/src/system.rs
- cited: crates/verify/ess-conformance/src/count_json.rs
- cited: crates/verify/ess-conformance/src/coverage_build.rs
- inferred: crates/verify/ess-conformance/src/coverage.rs
- inferred: website/docs/reference/spec-versions.md
- inferred: schemas/generated/ess.schema.json

confidence: medium

## Shared surfaces after this task

The `ess/37` row of `FORMAT_HISTORY` (`crates/specify/ess-domain/src/system.rs`), the generated
`website/docs/reference/spec-versions.md` table and `CHANGELOG.md` are append-only for the stories
of this epic: each implementor proposes its sentence, and the coordinator writes and reconciles it
at merge. They are not a typed scope entry of those stories, so `aep plan artifact waves` does not
serialise every format story on them.
