---
format: aep.planning-md/3
id: review-result:ess-054-w2-1-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W2-1 bulk removal
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-452
revision: 1
---
unit: W2-1 (story:feature-request-452), commit 1d655b92c in `~/.local/state/worktree/trees/b10x/ess/ess-054-w2-1`, plus two new test files left uncommitted
verdict: NEEDS-CHANGE
cases: executed 72→84, red 8
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths (`~/.cache/ess-054-wave/W2-1/adv1/`, `/dev/shm/ess-054/W2-1-base`, which is cleaned and removed)
needs-coordinator: I found no lease command in the `worktree` CLI, so I took no lease.

**1. Diff stat**

`git --no-pager diff --stat` prints nothing, because both added files are untracked. `git status --short`:
```
?? crates/specify/ess-domain/tests/adversary_w2_1_pass1.rs
?? crates/verify/ess-conformance/tests/adversary_w2_1_pass1.rs
```
Both are test files. No implementation file was touched.

**2. Cases added. Each was red on its first run alone, except the four conformance cases, which are green.**

`crates/specify/ess-domain/tests/adversary_w2_1_pass1.rs`: 8 cases, all red.

| case | asserts | first red output (verbatim, trimmed) |
|---|---|---|
| `ess16_preserves_beside_instances_keeps_its_refusal_text` | an ess/16 `preserves:` + `instances:` refusal keeps the base text | `left: "...admitted beside `moves:`, `updates:` and `deletes:` only"` / `right: "...admitted beside `moves:` and `updates:` only"` |
| `ess22_deletes_with_instance_and_instances_keeps_its_conflicting_declaration` | the base's `conflicting_declaration` "both `instance:` and `instances:`" | `[unsupported_construct] ...revoked.instances ... (hint: declare `format: ess/23`, or name one row with `instance:`)`, `[missing_declaration] ...revoked.instance`, `[empty_declaration] ...RevokeTokens.outcomes` |
| `ess22_bulk_delete_with_unparsable_filter_keeps_its_parse_refusal` | the base's `unparsable_predicate` is kept | only the two `unsupported_construct` format refusals |
| `ess23_misnamed_deleting_entry_keeps_later_entry_positions` | an undeclared entity in the 2nd entry is reported at `affects[1]` | `[undeclared_reference] ...deleted.affects[0].entity: `demo.auth.Nowhere` is not a declared entity` |
| `ess23_misnamed_deleting_entry_keeps_the_conflict_pair_positions` | the conflict is at `affects[2]` and names `affects[1]` and `affects[2]` | `...deleted.affects[1]: ... declares `affects[0]` and `affects[1]` over `demo.auth.Token`` |
| `ess23_deleting_entry_with_a_misspelt_entity_reports_the_undeclared_entity` | `entity: demo.auth.Tokn` is reported as undeclared | only `...affects[0]: ... (hint: name the entry's own entity: `deletes: demo.auth.Tokn`)` |
| `ess23_deleting_entry_over_another_domains_entity_is_refused` | a deleting entry over `demo.mail.Mailbox` is refused | `a deleting `affects:` entry over `demo.mail.Mailbox`, beside a `demo.auth` subject, is admitted` |
| `ess15_bulk_delete_is_refused_without_cascade` | no `missing_declaration` or `empty_declaration` after the ess/16 refusal | `[missing_declaration] ...revoked.deletes: ... declares no `instance``, `[empty_declaration] ...RevokeTokens.outcomes` |

One change after a red run: the first version of the `ess22_deletes_with_instance…` document still contained the entry's `deletes:` key, so 0.53.0 cannot parse it. I swapped that entry for `sets:` so the base can read the document, and reran it alone. Same red result; that output is the one quoted above.

`crates/verify/ess-conformance/tests/adversary_w2_1_pass1.rs`: 4 cases, all green. Each one synthesizes an admitted ess/23 shape the unit's fixture does not cover, checks there are no synthesis refusals and that rows are read absent, and checks the interpreted model passes every scenario.

**3. Suite run, after the cases existed**

The command was `cargo test --locked --offline --no-fail-fast`, using the brief's environment, on each lane:

| lane | result |
|---|---|
| ess-domain `set_effects` | 27 passed |
| ess-domain `adversary_w2_1_pass1` | 0 passed, 8 failed |
| ess-conformance `set_effects` / `interpreted_set_effects` / `adversary_w2_1_pass1` | 16 / 12 / 4 passed |
| ess-diff, ess-entity-runtime, ess-gen, ess-synth `set_effects` | 5 / 4 / 4 / 4 passed |

Domain lane EXIT=101; every other lane EXIT=0. The "before" count of 72 comes from the unit's `final-*.log`.

**4. Findings (cover 1d655b92c)**

