---
format: aep.planning-md/3
id: story:conform-author-and-run-say-what-they-need-before-reading-nothing
kind: story
status: draft
title: conform author reads the manifest's scenarios or says it read none, and conform run's help names the report format current suites need
summary: conform author silently reads 0 files; conform run needs --report-format 2 unannounced (adopter item 6, beyond10x/a2a); gap (UX).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
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
revision: 6
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 6 of 9, 2026-10-22). Two
silent requirements in the conformance verbs:

(a) `ess verify conform author --path .` without `--scenarios` reads nothing and succeeds:

```text
0 authored scenario(s) from 0 file(s)
```

exit 0, although the manifest lists scenario files.

(b) `ess verify conform run` on a current suite without the flag refuses:

```text
error: suite/8 through /57 require explicit --report-format 2 before execution
```

and `--help` does not say so up front.

## Where it lives

- (a) `crates/edge/ess-cli/src/main.rs:686-698` (`author --scenarios`, no default);
  `crates/edge/ess-cli/src/input_discovery.rs:80-82` (absent means empty, manifest ignored) and
  `:98-99` ("a model argument never implies scenarios"). Synthesize behaves the same
  (`main.rs:641-644`).
- (b) `main.rs:793-794` (default `--report-format 1`); refusal at
  `crates/edge/ess-cli/src/coverage.rs:195-200`.

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0: (a) "0 authored scenario(s) from 0 file(s)", exit 0; (b) exit 1
with the line above.

## Cause

(a) by design (`input_discovery.rs:98-99`), but silent. (b) the default stays 1 because AEP readers
accept only report/1 (`.engineering/planning/story/review-conformance-format-design.md:53`;
`docs/design/review-conformance-coverage.md:253`); changing the default is a contract change.

## Fit review

1. **Need.** An author running the documented verbs must not get an empty success or an
   unannounced requirement. Requester's wording: "conform author --path . reads 0 files silently
   without --scenarios; conform run needs --report-format 2 unannounced".
2. **Class: gap (UX).** Neither behaviour contradicts a documented rule; both leave the author
   without the information to proceed.
3. **Already expressible?** Yes, by passing `--scenarios` and `--report-format 2`; the gap is that
   nothing says so.
4. **Fit.** No authored surface. (a) When `--scenarios` is absent and the manifest names scenario
   files, either read them or warn naming the manifest entry and the flag; the choice must hold for
   `synthesize` too (sibling rule). (b) Keep the default (AEP readers), and name the requirement in
   `--help` for `--report-format`.
5. **Second adopter.** Any first-time adopter following the tutorial with a manifest.
6. **Cost.** No format version. Reading manifest scenarios by default changes behaviour for
   existing invocations that rely on "never implies scenarios"; a warning does not. Changing the
   report default is a contract change and is not proposed here.
7. **Alternatives.** (a) Change nothing. (b) Default report format 2: breaks AEP readers. (c)
   Chosen: warn in (a) (or read, if the design rule at `input_discovery.rs:98-99` is revisited), and
   document in (b).

## Decisions

**accept, redesigned.** No default changes. `author` and `synthesize` say they read no scenarios
and name the manifest entries they ignored; `run --help` names the report format suite/8 and later
require.

2026-10-22: acceptance covers both `author` and `synthesize` (sibling rule,
`input_discovery.rs:98-99` holds for both). The `--report-format` default stays 1 (AEP readers,
`docs/design/review-conformance-coverage.md:253`). Released in 0.70.0. The help text is pinned in
the generated `website/docs/reference/cli.md` (`crates/edge/ess-xtask/src/cli_reference.rs:22`),
which is regenerated in the same change.

## Acceptance

Fixture: a model directory whose manifest lists two scenario files, written by the test.

- **`author` names what it ignored.** Transition: `ess verify conform author --path DIR` without
  `--scenarios`. Before: stdout `0 authored scenario(s) from 0 file(s)`, nothing on stderr, exit 0.
  After: stderr names both manifest entries and `--scenarios`; exit status 0 and stdout unchanged.
  Test: `crates/edge/ess-cli/tests/authored_scenarios.rs`, case
  `author_without_scenarios_names_the_manifest_entries_it_ignored`. Failing before: stderr is empty
  on the current tree.
- **`synthesize` names what it ignored.** Transition: `ess verify conform synthesize --path DIR
  --out FILE` without `--scenarios`. After: the same stderr warning; the suite written is
  byte-identical to the current tree's. Same file, case
  `synthesize_without_scenarios_names_the_manifest_entries_it_ignored`.
- **Silent where nothing was ignored.** Transition: both verbs on a manifest with no scenario
  entries, and both with `--scenarios` given. After: stderr empty, output byte-identical. Same
  file, case `no_warning_when_no_manifest_scenario_was_ignored`. Mutation check: emitting the
  warning unconditionally fails this case.
- **`run --help` names the requirement.** Transition: `ess verify conform run --help`. Before: the
  `--report-format` entry does not say suite/8 and later need 2. After: it does, default still 1.
  Test: `crates/edge/ess-cli/tests/conform_report.rs`, case
  `run_help_names_the_report_format_current_suites_require`; `website/docs/reference/cli.md`
  regenerated (`cargo xtask cli-reference`), and its `--check` (Taskfile.yml:287) passes.

## Schedule

Release 0.70.0 (gap, UX; wave A unit A5, after A8 in its dispatch slot). Tagged `ess-0.70.0`
on 2026-10-22. No format version needed; the report-format default stays.
