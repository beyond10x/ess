---
format: aep.planning-md/3
id: review-result:ess-054-w3-3-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W3-3 validate reporting
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-434
- reviews: story:feature-request-437
revision: 1
---
**W3-3 adversary report, pass 1**

```
unit: W3-3 validate-reporting, commit 69d7c2d5f (range 97271a43d..69d7c2d5f), plus three uncommitted adversary cases
verdict: NEEDS-CHANGE
cases: executed 17→20, red 3
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (scratch ~/.cache/ess-054-wave/W3-3/adv1/, build dir /dev/shm/ess-054/W3-3, now cleaned)
needs-coordinator: yes. Story 434 accepted "slower on large models (unmeasured)". Measured on a 119-line committed model, validate goes from 5 ms to about 60 s in a release build. Should text mode run synthesis at all?
```

**1. `git --no-pager diff --stat`**
```
 crates/edge/ess-cli/tests/validate_completeness.rs | 38 +++++++++++++
 .../ess-cli/tests/validate_implied_relations.rs    | 66 ++++++++++++++++++++++
 2 files changed, 104 insertions(+)
```
Both paths are test files. No implementation file was touched.

**2. Cases added (each run alone first; all red now)**

| Case | Asserts | Red output (first run) |
|---|---|---|
| `validate_completeness.rs:518` `adversary_text_validate_does_not_wait_on_synthesis` | Text-mode `validate` on `crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml` exits 0 within 30 s | `text-mode `validate` gave no verdict within 30s on …/conditional-measures-generated.yaml` |
| `validate_implied_relations.rs:428` `adversary_the_hinted_relation_on_a_renamed_entity_validates` | When ESS-ENTITY-019 fires on `Grant.secret`, declaring the `references` relation its hint names still validates | `ESS-ENTITY-019 warns on `Grant.secret`, and the `references` its hint names is refused: … [unsupported_construct] command.demo.vault.RenameSecret.outcomes.renamed.sets.name: `demo.vault.Secret` is carried by `demo.vault.Grant.secret` … a re-key of its identity would leave every carrier naming a record that is gone` |
| `validate_implied_relations.rs:465` `adversary_the_owns_the_hint_offers_for_an_optional_field_validates` | For an `Optional<PoolId>` field, the `owns` alternative the hint offers validates | `the hint offers `… or an `owns` relation on `probe.staff.Pool` carried by `pool``, and the `owns` it names is refused: … [type_mismatch] entity probe.staff.Pool.relations.agents: … typed `Optional<probe.staff.PoolId>` (hint: `probe.staff.Agent.pool` must be typed `probe.staff.PoolId`)` |

