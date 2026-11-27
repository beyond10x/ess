---
format: aep.planning-md/3
id: story:a-mutation-audit-of-a-larger-model-ends-in-bounded-time-and-shows-its-progress-while-it-runs
kind: story
status: active
title: A mutation audit of a larger model ends in bounded time and shows its progress while it runs
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/mutate_cli.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/once_per_model_work.rs
- confidence: inferred
  path: docs/design/
- confidence: inferred
  path: website/docs/
- confidence: inferred
  path: website/docs/guides/verify/mutation-audit.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T22:32:35Z", actor: "human:timo", revision: 14}
- {from: "proposed", to: "active", at: "2026-11-22T22:32:37Z", actor: "human:timo", revision: 15}
---
## Outcome

`ess verify conform mutate` on a model with many mutation sites ends within a stated bound of
work, or says at its start how much work it has before it, and reports each mutant as it is
decided. A run cut off by a caller's time limit leaves the mutants it decided on standard error,
not nothing. The verdicts, the report and the manifest of every mutant the audit runs stay
byte-identical to 0.97.3.

## Need

A downstream specification reported that `--class error-swap` on `--target interpreted` ended at
a 600 s limit with no output, after 405 s of user CPU; its other classes finish. Reproduced on the
released 0.97.3 binary:

- One synthesis of that specification is 78e9 instructions, 20 s of user CPU. A hand-made
  error-swap mutant of it synthesizes in the same 78e9: no mutant is slower than the model.
- `emit-swap`: 10 mutants, 868e9 instructions, 228 s. `guard-negate`: 27 mutants, 2,141e9,
  559 s. That is 79e9 per mutant: one full synthesis of the mutated model plus about 1 %.
- `error-swap` has one site per refusal (`error:` outcome) of every command: 44 on that
  specification: 44 mutants, all killed, 45 syntheses, 3,546e9 instructions, 934 s of CPU (exit
  0 when nothing bounds it). Nothing is printed until the last mutant is decided; a run cut at
  900 s wall printed nothing after 645 s of CPU. A frame-pointer profile puts 99 % of the run in
  the syntheses (`mutate::synthesized`) and about 1 % in the rest of the driver.
- A generic model of K commands with N input-guarded refusals each (released binary, error-swap):
  mutants = K x N; the audit costs (K x N + 1) syntheses. K=2: N=4 0.56e9, N=8 2.9e9, N=16 27.7e9
  instructions; N=4: K=4 2.0e9, K=8 7.7e9. One synthesis grows with the model too, so the audit
  grows as refusals times synthesis: quadratic in commands, faster in refusals per command.

Cause, at tag 0.97.3: `outcome_sites` (`crates/verify/ess-conformance/src/mutate.rs:2462`) and
`error_payload_sites` (`mutate.rs:2149`) make one mutant per refusal; `audit_with_authored`
(`mutate.rs:5910`) measures each with `measure` (`mutate.rs:5773`), which synthesizes the whole
mutated model afresh (`run_checked`, `mutate.rs:5643`). Every memo of a synthesis is scoped to one
synthesis, so nothing of the baseline's work is kept for a mutant, and the CLI renders the report
only after the last mutant (`crates/edge/ess-cli/src/main.rs:4792`). The 0.97.2 synthesis fixes
apply to each mutant's synthesis and do not change the count.

## Design

Three parts, each its own decision below; none changes a verdict.

1. Progress. Before the first mutant, one line on standard error: the selected classes, the
   number of mutants and the sites listed as unmutated or unavailable. After each mutant, one
   line: its index, id and verdict. Standard output, the report and the manifest are unchanged.
2. Reuse across mutants. A mutant whose edit leaves a command's synthesis inputs unchanged takes
   that command's scenarios from the baseline's synthesis instead of synthesizing them again. The
   unit and its key must be shown sound before it is built: the suite is today a function of the
   whole model (precedence checks, grants, views, withdrawals after generation), so the key is the
   part of the compiled model a command's scenarios read, and a test compares the reused suite
   with a fresh synthesis byte for byte on every mutant of `examples/billing` and the generic
   models. An error-swap edit changes one outcome's error; the expected answers that name it move,
   the arrangements do not.
3. A bound. `--max-mutants <n>` (or a work bound) stops after `n` mutants, lists every mutant not
   run as unmutated with a named reason, and exits 3 as an unavailable site does. Without the flag
   nothing changes.

## Acceptance

- On the generic model of 2 commands with 16 refusals each, `mutate --class error-swap --target
  interpreted` prints the mutant count before the first synthesis and one line per mutant on
  standard error; standard output and `--report-out` are byte-identical to 0.97.3.
- With part 2, the same run starts at most 1 + (number of commands a mutant changes) whole or
  partial syntheses per mutant, counted by `syntheses_started` or a counter of command syntheses,
  and its report and an `--emit` manifest and suites are byte-identical to 0.97.3 on
  `examples/billing`, the generic models and every class of the downstream specification that
  finishes.
