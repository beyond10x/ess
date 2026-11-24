---
format: aep.planning-md/3
id: story:a-patch-tag-whose-change-set-holds-a-release-notes-file-or-the-ess-ui-schema-runs-the-targeted-gate-hotfix
kind: story
status: active
title: 'A patch tag whose change set holds a release-notes file or the ess-ui schema runs the targeted gate: hotfix-scope.sh counts `release-notes/` as prose and maps `schemas/ui/ess-ui.schema.yaml` to ess-ui'
summary: hotfix-scope.sh emits the full gate for release-notes/<version>.md, which every hotfix carries, and for the ess-ui schema, so the targeted Hotfix checks and Hotfix tests never run; tag 0.98.1 took 2 h 10 min against the 1 h target
tags:
- ci
- issues-2026-11-24
- release
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T06:29:04Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T06:29:05Z", actor: "human:timo", revision: 4}
---
## Outcome

A patch tag whose change set is code of workspace crates plus the files every hotfix carries
(`CHANGELOG.md`, `release-notes/<version>.md`, the version bump) runs the targeted gate (`Hotfix
checks`, `Hotfix tests`) and not the 20 test shards. `AGENTS.md` § Hotfix sets the target of one
hour from reproduction to a pushed tag with a published release; today the targeted gate cannot
run for any hotfix that follows the section's own step 3.

## Need

**Reading.** Verified at `origin/main` (`840db72ef`), `.github/scripts/hotfix-scope.sh`:

- Lines 64-66 are the whole list of paths that are not mapped to a crate and do not count as a
  change: `.engineering/*`, `changes/*`, `website/*`, `docs/*`, `CHANGELOG.md`, `README.md`,
  `WHATS-CHANGED.md`. `release-notes/*` is not in it.
- A path outside that list and outside every workspace crate directory reaches line 90 and emits
  `full` with the reason "`<path>` belongs to no workspace crate". So `release-notes/<version>.md`
  alone turns every hotfix into a full-gate run. The same holds for `schemas/ui/ess-ui.schema.yaml`.
- `AGENTS.md:466` (§ Hotfix, step 3) requires a `release-notes/<version>.md` in every hotfix, and
  `cargo xtask release verify` refuses a workspace version without a valid file, so the case is not
  an exception: it is every hotfix.

**Measured.** Tag `0.98.1` (commit `7b4b42f6c`, 2026-11-24 05:03 CET). The change since `0.98.0`
holds `release-notes/0.98.1.md` and `schemas/ui/ess-ui.schema.yaml`, beside crate files under
`crates/ui/`, `CHANGELOG.md`, the manifests, `README.md` and `website/`. Release run 45323728818:
`Hotfix checks` and `Hotfix tests` skipped, the `gate / Test k/20` jobs and the two feature-off and
workspace archive builds ran, the run ended 07:13:50 CET and the release was published 07:14:08
CET by the organization bot: 2 h 10 min from the tag push against the one hour of § Hotfix.

**The script has no test.** No file under `crates/` or `.github/` runs `hotfix-scope.sh`;
`ci_lanes.rs` (`a_release_runs_its_full_gate_unless_a_full_gate_passed_the_tagged_tree`, line 2199,
and the neighbouring cases) holds the wiring of the `resolve`, `hotfix-checks` and `hotfix-tests`
jobs in `release.yml`, not the decision the script makes. A change to the prose list therefore
had nothing to fail.

**Where `release-notes/` is held to the code.** `task release-check` (`Taskfile.yml:485-488`) runs
`cargo xtask release verify` and `cargo xtask release notes --check`, and it is a step of `ci-lint`
(`Taskfile.yml:627`), which `Hotfix checks` runs; the release preparation runs `cargo xtask release
verify "$TAG"` again (`release.yml:565`). So counting the file as prose leaves its validation in the
targeted gate.

**The schema.** `schemas/ui/ess-ui.schema.yaml` is embedded by `ess-ui` itself
(`crates/ui/ess-ui/src/lib.rs:54`, `include_str!`). The other crates that name it:
`ess-ui-check` (`src/schema.rs:1`), `ess-ui-docs` (`src/doc.rs:51`, its `SOURCE`; its tests read the
file from disk) and `ess-ui-react` (`tests/adversary_narrow_layout_pass1.rs:113`, read from disk).
All of them depend on `ess-ui` by path, so the script's
"direct dependents" step adds each of them once `ess-ui` is the changed crate; `ess-cli`,
`ess-synth` and `ess-ui-test` depend on it directly too and come in the same way.

## Acceptance

A script test that stages a Git history of a small workspace (two crates, one depending on the other, a
`cargo` stub on `PATH` that answers `metadata` with the matching JSON, so the test spawns no cargo
process) and runs `hotfix-scope.sh` on a patch tag whose change set is one crate file,
`CHANGELOG.md` and `release-notes/<version>.md` prints `scope=targeted` with that crate and its direct
dependents in `packages=`, and prints `scope=full` for the same change set when a file outside every
crate and outside the prose list is added; and a second case with `schemas/ui/ess-ui.schema.yaml` in
the change set prints `scope=targeted` with `ess-ui` among the packages.

## Decisions

- **`release-notes/*` is prose.** It carries no code; `release-check` still validates it in `Hotfix
  checks` (named above). Counting it as prose changes which crates are tested only by adding none.
- **`schemas/ui/ess-ui.schema.yaml` maps to `ess-ui`**, not to the prose list and not to "every
  crate". `ess-ui` embeds the file at compile time, every other reader is a crate that depends on
  `ess-ui` and is added as a dependent, and the file changes bytes `ess-ui` compiles, so it is
  code of `ess-ui`. The cost is real: the dependents include `ess-cli` and `ess-synth`, whose
  tests are the largest of the targeted set; a hotfix of the schema is still far below the full
  gate.
- **The test runs the script, in `ci_lanes.rs` or a test beside it.** The wiring cases of
  `release.yml` stay where they are.

## Out of scope

- The targeted gate's own contents and `terminate-after`.
- A file that belongs to no crate and is not prose still runs the full gate; that safety stays.

## Scope

- `.github/scripts/hotfix-scope.sh` (the prose list, the schema mapping)
- `crates/edge/ess-xtask/tests/ci_lanes.rs` (or a new test file of `ess-xtask`) for the script's cases
- `AGENTS.md` (§ Hotfix, one sentence if the mapping is named there)
