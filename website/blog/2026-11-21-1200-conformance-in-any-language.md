---
title: "0.97 — A conformance run holds a program in any language to its suite"
description: >
  0.97.0 adds ess verify conform run --target process, which starts a program written in any
  language and asks it the questions of a conformance suite over the new line protocol
  ess-conformance-target/1, and ess verify conform serve, which answers that protocol for a
  built-in reference. The text report escapes control, line-separator and bidirectional characters.
slug: conformance-in-any-language
tags: [release, ess]
date: 2026-11-21T12:00:00+01:00
release_tag: "0.97.0"
---

0.97.0 adds a conformance target that is a separate program, the reference peer for it, and an
escaping fix in the text report. No source format moves. The new format is the line protocol
`ess-conformance-target/1`, released in 0.97.0. The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change.

{/* truncate */}

## Before you upgrade

- The text report of `ess verify conform run` writes a C1 control, U+2028, U+2029 and a
  bidirectional override or isolate (U+202A to U+202E, U+2066 to U+2069) as `\uXXXX`, in values and
  in member names. Text without those characters keeps its bytes; a tool that compares text reports
  byte for byte sees a change only where a target reported such a character.
- No other output and no format an existing reader reads changes.

## A target in another process

- `ess verify conform run --target process -- <program> [args…]` starts the program without a
  shell, in its own process group and in `--target-dir` (the current directory when absent). It
  asks each question of the conformance target as one JSON object per line on the program's
  standard input and reads one answer per line from its standard output. The Rust runner judges the
  answers and writes the report a built-in target writes.
- `--target-timeout` is the seconds `ess` waits for each answer, 1 to 3600, 30 when absent, plus
  the time an operation asks to let pass.
- A fault ends that scenario `error`, kills the program's process group and restarts the program
  for the next scenario: the program exits or closes its output, answers after the limit, answers a
  line that is not valid for the operation or another request's `id`, writes a number binary64
  cannot hold or a line over 16 MiB.
- Exit 2, with no report, for no command after `--`, a program that cannot be started, a first
  handshake that fails or names a version `ess` does not read, and the process options with another
  target. SIGINT, SIGTERM and SIGHUP kill the program's process group and end `ess` by that signal.
- A process target's implementation `name` and `version` are refused over 256 bytes, and every
  string of a `target_error` is cut at 4096 bytes and escaped, with a backslash doubled.
- The guide page [An implementation in any
  language](https://github.com/beyond10x/ess/blob/main/website/docs/guides/verify/target-process.md)
  holds a complete adapter in Python that a test runs. The design is
  [conformance-target-process.md](https://github.com/beyond10x/ess/blob/main/docs/design/conformance-target-process.md).

## The reference peer

- `ess verify conform serve --target billing|oracle-fixture` answers `ess-conformance-target/1`
  for a built-in reference until its input ends. `ess verify conform run --target process -- ess
  verify conform serve --target billing` writes the report `--target billing` writes.
- The library half is public as `ess_conformance::process` (`ProcessTarget`, `serve`).
