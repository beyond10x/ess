---
format: aep.planning-md/3
id: story:overlap-witnesses-per-phase
kind: story
status: draft
title: Two overlapping branches of one phase are witnessed where both hold
refs:
- provider: github
  reference: beyond10x/ess#472
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:synthesis-reads-selection-plan
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/adversary-244b-base-digests.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/external-beside-held-guard-base.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs
- confidence: cited
  path: docs/design/input-guard-overlap-precedence.md
revision: 11
---
# Story: Two overlapping branches of one phase are witnessed where both hold

## Why

beyond10x/ess#472: `ess verify conform mutate` reports a `precedence-swap` survivor for any two
refusals whose guards can both hold, because no synthesized scenario sends an input both admit. The
order of overlapping branches is declared and never tested. With the plan, "two branches in one
phase whose guards can both hold" is a query, not a search per construct.

## What it delivers

For each pair of input refusals, held-state (`when_subject`) branches or stored-row branches that
the plan places in one phase, ordered by declaration, whose guards one request can satisfy
together, synthesis writes one overlap send that expects the first declared, appended to the first
branch's own scenario (as #178, #217 and #455 do). `mutate.rs` `precedence_sites` builds its
precedence-swap mutants from the same pairs.

Not in this story: related-row and row-set pairs (#283 already orders present-related refusals
across rows; a later story once the plan carries them), and a `Note` for an unsent pair (Scope
gap 3, design decision 4).

Before any code: run the #472 reproduction on the base. #455 (0.54.0) may already send that
overlap on the stateless path (Scope); if the mutant is killed there, the #472 case becomes a
regression test.

## Acceptance

- The #472 reproduction (`bad-quantity: quantity <= 0`, `bad-discount: discount < 0`) is run on the
  story's base first and its result recorded in the PR. It becomes a regression test in
  `tests/refusal_pair_overlap.rs`: the suite sends an input both guards admit (the first ladder
  candidate, `first_candidate`) expecting `bad-quantity`, and `mutate` reports the precedence-swap
  mutant killed. If the base already passes (#455), the PR says so and #472 is closed with that
  evidence.
- A held-state command with two `when_subject` refusals whose guards one request satisfies (a
  fixture added with the story) gets an overlap send expecting the first declared; on the base it
  gets none, so a target answering the second passes the base suite and fails the story's.
- The same for a stored-row command (`subject_fact` path) with two overlapping refusals.
- `mutate.rs` `precedence_sites` builds precedence-swap mutants from the plan's same-phase pairs,
  so the two fixtures above produce a precedence-swap mutant each, killed by their overlap sends;
  the pins `("precedence-swap", 0)` (`mutation_audit.rs:336,525`) and the 244b `mutants=` column
  move only where a listed model gained a pair.
- Repository digest tables change only for models with a same-phase overlapping pair, each listed
  in the PR.
- Closes beyond10x/ess#472.

## Scope

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong).

