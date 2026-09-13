---
format: aep.planning-md/1
id: review-result:adversary-wave22-unit2-pass-1
kind: review-result
status: active
title: Adversary pass 1 against the host-path gate lane
relations:
- reviews: story:fixtures-carry-workstation-paths
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-5677d3fc5e9b` at `ab02915b` over base `c8023067`.
Verdict **NEEDS-CHANGE**. Cases executed 137 → 142, red 4.
Origin: introduced 5, pre-existing 1, undecided 0 (seven findings; two carry no red case).

Five cases added in `crates/edge/ess-xtask/tests/host_paths_adversary.rs`. The file transcribes the
lane's `continues_a_path` and `home_paths` **verbatim** and proves the transcription with a case, so
the four red cases attack the lane's own bytes rather than a paraphrase. It contains no home-marker
literal — every marker is joined at run time, for the reason the lane itself stopped writing its
controls as literals.

## The measurement that decides finding 1

`host_paths.rs` depends on no crate item, so a copy compiles standalone with `rustc --test`.

```
/Users/ → /Userz/ (one macOS marker corrupted)
  the_detector_finds_each_home_directory_spelling_and_no_portable_one ... ok   exit 0

HOME_MARKERS replaced entirely with ["/nope-a/", "/nope-b/", "/nope-c/"]
  test result: ok. 3 passed; 0 failed;   exit 0
```

The whole lane — including the repository scan that is otherwise red in that tree with 571 `docs/`
findings — goes **fully green** when the marker set is replaced with three strings no host uses.
The detector case builds its expected control from `HOME_MARKERS` itself, and the scan case asserts
only that files were read. Nothing in the lane compares the constant to the world.

## Attacked and could not break

The two-step rehash holds: `input-catalog.json` → `66a5eabe…` matches the pin at
`semantic-plan.json:17`, and `semantic-plan.json` → `d1cc575f…` matches the pin at
`coverage_producer.rs:99`, recomputed from the committed blobs. The old digests survive only in the
append-only journal and as a dated citation. All ten new relative `original` values resolve to real
tracked files and each `original_sha256` equals that file's `sha256sum`. All 1016 tracked files
under the four trees are mode `100644` — no symlinks, no submodules — and extension is not used as
a filter, so extensionless files are scanned. `/Users/` is justified and `C:\Users\` is not needed:
CI runs `ubuntu-latest` and `macos-15`, no Windows runner. `file!()` is root-relative under this
build; `.cargo/config.toml` carries only an `[alias]`.

```findings
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 43
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the detector case derives its expected control from HOME_MARKERS itself, so replacing the whole marker set with three strings no host uses leaves all three cases of the lane green at exit 0, including the repository scan.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 138
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the scan silently drops the 2 of 1015 selected tracked files that are not UTF-8, without counting them, and the read > 0 guard cannot see the drop because read is incremented after the continue.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 125
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: a marker ending a sentence is reported as a finding whose text is the marker's parent directory, contradicting the module doc at :105-107; constructed, since no tracked file spells it today.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 82
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: git ls-files --cached quotes any non-ASCII path and the line-split selection drops the quoted line before reading or counting it; constructed, since no tracked path under the four trees needs quoting today.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 114
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: a home path escaped as \/ or encoded as %2F or \u002f is not collected although the acceptance refuses a tracked file containing one, and four tracked files under crates/ already carry the \/ escaping.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 180
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the self-exclusion assertion re-states the filter's own postcondition and cannot fail for any repository state, while the exclusion itself permanently exempts one tracked file under crates/ from the rule the lane enforces and currently guards nothing.
- file: .engineering/planning/journal.jsonl
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: 60 tracked files under .engineering/planning/ still carry absolute home-directory paths, outside the acceptance's four trees, so the repository is not clean after this unit even though the accepted sentence reads as though it is.
```
