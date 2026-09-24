---
format: aep.planning-md/1
id: review-result:adversary-wave23-unit2-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: a comment in ci.yml turns the gate red'
relations:
- reviews: story:host-path-lane-detector-bounds
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-0ea66f08768f` over base `46c7281e`.
Verdict **NEEDS-CHANGE**. Cases executed 25 → 30, red 3. Origin: introduced 2, pre-existing 1.

Five cases added in `crates/edge/ess-xtask/tests/host_paths_adversary_3.rs`; each calls
`assert_current()` and drives the transcription, so none can measure a fossil.

## The blocker: a comment in `ci.yml` turns the gate red

The expression form is recognised only when the value is **exactly** `${{ matrix.<key> }}`. A
trailing YAML comment, or a matrix written with `include:`, yields a "label" that is the raw
expression, and `the_markers_cover_the_home_root_of_every_platform_ci_runs_on` then panics naming a
platform that does not exist.

`ci.yml:77` is `runs-on: ${{ matrix.runner }}` today, and that same file already carries trailing
`#` comments on value lines (`ci.yml:65`). Adding one to line 77 turns `task check` red on an
otherwise unchanged repository. **The base parse was green on both shapes.**

## Verified claims

The 19-occurrence count is correct: 19 marker occurrences across the 1,023 tracked files under the
four scanned trees, predecessors backtick ×14 and `"` ×5, zero at line start, zero escaped or
`%2F`-spelled. Encoded as a green control. The re-copy is byte-identical and machine-checked —
`transcription_drift` compares each of the ten `TRANSCRIBED` functions signature to closing brace.

## Attacked and could not break

Every predecessor the brief named — start of file, newline, CR, tab, space, `(`, `[`, `<`, `=`, `:`,
`,`, `'`, `"`, backtick, `)`, `]`, `{`, `}`, `*`, `|`, `>`, `#`, `$`, `%`, `&`, `!`, `?`, `;`, NBSP,
U+2026, an emoji, a combining mark, U+FFFD, a euro sign — all still collected. The `file://` and
serialized-escape reasoning is correct and load-bearing: **4,528 real occurrences** in the planning
journal depend on the `escaped` branch and all survive it. `~` is refused as a predecessor and `home-path:sha256:b8b761895b57202fd4fda27490ddf4361180a61f87bbd87145a47c74df44f694`
stays uncollected. Account names starting with a digit, `_`, `-`, `.`, `+`, `@`, Cyrillic, CJK,
Devanagari, a Roman numeral, a circled letter, a superscript, a fullwidth letter: all collected
whole. `ci_runner_labels` over the real `ci.yml` resolves `matrix.runner` correctly; a `runs-on:`
echoed inside a `run: |` block is not picked up; an inline `[self-hosted, linux, x64]` array is read
and fails loudly as intended.

```findings
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 483
  category: mutant
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a trailing YAML comment on ci.yml's own `runs-on: ${{ matrix.runner }}` line, or a matrix written with `include:`, makes the new parse emit the raw expression as a runner label and the markers case panic naming a platform that does not exist, turning the gate red on a clean repository where the base parse was green."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 471
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "a `runs-on:` written as a YAML block sequence contributes no label at all, so a platform with no home root still arrives in silence, which is both what acceptance bound 3 asked to close and the opposite of what `list_items` own doc promises."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 794
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the repository gate now writes a fixture tree under `$TMPDIR`, prints 'retained fixture' on every run and leaks the directory on failure, through a `throwaway` that duplicates host_paths_adversary_2.rs:25 verbatim without being in TRANSCRIBED."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 156
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "`-` and `+` are path characters to `continues_a_path`, so an absolute home path in the content column of a unified-diff line is dropped by the new predecessor rule, and docs/reviews/ carries four tracked .patch files inside the scanned trees, though no tracked file reaches that state today."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 25
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: "the module doc states a Windows home contains no marker, but the forward-slash spelling of a Windows home directory that git and cargo print is collected with its drive letter chopped off, which is the path-on-no-host shape bound 1 exists to eliminate. The literal is omitted because this repository's own secret scanner reads it as a personal path."
```
