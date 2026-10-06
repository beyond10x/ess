---
format: aep.planning-md/3
id: story:feature-request-237
kind: story
status: implemented
title: Distinct list members and a count across records
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#237
relations:
- serves: vision:O2
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:27:22Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-04T15:27:22Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:58Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Distinct list members and a count across records (beyond10x/ess#237).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- Item 2 (a count of an entity's records) is also #229's follow-up (count, every, none over related rows). One story covers both. #229's part 1 shipped (ess/20), and its effect part is `story:related-record-effects`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-237 (distinct list members; count across records)

Tested with `ess 0.44.0`; rules re-read in the 0.48.0 read tree (`gaps-270`, `e3bc9a2ff`). Repros are `repro-237/*.yaml` from `base.yaml`.

1. **Need.**
   - (1) A list holds no two elements equal, as a whole or by one member: files unique by `path`.
   - (2) A command is refused when the number of an entity's rows matching a filter (state, scope) has reached a bound: at most N keys per issuer.
   - Repros (all 0.44.0):
     - `1a`: `tags.distinct` -> `ESS-ENTITY-003`.
     - `1b`: `{distinct: true}` -> "unknown operator".
     - `1c`: `files.path.count_distinct` -> `ESS-ENTITY-003`.
     - `2a`: `count(demo.bundle.Bundle) >= 3` -> parse error.
     - `2b`: reading the aggregate view `OpenPerOwner.open` in `when:` -> `ESS-COMMAND-003`.
   - *Requester's syntax:* `distinct` on `List<T>` with an optional member; a guard operand counting records by state and scope.
2. **Class.**
   - (1) gap.
   - (2) gap. A count is only a view aggregate (`docs/design/aggregate-views.md:108-111`). No guard reads another entity's rows except one row by identity (`cross-record-and-stored-field-guards.md:517-520`: "a lookup by any other field is a query, and stays out of scope").
3. **Already expressible?** No.
   - The pairwise-quantifier attempt `1e-nested-binder.yaml` (`forall a in tags: forall b in tags: a != b`) **validates but means `a != "b"`**. That is a silent defect: `text_literal` checks only declared fields (`crates/specify/ess-domain/src/expression.rs:1003` -> `root` at `:360-365`), not binders (`:480`).
   - Even fixed, a pairwise `forall` is false for `a` = `b`, so it cannot state distinctness.
4. **Fit.**
   - (2) as a free `count(Entity)` operand inside a predicate would be a second spelling for a row set beside `affects:`/`instances: {where}` (`docs/design/set-effects-over-filtered-instances.md:20-35`). It also could not say what answers a missing scope. It should be family F **B**:
     ```yaml
     when_related: {entity: E, where: <predicate over E's fields with input./subject.>, count: {gte: N}}
     ```
     The siblings are `exists: true|false` and `forall: <predicate>`.
     - It reuses `when_related`, `where:` (the set-effect filter grammar) and the map operators.
     - Synthesis reuses set-effect arrangement: the rows the filter selects, plus one per refuted conjunct (`set-effects...md:55-70`).
     - The scope rule follows aggregate views: count only where a `where` conjunct is a scopable `String`/`Uuid` set from input, else a named refusal like `ESS-SYNTH-016` (`aggregate-views.md:444-500`), because a target may be shared.
     - A bound larger than a synthesis cap (10,000 live cursors) is validated but refused by name for witnessing.
   - (1) fits as a quantifier-shaped key `distinct: {in: <list>, as: x, by: x.<member>}` (`by` optional). It reuses `in`/`as` (`predicates.md:450-466`) and the `count_distinct` equality of aggregate views (`aggregate-views.md:174-183`).
5. **Second adopter.**
   - (1) An order's line SKUs are distinct; a playlist holds no duplicate track.
   - (2) At most 5 active sessions per user; at most 3 open drafts per author.
6. **Cost.**
   - B: new `when_related` keys (`entity`, `where`, `count`, `forall`; `exists` reused), the ess/20 bundle gate, `ess-diff` classification (`outcome-guard-changed` family), Entity Runtime `RelatedGuardUnsupported` (existing), interpreter `unsupported`. No suite format: decided by arrangement plus `expect_outcome`.
   - `distinct`: a new `Predicate` variant in the Rust, Go and TypeScript evaluators, and a suite major when it reaches a `satisfies` expectation (precedent E7, `value-expressions.md:199-202`).
   - Each is a new keyword with no migration: absent means unchanged bytes.
7. **Considered.**
   - (2): change nothing (SQL triggers, per the issue); the requester's free count operand; B. B is chosen: one row-set spelling shared with #228 and the #229 follow-up ("every task finished", "no open high-risk ambiguity").
   - (1): change nothing; `.count_distinct` as a path suffix (cannot carry a member key on a list of structs); the `distinct:` quantifier shape. The quantifier shape is chosen.
   - The binder defect should be filed separately: today it is a silent miscompile.

## Decisions

- **accept, redesigned (proposed):**
  - (2) becomes family F part B, a row-set test `when_related: {entity, where, count|exists|forall}`. It reuses the `where:` filter of `affects:`/`instances:`, the map operators and set-effect arrangement. It is witnessed only when the scope is scopable and the bound sits under a synthesis cap, and refused by name otherwise.
  - (1) becomes `distinct: {in, as, by}` (family F part C).
  - The requester's free `count(...)` operand is dropped, because it would be a second spelling for a row set.
  - Overlaps: #228 is the `exists: true` case of the same construct; the #229 follow-up is the `forall`/`count` cases; #283 (several related rows) is adjacent.
  - **New defect to file:** a quantifier binder on the right side reads as text silently (`repro-237/1e-nested-binder.yaml`; `expression.rs:1003` checks fields only).
  - Not already fixed (tree `e3bc9a2ff`).

- Coordinator (2026-10-01): adopted as family F below. Format: ess/21, because ess/20 ships alone in 0.49.0 and family F lands as one bump. Family F (one design across #225, #228, #233, #237, #244): A1 a bare right-hand root names a field, input or binder; A2 one `± constant` offset (Integer or Timestamp); A3 `now` in `when_subject`/`when_related`; A4 `input.<dotted path>` in values; B `when_related: {entity, where, exists | count | forall}` over row sets; C `distinct` over lists and `.utf8_bytes`.

## Current design disposition

Second independent review approved the complete shared row-set/filtered-value contract at 8606b103c (review-result:filtered-related-read-design-20261003-r2). All seven first-round findings were fixed. Design review is complete; implementation and target acceptance remain pending. This supersedes the earlier pending-design-review wording, not the predecessor gates or draft implementation status. The shared normative page is docs/design/filtered-related-reads.md and the syntax allocation is source22. No source21 family allocation remains current. #299 depends on #285 and #228/#237; the shared page also binds the latter stories' row-set implementation.