- **#472's reproduction is probably already fixed** (inferred, walked not run): 0.54.0 shipped beyond10x/ess#455 (`32afe8ef4`), which sends the overlap of two input refusals on the stateless path; #472 was filed against 0.53.0 at 2026-10-06 15:14Z, about two hours before 0.54.0 was tagged (17:21Z). For the #472 shape, `boundaries` → `boundary_inputs` → `overlap_inputs` → `refusal_pair_overlaps`, and `keep`'s `selects_branch` refutes only refusals declared earlier (`synthesize.rs:6593-6609`), so `{quantity ≤ 0, discount < 0}` is sent requiring `bad-quantity`.
- **Primary surface:** `crates/verify/ess-conformance/src/synthesize.rs`, the overlap section — cited
- **Files:** `synthesize.rs:13168-13355` (`Overlap`, `overlaps` for refusal×accepting #178 and accepting×accepting #217, `overlap_inputs`, `refusal_pair_inputs`, `refusal_pair_overlaps`); `:13293-13297` `overlap_inputs` chains `refusal_pair_overlaps` (#455), reached from `boundary_inputs` at 13093 — cited
- **Why other paths get none:** refusal pairs are "kept apart from `overlaps`" and sent "only from the stateless boundary rows" (`synthesize.rs:13332-13334`) — cited
- **Gap 1:** a held-state command gets boundary rows only if `overlaps` is non-empty, and `overlap_inputs_in_state` reads `overlaps` only (`synthesize.rs:13670-13676`, `13363-13395`) — cited
- **Gap 2:** stored-row commands skip `boundaries` (`synthesize.rs:13659`); `subject_fact::overlaps` (`subject_fact.rs:6899-6940`) pairs a refusal only with accepting siblings; `refusal_pair_inputs` is called only from `unknown_identity_refusal` (`subject_fact.rs:5468`) — cited
- **Gap 3:** `unwitnessed_overlaps` (`synthesize.rs:13547-13595`) reads `overlaps` only (13555), so an unsent refusal pair raises no `Note` — cited
- **Also likely:** `synthesize/related_guard.rs:2158` (`overlaps`, related refusals only) — inferred
- **Mutation:** `src/mutate.rs:1483-1497` (`precedence_sites`, adjacent input-only pairs) and `:872-890` (`guarded_by_input_alone`) build the precedence-swap mutants; this story owns reading the plan there (epic, wave 5) — cited
- **Tests:** `tests/refusal_pair_overlap.rs` (#455), `tests/accepting_guard_overlap.rs` (#217), `tests/mutation_arms.rs:249-283` — cited; a #472 regression test beside them — inferred
- **Digest tables:** `tests/fixtures/external-beside-held-guard-base.tsv` (193 models); `tests/fixtures/adversary-285-base-digests.tsv` (88); `tests/fixtures/adversary-244b-base-digests.tsv` (slow probe, `mutants=` column, `adversary_244b_window_free_bytes.rs:177-193`); `suites/generated/` (3 suites) — cited
- **Models gaining scenarios:** 0 (inferred, yq scan of 190 single-file models): 6 commands have ≥2 input-guarded refusals, all in `tests/fixtures/`, none with held-state or related guards, all on the path #455 covers. If every phase is in scope: 2 commands with ≥2 `when_subject` refusals, 2 with ≥2 `when_subject` accepting branches, 12 with ≥2 `when_related` refusals. Raw-string models in tests were not scanned.
- **Documents:** `docs/design/input-guard-overlap-precedence.md:47` and "What synthesis does" (64-90): #455's overlap send is not recorded — cited
- **Confidence:** medium. Sites read; whether the #472 mutant is already killed was walked, not run.
- **Would collide with:** `story:synthesis-reads-selection-plan` (same functions; ordered by `depends_on`), any unit editing `synthesize.rs`'s overlap/boundary section or `subject_fact.rs`, any unit re-pinning the three digest tables — inferred

### Design decisions for the implementor (inferred)

1. Run the #472 reproduction on the base first. If the mutant is killed, acceptance 1 becomes a regression test and the remaining work is gaps 1-3.
2. Replace `overlaps` and `refusal_pair_overlaps` with one enumeration from the plan: pairs within one phase, in declaration order; emit today's pairs first, in today's order.
3. Append each overlap send to the first branch's `…/outcome/<first>` scenario, as #178, #217 and #455 do; no new scenario id.
4. Bring refusal pairs into `unwitnessed_overlaps`; notes are not in `model_line` digests (`external_beside_held_guard.rs:943-987`).
5. `precedence_sites` reads the plan's same-phase pairs; the pins `("precedence-swap", 0)` at `mutation_audit.rs:336,525` and the 244b `mutants=` column move.
6. Assert the first ladder candidate (`first_candidate`, `synthesize.rs:6663`), not a hard-coded `quantity: 0`.

## Evidence (adopter report, ess 0.56.0)

An adopter: no synthesized scenario sends a request where two input-guarded refusals with
different errors both hold, so their declared precedence is never pinned and precedence-swap
mutants survive (not https://github.com/beyond10x/ess/issues/517, which covers swaps with the same
error and payload). On `main`, 32afe8ef40 (in 0.54.0) sends that overlap only for commands whose
branches read the input alone (`refusal_pair_overlaps`,
`crates/verify/ess-conformance/src/synthesize.rs:13736-13758`). On the held-state path
`boundaries` returns early unless a refusal overlaps an accepting branch (`:14074-14079`;
`overlaps`, `:13606`, pairs refusals with accepting branches only); on the stored-row path
`subject_fact::routes` returns early (`:14062`) and `refusal_pair_inputs` (`:13712`) serves only
the unknown-identity refusal. `unwitnessed_overlaps` (`:13948`) reads `overlaps` only, so an
unsent refusal pair raises no note.

## Acceptance (added)

- A neutral fixture with two input-guarded refusals (different errors, guards that can hold
  together) on a command that also has a held-state branch: synthesis writes one scenario at
  their overlap expecting the declared-first refusal and its error; the precedence-swap mutant is
  killed.
- A refusal pair synthesis does not send is named in a note, as an unwitnessed refusal/accepting
  overlap already is.
