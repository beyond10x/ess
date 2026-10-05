---
format: aep.planning-md/3
id: story:ess-generate-check
kind: story
status: active
title: ess generate --check reports drift against committed output
tags:
- ess-0.54.0
refs:
- provider: github
  reference: beyond10x/ess#435
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/mod.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/generate_check.rs
- confidence: cited
  path: website/docs/reference/cli.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:31Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:32Z", actor: "human:timo", revision: 7}
---
## Outcome
`ess generate` gains `--check`: it regenerates in memory, compares with the output already under `--out`, writes nothing, and exits non-zero naming each file that drifted.

## Origin
beyond10x/ess#435, coordinator decision 2026-10-05. The fit review of #435 found that the guide tells adopters to drift-check with `cargo xtask generate --check`, a repository-only command (`website/docs/guides/generate-artifacts.md:786-797`, `website/docs/reference/cli.md:235`). The coordinator split the missing adopter command out of the documentation story into this one.

## Fit review
1. Need: an adopter who commits generated projections needs CI to fail when they are stale, without a dirty-tree check in their own scripts. `ess generate` has no `--check` (`website/docs/reference/cli.md:673` option table; `GenerateArgs`, `crates/edge/ess-cli/src/main.rs:223-244`).
2. Class: convenience. The git-based idiom works today: regenerate into the committed root and fail on a dirty tree (#435 Q3). But every sibling generator already has the flag, and this one does not.
3. Existing idiom: the siblings `ess generate cli` (`crates/edge/ess-cli/src/cli_binding.rs:25`), `ess schema typescript` (`crates/edge/ess-cli/src/schema.rs:67-69`, "Refuse when `--out` differs from the generated module, without writing it"), `ess specify realization generate` (`main.rs:1012-1015`; check arm `:1709-1730`) and `ess ui docs` each take `--check` (`cli.md:124`, `:566`, `:710`, `:994`, `:1024`, `:1699`).
4. Fit: the same flag, with the same meaning, on the one generator that lacks it. It reuses the output-ownership guard (`output_ownership::check`, `crates/edge/ess-cli/src/output_ownership/mod.rs:75`) and compares where `publish` (`:87`) would write. It adds no format and no new diagnostic.
5. Second adopter: any team committing `openapi/` for its clients wants CI to fail when a specification change was not regenerated.
6. Cost: one flag on `GenerateArgs`, a compare path in `generate` (`main.rs:3242-3357`), one test file, and the regenerated CLI reference block. No format, no diagnostic code.
7. Alternatives: (a) document the git idiom only: #435 then teaches a second drift-check spelling beside every sibling's `--check`; (b) chosen: the sibling flag.

## Decisions
Accept. `ess generate --check` requires `--out`. It regenerates the selected `--kind` (all projections when absent) in memory and compares with the files under `--out`. The files `.ess-output` records as owned are compared too, so a missing or no-longer-generated file is drift. It writes nothing. Exit 0 when current; exit 1 with one stderr line per drifted file, naming the file and saying `regenerate it with ess generate`. Same flag text and exit codes as `ess schema typescript --check`. No format bump. Story 435 depends on this one.

## Acceptance
- generate_check_reports_drift_and_exits_nonzero: after `ess generate --path examples/billing --kind openapi --out <dir>`, one generated file is edited and another deleted. `ess generate --path examples/billing --kind openapi --out <dir> --check` then exits 1 and names both files on stderr. `<dir>` is byte-identical before and after the check.
- generate_check_passes_on_fresh_output: right after the same generation, `--check` exits 0, prints no drift line, and writes nothing (no file under `<dir>` changes its bytes or modification time).
- generate_check_requires_out: `--check` without `--out` is refused by the argument parser, as `schema typescript` refuses it.
- cli_reference_lists_generate_check: the generated block of `website/docs/reference/cli.md` lists `--check` for `ess generate generate`, and the CLI reference drift check (`crates/edge/ess-xtask/src/cli_reference.rs`) passes.

## Scope
- crates/edge/ess-cli/src/main.rs  cited — `GenerateArgs` (223-244) gains `check`; `generate_projections` (1583-1596) passes it; `generate` (3242-3357) compares instead of publishing; precedent: the `RealizationCommand::Generate` check arm (1709-1730). Disjoint from `validate` (2491-2561), which #434 and #437 edit.
- crates/edge/ess-cli/src/output_ownership/mod.rs  cited — `check` (75) guard reused; `publish` (87) is the write path the comparison mirrors
- crates/edge/ess-cli/tests/generate_check.rs  inferred — the three CLI scenarios above
- website/docs/reference/cli.md  cited — generated `ess generate generate` table (:673-690), regenerated through `crates/edge/ess-xtask/src/cli_reference.rs`
