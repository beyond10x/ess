# Truthful conformance counts

A skipped scenario is not a failed one. `ess-conformance-report/1` carries `scenarios_total` and
`scenarios_failed` and no skipped count, so every non-pass is booked as a failure: a measured run of
340 scenarios with 275 skips and one genuine failure reports 276 failures, and `failed_scenarios`
holds 276 rows of which 275 are prefixed `skipped `. The real number is recoverable only by parsing
a prefix out of a string list, which is what a reader taking the count from the report is there to
avoid.

## The counts

`ess-conformance-report/2` carries the categories separately, and `scenarios_failed` means
failures:

| field | what it counts |
|---|---|
| `total` | every selected terminal result |
| `passed` | final passes |
| `failed` | final failures, and nothing else |
| `error` | runner or target errors |
| `unsupported` | observations the runner cannot make |
| `skipped` | scenarios the run did not execute |

`ess-conformance-run/2` is the separate detailed form, paired with the original suite bytes. A
category is a count and a list, so a reader that wants the ids does not parse them out of prose.

## Opt-in, and the old bytes

`/2` is explicit: `--report-format 2`, or `ESS_REPORT_FORMAT=2` for a generated runtime. Strict
conformance requires it, and asks for it by name rather than choosing silently —
`--allow-incomplete` is the way to keep diagnostic `report/1`.

`ess-conformance-report/1` is unchanged: same fields, same canonical bytes, same documented
meaning, including that `scenarios_failed` there counts every non-pass. A reader pinned to `/1`
reads what it always read. Nothing is rewritten in place, and no field is added to `/1` to make its
old meaning quietly different.

## Downstream

A reader that records evidence from a report — taking the count from the artifact rather than from
a person — is correct against `/2` and wrong against `/1` for any run with skips. Adopting `/2` is
therefore the reader's move as much as the runner's, and a reader that cannot yet import `/2`
should say so rather than book `/1`'s non-pass count as failures.
