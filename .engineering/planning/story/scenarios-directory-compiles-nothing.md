---
format: aep.planning-md/1
id: story:scenarios-directory-compiles-nothing
kind: story
status: draft
title: A --scenarios directory of directories compiles nothing and exits 0
summary: The flag does not descend, and a corpus root silently yields a suite with none of the corpus
revision: 1
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
