---
format: aep.planning-md/1
id: review-result:adversary-area-layout-round-1
kind: review-result
status: active
title: 'Adversary, round 1: crates under area directories'
relations:
- reviews: story:crates-under-area-directories
revision: 1
---
# Adversary, round 1 — story:crates-under-area-directories

Verdict: NEEDS-CHANGE. Cases executed 1307 → 1310, red 3. Origin: introduced 5 / pre-existing 0.
Agent: `adp:adversary` (opus). Full transcript-derived report kept by the orchestrator; findings verbatim below.

Mutation probe (scratch copy of `layout.rs`, 2-crate workspace):

| Mutation | `every_workspace_crate_lives_under_an_area_directory` | `every_literal_path_naming_a_workspace_crate_exists` |
|---|---|---|
| member at `crates/ess-orphan` (no area) | FAILED (guard works) | — |
| member at `crates/misc/ess-stray` (unknown area) | FAILED (guard works) | — |
| stale pre-move literal `crates/ess-domain/src/gone.rs` in a `.rs` file | — | FAILED (guard works) |
| stale post-move literal `crates/specify/ess-domain/src/gone.rs` | — | ok — 1 passed |

Cases added by the adversary (red when written): `crates/edge/ess-xtask/tests/layout.rs:194`,
`:226`; `crates/edge/ess-xtask/tests/layout_acceptance.rs:101` (72 dangling paths in 19 files
under the story's own acceptance command; 52 in `docs/`, 11 in `.engineering/planning/journal.jsonl`).

Attacked, did not break: every area-qualified `crates/…` literal (0 dangling over 495 files); every
`CARGO_MANIFEST_DIR`/`include_str!`/`include_bytes!` site; `cargo xtask generate --check` from a
subdirectory; `ess synthesize --target rust|web` emits no path into `crates/`; 28 manifests incl.
excluded workspaces reach no `crates/` path; `examples/billing-realization` builds and tests from its
own directory; `.github/workflows/*` name no crate path; `ess-kubernetes` redaction tests: same 2 by
name before and after.

```findings
- file: crates/edge/ess-xtask/tests/layout.rs
  line: 91
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: >-
    quoted_crate_paths tests only the segment directly after `crates/`, which is now always an area and never a crate name, so a stale `crates/<area>/<crate>/...` path leaves the scan green — proven on a scratch copy where the pre-move spelling of the same path turns it red
- file: crates/edge/ess-xtask/tests/layout.rs
  line: 153
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: >-
    every_literal_path_naming_a_workspace_crate_exists reads 492 files and collects zero paths, so it asserts that an empty set is empty and would pass on an empty repository; it lacks the non-empty half infra-spec's equivalent source scan writes
- file: .engineering/planning/story/crates-under-area-directories.md
  line: 49
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: >-
    the acceptance statement's own rg command finds 72 non-existent paths in 19 files where the same scan against origin/main finds none, because the unit excluded docs/ and .engineering/ in its new test although the story's Context puts everything outside target/ and CHANGELOG.md in scope
- file: crates/edge/ess-xtask/tests/layout.rs
  line: 26
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: >-
    UNSCANNED_DIRECTORIES matches a bare directory name, so website/docs/ — the published documentation source per b10x.docs.yaml's `root: website` — is skipped by the scan under a comment that justifies skipping the engineering record instead
- file: AGENTS.md
  line: 21
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: >-
    the new Crate tree section and README:117 restate `[workspace] members` as prose in two further places with nothing comparing the three lists, which is the drift the unit's own layout test was written to prevent for paths
```