- With part 3, `--max-mutants 5` on that model runs 5 mutants, lists 27 as unmutated with the
  reason, and exits 3.
- A work-bound test in an existing `*_work.rs` target of `ess-conformance` holds the count of
  part 2; no test over 60 s.

## Decisions

- D1 (default A): progress lines. A: on standard error in every format. B: only with `--format
  text`. C: none.
- D2 (default A): reuse across mutants. A: design it as a design page first, then build it with
  the byte-identity test. B: skip it and rely on parts 1 and 3. C: run mutants on several threads
  instead (same CPU, less wall time; no reuse).
- D3 (default B): a bound. A: `--max-mutants <n>`. B: no bound; parts 1 and 2 are enough, and a
  caller who wants fewer mutants selects classes. C: a sampled class (every k-th site), listed as
  unmutated with a reason.

## Scope

Derived 2026-11-22 by `story-scoper`. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/mutate.rs` (7747 lines) and the `Mutate` verb of `crates/edge/ess-cli/src/main.rs` — cited
- **Files:** `mutate.rs`: `audit_with_authored` (5861), `measure` (5773), `run_checked` (5637, one `synthesized(ir)` per mutant), `selection` (1792), `UnmutatedReason` (1628, `class()` at 1666), `UnmutatedSite::check` (1722), `report_format` (/8), `unaudited_sites` (4476), `sites_without_a_live_mutant` (4486), `coverage` (4493). The line numbers differ from the story's, which are at 0.97.3. — cited
- **Files:** `main.rs`: `Mutate` args and help (~1000-1060), `conform_mutate` (4713), `finish_mutation_audit` (4974, the exit table), `write_emission` (4852) — cited
- **Tests to write or extend:** `crates/edge/ess-cli/tests/mutate_cli.rs` (stderr, exit 3, byte identity); `crates/verify/ess-conformance/tests/mutation_audit.rs` and `mutation_site_coverage.rs` (the report format and unmutated sites); the work bound goes in `once_per_model_work.rs` (inferred; the story says "an existing `*_work.rs`"); `crates/edge/ess-xtask/tests/` pins for the reference pages — inferred
- **Symbols:** `crate::synthesize::syntheses_started` (`synthesize.rs:3505`, counted at 3599 in `synthesize_in`, whole syntheses only), `UnmutatedReason`, `MutationOut`, `KeepingGrants` — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs`, only for part 2 (a per-command synthesis and a counter of command syntheses) — inferred
- **Documents:** `website/docs/reference/cli.md` (generated by `crates/edge/ess-xtask/src/cli_reference.rs` from the clap doc comments; regenerate it, do not hand-edit), `website/docs/reference/formats.md` (rows for /8 at line 430), `website/docs/guides/verify/mutation-audit.md`, `docs/design/mutation-audit-and-model-runner.md` and a new design page for part 2 (D2 A), `CHANGELOG.md` — inferred
- **Generated schema:** none found. `schemas/` holds no mutation-report schema, and `RawSpecFile` is not touched, so `cargo xtask schema` is not needed — cited, from `git grep`
- **Confidence:** high for parts 1 and 3 (the source site is named and read); medium for part 2 (the soundness key is unwritten)
- **Would collide with:** any unit in `mutate.rs`, the `Mutate` verb in `ess-cli/src/main.rs`, or `synthesize.rs` (`synthesize_in` and its memos) — part 2 only
- **Safety fact:** parts 1 and 3 change no verdict because `measure` is untouched and the printing sits beside the loop at `mutate.rs:~5905`; stderr is checked only by `mutate_sets_entries.rs:117` through a `note:` filter, so progress lines must not start with `note:` — step 2, unproven

## Decisions taken

Taken for this wave on 2026-11-22, 22:47 CET, as part of decision D24 (option A): the story's own
defaults stand.

- D1 A: progress lines go to standard error in every format.
- D2 A: a design page for reuse across mutants comes first.
- D3 B: no bound on the number of mutants.

This wave builds the progress unit and the design page. The reuse itself follows the page.

Its units are scheduled for the wave after the wave of 2026-11-23 (decision D33).

## Cost of a mutant, 2026-11-27

Cost of a mutant, reported 2026-11-27 on a downstream model: about 20 s per mutant, so the whole class set of that model exceeds a pool job's 30-minute limit (the most a job may name, `slot --timeout 30m`). It agrees with the measure under Need (one full synthesis of the mutated model per mutant, about 79e9 instructions on that report's model) and means a caller must split the audit by class or by site to stay inside one job. The progress lines of D1 A show how far a cut run got; the reuse across mutants (D2 A, the design page) is what lowers the per-mutant cost, and nothing here lowers it yet. Acceptance to add: a run over the model of this report ends within a stated bound of work, or names at its start how many mutants it will decide and the cost of one, so a caller can split it before a job limit cuts it.
