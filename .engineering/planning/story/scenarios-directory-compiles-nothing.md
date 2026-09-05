---
format: aep.planning-md/1
id: story:scenarios-directory-compiles-nothing
kind: story
status: active
title: A --scenarios directory of directories compiles nothing and exits 0
summary: The flag does not descend, and a corpus root silently yields a suite with none of the corpus
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/authored_scenarios.rs
revision: 9
---
## The defect

`--scenarios <dir>` reads the files directly in that directory and does not descend. A directory
holding only subdirectories therefore compiles **nothing**, reports the run as normal and exits 0.

Observed 2026-09-04 against `acd/specs` at `d87c009`, whose 67 authored scenarios live in
`authored/routing/`, `authored/slotmatcher/` and `authored/e2e/`:

```console
$ ess verify conform synthesize --path model --scenarios authored --target go --out <dir>
92 scenario(s) (0 authored), 20 refusal(s), 5 file(s) written
$ echo $?
0
```

```console
$ ess verify conform synthesize --path model --scenarios authored/e2e --target go --out <dir>
114 scenario(s) (22 authored), 20 refusal(s), 5 file(s) written
```

The `(0 authored)` is the only thing said, in a line whose other numbers are large, and the suite
that comes out is a valid suite. A CI job that pointed at the corpus root would publish a green
result over a suite holding none of the corpus.

## Why it matters here

`acd/specs/.gitlab-ci.yml` names each of the three areas as a separate invocation and asserts a
scenario count, which is the workaround, arrived at by hitting this. Nothing warned; the count
assertion caught it.

## What it should do

A `--scenarios` path that resolves to zero `ess-scenario/1` documents is a refusal, named as one,
exiting non-zero. Where the directory contains subdirectories that do hold documents, the refusal
should say so — that is the case somebody will actually hit.

Whether the flag should recurse instead is a second question and a larger one; a suite whose
contents depend on directory depth is its own hazard. The refusal is worth having either way.

## Scope

Derived 2026-09-06 by `aep-drive:story-scoper` against clean published ESS `ba43fda29de637ad9323d96c4bb9aac10f48ae64`; every finding below distinguishes observed scope from proposed implementation — cited.

- **Primary surface:** `crates/edge/ess-cli` — cited; this crate owns explicit scenario-path discovery and every command that invokes it.
- **Production file:** `crates/edge/ess-cli/src/main.rs` — cited; `authored_sources` at line 2682 returns an empty successful collection for an explicit directory with no immediate matching entries, while line 2692 intentionally returns an empty collection when `--scenarios` is omitted.
- **Symbols and callers:** `authored_sources`, `synthesize_suite`, `author_suite`, `conform_web`, and the synthesized-suite branch of `conform` / `ConformCommand::Run` — cited; all four discovery paths propagate the helper's `Result` before their output writes or runner invocation.
- **Implementation boundary:** refuse an empty selection from an explicit directory in the shared discovery helper, identify the requested path, explain immediate `.yaml`/`.yml` selection and how to select the intended child directory or file, and make the subdirectory case actionable without adding recursive selection — inferred; this is the smallest change satisfying the story.
- **Preserved behavior:** omission selects no authored sources; an explicit file is read without extension filtering; directory entries are selected by the existing lowercase `.yaml`/`.yml` rule and sorted by path; `Run` with `--suite` does not read `--scenarios` — cited; these are separate existing branches and must not be collapsed by an unconditional emptiness check.
- **Tests:** `crates/edge/ess-cli/tests/authored_scenarios.rs` — inferred; a focused new subprocess regression file can cover the shared caller matrix, refusal before output creation or replacement, omitted-flag controls, direct-file and shallow-directory success, and committed-suite bypass without modifying unrelated existing test files.
- **CLI help:** the `ConformCommand::Synthesize.scenarios` description in the production file — cited; lines 421–423 incorrectly claim an implicit `scenarios/` default, contradicting the helper and the story's explicit preservation requirement.
- **Documents:** no separate design or public documentation file is required for this bounded refusal — inferred; CLI help is the directly affected explanation, and this change requires no new persisted format or discovery contract.
- **Confidence:** high — cited; the exact empty-success branch and all four shared discovery callers are present in the inspected source.
- **Would collide with:** any unit changing the ess-cli crate, especially its main command dispatch, scenario discovery, CLI help, or authored-scenario regression tests — inferred; retain the existing crate-level scope token for wave collision computation.

## Acceptance

An explicitly supplied scenario path resolving to zero authored ESS scenario documents exits non-zero with an actionable diagnostic rather than a successful empty collection.

## Remediation ownership

Owns the immediate F10 empty-result refusal. Omitted --scenarios retains its existing intentional behavior. Broader typed/recursive mixed-document discovery belongs to story:review-authored-discovery.
