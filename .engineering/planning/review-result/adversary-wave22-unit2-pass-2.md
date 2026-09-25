---
format: aep.planning-md/2
id: review-result:adversary-wave22-unit2-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: the host-path lane''s own controls'
relations:
- reviews: story:fixtures-carry-workstation-paths
revision: 1
---
## Pass

`aep-drive:adversary`, pass 2, against `wt-5677d3fc5e9b` at `44a67861`, base `c8023067`.
Verdict **NEEDS-CHANGE**. Cases executed 12 → 21, red 7 (6 new, 1 the known `docs/` case).
Origin: introduced 6, pre-existing 1, undecided 0 (eight findings).

Two jobs. **Job 1 repaired pass 1's own harness**: `host_paths_adversary.rs` was 0/5 because it
froze a copy of the old detector and carried obsolescence guards every possible fix trips. Rewritten
to 7 cases, all green, with the transcription made mechanical — `tests/host_paths_lane/mod.rs`
copies seven of the lane's functions with **no substitution** and reads `HOME_MARKERS`,
`SEPARATOR_SPELLINGS`, `SCANNED_PREFIXES` and `RUNNER_HOME_ROOTS` out of the lane's source at run
time, so a constant cannot drift and a drifted function names itself. `assert_current()` runs at the
head of every case in both files, so no case can report a fossil.

**Job 2 attacked the correction's new controls**, in `host_paths_adversary_2.rs`: 7 cases, 6 red.

## The mutant that still survives

Dropping `home-path:sha256:05292b25783aaf4e4f66300d2a83cc809bae6ec68927bf615cedbfd4ca174e6f` and its control tail from `HOME_MARKERS` leaves the lane's six non-scan cases
green, byte-for-byte identical to the unmutated copy. Dropping `/Users/` **is** caught. The hole is
specific to the one marker no CI runner names — the CI-label control never reaches the superuser,
and everything else derives from the constant. Same class as pass 1's finding 1, one marker
narrower.

## Attacked and could not break

The `\/` / `%2F` / `\u002F` normalisation over all 1016 selected files produces **0** findings the
plain spelling did not already produce; every one of the 571 is spelled plainly in its file — now a
case. The `candidate.len() > directory.len()` rule survived a 36-input grid. `git ls-files -z` with
a NUL split survives a real repository tracking `docs/straße.md`. `ci_runner_labels` against the
real `ci.yml` returns `{ubuntu-latest, macos-15, macos-15-intel}` with no phantom label, and all
seven workflow files run only ubuntu or macos, so reading `ci.yml` alone misses no platform today.

```findings
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 43
  category: mutant
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: dropping home-path:sha256:05292b25783aaf4e4f66300d2a83cc809bae6ec68927bf615cedbfd4ca174e6f and its control tail from HOME_MARKERS leaves every case in the lane green, because the CI-label control never names the superuser and every other expectation derives from the constant.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 371
  category: judgement
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the gate's verdict now depends on the HOME of the account that runs it — outside the three marker roots it refuses a clean repository and blames the marker set, and unset it panics on an expect that POSIX does not guarantee.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 184
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: the reader panics on a selected file the working tree no longer holds, so one unstaged deletion aborts the scan before it reports the other 1015 files, and the assert_eq!(read, selected.len()) guard it bypasses cannot fail anyway because read is incremented unconditionally in a loop no branch leaves early.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 305
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: RUNNER_HOME_ROOTS maps windows to the macOS home root, so a Windows runner would be reported as covered while the native spelling C:\Users\... matches no marker, which is the one case the table exists to fail.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the commit message's stated bound is wrong in the lane's favour — /export/home/... and /var/home/... do contain a marker and are collected, truncated to a path that exists on no host, rather than being outside the markers as claimed.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 313
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: ci_runner_labels is documented as failing when CI adds a platform with no home root recorded, but it only searches for families RUNNER_HOME_ROOTS already names, so an added platform produces no label and the case passes on the runners it knows.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 158
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: the detector never asks what precedes a marker, so a relative path or URL with a component named home, root or Users is collected and reported as an absolute path that exists on no host.
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 110
  category: acceptance
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: continues_a_path accepts ASCII only, so a home-directory path whose account name starts with a non-ASCII byte is collected as nothing.
```
