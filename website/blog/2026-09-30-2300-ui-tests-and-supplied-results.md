---
title: "0.48 — UI tests by node path, and reports from an outside runner"
description: >
  0.48.0 runs ess-ui-test/1 tests against the terminal renderer and emits a Playwright spec from
  the same file, turns an outside runner's per-scenario results into a conformance report, and
  marks a served 501 whose effect was committed.
slug: ui-tests-and-supplied-results
tags: [release, ess]
date: 2026-09-30T23:00:00+02:00
release_tag: "0.48.0"
---

0.48.0 adds three formats, `ess-ui-test/1`, `ess-ui-test-report/1` and
`ess-conformance-results/1`, described in [the formats reference](/ess/docs/reference/formats).
Specifications, suites and deltas keep their earlier formats.

{/* truncate */}

## UI tests by node path

An `ess-ui-test/1` file tests an `ess-ui/1` document. Each step is one line: open a page, select
a node by its canonical node path (a row by its key), type, choose, act, page, expect text, rows
or a section's state, play a live event, advance time, or expect a command with given input.
A test can replace the document's fixtures.

`ess ui test --path <document> <tests…>` runs the tests headless against the terminal renderer
and writes `ess-ui-test-report/1`. `--playwright <out>` writes a Playwright spec for the
generated React project from the same file; it selects `[data-ui-path]`, and a step the terminal
cannot check is `test.fixme` in the spec with the same reason, so one file has one verdict. The
[reference](/ess/docs/reference/ess-ui-test) compares the language with selector-based browser
tests.

## Reports from an outside runner

`ess verify conform report --suite <suite> --results <results.json> --implementation <name>
--report-out <path>` writes an `ess-conformance-report/2` from the per-scenario results of a
runner written in any language. Coverage, the suite reference and policy come from ESS's own
admission of the suite; the producer profile, `external-scenario-status/1`, says ESS executed
nothing. aep 0.66.0 records such a report as evidence.

## A served 501 says whether the effect was committed

Every served `501` body carries `committed`: `false` for an unmet obligation, where nothing was
written, and `true` when the command took effect and delivering what it published failed. Rust
shells see the second case as `Refused::Undelivered`.

## Also

- `ess ui check --model <dir>` reads the specification through the directory's `ess-inputs.yaml`.
- An `ess` that delegates to a pinned release says so on stderr for every command
  (`ESS_TOOLCHAIN_QUIET=1` silences it).
