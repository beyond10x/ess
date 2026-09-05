---
format: aep.planning-md/1
id: story:scenarios-directory-compiles-nothing
kind: story
status: draft
title: A --scenarios directory of directories compiles nothing and exits 0
summary: The flag does not descend, and a corpus root silently yields a suite with none of the corpus
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
revision: 4
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

Derived 2026-09-05 by `aep-drive:story-scoper`; scope follows the immediate explicit-path refusal — cited.

- **Primary surface:** `crates/edge/ess-cli` — cited; `authored_sources` discovers explicit scenario inputs and currently returns an empty successful collection for directories containing only subdirectories.
- **Symbols:** `authored_sources`, `conform_synthesize`, `author_suite`, `conform_web`, and the `ConformCommand::Run` dispatch — cited; the shared discovery function feeds these conformance operations.
- **Tests:** refusal, non-zero exit, and nested-document diagnostic regressions within the primary crate — inferred; existing CLI tests provide the subprocess pattern.
- **Documents:** none required — cited; the story requests an operational refusal and diagnostic.
- **Confidence:** high — cited; the shared function visibly accepts an empty directory result, and every relevant caller resides in the primary crate.
- **Would collide with:** any unit changing the ess-cli crate, including its source-discovery code or CLI tests — inferred.

## Acceptance

An explicitly supplied scenario path resolving to zero authored ESS scenario documents exits non-zero with an actionable diagnostic rather than a successful empty collection.

## Remediation ownership

Owns the immediate F10 empty-result refusal. Omitted --scenarios retains its existing intentional behavior. Broader typed/recursive mixed-document discovery belongs to story:review-authored-discovery.