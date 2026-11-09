---
format: aep.planning-md/3
id: story:conform-author-and-run-say-what-they-need-before-reading-nothing
kind: story
status: draft
title: conform author reads the manifest's scenarios or says it read none, and conform run's help names the report format current suites need
summary: conform author silently reads 0 files; conform run needs --report-format 2 unannounced (adopter item 6, beyond10x/a2a); gap (UX).
tags:
- adopter-report
- defect
- ess-0.70.0
- feature-request
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/input_discovery.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/authored_scenarios.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/conform_report.rs
- confidence: cited
  path: website/docs/reference/cli.md
revision: 8
---
## Need

Reported by an adopter session (item 6 of 9, 2026-10-22). Two of the conformance verbs have
requirements they never state:

(a) `ess verify conform author --path DIR` without `--scenarios` reads nothing and succeeds, although
the manifest in `DIR` lists scenario files. `synthesize` behaves the same.

(b) `ess verify conform run` on a current suite without `--report-format 2` refuses, and `--help`
does not say so up front.

Reproduction on 0.92.0: a model directory whose `ess-inputs.yaml` lists five specification files
and two scenario files under `scenarios:`.

```text
$ ess specify validate --path model
billing v3 — 5 file(s), 2 scenario(s), valid
$ ess verify conform author --path model                       # stderr empty, exit 0
0 authored scenario(s) from 0 file(s), 0 refusal(s), suite ess-conformance/34, nothing written
$ ess verify conform author --path model --scenarios model/scenarios
2 authored scenario(s) from 2 file(s), 0 refusal(s), suite ess-conformance/34, nothing written
$ ess verify conform synthesize --path model --out suite.json  # no word about the listed scenarios
32 scenario(s) (0 authored), 0 refusal(s), written to suite.json
$ ess verify conform run --suite suite.json --path model --target interpreted
error: suite/8 through /63 require explicit --report-format 2 before execution
$ ess verify conform run --help        # the --report-format entry:
      --report-format <REPORT_FORMAT>
          Report contract version; JSON/YAML detailed v2 is ess-conformance-run/2
```

So `validate` reads the manifest's scenarios, while `author` and `synthesize` silently do not.

## Where it lives

- (a) `crates/edge/ess-cli/src/main.rs:694` (`author --scenarios`, no default; dispatched at
  `:3763`); `crates/edge/ess-cli/src/input_discovery.rs:80` (`authored`: absent means empty) and
  `:96-101` ("a model argument never implies scenarios"; `listed_scenarios`, which `validate` uses).
- (b) `crates/edge/ess-cli/src/main.rs:795-797` (default `--report-format 1`, help text "Report
  contract version"); refusal at `crates/edge/ess-cli/src/coverage.rs:202`.

## Cause

(a) is by design (`input_discovery.rs:98-99`), but silent. (b): the default stays 1 because AEP
readers accept only report/1 (`docs/design/review-conformance-coverage.md:253`). Changing the
default is a contract change.

## Fit review

1. **Need.** An author who runs the documented verbs must not get an empty success or a requirement
   nobody announced. The requester's words: "conform author --path . reads 0 files silently
   without --scenarios; conform run needs --report-format 2 unannounced".
2. **Class: gap (UX).** Neither behaviour contradicts a documented rule. Both leave the author
   without what they need to know to go on.
3. **Already expressible?** Yes: pass `--scenarios` and `--report-format 2`. The gap is that
   nothing says so.
4. **Fit.** No authored surface. (a) When `--scenarios` is absent and the manifest names scenario
   files, warn naming the manifest entries and the flag. The same rule holds for `synthesize`
   (sibling rule). (b) Keep the default (AEP readers), and name the requirement in `--help` for
   `--report-format`.
5. **Second adopter.** Any first-time adopter who follows the tutorial with a manifest.
6. **Cost.** No format version. A warning changes no existing invocation's output on stdout or its
   exit status. The report default is not changed.
7. **Alternatives.** (a) Change nothing. (b) Default report format 2: breaks AEP readers. (c) Read
   manifest scenarios by default: reverses `input_discovery.rs:98-99` for existing invocations.
   (d) Chosen: warn in (a), and document in (b).

## Decisions

**accept, redesigned.** No default changes. `author` and `synthesize` say they read no scenarios
and name the manifest entries they ignored. `run --help` names the report format that suite/8 and
later require. The help text is pinned in the generated `website/docs/reference/cli.md`
(`crates/edge/ess-xtask/src/cli_reference.rs`), which is regenerated in the same change.

2026-11-09: the earlier revision recorded this as released in 0.70.0. It was not. 0.92.0 still
behaves as reproduced above, the named test cases do not exist, and `CHANGELOG.md` has no entry for it.

## Acceptance

Fixture: a model directory whose manifest lists two scenario files, written by the test.

- **`author` names what it ignored.** Transition: `ess verify conform author --path DIR` without
  `--scenarios`. Before: stdout `0 authored scenario(s) from 0 file(s), ...`, nothing on stderr,
  exit 0. After: stderr names both manifest entries and `--scenarios`, and the exit status and stdout
  stay the same. Test: `crates/edge/ess-cli/tests/authored_scenarios.rs`, case
  `author_without_scenarios_names_the_manifest_entries_it_ignored`.
- **`synthesize` names what it ignored.** Transition: `ess verify conform synthesize --path DIR
  --out FILE` without `--scenarios`. After: the same warning on stderr, and the suite written is
  byte-identical to the current tree's. Same file, case
  `synthesize_without_scenarios_names_the_manifest_entries_it_ignored`.
- **Silent where nothing was ignored.** Transition: both verbs on a manifest with no scenario
  entries, and both with `--scenarios` given. After: stderr empty, output byte-identical. Same
  file, case `no_warning_when_no_manifest_scenario_was_ignored`. Mutation check: emitting the
  warning unconditionally fails this case.
- **`run --help` names the requirement.** Transition: `ess verify conform run --help`. Before: the
  `--report-format` entry reads "Report contract version; JSON/YAML detailed v2 is
  ess-conformance-run/2". After: it says suite/8 and later need `2`, and the default is still 1. Test:
  `crates/edge/ess-cli/tests/conform_report.rs`, case
  `run_help_names_the_report_format_current_suites_require`. `website/docs/reference/cli.md` is
  regenerated (`cargo xtask cli-reference`), and `cargo xtask cli-reference --check` passes.

## Scope

- cited: `crates/edge/ess-cli/src/main.rs`
- cited: `crates/edge/ess-cli/src/input_discovery.rs`
- cited: `crates/edge/ess-cli/tests/authored_scenarios.rs`
- cited: `crates/edge/ess-cli/tests/conform_report.rs`
- cited: `website/docs/reference/cli.md` (generated)

confidence: high
