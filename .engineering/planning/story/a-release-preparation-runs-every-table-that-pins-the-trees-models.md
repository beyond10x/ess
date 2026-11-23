---
format: aep.planning-md/3
id: story:a-release-preparation-runs-every-table-that-pins-the-trees-models
kind: story
status: draft
title: A release preparation runs every table that pins the tree's models, so adding a model file cannot reach a tag with a table unpinned
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/lowered_definitions_table.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/selection_precedence_table.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
revision: 12
---
## Outcome

A change that adds, renames or removes a model file (a YAML file whose first lines carry
`format: ess/N`) cannot reach a release tag while a committed table that pins the tree's models
does not list it. A fast check in the release preparation names the table, the missing or stale
labels and the command that re-pins it, before the tag exists.

## Need

Tag `0.97.2` was never released. Its release gate failed on two tests, and 0.97.3 (tag `0.97.3`,
commit `cd9698fd65`) carries its content:

- `ess-compiler::selection_precedence_table`: "the tree holds models the table does not pin";
  0.97.2 added 8 fixture models under `crates/verify/ess-conformance/tests/` and the table held 393
  of 401 (re-pinned by commit `723b6a0641`: models 393 to 401, commands 781 to 824).
- `ess-entity-runtime::lowered_definitions_table`: models 473 to 483 (re-pinned by commit
  `dff06d5ece`).

Both tests walk the repository for model files and compare with a committed table. The local
checks of the release were the named test targets of `AGENTS.md` § Gate, "A gate of named test
targets includes the repository-wide byte pins". That list names `synthesis_suite_bytes_table`,
`synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`,
`external_beside_held_guard` and five more targets (nine in all), and neither of these two. Cost, as reported by
the session that cut the release (not re-measured here): one red release run of 73 minutes and a
second release.

The tables are three test files with four committed tables, each with its own re-pin variable:

| test file | table | re-pin variable |
|---|---|---|
| `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` | `synthesis-suite-bytes-table.tsv` | `SYNTHESIS_SUITE_BYTES_TABLE_WRITE` |
| the same file | `synthesis-component-suites-table.tsv` | `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` |
| `crates/specify/ess-compiler/tests/selection_precedence_table.rs` | `selection-precedence-table.tsv` | `SELECTION_PRECEDENCE_TABLE_WRITE` |
| `crates/generate/ess-entity-runtime/tests/lowered_definitions_table.rs` | `lowered-definitions-table.tsv` | `LOWERED_DEFINITIONS_TABLE_WRITE` |

Each test carries its own copy of the walker: it skips `target`, `node_modules`, `.git` and
`.engineering`, keeps a YAML file with a line starting `format: ess/`, and labels it by its path
from the repository root; the label rules differ at the edges (the precedence table also reads
each `examples/<name>` directory as one system, the lowered table reads every directory holding a
`system.yaml` as `system-dir:<dir>`, the bytes table has an exemption list). The table's first
line is `#<TAB>models=N...`, and every later line starts with the label. A table is slow to
re-pin (the bytes table synthesizes every model) and fast to compare by label.

## Design

Options, each a lettered decision below:

- A. One fast `ess-xtask` check, `cargo xtask model-tables --check`, called by `release-check` in
  `Taskfile.yml` (which `ci-lint` runs, so `task release-check` and `task ci-lint` both run it).
  It walks the tree once with the rules above, reads the label column of each of the four
  tables, and compares the sets: no parse, no compile, no synthesis. It fails naming the table,
  the labels the tree holds and the table does not (and the reverse), and the re-pin command of
  that table, which it prints from the test's own `REPIN` text.
- B. Add `selection_precedence_table` and `lowered_definitions_table` to the list of byte pins in
  `AGENTS.md` § Gate. It costs a sentence and a reader who follows it; it would have caught
  0.97.2 only for a session that reads the list.
- C. Both.

Drift between the check and the three walkers is the check's risk: it holds its own copy of the
rules. An `ess-xtask` test runs the check against the committed tables on the tree at the commit
(all four must pass), so a change to a walker that moves its label set fails there too.

## Acceptance

- `cargo xtask model-tables --check` passes on the tree, and a commit that adds a model file
  (for example a copy of a fixture with `format: ess/N` on its first lines) without re-pinning
  fails it, locally, naming the table and the label and printing the re-pin command, in under 10 s
  wall time (the time on the tree, whose tables hold 401 to 483 models, is in the pull request; the three
  tests of this story's tables take minutes on the same tree).
- The same holds for a removed model file and for a renamed one (a stale label, a missing label),
  one `ess-xtask` test per case on a temporary copy of the tables.
- `task release-check` and `task ci-lint` run it (`Taskfile.yml`: one `cargo xtask` line in
  `release-check`); `crates/edge/ess-xtask/tests/ci_lanes.rs` fails when the line is missing.
- `AGENTS.md` § Gate names all three table tests (`synthesis_suite_bytes_table`,
  `selection_precedence_table`, `lowered_definitions_table`) beside the byte pins, with the four
  re-pin variables and the new check as the fast way to see which table is stale.
- The change adds no test over 5 s: the new tests time under 5 s each (times in the pull request).

## Decisions

A. How a model file added without re-pinning is stopped:
   1. option C, the check and the `AGENTS.md` names (recommended: the check stops it for every
      author without a rule to read, the names keep the three table tests in a named-target gate
      that skips `release-check`);
   2. option A only (the three table tests stay unnamed in `AGENTS.md`; the acceptance line on
      `AGENTS.md` is dropped);
   3. option B only (a sentence, no program: 0.97.2 shows what a list that must be read costs).

## Out of scope

- Re-pinning a table: that stays the three tests' own `*_TABLE_WRITE` runs.
- Making the three walkers one shared walker: that is a change of the tests, not of the release
  preparation, and the bytes table's exemption list stays where it is.
