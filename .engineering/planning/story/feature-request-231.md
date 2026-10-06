---
format: aep.planning-md/3
id: story:feature-request-231
kind: story
status: implemented
title: 'Entity Runtime lowering refuses constructs ess/15-16 validate: unknown_instance, existing_instance, {related:}, {increment}, {cleared}, alphabet:, text .count, now'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#231
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T14:28:42Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T14:28:42Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T09:47:47Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Outcome

Entity Runtime lowering refuses constructs ess/15-16 validate: unknown_instance, existing_instance, {related:}, {increment}, {cleared}, alphabet:, text .count, now (beyond10x/ess#231).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-231 (beyond10x/ess#231)

Read tree: `gaps-270` at `e3bc9a2ff`. Lowering crate: `crates/generate/ess-entity-runtime` (ESS workspace member, `Cargo.toml:19`), pinned to `entity-core` tag 0.24.1 (`Cargo.toml:102`). Entity Runtime 0.24.1..0.25.1 has no `crates/entity-core` diff (`git diff --stat 0.24.1 0.25.1 -- crates/entity-core` is empty), so the issue's ER 0.25.1 is the same target. Neither `ess` 0.44.0 nor 0.48.0 has a command that runs this lowering (`ess generate --help`), so the evidence below is source and tests, not a CLI run.

1. **Need.** A downstream specification whose component is lowered to Entity Runtime finds out only at host build time, from the ER host, that a construct `ess specify validate` accepted cannot be lowered. There is no published list of the lowerable subset and no `ess` command that runs the lowering. Requester's two asks, labelled as theirs: (a) "lower each" of 9 constructs; (b) "document which subset of ess/15-16 a lowered component may use". Status of each construct in the read tree (all **still refused**):
   - `unknown_instance:` on an update → `OutcomeShapeUnsupported`, `src/lib.rs:2045-2051`; test `tests/outcome_shapes.rs:94`.
   - `existing_instance:` / create-on-unknown → `ExistenceSelectionUnsupported`, `lib.rs:1358-1372` (early `return`), `lib.rs:2052-2071`; test `tests/upsert_by_existence.rs:131,146`.
   - `{related: {via, field}}` → `ValueExpressionUnsupported`, `lib.rs:3427-3436` (`is_value_expression` includes `RelatedField`), `lib.rs:2588-2598`; test `tests/related_values.rs:94`.
   - `{increment: 1}` → `ValueExpressionUnsupported`, same match arm (`Increment`).
   - `{cleared: true}` on update → `ClearedValueUnsupported`, `lib.rs:2303-2309`; test `tests/lowering.rs:1235-1249`.
   - `alphabet:` → `AlphabetUnsupported`, `lib.rs:1097-1106`; test `tests/lowering.rs:1812`.
   - `.count` on text → `TextLengthUnsupported`, `lib.rs:600-611`; tests `tests/lowering.rs:1817`, `tests/adversary_guards_lowering.rs:124`.
   - `now` in a guard → `CurrentTimeUnsupported`, `lib.rs:590-598`; test `tests/current_time_guard.rs:126`.
   - Optional input into an Optional field on update → `OptionalBoundOutputUnsupported`, `lib.rs:2294-2300`; test `tests/lowering.rs:461`.
   - Since the issue was filed, #229 (`c1f4353df`, merged in this tree) **added** another refusal, `RelatedGuardUnsupported` (`lib.rs:452-455`). The refused set is still growing.
   - Minimal reproduction: the existing tests above each build a minimal case from ESS's own examples. No new spec was written because 0.44.0 has no command that lowers one.
2. **Class.** Split.
   - (b) is a **defect**. The design doc's "Finite source forms refused by this target" table (`docs/design/ess-evolution/entity-runtime-lowering.md:565-583`) lists 15 codes. The enum (`lib.rs:392-456`) carries 11 more source-form refusals the table omits: Alphabet, TextLength, ValueExpression, CaseFold, OutcomeShape, InputAbsent, CurrentTime, ExistenceSelection, Caller, SetEffect, RelatedGuard. The website mentions only 5 of them, scattered across pages (`commands-and-outcomes.md:157,221`, `predicates.md:583,733`, `fields-and-invariants.md:140`).
   - Also, "returned together" (`lib.rs:5`) does not hold everywhere: `lib.rs:1372` and `lib.rs:1380` `return` early per command, so one refusal hides the others in that command.
   - (a) is a **gap** per construct. Each refusal is deliberate and named. The target cannot state the rule.
3. **Already expressible?** No, for every listed construct. A lowered component can only leave the rule out. That is what the issue reports, and the tests above assert it.
4. **Fit / where it belongs.** The lowering, the refusal codes and the docs live in ESS, so (b) and any lowering change are **ESS** work. Several constructs first need a primitive in the **entity-runtime** repository. Per construct (inferred from the code comments, not designed or tested):
   - ESS-only, through a host-supplied argument or binding obligation (the pattern the slots and `BindingRequirement` already use):
     - `now`: ER refuses `$now` for good (`entity-runtime crates/entity-core/src/definition.rs:1138-1139`), so the clock must be an argument.
     - `{related:}`: the host loads the row and passes the value in.
     - `unknown_instance` / `existing_instance`: the host's own not-found or exists answer, mapped to the declared refusal.
   - Need an entity-core primitive first. None exists in the `Condition` enum (`definition.rs:1105` ff.):
     - `{increment}`: no arithmetic.
     - `alphabet:`: no character class.
     - text `.count`: `count` works on arrays and maps only (`lib.rs:416-420` doc).
     - `{cleared}`: ER admits `Remove` only as a host action (design table, `ClearedValueUnsupported` row).
     - Optional→Optional on update: `PresentArgument` is top-level create, event and response only (design table, `OptionalBoundOutputUnsupported` row).
   - Every target already refuses each construct by name. Nothing is silently ignored.
5. **Second adopter.** Any ER-hosted service with an optimistic revision counter (`revision: {increment: 1}`), or with fixed-length lowercase-hex tokens (`alphabet:` + `.count`). Both are common and unrelated to the requester's domain.
6. **Cost.**
   - (b): no format bump and no new keyword. It needs a reference page (ideally generated from `LoweringCode`), a fixed design table, and possibly an `ess` command or flag that runs the lowering for one component so every refusal is seen at once. A new CLI surface means new docs and one new diagnostic rendering.
   - (a): each ESS-only lowering adds binding obligations, which changes the host-facing plan for every ER adopter. Each primitive-dependent lowering needs an entity-core release plus an ESS pin bump (currently 0.24.1).
7. **Alternatives.**
   - (i) Change nothing. Refusals are already by name, but adopters keep finding them late.
   - (ii) Requester (a) as one story: 9 lowerings across two repos in one unit. Too wide, and 5 of them are blocked on entity-core.
   - (iii) Chosen: deliver (b) now in ESS (complete table and reference, no early-return masking, a way to run the lowering from `ess`). File per-construct lowering as separate stories, ESS-only first. The 5 primitive-dependent ones get a decision-blocker naming the entity-core capability, raised with the entity-runtime repository.

## Decisions

- **accept, redesigned (proposed):** Scope the story to the ESS-side defect:
  - publish the complete lowerable-subset table, generated from `LoweringCode` (the design table at `docs/design/ess-evolution/entity-runtime-lowering.md:565-583` omits 11 of the codes);
  - stop the per-command early returns (`lib.rs:1372,1380`) from hiding other refusals;
  - give `ess` a way to run the lowering for one component.

  Split the 9 lowerings into follow-up stories:
  - ESS-only via host arguments or binding obligations: `now`, `{related:}`, `unknown_instance`/`existing_instance`;
  - a `decision-blocker` on entity-core primitives, owned with the entity-runtime repository: `{increment}`, `alphabet:`, text `.count`, `{cleared}`, Optional→Optional on update.

  None of the 9 is fixed in the read tree. Overlaps:
  - #229 (merged here) added `RelatedGuardUnsupported` to the refused set;
  - #233 (value expressions, byte length) and #244 (time guards) will add further constructs this list must cover;
  - #285 (`{related:}` through an Optional reference);
  - closed #232 (ER guard-refusal mapping) is related, not a duplicate.

  No duplicate.

- Coordinator (2026-10-01): adopted as proposed above. The 5 constructs that need entity-core are blocked by decision-blocker:entity-core-lowering-features.