| # | file:line | what breaks | case | verdict | origin | what reaches it |
|---|---|---|---|---|---|---|
| 1 | `crates/specify/ess-domain/src/command/set_effects.rs:209` (also `:422`) | In ess/16–22 documents, the refusal of `preserves:` + `instances:` now says `deletes:` is admitted beside `instances:`, and the hint offers `deletes:`. The `creates:`/`preserves:` + `affects:` hint now says "moves, updates or deletes". All of these are formats that refuse it. The probe shows base and head differ on exactly these lines. | `ess16_preserves_…` | NEEDS-CHANGE | introduced | Any ess/16–22 document with `preserves:` + `instances:`, or `creates:`/`preserves:` + `affects:`. |
| 2 | `set_effects.rs:1360` | Below ess/23 the pre-pass removes `instances:` and `deletes:` before conversion. That throws away the refusals the base gave first. With `instance:` present, the base `conflicting_declaration` is replaced by a format refusal whose hint asks for the `instance:` already written. It also brings back a `missing_declaration` plus `ESS-COMMAND-007` cascade, which the acceptance says must not happen. With a filter that does not parse, `unparsable_predicate` is lost. | `ess22_deletes_with_instance…`, `ess22_bulk_delete_with_unparsable…` | NEEDS-CHANGE | introduced | ess/22 documents; the base output for both was captured by the probe. |
| 3 | `set_effects.rs:1383` (same at the conversion fallback `:354`) | A misnamed deleting entry is dropped from the list with `retain`. Every later entry's refusal then names a position one lower than the author wrote. A typo in `entity:` is never reported as undeclared, and the hint steers the author to `deletes: <typo>`. The sibling `refuse_affect_moves` removes only the move and keeps the entry. | 3 `ess23_…` position/misspelt cases | NEEDS-CHANGE | introduced | Any ess/23 document with a misnamed deleting entry. |
| 4 | `website/docs/guides/specify/selection-effects.md:139`, `docs/design/set-effects-over-filtered-instances.md:189` | Both pages say `affects:` stays inside its outcome's domain and that removal in other domains is not an `affects:` entry. In fact a deleting entry over another domain's entity is accepted, which is the cross-domain cascade the story declined. A cross-domain *setting* entry is already accepted at base (probe). | `ess23_deleting_entry_over_another_domains…` | NEEDS-CHANGE | introduced | Any multi-domain system; the two-file document validates. |
| 5 | `set_effects.rs:1052` (`refuse_below_ess_16`) | Below ess/16, bulk `deletes:` still cascades into `missing_declaration` plus `ESS-COMMAND-007`, because `deletes` is kept after `instances:` is removed. Base output is byte-identical (probe), so this is not the unit's doing, but it misses the acceptance's "no cascade" below ess/23. | `ess15_bulk_delete…` | CONFIRMED | pre-existing | ess/≤15 documents. |

Fixes I would suggest (not applied):
- **#1:** keep the base wording below ess/23 and add an "and, from ess/23, `deletes:`" clause, as the deletes-specific refusals already do.
- **#2:** skip the pre-pass when `instance:` is present or the filter does not parse, and leave those refusals to conversion.
- **#3:** remove only the entry's `deletes` key, not the whole entry.
- **#4:** either refuse a deleting entry whose entity is in another domain, or correct both pages.
- **#5:** also clear `deletes` in `refuse_below_ess_16`.

**5. Attacked and could not break**

- **Old documents (focus 1):** I ran a throwaway probe, built separately against base 97271a43d and against head, over 179 repository documents before ess/23. Validation reports are identical for all of them. IR bytes (130 documents) and suite bytes (130 suites) are identical. The ess/16, /20 and /22 variants of `set-effects.yaml` are identical too, which confirms that `ResolvedAffect.deletes` is skipped from the output when false.
- **ess/22 `deletes:` subject + `affects:`:** one refusal, no cascade, as claimed.
- **Honest target (focus 2):** the interpreted target passes the whole suite for four more shapes, and removed rows are read absent:
  - a deleting entry over the subject's own entity
  - a deleting entry beside an updating subject
  - two deleting entries over two entities
  - a setting entry beside a deleting subject
- **Too few / too many / leftover rows:** the unit's five mutants cover these. I added no mutant targets of my own.
- **ess-diff:** I read the test and believe a mutant that drops the `deletes` branch would fail it. I did not run that.

**6. Paths written outside the worktree**

- `~/.cache/ess-054-wave/W2-1/adv1/` (114M): `cargo.sh`, `run-probe.sh`, `run-probe2.sh`, `suite.sh`, `logs/`, `tmp/`, `base-src/` (a `git archive` of 97271a43d), `probe-head/`, `probe-base/`, `corpus.txt`, `corpus-extra/`, `corpus-extra2/`, `corpus-extra3/`, `out-head/`, `out-base/`, `out2-head/`, `out2-base/`, `out3-head/`, `out3-base/`
- `/dev/shm/ess-054/W2-1`: cleaned with `cargo clean` (1.9GiB)
- `/dev/shm/ess-054/W2-1-base`: cleaned with `cargo clean` (423.7MiB) and the directory removed

**7. Findings block**

```findings
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 209
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'ess/16-22 refusals of preserves/creates beside instances or affects now name deletes: as admitted and offer it as the remedy, though those formats refuse it'
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 1360
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'below ess/23 the bulk-deletion pre-pass pre-empts the base conflicting_declaration (instance beside instances) and unparsable_predicate, and reintroduces a missing_declaration plus ESS-COMMAND-007 cascade'
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 1383
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'dropping a misnamed deleting entry shifts every later entry refusal position and hides an undeclared entry entity behind a hint to write deletes: <typo>'
- file: website/docs/guides/specify/selection-effects.md
  line: 139
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'the guide and design note say affects: stays inside its domain and is not how other domains remove rows, yet a deleting entry over another domain entity validates'
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 1052
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: 'below ess/16 a bulk deletes: keeps its verb after instances: is refused, so missing_declaration and ESS-COMMAND-007 still cascade'
```
