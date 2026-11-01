---
format: aep.planning-md/3
id: review-result:adversary-d357-u1-pass-1
kind: review-result
status: active
title: 'D357 U1 adversary pass 1: citing the file a refusal was read from'
relations:
- reviews: story:a-refusal-records-the-document-it-was-read-from
revision: 1
---
unit: D357 U1 (story:a-refusal-records-the-document-it-was-read-from), c536cce11 on unit/ess-d357-u1, reviewed against 700b4faa6
verdict: NEEDS-CHANGE (1 blocker, 1 warning); fixed at df9b2c6ed
cases: executed 6→9, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u1/adv-scratch (cargo log only)
needs-coordinator: none

**Cases.** `crates/specify/ess-compiler/tests/adversary_d357_u1_pass1.rs`: a command declared in `first.yaml` and again in `second.yaml`, where only the second copy writes the refused key. A premise case through the unchanged `diagnose_locating` (green: the old search cited `second.yaml` line 6); `instances:` under `format: ess/1` through `diagnose_traced` (red: cited `first.yaml` line 9); a per-row event `{event: …, each: changed}` with no `instances:` at `format: ess/25` (red: cited `first.yaml`).

**Findings.**
- F1 (blocker, introduced), `crates/specify/ess-compiler/src/resolve.rs:1054`: refusals pushed before the read loop (the set-effect passes and the outcome-group expansion) had no read answer, so `read_from` answered from the kept copy and cited the wrong file and line with confidence. Fixed at df9b2c6ed: the provenance record is a list of marks; each pre-read pass marks each file before refusing in it.
- F2 (warning, introduced), `crates/specify/ess-domain/src/spec.rs:997`: the `Provenance` doc comment promised a fallback the code did not take. Fixed at df9b2c6ed: the comment says which refusals the marks, the kept copy and the search answer.

**Attacked, not broken.** Single-file labels are consistent (`Source` Display equals `as_str`, texts keyed alike); `ConstructKind::as_str` agrees with the kind strings in `kept`; `ValidationErrors` only appends, so indices stay valid; the unit's tests assert literal files and lines; `view.from` refusals carry their own span.

**Residue.** No case exercises the outcome-group marks (the cases reach the set-effect passes); the `--format json` output was not compared with the text output by the adversary (the unit's own CLI test covers both). Accepted at close: the marks share one mechanism.

**Fix-round run (df9b2c6ed).** ess-domain lib 366 and masked-declaration lanes 14/1/2, ess-compiler lib 30, locator_citations 6, adversary_d357_u1_pass1 3, ess-cli validate_names_the_file_read_from 2 and validate_parse_refusals 13; clippy -D warnings and fmt-check clean. No second pass: the fix turns the adversary's own red cases green.

```findings
[
{"file":"crates/specify/ess-compiler/src/resolve.rs","line":1054,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a refusal pushed before the read loop on the dropped copy of a command declared in two files was cited at the kept copy's file and name line; fixed at df9b2c6ed by marking each file in the pre-read passes"},
{"file":"crates/specify/ess-domain/src/spec.rs","line":997,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Provenance documented a search fallback for pre-read refusals while read_from answered from kept; doc comment matches the code at df9b2c6ed"}
]
```
