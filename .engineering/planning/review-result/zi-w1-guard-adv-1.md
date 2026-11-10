---
format: aep.planning-md/3
id: review-result:zi-w1-guard-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
relations:
- reviews: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
revision: 2
---
unit: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes (tree zi-t1-10, HEAD b59e2c0b2a plus my uncommitted cases)
verdict: weak
new failing cases: 3 (system_issued_synthesis.rs::adversary_a_held_state_beside_an_input_guard_states_the_input, ::adversary_a_stored_guard_reading_an_input_states_it_for_the_default, ::adversary_a_related_default_states_the_input_naming_the_row)
mutants survived: 1 (removing the `RelatedSet` arm of `guard` left the unit's suite green, because no case uses a row-set model; the new `adversary_a_row_set_guard_beside_an_input_guard_states_the_input` catches it)

`git diff --stat`: `crates/verify/ess-conformance/tests/system_issued_synthesis.rs | 156 +` and nothing else. Only the test file changed. Cases went from 11 (the implementor's green.txt) to 15; 3 are red.

- **The story's acceptance is met.** The `when_subject:` and `when_related:` cases pass on HEAD, and both byte-pin tests pass (run4-suite.txt). I checked the fix in a scratch copy of HEAD with the new lines of `guard` taken out, so it matches the base. There the `when_related:` case fails at `:998` with `left: None` (run3-stub-all.txt). That model's only input-guarded branches are `Related`, so it fails without that arm.
- **The fix is narrower than the bug it fixes.** All 3 red cases fail at HEAD (run2-cases.txt, run4-suite.txt) and in the base-equivalent copy (run3-stub-all.txt), so they were already there before this change. They are beside the acceptance: backlog items, not blockers.
  1. `when_subject: {predicate: paused == input.hold}`: `claim_of` refuses `held` with the story's own reason ("an input guard beside the stored-field guard", system_issued.rs:279-284). But `guard` reads only the separate `input` part, so the default branch's await states `{}` (`:1218`).
  2. `when_subject_state: Open` with `when: mode == fast` (the story's repro with a lifecycle-state guard instead of a stored-field one): the default's await states no `mode` (`:998`, `left: None`). `guard` has no arm for `SubjectState`, `SubjectField` or `StateChange`.
  3. In the unit's own `when_related:` model, the default's await states `mode: slow` but not `owner_id` (`:1235`). The `missing` branch (`exists: false` via `input.owner_id`) is avoided only when `owner_id` names an existing row. The interpreter only tries the IDs of rows it holds, so it never notices; a real target that makes up a new ID would end up in `missing`. Same gap for a row-set `where:` that reads input.
- **Fix I'd suggest, not applied:** `guarded_input` should collect every input path a branch's condition reads: the `via` input field, inputs read by a row-set `where:` or a `when_subject` predicate, and the predicate on held-state conditions.
- **Tried and could not break:** a row-set guard next to an input guard. On this model, synthesis avoids `held` through the row condition and sends `mode: fast`, and the await states that. The interpreted run passes. That case stays in as the pin for the `RelatedSet` arm.
- **Paths written outside the worktree**, all under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes/adv/`:
  - `cases.rs`, `guard-mutant.rs`
  - `stub/`: a copy of HEAD without `website/`, with the mutant `guard`
  - run logs: `run1-cases.txt`, `run1-cases.kept.txt`, `run2-cases.txt`, `run3-stub-all.txt`, `run4-suite.txt`
  - build.sh launch logs: `nohup1.txt` to `nohup4.txt`
  - Running the mutant binary directly was not valid evidence: another tree had rebuilt slot B. I deleted those 2 outputs.

```findings
- file: crates/verify/ess-conformance/src/synthesize/system_issued.rs
  line: 363
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: a when_subject predicate that reads an input is refused with the story's own reason, yet guard returns only the separate input half, so the default branch's await states nothing for that input
- file: crates/verify/ess-conformance/src/synthesize/system_issued.rs
  line: 363
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: guard ignores the input predicate of SubjectState/SubjectField/StateChange, so a default beside when_subject_state plus when awaits without stating mode, the story's defect on a held-state guard
- file: crates/verify/ess-conformance/src/synthesize/system_issued.rs
  line: 363
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the default beside when_related through input.owner_id does not state owner_id, so a target that picks an unheld identity takes the exists-false branch; the interpreter masks it by trying only held rows
- file: crates/verify/ess-conformance/tests/system_issued_synthesis.rs
  line: 1128
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the unit's suite has no row-set model, so removing the RelatedSet arm of guard survived it; the new adversary case pins it
```
