---
format: aep.planning-md/3
id: review-result:ess-054-w2-4-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W2-4 precedence guide
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-454
- reviews: story:feature-request-455
revision: 1
---
unit: W2-4 (stories 454, 455), range 6192d9be25..32afe8ef4, working tree at 32afe8ef4 plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 143→146, red 3
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths (see part 6)
needs-coordinator: whether a plain refusal pair beside a `when_subject:` branch is in #455's scope, or whether the docs get narrowed and a note added

None of your three focus points broke. I found no case where the new #454 or #455 sends make an honest target fail. The byte-identity claim holds over 706 models. The one finding: a target that takes two overlapping plain refusals in the wrong order still passes when the command also has a `when_subject:` branch. That is exactly the shape the #454 guide recommends.

**1. `git --no-pager diff --stat`**

Empty: nothing tracked was changed. `git status --short` shows one untracked file, a test file:
`?? crates/verify/ess-conformance/tests/adversary_454_455_pass1.rs`. No implementation file was touched.

**2. Cases added** (all in `crates/verify/ess-conformance/tests/adversary_454_455_pass1.rs`)

The model is the #454 fixture plus two plain refusals on `Pause`: `reason-required: reason == ""`, declared before `pause-refused: pause == true`.

| case | asserts | now |
|---|---|---|
| `adv1_refusal_pair_overlap_is_sent_beside_a_held_state_refusal` (:223) | the `reason-required` scenario sends `{reason: "", pause: true}` and requires `reason-required` | red |
| `adv1_pause_first_target_fails_beside_a_held_state_refusal` (:240) | the honest interpreter passes everything (it does), and a target checking `pause` first fails `reason-required` | red |
| `adv1_plain_refusal_is_sent_for_an_unknown_identity_beside_a_held_state_refusal` (:267) | a blank reason is sent for an unknown identity and requires `reason-required` (step 2 before step 3) | red |

Red output from the first run of this file alone (`cargo test -p ess-conformance --test adversary_454_455_pass1`), exit 101:
```
`{reason: "", pause: true}` is sent requiring `reason-required`, the first declared: [(true, Some("reason"), Some("true"), None), (false, ...
checking `pause` before `reason` fails `reason-required`: { ... "demo.session.Pause/outcome/reason-required": Passed, ... all 8 Passed }
a blank reason is sent for an identity nothing stores and requires `reason-required`: [(true, Some("reason"), Some("true"), None), ...
test result: FAILED. 0 passed; 3 failed
```
I ran the same file against a `git archive` copy of the base, and all 3 are red there too.

**3. Suite run** (after the cases existed)

`cargo test -p ess-conformance --locked --offline --no-fail-fast` over 15 test targets: mine, `held_state_input_refusal`, `refusal_pair_overlap`, `arrangement_creators`, `adversary_arrangement_pass1`/`pass2`, `adversary_e_u9_pass1`, `authored_outcome_guards`, `typed_text_operands`, `adversary_222_pass1`, `enum_presence_guard`, `expression_a1`/`a2`, `expression_utf8_bytes`, `mutation_offsets`.

Exit 101. 146 executed, 143 passed, 3 failed (all mine), 4 ignored (already ignored in `adversary_e_u9_pass1`). The 143 is the same run with my file left out.

**4. Findings**

