---
format: aep.planning-md/3
id: story:a-caller-read-in-a-row-reading-guard-is-witnessed-by-two-callers
kind: story
status: draft
title: A caller read in a row-reading guard is witnessed by two callers
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#636
- provider: github
  reference: beyond10x/ess#638
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:caller-attributes-are-readable-in-a-when-related-predicate
revision: 3
---
## Need

Once a related-row predicate, a row-set `where:`/`forall:` or a set effect's `where:` may read
`caller.<attribute>` (`story:caller-attributes-are-readable-in-a-when-related-predicate`), the suite
must witness it from both sides, as it does for `when_subject` (`docs/design/caller-values.md`
§ Synthesis): one caller arranges the row, a second caller whose attribute differs sends the
command. Today synthesis writes the caller's literal into `when:` and `when_subject` guards only and
leaves a related or row-set predicate untouched ("The related row is another entity's, which reads
no caller", `crates/verify/ess-conformance/src/synthesize/caller.rs:1163-1169`), and the interpreter
evaluates a row-set or set-effect selection with no caller
(`crates/verify/ess-conformance/src/interpret/execute/row_set.rs:199`).

Minimal model: the `example.g.Bind` command of the parent story (`missing` / `elsewhere` / `bound`),
`example.g.Open` creating the `Parent` with `account_id: {caller: account_id}`.

## Acceptance

- `ess verify conform synthesize` on the minimal model reports 0 refusals and writes
  `example.g.Bind/outcome/elsewhere`, whose steps send `example.g.Open` as the first caller and
  `example.g.Bind` as the second and expect `example.g.ParentNotFound`, and
  `example.g.Bind/outcome/bound`, which sends both as one caller and expects `example.g.Bound`.
- The same holds with `elsewhere` written as a row-set guard over `account_id == caller.account_id`
  and as a set effect `instances: {where: account_id == caller.account_id}`: the scenario arranges
  one row per caller and the effect changes only the sender's row.
- `ess verify conform run --target interpreted` passes every scenario of the three models; a mutant
  interpreter that evaluates the predicate with no caller (or with the first caller fixed) fails the
  `elsewhere` scenario.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/caller.rs
- cited: crates/verify/ess-conformance/src/interpret/execute/row_set.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
- inferred: crates/verify/ess-conformance/src/synthesize/caller/arranged.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/tests/caller_values.rs
confidence: medium

## Fit review

1. **Need.** A caller read in a row-reading guard must be checked by the suite, or a target that
   ignores the caller passes. Reproduction: the parent story's model; today it does not validate
   (triage run, ESS-COMMAND-003), so there is nothing to synthesize yet.
2. **Class.** Gap, following the parent story; without it the parent would admit a construct the
   suite does not check (`.agents/skills/assessing-external-requests/SKILL.md`, red flag "a target
   would silently ignore the new construct").
3. **Already expressible?** No: `write_predicate` is applied to `Related`/`RelatedSet` input guards
   only (`synthesize/caller.rs:1163-1169`).
4. **Fit.** Same grid R column as the parent. Reuses the two-caller assignment and the swapped run
   (`docs/design/caller-values.md` § Synthesis, steps 1-3); no new scenario id family.
5. **Second adopter.** The ticket-queue example of the parent story needs the same witness.
6. **Cost.** No source or suite format (suite/26 `caller` exists); synthesis output for models
   without the construct keeps its bytes.
7. **Alternatives.** (a) Leave synthesis as is and require an authored scenario: rejected, the
   figure would be checked only where an adopter writes it. (b) A new caller-specific scenario
   family: rejected, the existing assignment already produces the two-caller arrangement.
   (c) Rewrite the related and row-set predicates under the assignment, as `when_subject` is: chosen.

## Decisions

accept, redesigned: as the parent story; this story carries its synthesis and interpreter half.

## Source

github:beyond10x/ess#636
