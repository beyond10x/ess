---
format: aep.planning-md/3
id: story:feature-request-251
kind: story
status: draft
title: A when_subject branch at an invariant upper bound is synthesized again
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#251
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
---
## Outcome

A `when_subject` branch whose predicate is an entity invariant's upper bound, followed by a second `when_subject` branch, is synthesized again, as it was through 0.42.0.

## Acceptance

- The #251 reproduction synthesizes 7 scenarios and 0 refusals.
- `Revise/outcome/exhausted` is witnessed at the bound.

## Origin

beyond10x/ess#251, a regression since 0.43.0 reported downstream. Candidate cause, unverified: `72e532a0aa` (stored counter at its limit, #226).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: story:feature-request-251 (beyond10x/ess#251)

Read tree `gaps-270` HEAD `e3bc9a2ff` (contains 0.48.0). Runs: `ess` 0.44.0. Note: the read tree's store holds no `story:feature-request-251` (`aep plan artifact show` → "holds no `story:feature-request-251`"), so the story still has to be created.

1. **Need.** A stored-field refusal at an invariant's upper bound (`exhausted: when_subject revision >= MAX`, with the invariant `revision lte MAX`), followed by a second `when_subject` refusal that compares the row with the input (`stale: revision >= input.revision`), has no scenario. It had one through 0.42.0 (issue table). Requester's syntax: none; this is a regression report. Repro (the issue's own minimal specification, brand-free): `~/.cache/ess-gaps/fit2/repro-251/spec` → `ESS-SYNTH-003 … no candidate of the 2 tried … over the rows 2 bounded arrangements left`, 6 scenarios and 1 refusal (ess 0.44.0, reproduced).
2. **Class: defect** (regression).
   - The branch is satisfiable: a row at MAX meets both the invariant and the guard.
   - The precedence order puts held-state branches before accepting ones, and the declared order decides among overlapping branches (`docs/design/cross-record-and-stored-field-guards.md:605-617`). `exhausted` is declared first, so it answers at MAX whatever `stale` says.
   - 0.40.0–0.42.0 witnessed it.
3. **Already expressible?** Not applicable to a defect. Author-side workarounds found by probing (ess 0.44.0, the repro with MAX replaced by 100):
   - raising the invariant bound one above the guard literal synthesizes 7 scenarios: `lte: 101` with `>= 100` arranges `Configure revision 100` and sends `Revise revision 101`;
   - writing `stale` as `revision <= input.revision` synthesizes `exhausted` (sent `Revise revision 1`).
   - Neither is acceptable as an answer, because each changes the domain.
4. **Is it still refused on the integration tree? Inferred yes, not executed** (no build permitted).
   - No commit after 0.45.0 that touches `witness.rs` or `synthesize/subject_fact.rs` names #251 or invariant bounds: `git log 0.45.0..HEAD` gives `e3bc9a2ff`, `c1f4353df`, `8aaa71961`, `ccd32f4ee` and `f232e5300`, all `when_related`, caller or error-source work.
   - No test or fixture mentions `ess#251` (`git grep`).
   - **Cause: the candidate `72e532a0aa` is refuted by ancestry.** It is already in 0.42.0 (`git merge-base --is-ancestor 72e532a0aa 0.42.0` → true), the last release that synthesized the outcome. It also follows only `{increment}` counters, and the repro has none.
   - **Hypothesis, labelled:**
     - `cb3ae4906` (0.43.0, #234) holds every input to the entity invariants of the branches it can reach (CHANGELOG `:514-518`, "An input that still breaks one is never sent").
     - The stored-row search for `exhausted` appears to require an input that also refutes the later `stale` (`row >= input`), which needs `input.revision > MAX`. That input now breaks `lte: MAX` on the `revised` branch it can reach, so no candidate is left.
     - The probes fit this pattern: the refusal disappears with the bound at literal+1 (input 101 sent), with `stale` flipped (input 1 refutes it), with `stale` removed, or with the bound removed (issue).
   - The flaw is then the refutation demand itself: refuting a later branch is unnecessary when an earlier declared branch answers first. To be confirmed by a bisect over `0.42.0..0.43.0` (only 8 synthesis commits, `git log 0.42.0..0.43.0 -- …/synthesize*`) under `aep:diagnosing`.
5. **Second adopter.** A quota entity with `used lte limit`: a `full` refusal at `used >= limit`, followed by `over: used + input.amount > limit`. The same shape arises for any counter at its cap followed by an input comparison.
6. **Cost.**
   - Synthesis-internal fix and a regression test pinning the issue's two-branch shape.
   - Suite bytes change only for specifications that hit this refusal.
   - No format, keyword or diagnostic.
   - The precedence doc should also state the order among held-state branches explicitly. Step 4 says they "select by it" but does not state their mutual order (`cross-record…:614`).
7. **Alternatives.**
   - (a) Change nothing; adopters raise the bound: refused, because it changes the domain.
   - (b) Undo the #234 invariant holding: refused, because #234 fixed scenarios that failed against correct targets.
   - (c) Chosen: when witnessing a held-state branch, demand refutation only of branches declared **before** it, as `earlier_accepting` already does for accepting `when:` branches (`input-guard-overlap-precedence.md:113-120`).

## Decisions

- **accept as proposed (proposed):**
  - A regression with no new surface: restore synthesis of the `when_subject` branch at an invariant upper bound.
  - Scope the fix to "refute only earlier-declared held-state branches". Bisect `0.42.0..0.43.0` first under `aep:diagnosing`.
  - The candidate `72e532a0aa` is refuted, since it already shipped in 0.42.0. Likeliest cause (hypothesis): `cb3ae4906` (#234, inputs held to entity invariants).
  - Not fixed at HEAD `e3bc9a2ff` (inferred from code and history, not executed).
  - Overlaps: #234 and #226 (closed, the two suspect changes); #278 (open, a different `ESS-SYNTH-003` shape with an input guard beside a stored guard); #282 (open, precedence statement). Not a duplicate.
  - No `story:feature-request-251` exists in the store yet.

- Coordinator (2026-10-01): adopted as proposed above. Priority 1: a regression.