| file:line | what breaks | case | verdict | origin |
|---|---|---|---|---|
| `crates/verify/ess-conformance/src/synthesize.rs:12441` | `refusal_pair_overlaps` only feeds the stateless boundary rows. On a command with any `when_subject:` branch, plain refusals are sent on an arranged row, and that path never sends the pair's overlap. A target taking them in the other order passes, and no unwitnessed-overlap note is raised. The new docs say the overlap is sent with no such limit (`fields-and-invariants.md:48`, `guards-and-predicates.md:220`). | the first two cases | NEEDS-CHANGE | introduced |
| `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (plain refusals sent on an arranged row) | On the same commands, a plain input refusal is never sent for an unknown identity requiring the refusal. A target that looks the record up first passes. The new guide section's step 1 ("checked before the record is looked up") is therefore not witnessed for any command that adopts the #454 idiom. | the third case | CONFIRMED | pre-existing |

- **Why the first row is "introduced" although its cases fail at the base too:** the base has no #455 feature, so any test of it fails there. The unit introduced the rule and the unqualified doc claims; this command shape is a path the rule doesn't reach.
- **What reaches it:** the #454 guide's recommended `when_subject: {predicate: state == ...}` refusal, next to two overlapping plain refusals. That is the reporter's own setup: #455 came from the same run as #454.
- **Fix:** either have the arranged-row refusal path (and its overlap rows) send `refusal_pair_overlaps` too, or limit the two doc sentences to commands without stored-row branches and add an unwitnessed-overlap note for those that have them.

**5. What I attacked and could not break**

- **Wrong-state overlap where the held refusal still holds:** with `state != Closed`, the overlap is skipped on `Paused` and sent on `Closed`. The interpreter passes.
- **Stored-field guards:** a guard on `locked` that holds on the wrong-state row sends nothing. One the row rules out is sent, with a view check first. The interpreter passes.
- **No view:** the overlap is dropped, and the refusals are the same as at base.
- **Create-or-update commands:** validation refuses `when_subject` beside a creating `unknown_instance:` (ESS-COMMAND-004). Separately, `unknown_instances` skips upserts.
- **Unknown-identity sends:** none overlaps a plain refusal; that filter works.
- **Picking the first refusal's main witness:** the new search keeps base's search order and checks every later guard, not just one. A search error falls back to the old witness.
- **Byte identity:** I synthesized 706 models with a separately built base binary and the head binary. That is 169 `.yaml` specs, 46 `system.yaml` directories and 491 models embedded in Rust tests. 697 are byte-identical. The 9 that differ all have overlapping input refusals or a held-state input refusal:
  - the 2 new fixtures
  - `arrangement-input-refusal.yaml` and `typed-text-operands.yaml`
  - the models embedded in `adversary_222_pass1`, `expression_a1`, `expression_a2`, `expression_utf8_bytes` and `mutation_offsets`

  The implementor's "2 fixtures changed" is about edited test assertions. Seven existing models also changed bytes, while their tests still pass. All 7 pass the interpreter at base and head. Those 5 test models were outside the implementor's 148-model list.
- **Weak test (not a separate finding):** `no_overlap_models_bytes_unchanged` (`refusal_pair_overlap.rs:301`) counts the sends of one scenario and never compares bytes. The measurement above is what supports that acceptance.
- **Build trap:** a base build in the shared target dir produced a binary identical to head's (same md5). I rebuilt base in its own target subdirectory before measuring.

**6. Paths written outside the worktree**

- `~/.cache/ess-054-wave/W2-4/adv1/` (38M, kept): probe models, both measurement scripts, the per-model suites under `out/`, and the logs.
- `~/.cache/b10x-target/ess-054-W2-4` and its `base/` subdirectory: removed with `cargo clean`.
- `~/.cache/b10x-go-cache/W2-4-adv1` (167M, created by the test build): deleted.

The base source copy and the two binaries in scratch are also deleted. My worktree session lease is released.

**7. Findings block**

```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 12441
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'On a command with a when_subject branch, refusal_pair_overlaps never reaches the arranged-row refusal path, so the overlap of two plain refusals is never sent and no note is raised; a target taking them in the other order passes, which contradicts fields-and-invariants.md:48 and guards-and-predicates.md:220 (cases adv1_refusal_pair_overlap_is_sent_beside_a_held_state_refusal and adv1_pause_first_target_fails_beside_a_held_state_refusal).'
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: 'On a command with a when_subject branch, a plain input refusal is never sent for an unknown identity requiring the refusal, so a target that looks the record up first passes and step 1 of the new Which-branch-answers section goes unwitnessed (case adv1_plain_refusal_is_sent_for_an_unknown_identity_beside_a_held_state_refusal, also red at the base).'
```
