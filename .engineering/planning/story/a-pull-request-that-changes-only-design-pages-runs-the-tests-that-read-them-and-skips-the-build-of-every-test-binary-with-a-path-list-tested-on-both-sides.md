---
format: aep.planning-md/3
id: story:a-pull-request-that-changes-only-design-pages-runs-the-tests-that-read-them-and-skips-the-build-of-every-test-binary-with-a-path-list-tested-on-both-sides
kind: story
status: draft
title: A pull request that changes only design pages runs the tests that read them and skips the build of every test binary, with a path list tested on both sides
tags:
- ci
- test-budget
relations:
- serves: vision:O2
- informed_by: story:the-test-step-meets-the-test-budget
revision: 1
---
## Need

The `diff` job of `ci.yml` lets a pull request that changes only `.engineering/` (and not
`.engineering/repro/`) pass its required checks without a build or a test: the one path list sits at
the top of its `plan-only` step as `plan=('.engineering/?*')` and `tested=('.engineering/repro/*')`.
A pull request that changes only a design page under `docs/design/` is not covered, so it runs the
full gate: `build-tests`, `build-feature-off` and the 20 test jobs. Pull request 841 ("design: outward
events and an event-bus contract", one file, `docs/design/outward-events-and-event-bus.md`) is a
case, seen 2026-11-26 and open when this was written.

The design pages cannot simply join the `plan` list, because tests read them: `AGENTS.md` says a
change to `docs/` or `website/` also needs `task test-xtask`, and `docs/design/review-typed-diagnostics.md`
counts every string-located `ValidationError` per file for `typed_diagnostics`. Many test files under
`crates/*/tests/` name `docs/design` paths (for example in `ess-xtask`: `layout.rs`,
`d2_constraint_home.rs`, `host_paths.rs`; in `ess-cli`: `predicate_reference_page.rs`,
`http_presentation.rs`); which of them read the file and which only name it is the first thing the
story establishes.

The need, apart from any spelling: a pull request that changes only prose pages runs the tests that
read those pages, and not the compile of every test binary and the 20 shards, as the planning store runs none.

## Outcome

A pull request whose changed paths all lie under `docs/design/*.md` runs a lane of its own: the
tests that read `docs/`, named in the workflow by one list, and `Checks`; it skips `build-tests`,
`build-feature-off` and the shards. `Gate` passes the skipped jobs for that reason only and says so
in its log, as it does for the plan-only case. A path outside the list, or a test that reads the
pages and is not in the lane, keeps the full gate.

## Acceptance

- One path list for design pages, written once in the workflow beside `plan` and `tested`, with a
  test in `crates/edge/ess-xtask/tests/ci_lanes.rs` that runs the decision against canned inputs on
  both sides of it: a diff of one `docs/design/*.md` file (lane), the same file with one `crates/`
  file (full gate), a file under `docs/` that is not `docs/design/*.md` (full gate), a diff of
  `.engineering/` files only (plan-only, unchanged), an error in the diff lookup (full gate).
- A test in `ci_lanes.rs` fails when a test source under `crates/*/tests/` opens a file under `docs/`
  and is in neither the lane's list nor documented as a full-gate reader; the failure names the
  file.
- A `main` push, the nightly run, a `queue/**` head, a `merge_group` run and a call from
  `release.yml` never take the lane, as they never take the plan-only skip; such a run never counts
  for a release.
- A design-page-only pull request ends its required checks without a test shard: shown by a run's
  job list, the first one named in the story's delivery.

## Fit review

1. Need: runner time and wait for a pull request whose only change is prose, with the tests that
   read prose still run. Reproduction: pull request 841.
2. Class: gap in the CI lanes (a defect of the budget, `AGENTS.md` of the organization,
   "Planning-only pull requests skip the tests": it names `*.md` outside a built docs site).
3. Already expressible: the plan-only path list covers `.engineering/` only; the organization rule
   says `*.md` outside a built docs site, so this closes the gap between the rule and the list.
4. Fit: same step, same `Gate` handling, one more list.
5. Second adopter: any repository whose design pages sit beside tests that read them.
6. Cost: a workflow change and its tests; a lane that must name its readers, which is the risk
   (a reader missing from the list lets a stale page pass).
7. Considered: change nothing; add `docs/design/*.md` to `plan` (refused: tests read them); a
   lane of the readers (chosen).

## Decisions

- DP-D1, open. A: the lane of the readers (recommended; fails closed where a reader is not in the
  list, because the acceptance test names it). B: `docs/design/*.md` joins the plan-only list;
  fails open, a page a test reads can break the gate after merge. C: change nothing; every design
  page pull request runs the full gate.
- The ruleset is the owner's and is not changed by this story.

## Scope

- inferred: .github/workflows/ci.yml
- inferred: crates/edge/ess-xtask/tests/ci_lanes.rs
- inferred: AGENTS.md
