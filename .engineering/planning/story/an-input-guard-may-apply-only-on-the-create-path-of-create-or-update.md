---
format: aep.planning-md/3
id: story:an-input-guard-may-apply-only-on-the-create-path-of-create-or-update
kind: story
status: draft
title: An input guard may apply only on the create path of create-or-update
summary: 'when: beside unknown_instance: is refused, and an input refusal answers before existence on both paths, so a check that only a new record must pass cannot be stated (adopter item 13, 2026-10-22, related to beyond10x/ess#554); same on 0.67.0.'
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
---
## Need

Reported by an adopter on 2026-10-22 (item 13 of the recheck batch, the second team's limit file L4, relayed by the
orchestrator; adopter names kept out). A create-or-update command (`updates:` beside a creating `unknown_instance:` branch)
requires an input only when it creates: an update may leave `label` empty, a creation may not. The guard belongs to the
creation path alone, answered after existence. ESS refuses any condition beside `unknown_instance:`, and an input-guarded
refusal (`when:` with `error:`) answers before existence on both paths
(`docs/design/cross-record-and-stored-field-guards.md` § "The precedence order", as `story:the-precedence-rule-places-every-branch`
records it: "Request-only refusals stay before existence").

Needed: an input guard that applies only when no record carries the identity, either `when:` beside `unknown_instance:` (on
a refusal or on the creating branch), or a recorded idiom. This touches the precedence rule's answer 1, so it needs a fit
review against beyond10x/ess#554 before a design.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Item` with `label: String`, `PutItem(item_id, label)`
with `updated` (`updates:`), `create-needs-label` (`unknown_instance: true`, `when: label == ""`,
`error: demo.server.LabelRequired`) and `created` (`unknown_instance: true`, `creates:`), and an immediate view.
`ess specify validate --path spec.yaml`, 0.68.0 (integration build, also at `format: ess/30`) and 0.67.0 identical:

```text
error[ESS-COMMAND-004]: outcome `create-needs-label` declares `unknown_instance` beside another condition; the branch an
identity no record carries takes is decided by nothing else
help: keep `unknown_instance: true` alone, naming its `error:` or `refuses: false`
```

Variant: `when: label != ""` on the creating branch itself is refused the same way. Control, both versions, byte-identical
suites: the refusal without `unknown_instance:` (an input refusal on both paths) validates and passes 3/3, which is the
behaviour the adopter does not want for updates.

## Where it lives

- `crates/specify/ess-domain/src/command.rs:7280-7300`: `unknown_instance` beside `when`, a subject fact, a subject state,
  `when_state_changes`, `external` or `wrong_state` is a `conflict`.
- Related, not the same: `story:unknown-identity-answered-by-an-existing-refusal` meets the same refusal for a different need
  (an unknown identity answered by an existing refusal). `story:every-guard-combination-validates-under-the-rule` admits
  `when_related:` beside `unknown_instance:`, not `when:`.

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0 and 0.68.0. Gap. Does not block 0.68.0.

## Fit review

Assessed 2026-10-22 against the integration tree at 0.68.0 (`ess/30` the newest released format), with
`.agents/skills/assessing-external-requests/SKILL.md`.

**1. The need, apart from the proposed syntax.** A create-or-update command must refuse some inputs
only when it would create the record. An update may leave `label` empty, but a creation may not. The
domain fact is a condition on the request that holds only for an identity no record carries. Today
ESS can state an input refusal on both paths or an unconditional answer for an unknown identity. It
cannot state a refusal that depends on both. Minimal reproduction: the brand-free `demo.server`
fixture in `## Reproduction` above, `ess specify validate` on 0.67.0 and on 0.68.0 →
`ESS-COMMAND-004`, "declares `unknown_instance` beside another condition".
*Requester's proposal (theirs):* `when:` beside `unknown_instance:`, on a refusal or on the creating
branch, or a recorded idiom.

**2. Class: gap.** The refusal rests on a premise in the code comment that does not hold for input
guards: "an identity no record carries has no state, no stored field and no input guard that could
narrow it further" (`crates/specify/ess-domain/src/command.rs:7280-7282`). An identity no record
carries has no state and no stored field, but the request is still there. It is not a defect,
because ESS documents the refusal on purpose (`command.rs:7283-7300`;
`crates/specify/ess-domain/src/command/outcome_shapes.rs:483-509`, at most one `unknown_instance:`
branch per command).

