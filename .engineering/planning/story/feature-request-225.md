---
format: aep.planning-md/3
id: story:feature-request-225
kind: story
status: implemented
title: a guard comparing two identity-typed inputs (a self-edge check)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#225
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:27:20Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T15:27:20Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:54Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

a guard comparing two identity-typed inputs (a self-edge check) (beyond10x/ess#225).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-225 (guard comparing two identity inputs)

Tested with `ess 0.44.0` on PATH; rules re-read in the 0.48.0 read tree (`gaps-270`, `e3bc9a2ff`). A newer release may differ only where noted.

1. **Need.** A command that takes two inputs of one type cannot refuse when they are equal (a dependency edge from a task to itself). Nothing here is about identity types: any two top-level inputs hit the same wall. Repro `repro-225/a-bare.yaml`: `when: task_id == depends_on` -> `ESS-COMMAND-002` "reads `depends_on` as the text literal ... a right-hand side without a dot is a literal" (0.44.0). *Requester's syntax:* admit `==`/`!=` between two same-identity-typed inputs in `when:`.
2. **Class: gap.** The documented idiom moves both inputs into one struct (`website/docs/reference/predicates.md:191-193`), which changes the command's wire contract. That contract is a domain fact, so the idiom does not cover this. It also breaks when the members have to be stored: `repro-225/c-struct.yaml` `sets: {task_id: input.edge.task_id}` -> `ESS-COMMAND-001` (0.44.0; tree `crates/specify/ess-domain/src/command.rs:2529-2545` checks top-level input names only). That is #233 item 1.
3. **Already expressible?** No, short of reshaping the wire contract.
   - `b-input-prefix.yaml`: `when: task_id == input.depends_on` -> `ESS-COMMAND-003` "reads `input`" (0.44.0). `when:` has no `input.` namespace, although `when_subject` and `when_related` have one (`predicates.md:744-752`, `:27`).
   - `d-related-idiom.yaml`: `when_related: {via: input.task_id, predicate: task_id == input.depends_on}` -> `unobservable_fact`, because the identity is not a readable field.
   - `e-subject-idiom.yaml`: `when_subject` on the task, same refusal (0.44.0).
   - The two-fact comparison machinery already works. `c2-struct-unstored.yaml` synthesizes with 0 refusals, sending equal and different ids (`c2-suite.yaml`, scenario `.../outcome/self-edge`).
4. **Fit of the requester's design: fails.**
   - It is one construct (`when:`) and one type class (identity). The same question arises in `when_subject` (#233 comment: `leased >= capacity`), entity invariants (#233 item 3) and quantifier bodies.
   - Restricting it to identity types would be a red flag: one capability for one sibling only.
   - The fit is family F part A1 (see `233-fit.md` Q7). The right-hand side reads a bare word naming a root of the place as that fact, and `when:` admits `input.<path>` as `when_subject` already does.
   - Synthesis already grounds a two-fact comparison (`docs/design/value-expressions.md:163-171`).
   - Entity Runtime lowers `input.<field>` (`value-expressions.md:173`).
5. **Second adopter.** A transfer refuses `from_account == to_account`. A merge refuses `source_branch == target_branch`.
6. **Cost.**
   - A bare RHS root changes meaning, because IR stores predicate text (`repro-237/1e.ir.json` shows `"that": "a != b"`). So it is gated at the next format, bundled with ess/20 (`spec-versions.md:59`, unreleased).
   - No suite format: input guards are decided at synthesis (`value-expressions.md:172`).
   - No new keyword or diagnostic. The `text_literal` refusal (`expression.rs:1003-1016`) retires under the new format.
7. **Considered.**
   - (a) Change nothing and point to the struct idiom. Rejected: it reshapes the wire and cannot be stored (Q2).
   - (b) The requester's identity-only `==`/`!=`. Rejected (Q4).
   - (c) Family F A1. Chosen: no keyword, and every comparison place gets it at once.
   - The requester's proposal is subsumed, not refused.

## Decisions

- **accept, redesigned (proposed):**
  - Merge into one "comparison operands" story with #233 items 1–3 and 5, and #244 item 1. This one is family F part A1: a bare right-hand-side word naming a root of the place is that fact, and `input.<path>` is admitted in `when:`. It goes in under the ess/20 bundle.
  - The requester's identity-only `==`/`!=` is dropped, because #233's sibling-field and invariant cases need the same rule.
  - Overlap: #233 (superset), #200 (`param.` operand), and the binder defect found in `repro-237/1e-nested-binder.yaml`.
  - Not already fixed (tree `e3bc9a2ff`).

- Coordinator (2026-10-01): adopted as family F below. Format: ess/21, because ess/20 ships alone in 0.49.0 and family F lands as one bump. Family F (one design across #225, #228, #233, #237, #244): A1 a bare right-hand root names a field, input or binder; A2 one `± constant` offset (Integer or Timestamp); A3 `now` in `when_subject`/`when_related`; A4 `input.<dotted path>` in values; B `when_related: {entity, where, exists | count | forall}` over row sets; C `distinct` over lists and `.utf8_bytes`.
