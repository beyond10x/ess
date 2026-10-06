---
format: aep.planning-md/3
id: story:feature-request-212
kind: story
status: implemented
title: ess verify conform mutate does not emit sets-drop, outcome-order-flip or the ==/!= and ±1 guard-boundary arms
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#212
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:48Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:49Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:52Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

ess verify conform mutate does not emit sets-drop, outcome-order-flip or the ==/!= and ±1 guard-boundary arms (beyond10x/ess#212).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: story:feature-request-212 (beyond10x/ess#212)

Read tree `gaps-270` HEAD `e3bc9a2ff` (contains 0.48.0). Runs: `ess` 0.44.0; 0.45.0–0.48.0 may differ, but HEAD source still has the nine classes.

1. **Need.** A mutation audit on ESS's generator cannot measure three faults: (a) an updating branch's field write that no scenario reads back; (b) branch precedence between overlapping guards that no scenario checks; (c) a guard boundary or equality checked from one side only. Requester's syntax: classes `sets-drop`, `outcome-order-flip`, plus `==`↔`!=` and literal ±1 arms (issue body). Repro: `~/.cache/ess-gaps/fit2/repro-212/{base,drop}`, a note updated by `Edit` (`sets: title: input.title`); `drop` removes that entry.
2. **Class: gap**, with one deliberate exclusion that needs re-arguing. ESS leaves out "drop" on purpose as a weakening change that can never be killed (`docs/design/mutation-audit-and-model-runner.md:42-47`, `:174-179`; `website/docs/guides/verify/mutation-audit.md:24-27`). The repro shows that is not true for an updating branch. With the entry dropped, the mutant's suite still asserts the row (`expect_view contains title: "title"`). It only survives because the witness then sends `Edit` `title: "title"`, which equals the stored value, where the baseline sends `"title-1"` (`diff <(jq -S . base.json) <(jq -S . drop.json)`, 4 value lines, ess 0.44.0). So dropping a write is an altering change: the field is kept rather than set to the input. That makes it killable, and it is not expressible with `sets-retarget`, which needs a same-typed sibling input (`mutate.rs:678-684`).
3. **Already expressible?** No. `--class` has the nine values (`ess verify conform mutate --help`, 0.44.0; `MutantClass::ALL`, `crates/verify/ess-conformance/src/mutate.rs:95-106`). `guard-boundary` only swaps strictness on `< <= > >=` (`mutate.rs:525-533`). `guard-negate` negates the whole `when`, not one equality leaf.
4. **Fit.**
   - Vocabulary: `outcome-order-flip` clashes with the existing `order-flip`, which flips a view ranking key (`mutate.rs:89-90`). That is a red flag, so the class gets another name (`precedence-swap`). The `==`/`!=` and ±1 arms are new sites of the existing `guard-boundary` concept, not a new concept.
   - Killers exist: precedence has been witnessed at overlap points since 0.42.0 (`docs/design/input-guard-overlap-precedence.md:107-131`, `b2d39b5fb`, #217); a boundary is witnessed from the accepting side (#160, closed); unsatisfiable guard mutants are scored equivalent (#218, `ESS-MUTATE-005`).
   - `sets-drop` is killable only if synthesis sends an input apart from the stored value when nothing writes it. That extends the #161/#202 separation rule ("a written value is arranged over a different prior value", #161 closing comment). Without it every `sets-drop` mutant survives, which is the noise the design rejects (`design:709`).
   - Siblings: only `when` guards are mutated (`design:180-182`). The new arms inherit that limit. Two `when_subject` branches that overlap have no stated order (`docs/design/cross-record-and-stored-field-guards.md:614`), so `precedence-swap` there must be left out by name.
   - Targets: the classes are generator-only. Every target sees ordinary suites.
5. **Second adopter.** An order service whose `Amend` updates `quantity` and `note`: a dropped `note` write has to be caught. A pricing command with `discount` (`total >= 100`) before `standard`, where a swap must be caught. Neither is local policy.
6. **Cost.**
   - No authored format change.
   - New `--class` values: additive, but `MutantClass` is a public Rust enum with no `#[non_exhaustive]` (`mutate.rs:68-72`).
   - The pinning test is rewritten (`tests/mutation_audit.rs:688`).
   - The report and manifest stay `/3`, since class names are strings.
   - The design and the guide are corrected on "weakening".
   - The synthesis witness changes for `sets-drop`, so baseline suite bytes move for updating branches.
   - The mutant count rises: the requester's audit grows from 510 to about 1,290 sites (issue).
7. **Alternatives.**
   - (a) Change nothing and document why: refused, because the documented reason is wrong for updating branches (repro).
   - (b) Take the request as written: refused for the name clash and because a `sets-drop` on a creating branch really is weakening.
   - (c) Chosen:
     - `sets-drop` on non-creating branches only, with the witness separation;
     - `precedence-swap` on adjacent overlapping accepting `when:` branches and input-guarded refusals;
     - `guard-boundary` extended to `==`↔`!=` on a leaf that is not the whole guard (otherwise it duplicates `guard-negate`) and to an integral literal moved outward by 1 (the direction the strictness swap does not already give).

## Decisions

- **accept, redesigned (proposed):**
  - Split into three units:
    - `sets-drop`, only on non-creating branches. It needs a synthesis change that sends an updating input apart from the stored value (extends #161/#202). Without that change every `sets-drop` mutant survives, as in the 0.44.0 repro.
    - `precedence-swap`, renamed because `order-flip` already means a view ranking key. It needs no new witness: overlap points have been witnessed since 0.42.0 (#217).
    - New `guard-boundary` sites: `==`↔`!=` on a sub-leaf, and an integral literal moved outward by 1.
  - The design's "a dropped `sets` entry is weakening" statement is corrected.
  - Overlaps: #160, #161, #202, #217 and #218 are all closed, and each supplies part of a killer. Not a duplicate. Nothing in the request is already fixed: HEAD `e3bc9a2ff` still has nine classes.

- Coordinator (2026-10-01): adopted as proposed above.