**3. Can it already be expressed? No.**
- An input refusal without `unknown_instance:` answers at step 1, before existence, on both paths
  (`docs/design/cross-record-and-stored-field-guards.md` § "From `ess/27`: the one rule", step 1). In
  the fixture's control variant it validates and passes 3/3 on 0.67.0 and 0.68.0. The synthesized
  `updated` witness uses a non-empty label, so an implementation that refuses an empty label on
  updates also passes that suite.
- `when:` on the creating branch is refused the same way (variant in `## Reproduction`).
- Splitting the operation into a create command and an update command changes the protocol's
  surface (two operations where the domain has one put). That moves the figure out of the
  specification, so policy 1 rules it out as an answer
  (`docs/design/generic-language-features.md` § Policy, policy 1).

**4. Fit with what is there.**
- *The rule decides the placement.* G4's rule answers a branch "in the order of what it must read"
  (`generic-language-features.md` § G4 "The rule"). `unknown_instance:` is sugar for
  `not exists(subject)` at step 2 (§ G4, "The markers become sugar for G1 predicates"). A branch that
  reads the request and existence therefore answers at step 2, the later of its two reads.
- *No conflict with `story:the-precedence-rule-places-every-branch`.* That story's line "Request-only
  refusals stay before existence" is answer 1 of the record (`generic-language-features.md` § "Open
  questions, answered", 1). A branch guarded by `unknown_instance:` and `when:` is not request-only:
  it also reads existence. Answer 1 is untouched, and every input refusal without a marker still
  answers at step 1. beyond10x/ess#554 asked for "first the addressed record's existence, then its
  held state …, then related rows". A step-2 branch with an input conjunct keeps that order.
- *Grid triage.* The cell is grid R row `when:` (input, caller, `now`;
  `docs/design/language-grids.md` grid R) read on a step-2 branch. The decision opens the whole row
  for both existence markers. `when:` is admitted beside `unknown_instance:` and beside
  `existing_instance:` (`outcome_shapes.rs:264-287`, `existing_alone`, refused today for the same
  reason). Everything `when:` reads becomes readable on the create path: input, caller and `now`. The
  rest of the step-2 column stays refused: subject facts, held state, `when_state_changes:` and
  `wrong_state:`, because there is no record to read. `external:` beside a marker is a separate
  cell, at step 5, and is not opened here.
- *Composition.* The marker and `when:` read as `not exists(subject) and <when>` at step 2. Within
  step 2, declaration order applies, as in every step. Several `unknown_instance:` branches are
  admitted. The guarded ones come first, and exactly one unguarded one is declared last, which keeps
  the unknown path total. `when_related:` beside `unknown_instance:` is already admitted from
  `ess/27` and keeps its step-4 placement.
- *Disclosure.* No new existence disclosure. On a create-or-update, `created` and `updated` already
  tell the caller whether the record existed. On a command that refuses an unknown identity, the
  `unknown_instance:` answer already discloses it. `before_row: true` stays the way to refuse without
  disclosing (`cross-record-and-stored-field-guards.md:883-889`, as cited in § G4).
- *Every target handles it or refuses it by name.* The interpreter, synthesis and generated Rust and
  Go read the plan's step 2. Entity Runtime lowers it or refuses it by name (record answer 10).
  `ess verify diff` reports an added or removed conjunct as the existing `outcome-condition-changed`
  kind (§ "Migration and format consequences").

**5. A second, unrelated adopter.** An inventory service with
`PutShelf(shelf_id, capacity)`: any granted caller may update a shelf, but only an administrator may
create one (`when:` over the caller on a guarded `unknown_instance:` refusal). Another example: a create-or-replace command that refuses a duplicate
identity unless `replace == true` (`existing_instance:` beside `when: replace == false`).

**6. Cost.**
- Format: the next `ess/N`, `ess/31` (planned for 0.69.0). Below `ess/31` the refusal and its bytes
  are unchanged, and its hint names `ess/31`.
- No `RawSpecFile` key and no schema regeneration: `when:`, `unknown_instance:` and
  `existing_instance:` all exist.
- IR: `OutcomeCondition::UnknownInstance` and `ExistingInstance` carry an optional input predicate,
  omitted when absent, so every IR below `ess/31` keeps its bytes and digest.
- Diagnostics: `ESS-COMMAND-004`'s text for the marker changes under `ess/31` and keeps refusing
  subject, state and `external:` conditions. Two existing codes get new uses. `outcome_shapes.rs:495`'s
  "more than one `unknown_instance:`" becomes "the unguarded one is not last, or there are two". An
  unknown path with no unguarded branch becomes `non_exhaustive_branches`. Each needs
  `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`.