**3. Suite run, made after the cases existed (the unit's two named targets)**
```
cargo test --locked --offline -p ess-cli --test validate_completeness --test validate_implied_relations
validate_completeness: test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.08s
  failed: adversary_text_validate_does_not_wait_on_synthesis
(cargo stops after the first failing target, so the second target was run on its own)
cargo test --locked --offline -p ess-cli --test validate_implied_relations
test result: FAILED. 8 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s   EXIT=101
  failed: adversary_the_owns_the_hint_offers_for_an_optional_field_validates, adversary_the_hinted_relation_on_a_renamed_entity_validates
```
`executed 17` is these runs with my three cases left out (9 + 8). `rustfmt --check` on both files: exit 0.

**4. Findings (they cover commit 69d7c2d5f)**

| file:line | What breaks | Case | Verdict / origin |
|---|---|---|---|
| `crates/edge/ess-cli/src/main.rs:2531` | Validate now runs a full conformance synthesis in every format, and text mode throws the result away. On the fixture above: debug validate 241 s against under 1 s for compile. Release 0.53.0 takes 60 s for `conform synthesize` and 5 ms for `validate`. Exit status still ends as 0. The sweep's 120 s timeout recorded 124 for this one file. **What reaches it:** every `ess specify validate` call on a model whose synthesis is slow. **Fix:** build `Completeness` only for json/yaml, keeping the unknown-`--component` check. | `adversary_text_validate_does_not_wait_on_synthesis` | NEEDS-CHANGE / introduced |
| `crates/specify/ess-domain/src/entity.rs:1521` | **Warning that cannot be silenced.** On an `ess/23` model that renames entity X (re-key), a stored X-identity field on another entity warns. Both repairs the hint names (`references`, or X's `owns`) make the rename refused (`identity_write.rs`, "an entity a declared relation carries"). The guide is wrong for this case: `layout-and-validation.md:188` says "Declaring the relation the hint names silences the warning". **What reaches it:** the #429 rename shape plus any entity storing the renamed entity's identity. **Fix:** skip the lint when the target's identity is written by an identity-changing update, or say so in the hint and the guide. | `adversary_the_hinted_relation_on_a_renamed_entity_validates` | NEEDS-CHANGE / introduced |
| `crates/specify/ess-domain/src/entity.rs:1570` | The hint offers `owns` for `Optional<…>` and `List<…>` fields, but `owns` is only carried by a field typed exactly as the owner's identity (`carried_types`, entity.rs:1768). The same hint text is in `related_guard.rs` (for example `related-guard-stored-reference.yaml`, `Optional<TaskId>`). **Fix:** offer `owns` only when the field's type is exactly the identity. | `adversary_the_owns_the_hint_offers_for_an_optional_field_validates` | CONFIRMED / introduced |
| `crates/edge/ess-cli/src/main.rs:2531` | Judgement call. Fit review §4 of story 434 says "Text mode adds one summary line". It was not implemented, and the guide now says text output does not change. In text mode, `--component` does nothing except refuse an unknown name. No case written. | none | CONFIRMED / introduced |

**5. Attacked and could not break**
- **Exit status and stdout:** compile vs validate (JSON) over 175 specs in examples/, models/ and crates/**/fixtures. Exit codes are identical except the timed-out file, and there are no panics (no 101). Text stdout and the refused path are unchanged by construction.
- **IR:** the compile path passes `advise=false`. The `referenced_entity` refactor (`related_value.rs`) is equivalent to the base.
- **Completeness:** `counts` are built from the list lengths, so they cannot disagree. `--component` changes only `outside`, because `synthesize_for_in` passes on the whole model's refusals and notes. The object is left out exactly when all four lists are empty.
- **Self-reference and Optional/List:** `Node.parent: Optional<NodeId>` is silenced by a self `references`. Optional/List fields are silenced by `references` one/many.
- **Shared identity and old formats:** an identity shared by two entities is skipped. Relations are admitted at ess/10 and ess/17, so warnings on old formats can be silenced.
- **Rule (b) via a subject field:** an ess/22 subject-field `via` warns ESS-COMMAND-019. All 17 warned fixtures in the sweep are true positives.
- **Not run:** `cargo xtask diagnostics --check` (drift check of the generated diagnostics page) and the Go/TypeScript lanes.
- **Charter:** no worktree session lease was acquired.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W3-3/adv1/`: `build.log`, `sweep.tsv`, `slow.json`, `slow.err`, `slow.status`, `rel.out`, `probe-rekey/`, `probe-rekey-declared/`, `probe-self/`, `tmp/`. The throwaway `sweep.sh` is deleted.
- `/dev/shm/ess-054/W3-3`: build dir and test scratch; `cargo clean` removed 1.2 GiB.
- Harness task output files under `~/.cache/claude-tmp/claude-1000/`.

**7. Findings block**
```findings
- file: crates/edge/ess-cli/src/main.rs
  line: 2531
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'validate synthesizes the whole suite in every format and text mode discards it; on a committed 119-line model validate goes from 5 ms to about 60 s in release (241 s debug) with no output change'
- file: crates/specify/ess-domain/src/entity.rs
  line: 1521
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'on an ess/23 model that re-keys entity X, ESS-ENTITY-019 on a field storing an X identity cannot be silenced: both repairs its hint names make the rename refused, contradicting layout-and-validation.md:188'
- file: crates/specify/ess-domain/src/entity.rs
  line: 1570
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the hint offers an owns relation for Optional and List fields, which validate_relations refuses because owns is carried only by a field typed exactly as the owner identity (same text in related_guard.rs)'
- file: crates/edge/ess-cli/src/main.rs
  line: 2531
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'fit review 434 section 4 promised a text-mode summary line; none is printed and --component has no effect in text mode beyond refusing an unknown name'
```
