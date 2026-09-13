---
format: aep.planning-md/1
id: review-result:adversary-wave24-unit2-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: the decision''s load-bearing comparison is false in its own unit'
relations:
- reviews: story:planning-store-carries-workstation-paths
revision: 1
---
## Pass

`aep-drive:adversary`, pass 1, against `wt-5cf844fd8563` over base `bd722fa9`.
Verdict **NEEDS-CHANGE**. Cases executed 41 → 46, red 4. Origin: introduced 5, pre-existing 1.

Five cases added in `crates/edge/ess-xtask/tests/host_paths_adversary_5.rs`.

## The decision's load-bearing sentence is false in its own unit

The bullet argues the store must stay unscanned because carving the journal out of a widened scan
*"would report the store covered while exempting the file holding the largest single share of the
defect"*. In the unit the same bullet counts in two clauses earlier — lines — the journal holds
**87 of 32,806** carrying lines, 0.27%, and
`.engineering/planning/review-result/authored-discovery-source-pass1.md` holds **28,730**, 87.6%.
A journal carve-out would therefore **cover 32,719 of the carrying lines and 59 of the 60 files**,
not exempt the largest share.

It is true only under a second, unstated unit — findings per line, 32,163 against 28,730 — and the
two units answer the decision oppositely. The bullet does not say which it means.

## The other four

The module doc's **summary line** is the whole of what rustdoc renders standalone, and it claims no
tracked file names a home-directory path while 60 do. The unit redefined it 72 lines further down
rather than scoping the sentence, and a redefinition does not travel with the summary into an index
or a search result. `documented_unread_trees` reads each bullet's **first backtick token and nothing
else**, so a bullet asserting `.engineering/` is *read in full and clean* parses to exactly the same
set as the one asserting it is unread and carries sixty files — every count, comparison and citation
in the decision is unread prose. A continuation line wrapped with a tab ends the section and
silently drops every bullet after it, which suppresses the `stale` half for the dropped tree. And
the four helpers the unit added to the lane were not added to `TRANSCRIBED`, which walks that list
only and never checks the converse.

## Numbers confirmed independently

60 tracked files, 87 journal lines, 699 unread files, 65,838 findings, examined == 699 with no file
skipped. `tracked_files` is byte-equal to `git ls-files --cached -z`, `scanned ⊂ tracked`, and the
two partition 1730 = 1031 + 699.

## Attacked and could not break

Anchor deletion panics rather than passing. Near-miss spellings, a deeper path and a prefix
collision each fire both assertions. A tree named outside the section is not seen, and the
undocumented half then fires. Bullet reordering is red both ways. Mass unreadability empties
`carrying` and the `stale` half fires, so the case is self-protecting. No carrying tree exists
outside `.engineering/`.

```findings
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 1
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "the module doc summary line rustdoc renders standalone claims no tracked file names a home-directory path while 60 do, and the unit redefined it 72 lines down instead of scoping it."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 61
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the decision rests on the journal holding the largest single share of the defect, and in the bullet's own unit it holds 87 of 32806 carrying lines against 28730 in one review-result, so a carve-out would cover 99.7% of the defect rather than exempt the largest share."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 1227
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "documented_unread_trees reads only each bullet's first backtick token, so a bullet asserting the tree is read in full and clean parses identically to the one asserting it is unread and carries sixty files."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 1244
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a continuation line wrapped with a tab or two spaces ends the section and silently drops every bullet after it, which suppresses the stale half and leaves wrong documentation with a green lane."
- file: crates/edge/ess-xtask/tests/host_paths_lane/mod.rs
  line: 41
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the helpers this story added to the lane were not added to TRANSCRIBED, and transcription_drift never checks the converse, so the transcription warranty silently does not cover the new surface."
- file: crates/edge/ess-xtask/tests/host_paths.rs
  line: 64
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the bullet's closing appeal that layout.rs excludes the same tree for the same reason contradicts the SCANNED_PREFIXES doc 25 lines above, which states the two scans refuse different things."
```