- Synthesis: new witnesses on both sides of the guard (below).
- No suite-format change.

**7. Alternatives.**
- (a) *Change nothing* and use the control's step-1 refusal. Rejected: it refuses updates the domain
  allows, and the suite cannot tell the two behaviours apart.
- (b) *Move input refusals after existence.* Rejected: it reverses answer 1, changes the answer of
  every validating command, and reopens the disclosure `before_row` closed.
- (c) *`when:` on the creating branch only.* That is the requester's second shape. It is subsumed:
  the same conjunction on any step-2 branch also covers the refusal shape. Rejected as the only form,
  because a refusal must name its own error.
- (d) *Chosen:* `when:` beside either existence marker, placed at step 2. It adds no keyword, reuses
  the rule's derivation, and opens the row rather than one cell.

## Decisions

**Accept, redesigned** (verdict: redesigned).

- From `ess/31`, `when:` is admitted beside `unknown_instance:` and beside `existing_instance:`. The
  branch is `not exists(subject) and <when>` (respectively `exists(subject) and <when>`) and
  answers at step 2, in declaration order. Request-only refusals stay at step 1, so answer 1 of the
  G4 record holds.
- A command may declare several `unknown_instance:` branches. Every branch but the last carries
  `when:`, and the last carries none. A guarded branch after the unguarded one is
  `unreachable_branch`. An unknown path with no unguarded branch is `non_exhaustive_branches`. The
  same rule applies to `existing_instance:`.
- Subject facts, held state, `when_state_changes:`, `wrong_state:` and `external:` beside a marker
  stay refused in every format.
- What changed from the request:
  - extended to `existing_instance:` (same row of grid R);
  - "at most one `unknown_instance:`" relaxed to "one unguarded, last";
  - a cross-path witness added, so the suite catches a guard applied on both paths.
- Counted under G4 (`generic-language-features.md` § Policy, policy 2, and answer 2: the rest of a
  decision moves to the next format). Changelog count: general (it opens the `when:` row on step-2
  branches).

Acceptance, each test written to fail first, as a module of an existing test binary:

- `crates/specify/ess-domain/tests/precedence_classification.rs`:
  `ess_31_places_an_input_guarded_unknown_instance_at_existence`,
  `ess_31_keeps_input_refusals_without_a_marker_before_existence` (answer 1).
- `crates/specify/ess-domain` validation tests:
  - `ess_31_admits_when_beside_unknown_instance_and_existing_instance`;
  - `ess_31_refuses_a_guarded_unknown_instance_after_the_unguarded_one`;
  - `ess_31_refuses_an_unknown_path_with_no_unguarded_branch`;
  - `below_ess_31_when_beside_a_marker_keeps_its_refusal_and_names_ess_31`;
  - subject, state and `external:` conditions beside a marker are still refused.
- `crates/verify/ess-conformance/tests/precedence_plan_answers.rs`, on a fixture shaped like the
  reproduction:
  - `PutItem/outcome/create-needs-label`: unknown identity, `label == ""` → refused `LabelRequired`,
    no row;
  - `PutItem/outcome/created`: unknown identity, `label != ""` → created;
  - `PutItem/outcome/updated`, witnessed on an input the step-2 guard holds (existing record,
    `label == ""`) → updated.
  - The suite passes on the interpreted target and the generated Rust and Go behaviour.
- `crates/verify/ess-conformance/tests/mutation_arms.rs`: three mutants each turn a scenario red:
  dropping the `when:` conjunct, dropping the marker (the guard applied on both paths), and swapping
  the order with the creating branch.
- `crates/generate/ess-entity-runtime/tests/selection_precedence.rs`: lowers the command in plan
  order or refuses it by name.
- `crates/specify/ess-domain/tests/format_history.rs`: the `ess/31` row names the admission.

The requester is told: accepted, redesigned. Their first shape (`when:` beside `unknown_instance:` on
a refusal) works as written from `ess/31`. Their second shape works when the creating branch is the
last, unguarded `unknown_instance:` branch.

## Format retarget (2026-10-28)

`ess/31` shipped in 0.69.0 and `ess/32` in 0.70.0 without this story (`FORMAT_HISTORY` in
`crates/specify/ess-domain/src/system.rs`). Its planned format gate is now the next one, `ess/33`,
opened once for the open-issue batch that carries the `ess/33` language changes
(`docs/plan/wave-ess-issues-1.md` § Batches). Read `ess/31` above as `ess/33`, and `ess/30` as
"below `ess/33`". Store drift fixed under D136 (`approval-record:d136-synthesis-defects-and-store-drift`).
