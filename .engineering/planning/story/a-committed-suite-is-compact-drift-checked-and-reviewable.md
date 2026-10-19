---
format: aep.planning-md/3
id: story:a-committed-suite-is-compact-drift-checked-and-reviewable
kind: story
status: draft
title: A committed suite is compact, drift-checkable and reviewable as a change
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#570
- provider: github
  reference: beyond10x/ess#571
- provider: github
  reference: beyond10x/ess#572
relations:
- serves: vision:O2
revision: 3
---
## Outcome

A committed `--target ir` suite stays reviewable: its size is proportional to what it asserts (rules
that hold for every scenario, such as the declared event catalog, the one-time response authority
and "scan every declared event after each operation", are written once per suite, not per scenario
per event per operation); `ess verify conform synthesize --check` drift-checks a committed suite
byte for byte; a command describes what changed between two suites (scenarios added, removed and
changed, grouped by construct, with a JSON form); and the commit guide has a row for the `--target
ir` suite.

Fits none of the four generic language features: suite format, tooling and guidance.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/570, https://github.com/beyond10x/ess/issues/571
and https://github.com/beyond10x/ess/issues/572 (a downstream IAM service specification, ess 0.59.0).

1. **Need.** One `one_time_response` field took the suite from 145 to 918 scenarios and 37 MB (20 MB
   `--compact`), about 40 KB per scenario against about 4 KB in a minimal model. The weight is the
   one-time trace: the full event catalog and up to 2 x E x S event windows per scenario, and the
   complete response authority per origin (https://github.com/beyond10x/ess/issues/570, citing
   `crates/verify/ess-conformance/src/one_time_response/produce.rs:10-97` at 0.59.0). A textual diff
   of such a suite cannot be reviewed, and no command compares two suites
   (https://github.com/beyond10x/ess/issues/572). The commit table covers Go and TypeScript suites,
   not `--target ir`, and `synthesize` has no `--check`
   (https://github.com/beyond10x/ess/issues/571, citing
   `website/docs/guides/commit-generated-files.md:22`).
2. **Class.** Defect for the size (the design asks for the scan, `docs/design/one-time-response-values.md:114-116`,
   not for its enumeration); gap for `--check` and the suite diff; documentation gap for the row.
3. **Already expressible?** `--compact` exists (`story:conformance-compact-json-output`); it removes
   whitespace only. `ess verify impact --suite` lists scenarios put back to owed and by design not
   additions or step changes (`crates/verify/ess-diff/src/impact.rs:1-40`).
4. **Fit.** The same argument was made on 2026-09-11 (`story:a-synthesized-suite-has-a-compact-form`,
   archived, folded): 93% of steps were repeated negative obligations, and its option 2 ("carry the
   set once") is this design. `--check` reuses `ess generate --check` (`story:ess-generate-check`).
   The suite diff is a reader of two suites, beside `ess verify diff` (two specifications) and
   `impact`. A suite format version is required for the suite-level catalog; the Go and TypeScript
   runners read it (`story:go-and-typescript-read-current-suites`).
5. **Second adopter.** The 13 MB internal suite pair of 2026-09-11; any model with many events.
6. **Cost.** One suite format version; runner support in Go, TypeScript and the interpreter; one new
   subcommand and one flag; a guide row.
7. **Alternatives.** (a) Change nothing: adopters stop committing suites, and `ess verify impact`
   loses its earlier revision. (b) Guidance only (commit `--compact` or a digest). (c) Encode rules
   once, `--check`, a suite diff and the guide row (chosen); (b) ships first as the guide row.

## Decisions

Accept, redesigned: the requester's three asks are kept; the suite-level encoding is the format
change, and the order of growth (cells x events x operations) is documented in the one-time
response guide by `story:one-time-disclosure-inventory-is-published`.

## Acceptance

- `suite_states_event_catalog_once`: the https://github.com/beyond10x/ess/issues/550 minimal model and
  a model with ten events synthesize suites whose per-scenario size does not grow with the event
  count; every runner passes and fails exactly as before on the existing mutants.
- `synthesize_check_reports_drift`: `ess verify conform synthesize --check` exits non-zero on a
  committed suite that differs, zero on one that does not, writing nothing.
- `conform_diff_groups_changes_by_construct`: two suites from two revisions yield added, removed and
  changed scenario ids per construct, in text and JSON.
- `commit_guide_names_the_ir_suite`: the commit table has a `--target ir` row with its reason.

## Order

Wave 4 of `release-plan:ess-063-downstream`, independent of the source format. The suite-level catalog is a
suite format change: it rides the batch's open pair `ess-conformance/52`,`/53` if it lands before
the release closes them (the pair then carries the await steps, the wrapped-invariant observations
and the once-per-suite rules together), otherwise it opens `/54`,`/55`. The Go and TypeScript
runners and the interpreter read the new envelope; `ess verify conform synthesize --check` and the
suite diff command need no format. Not in flight on 2026-10-18; no branch holds any of it.
