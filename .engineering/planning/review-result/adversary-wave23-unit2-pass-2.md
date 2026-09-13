---
format: aep.planning-md/1
id: review-result:adversary-wave23-unit2-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: the control and the shape guard both fail to measure what they say'
relations:
- reviews: story:host-path-lane-detector-bounds
revision: 1
---
## Pass

`aep-drive:adversary`, pass 2, against `wt-0ea66f08768f` over base `46c7281e`.
Verdict **NEEDS-CHANGE**. Cases executed 30 → 35, red 5. Origin: introduced 6, pre-existing 1.

Five cases added in `crates/edge/ess-xtask/tests/host_paths_adversary_4.rs`, all red, all red the
first time anything executed them. **Every YAML shape was put through `yaml.safe_load` before being
written into a case**; one candidate attack — a tab before `#` — is not legal YAML and is therefore
not in the file.

## The two that matter

**The control that replaced the panic compares a count of distinct labels against a count of
`runs-on:` lines**, and `labels` is a `BTreeSet`. Its own sentence says every line must contribute
one. Both directions are wrong: it refuses a clean workflow whose two jobs share a runner, and
accepts one whose job the parse cannot see at all. `ci.yml` has 2 `runs-on:` lines and 3 labels, so
exactly one duplicate survives and two do not — two added ubuntu jobs, or retiring `macos-15-intel`
plus one added job, each turn `task check` red on a clean tree. The false-green half needs no edit
at all.

**The shape guard is an assertion inside the unit's own case**, applied to the nine workflows in
that case's own table and to **no label read out of a file**. `runner_labels`, `ci_runner_labels`
and `the_markers_cover_…` never consult it, so it cannot fail for any workflow this repository
holds. Findings 2 and 3 are its two halves: a label it would refuse and a label it accepts, both
produced from ordinary workflows without the guard running.

## Attacked and could not break

The diff-column rule cannot drop a real leak — `escaped || diff_column || (…)` only widens
acceptance. A tab before `#` is rejected by PyYAML outright, so the junk label it produces is
unreachable through legal YAML. YAML anchors pass the shape guard and are not runner labels, but
GitHub Actions does not support anchors. `without_comment` on a URL fragment is correct: YAML only
opens a comment after whitespace. `list_items` invents no label on an empty item, a lone `-`, or a
nested list. Every `runs-on:` line in today's `ci.yml` contributes at least one label under the
per-line measurement, so the control is green today for the right reason as well as the wrong one.
`transcription_drift` covers all thirteen functions and `assert_current()` passed in all five cases.

```findings
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 990
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the control that replaced the panic compares a count of distinct labels against a count of runs-on lines, so it refuses a clean workflow whose two jobs share a runner and accepts one whose job the parse cannot see at all."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 965
  category: judgement
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the shape guard that stands in for a hand-maintained expectation list is an assertion over the nine fixtures in its own case and is never applied to a label read out of a workflow file, so it cannot fail for any workflow this repository holds."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 530
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "runner_labels documents without qualification that an unresolvable expression contributes no label, and applies that rule to the runs-on value only, so a matrix entry spelled as an expression is returned as a runner label and the markers case panics naming a platform that does not exist."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 492
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "matrix_values reads a key across the whole file and its doc calls the over-collection harmless because an extra label is merely checked, when an unmatched label is panicked on — a second job matrixing os over container images turns alpine and debian into runner labels, both accepted by the shape guard."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 468
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "block_sequence reads one of the three legal spellings: items at the key's own indentation yield no label at all, and a comment or blank line between items truncates the list, which is the silence on an unknown platform its own doc says it exists to end."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 530
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a runs-on written as the documented group/labels mapping contributes no label, and unlike the block-sequence spellings this is a regression, because the base commit's substring parse did see that job's runner."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 433
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "without_comment cuts at the first space-hash with no notion of quoting, so a legal quoted value containing a hash becomes a shorter label; nothing was shown to reach it."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 530
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "a runs-on line inside a run heredoc is read as a runner label and panics the markers case where the base parse emitted nothing, but no step in this repository writes one."
```
