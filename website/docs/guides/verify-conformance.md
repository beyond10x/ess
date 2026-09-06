---
title: Verify conformance
description: Generate the semantic suite a specification requires, run it, and emit a standalone ESS conformance report.
---

# Verify conformance

A specification declares more than an interface shape. Command outcomes, refused branches, state
transitions, emitted events, and invariants become semantic scenarios an implementation can be
checked against.

## Generate the suite

```shell-session
$ ess verify conform synthesize \
    --path examples/billing \
    --out target/billing-suite.json
```

The suite is deterministic. Its provenance names the model and contract digests from which it was
derived.

## Run a supported target

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-out target/billing-conformance.json
```

The command executes the generated or committed scenarios against the selected reference target.
The default standalone report is `ess-conformance-report/1`. Its historical `scenarios_failed`
count includes every non-pass, including Go skips and Rust errors or unsupported results. Those
legacy bytes and meanings remain unchanged.

To run directly from a specification instead of a pre-generated suite:

```shell-session
$ ess verify conform run \
    --path examples/billing \
    --target billing \
    --format json
```

## Opt into explicit outcome counts

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-format 2 \
    --allow-incomplete \
    --report-out target/billing-counts.json \
    --format json
```

`--report-format 2` selects `ess-conformance-report/2` for `--report-out`. It records separate
`passed`, `failed`, `error`, `unsupported` and `skipped` counts and sorted scenario IDs. Rust uses
error/unsupported; generated Go uses skipped and keeps ordinary target errors as failed. The report
binds every original suite byte, including its final newline, under `sha256-json-bytes/1`. Retain the
original suite beside the report; reformatting it changes that identity.

The admitted suites remain versions 1–4. Their coverage is exactly `unknown`, so even an empty or
all-pass execution has inconclusive conformance. `--allow-incomplete` explicitly chooses diagnostic
execution: Rust exits 0 for passed execution, 1 for failed and 3 for inconclusive execution.
`--strict` requires explicit `--report-format 2` and succeeds only for passed conformance; unknown
coverage therefore exits nonzero. Strict and allow-incomplete cannot be combined. Neither control
changes the report format automatically, and suite/5 is not admitted at this stage.

With report format 2, `--format json` or `--format yaml` presents a separate closed
`ess-conformance-run/2` object containing `format`, the standalone `summary`, `started_at` and ordered
`scenarios`. `--report-out` always writes the standalone JSON summary. Both new formats preserve
unsigned epoch milliseconds exactly through the full u64 range. The default detailed output remains
the legacy unversioned structure.

Regenerate a Go package explicitly to use the new writer, then run:

```shell-session
$ ESS_REPORT_FORMAT=2 ESS_CONFORMANCE_ALLOW_INCOMPLETE=1 ESS_REPORT_OUT=report.json go test ./...
```

Generated Go defaults remain report/1 and diagnostic execution. `ESS_CONFORMANCE_STRICT=1` requires
report/2 and fails the enclosing test even when every subtest skipped. The strictness variables accept
only unset or `1`; conflicting settings refuse before target construction. Omitted or abnormally
terminated selected subtests, a negative report/2 clock or a failed report write cannot publish a
complete report. These checks also apply when no report destination is requested. Retained generated
packages keep their own runtime behavior until regenerated; upgrading the standalone binary does
not update them.

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.

## Add a target

The built-in targets demonstrate the runner contract. A new target implements the Rust
`ConformanceTarget` boundary and must preserve scenario identity, request/response correlation,
refusal semantics, and deterministic reporting. It must not report its own unobserved success as a
verifier result.
