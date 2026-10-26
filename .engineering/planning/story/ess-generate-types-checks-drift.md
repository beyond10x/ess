---
format: aep.planning-md/3
id: story:ess-generate-types-checks-drift
kind: story
status: active
title: Ess generate types checks drift
summary: 'Adopter report S6: ess generate types has no --check (ess#654).'
refs:
- provider: github
  reference: beyond10x/ess#654
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:57Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:57Z", actor: "human:timo", revision: 7}
---
## Problem

`ess generate types` has no `--check`, unlike the other generators. Adopters generate into a temporary directory and `diff -r -x .ess-output`, because `.ess-output/state.json` carries a fresh `anchor_id` per directory. Adopter report S6, ess 0.68.0.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary
(`crates/edge/ess-cli/tests/model_types.rs`).

- `generate_types_check_passes_on_fresh_output_and_writes_nothing`: after generating into
  `<dir>`, `--check` exits 0 and every file's bytes and modification time are unchanged (the
  snapshot helper `generate_check.rs` uses).
- `generate_types_check_names_a_drifted_file`: after editing `types.rs`, `--check` exits 1 and
  names `types.rs` with `ess generate types` in its remedy.
- `generate_types_check_names_a_missing_file_and_one_no_longer_generated`: a deleted
  `types-report.json` is named missing; output generated with `--target rust` then checked with
  `--target typescript` names `Cargo.toml` as recorded and no longer generated.
- `generate_types_check_passes_on_a_copied_output_root`: a copy of the generated root at another
  path checks clean (the anchor is not drift); an unrecorded extra file there is not named.
- `generate_types_check_requires_out`: clap refuses `--check` without `--out` (as
  `generate_check.rs::generate_check_requires_out`).

## Fit review

Checked on ess 0.69.2 (main `a3bc2df3d`) against minimal specifications in the fit scratch
directory.

1. **Need.** A repository that commits a generated type library cannot ask ESS whether it is
   current: `ess generate types … --check` answers `error: unexpected argument '--check' found`
   (reproduced). Generating into a second directory differs only in `.ess-output/state.json`
   (`anchor_id` and checksum), so the workaround is a temporary directory and
   `diff -r -x .ess-output`. Requester's proposal (theirs): a `--check` that exits non-zero, names
   the drifted files and writes nothing, as the other generators do.
2. **Class.** Gap in a sibling: every other generating verb has `--check` (`ess generate`,
   `crates/edge/ess-cli/src/main.rs:268-275`; `ess verify conform synthesize`, `main.rs:652-659`;
   `ess generate schema typescript`, `src/schema.rs:68-70`; the run-mode guide, `main.rs:1131-1134`);
   `ess generate types` has none (`src/model_types.rs:19-46`). CLI surface only, no authored key.
3. **Already expressible?** Only outside ESS (`diff -r -x .ess-output`), which moves the check
   into the adopter's tooling and relies on knowing which files ESS owns.
4. **Fit.** The output is already an owned tree (`write_owned_files(…, "model-types", …)`,
   `model_types.rs:107-113`), so the shared comparison `output_ownership::drift`
   (`src/output_ownership/mod.rs:147`) applies unchanged, as `check_owned_suite` uses it for a
   suite package (`main.rs:4895-4914`). Its contract: drift is a missing file, other bytes, a file
   recorded for the owner that is no longer generated, or a missing record; an unrecorded extra
   file is not drift (reproduced with `ess generate --kind schema --check`: exit 0 with an added
   file, exit 1 naming a deleted one). The anchor is no issue for a committed root: a copied output
   root checks clean (reproduced), because `ess-output-state/3` records no absolute path
   (`mod.rs:96-110`).
5. **Second adopter.** A service repository that commits generated Go types for review and wants
   CI to fail when the specification moved without a regeneration.
6. **Cost.** One CLI flag (`--check`, `requires = "out"`), no format, IR or generated-byte change;
   the CLI reference page is regenerated (`crates/edge/ess-xtask/src/cli_reference.rs:22`) and the
   commit-generated-files table gains a row.
7. **Alternatives.** (a) Change nothing, document the `diff` idiom: rejected, it re-implements
   ownership outside ESS. (b) A new comparison ignoring `.ess-output`: rejected, a second drift
   rule. (c) Reuse `output_ownership::drift` (chosen, the requester's shape).

## Decisions

- Accept as proposed: `ess generate types --check` with `--out`, comparing through
  `output_ownership::drift` with the files the same command would write, writing nothing; exit 0
  when current, 1 with one line per drifted file. The remedy line names `ess generate types`, as
  synthesize substitutes its own (`main.rs:4875,4903-4907`).
- An unrecorded extra file in `--out` is not drift, as for every other `--check`; the story's
  "extra file is named" line is replaced by a file recorded and no longer generated.
- No format gate. Not already fixed on main (`model_types.rs` has no `check` field; 0.69.2
  refuses the flag).

## Scope

| unit | files | confidence |
|---|---|---|
| flag and delivery | `crates/edge/ess-cli/src/model_types.rs` (`Args` `:19-46`, `run` `:107-118`: call `output_ownership::drift` instead of `write_owned_files` when checking) | cited |
| shared comparison | `crates/edge/ess-cli/src/output_ownership/mod.rs` (`drift` `:147`, `Publication::tree` `:28`), unchanged | cited |
| remedy text | beside `SUITE_REMEDY`/`check_owned_suite` in `crates/edge/ess-cli/src/main.rs:4875-4914`, or a shared helper | inferred |
| tests | `crates/edge/ess-cli/tests/model_types.rs` | cited |
| docs | `website/docs/reference/cli.md` (regenerated by `cli_reference.rs`), `website/docs/guides/commit-generated-files.md` table (`:23` neighbour row) | cited |

Crate: ess-cli only. No new spec format version. Test binary: ess-cli `model_types`. Sizing: S.
No overlap with the other eleven stories' files.

confidence: high
