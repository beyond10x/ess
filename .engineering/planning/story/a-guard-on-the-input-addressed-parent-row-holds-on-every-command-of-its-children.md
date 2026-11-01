---
format: aep.planning-md/3
id: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
kind: story
status: implemented
title: A guard on the input-addressed parent row holds on the creator, reader and deleter of its child rows
summary: Creator predicate guard starves the reader's found (ESS-SYNTH-001); when_subject beside a row set is ESS-COMMAND-009; the deleter's synthesized scenario fails past when_subject.
tags:
- downstream
- ess-issues-3
relations:
- informed_by: story:every-guard-combination-validates-under-the-rule
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:37Z", actor: "human:timo", revision: 16, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:38Z", actor: "human:timo", revision: 17, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:10Z", actor: "human:timo", revision: 18, decided_on: {"recorded":{"test_result":1}}}
---
## Problem

A downstream protocol answers "not found" when a request names a parent row of another tenant.
The parent row is addressed through the input (`input.task_id`), and its child rows (push
configurations) are created, read and deleted by three commands. Each of the three loses the
tenant rule for a different reason. Reported 2026-10-29 by an adopter on ESS 0.75.0. Checked on
the 0.77.0 head (84671d6, `ess 0.77.0`) against the adopter's model at its then-current commit, with the
missing guard added by hand:

- Creator, `when_related: {via: input.task_id, predicate: <tenant differs>}` declared after
  `task-not-found`: it validates and synthesizes. `other-tenant` and `created` both get
  scenarios, and `created` arranges the parent under the tenant it sends. On 0.67.0 this was
  ESS-SYNTH-003. With the guard present, the reader's `found` loses its scenario: ESS-SYNTH-001,
  "`task_id` created by `CreateTaskPushNotificationConfig`, whose `task-not-found` refuses a
  missing `demo.tasks.Task` ... is `row set`, which has no arrangement of rows its selector
  selects".
- Reader: `when_subject:` (the same tenant predicate) beside the row-set guard `config-not-found`
  is refused, ESS-COMMAND-009: "selects on a row set ... and on a `when_subject` guard; one
  command reads one kind of related row in this cut".
  `story:every-guard-combination-validates-under-the-rule` leaves this pair out.
- Deleter: the same `when_subject:` validates and synthesizes. The synthesized `deleted` scenario
  creates its rows under `tenant-1` and deletes with input tenant `tenant`. Against the
  interpreted target it fails with "expected outcome = deleted, observed outcome = other-tenant",
  plus three ESS-CF-VIEW checks.

Repros: outside the repository (the adopter's gate output and the 0.77.0 check).

## Acceptance

The new tests go in `crates/verify/ess-conformance/tests/parent_row_guard.rs`. Each builds its own
inline `demo.d` model, so the suite-bytes tables gain no line. The model:

- `Parent` has an Optional `tenant` and owns `Child` through `parent_id`.
- `CreateParent` sets `tenant` from its input.
- `CreateChild` refuses a missing parent, then refuses a parent of another tenant:
  `when_related: {via: input.parent_id, predicate: <tenant differs, absent matching absent>}`.
- `GetChild` addresses the parent (`unknown_instance: true`, `found` preserves it on `instance:
  parent_id`) and refuses through the row set `{parent_id == input.parent_id, child_id ==
  input.child_id}`.
- `DeleteChild` addresses the parent, refuses another tenant with `when_subject:`, and removes the
  selected child through `affects:`.

Each test runs as `cargo test -p ess-conformance --locked --test parent_row_guard <fn>`:

- `a_creator_guard_on_the_addressed_parent_leaves_the_readers_found_witnessed`:
  - `GetChild/found` and `GetChild/child-not-found` are synthesized with no refusal. 0.77.0
    refuses `found` with the guarded-creator `ESS-SYNTH-001`.
  - Every `CreateChild` step sends the tenant its parent was created with, and sends it absent
    where the parent's is absent.
  - The interpreted target passes every scenario.
- `a_required_tenant_guard_on_the_addressed_parent_is_witnessed`: the same with `tenant` a
  required `String` and `tenant != input.tenant`. `found` is synthesized and passes.
- `a_tenant_view_parameter_beside_the_creator_guard_keeps_child_rows_under_the_parents_tenant`:
  - The model adds a view `Children(parent_id, tenant)` and a `DeleteChild` with no `when_subject:`.
  - In `GetChild/found` and `DeleteChild/deleted`, every `CreateChild` step sends its parent's
    tenant, and no `tenant-<n>` is sent under a parent created with another tenant.
  - The interpreted target passes.
- `an_affects_segment_past_a_subject_refusal_sends_an_input_its_subject_accepts`:
  - In `DeleteChild/outcome/deleted`, the command under test is sent a tenant its subject's tenant
    accepts (absent where the subject's is absent).
  - The interpreted target passes it. 0.77.0 fails it with "observed outcome = other-tenant".
- `an_affects_segment_no_subject_takes_is_refused_by_name`:
  - The creator refuses a parent holding a tenant, and the deleter's `when_subject:` refuses one
    without a tenant.
  - `DeleteChild/deleted` is refused as `ESS-SYNTH-001`, and the refusal names `other-tenant`.
  - No scenario for it is written. 0.77.0 writes one that the interpreted target fails.
  - This test pins the one new refusal (Decisions 3).
- `a_subject_refusal_the_first_input_misses_leaves_the_segment_as_it_was`:
  - The model is the deleter whose `when_subject:` refuses a subject on its stored field alone
    (`tenant == "blocked"`), and compares nothing with the input.
  - The `deleted` segment's `DeleteChild` step sends the input `reach` gives (the base witness,
    the tenant `"tenant"`).
  - The input is chosen again only where the first one takes an earlier refusal. Repository
    models are held by the tables below.

These must stay green, unchanged, each as `cargo test -p ess-conformance --locked --test <file>`:

- `a_row_set_branch_arranges_rows_past_the_creators_guard` (beyond10x/ess#682, including
  `where_no_related_row_can_be_arranged_the_refusal_names_the_creators_guard`).
- `adversary_b_creator_predicate_guard` and `adversary_b_creator_predicate_guard_2`.
- `owned_subject_arrangement` (beyond10x/ess#553, rows under the addressed subject).
- `related_guard_owner_link`, `row_sets`, `row_sets_suite` and `arrangement_creators`.
- `set_effects`, `interpreted_set_effects`, `affects_past_emitting_not_found`,
  `related_record_effects` and `subject_guard_input`.
- `external_beside_held_guard` (`repository_model_suites_change_only_where_claimed`).
- `synthesis_suite_bytes_table` (`every_repository_model_keeps_its_synthesized_bytes`,
  `every_repository_component_keeps_its_synthesized_bytes`):
  - Expected unchanged (Decisions 3).
  - A moved line is re-pinned with `SYNTHESIS_SUITE_BYTES_TABLE_WRITE` /
    `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE` only where the model is one 0.77.0 refused for (a), or
    one whose `affects:` segment it wrote failing for (b), and `CHANGELOG.md` names it.

`CHANGELOG.md` gets one `Fixed` entry for (a) and (b). It names the two kinds of model whose suite
bytes move and the new `ESS-SYNTH-001` refusal.

## Downstream update 2026-10-29

The adopter's next release scopes its configurations view to the caller's tenant (the view takes
a `tenant` parameter). Since then, the synthesized scenarios for the reader's `found` and the
deleter's `deleted` arrange their configuration rows under fresh tenants on a task created under
another tenant. The adopter's server answers those scenarios "not found", and the adopter lists
them as rearranged in its conformance runner.

Checked on the 0.77.0 head (84671d6) against the adopter's main branch. The steps are
(command, input tenant):

- `GetTaskPushNotificationConfig/outcome/found`: SendMessage `tenant`; three
  CreateTaskPushNotificationConfig calls under `tenant-1`, `tenant-2` and `tenant-3`; then
  GetTaskPushNotificationConfig with no tenant. Before the view took `tenant`, the three creates
  sent no tenant.
- `DeleteTaskPushNotificationConfig/outcome/deleted`: SendMessage under `tenant`, `tenant-1` and
  `tenant-2`; five creates under `tenant-1` to `tenant-5`; then the delete with no tenant.
- The interpreted target passes both, because the model leaves the parent-row tenant guard
  unmapped, as described above. A target that enforces the rule refuses them.

Added acceptance:

- Synthesis arranges a child row under the tenant (or other input) its parent row was created
  under, whenever the parent row's guard or a view parameter reads that input. It does not
  diversify the input across tenants it never created a parent row under.

## Fit review

Reproduced on `ess 0.77.0` with a neutral model kept outside the repository: `demo.d.Parent` (Optional
`tenant`) owns `demo.d.Child` (Optional `tenant`, `parent_id`, `child_id`) through `parent_id`;
`CreateChild` refuses a missing parent (`when_related: {via: input.parent_id, exists: false}`);
`GetChild` addresses the parent (`unknown_instance: true`, `found` preserves it on `instance:
parent_id`) and guards on the row set `{parent_id == input.parent_id, child_id == input.child_id}`;
`DeleteChild` addresses the parent and removes the selected child through `affects:`.

1. **Need.** Four behaviours, in neutral terms:
   - (a) A creator refusing a child when the parent its input names holds another tenant
     (`when_related: {via: input.parent_id, predicate: <tenant differs>}`) validates, and the reader
     addressing that parent loses `found`: `ESS-SYNTH-001`, "`parent_id` created by
     `demo.d.CreateChild`, whose `parent-not-found` refuses a missing `demo.d.Parent` ... has no
     arrangement of rows its selector selects". The same with a required `String` tenant and
     `tenant != input.tenant`. Without the reader's subject (no `unknown_instance`, `found` emits
     an event) the same creator guard synthesizes 6 scenarios, 0 refusals, each child created under
     its parent's tenant.
   - (b) `DeleteChild` with `when_subject: {predicate: <tenant differs>}` before `deleted`
     synthesizes 9 scenarios, and the interpreted target fails `DeleteChild/outcome/deleted`: the
     `affects:` segment creates its subject with no tenant and deletes with input tenant `tenant`,
     so the command answers `other-tenant`.
   - (c) `GetChild` with the same `when_subject:` beside its row set is refused, `ESS-COMMAND-009`.
   - (d) With no creator guard and a view `Children(parent_id, tenant)`, `found` creates its
     children under `tenant-1` to `tenant-3` and `deleted` under `tenant-1` to `tenant-5`, on a
     parent created with no tenant; the interpreted target passes them.

   The requester's wording, theirs: "Synthesis arranges a child row under the tenant (or other
   input) its parent row was created under, whenever the parent row's guard or a view parameter
   reads that input"; "`when_subject:` beside a row-set `when_related:` validates".
2. **Class.**
   - (a) Defect. Synthesis documents arranging a row set past a guarded creator
     (`crates/verify/ess-conformance/src/synthesize/row_set.rs:3060-3072`, beyond10x/ess#682), and
     does so where the reader addresses no subject. Rows a selector reads under the addressed
     subject (beyond10x/ess#553) skip that path: `linked` returns `None` for them
     (`row_set.rs:4203`), and `arrange_row` creates them through `arrange_toward_owned`
     (`row_set.rs:2356`), which hands `arrange_toward_with` no held rows. So its second pass, which
     steers a creator's input on a captured row's settled values (`synthesize.rs:12563`, "row 29"),
     never runs, and every child is sent at its own distinction's tenant.
   - (b) Defect: a synthesized scenario the interpreted target fails. `affects_segment_on`
     arranges the segment's subject with `prepare_in` (`set_effects.rs:2795`) and picks the input
     with `reach` (`set_effects.rs:2806`), each without the other, so a `when_subject:` refusal
     comparing the subject with the input can answer first.
   - (c) Gap, refused on purpose at every format: "a `when_subject` guard beside a row set keeps
     it at every format" (`CHANGELOG.md:112`, 0.70.0; `crates/specify/ess-domain/src/command/row_set.rs:39-42`).
   - (d) Not a defect. Without the creator guard the specification admits a child under any
     tenant on any parent, and moving every leaf of a further instance is rule 5 of
     `crates/verify/ess-conformance/src/witness.rs:77-80`: the spread tenants are what catch a
     target ignoring the view's tenant filter. A server refusing those rows enforces a rule its
     specification does not state.
3. **Already expressible?**
   - (a), (b): no.
   - (c): in part. The comparison can sit in the row-set `where:` itself: with a required tenant,
     `tenant == input.tenant` there synthesizes 7 scenarios, 0 refusals, and the interpreted target
     passes them. With Optional tenants written as a presence disjunction (absent matches absent)
     the reader's `found` starves with or without a subject (`ESS-SYNTH-001`, same text as (a)),
     while plain `tenant == input.tenant` over the Optionals synthesizes and passes. That
     starvation is a separate defect, split out (Decisions).
   - (d): yes, with the creator guard: with it and the same view parameter (reader without a
     subject), 8 scenarios, 0 refusals, the interpreted target passes, and every child of `found`
     and `deleted` carries its parent's tenant. With (a) delivered the same holds beside the
     reader's subject. The idiom keeps the figure in the specification
     (`docs/design/generic-language-features.md` § Policy, policy 1).
4. **Fit.** (a) and (b) add no surface, keyword, diagnostic or format: they reuse the per-input
   steering the bound row search already has (`related_guard::steering_for`, `present_for`, through
   `synthesize.rs` `arrange_toward_held`) and the steered subject the `affects:` segment already
   builds for creators reading the subject (`set_effects.rs` `affects_segment`, `prepare_steered`).
   No grid cell moves. (c) is grid R, row `when_related:` predicate, column row set, "no: one kind
   of related row per command" (`docs/design/language-grids.md:25`); the design that lifts it is
   decided ("ESS-COMMAND-009's one-kind-of-row refusal is lifted with G2",
   `docs/design/generic-language-features.md:470-471`), its format is not allocated.
5. **Second adopter.** Orders and their line items: `AddLine` refuses an order of another region
   (`when_related: {via: input.order_id, predicate: region != input.region}`), `GetLine` addresses
   the order and selects the line by `(order_id, line_id)`, `RemoveLine` refuses a caller of
   another region with `when_subject:` and removes the line through `affects:`. Each of (a) to (c)
   arises unchanged.
6. **Cost.** (a), (b): no source, IR, generated-code or wire change; suite bytes move only for a
   model that 0.77.0 refuses with `ESS-SYNTH-001` for (a) or whose `affects:` segment the
   interpreted target fails for (b). (b) adds one synthesis refusal: a segment for which no subject
   passes the earlier `when_subject:` refusal, which 0.77.0 writes as a failing scenario. (c) is a
   source format, which D214 keeps out of batch 3.
7. **Considered.**
   - Change nothing: the adopter keeps rearranging scenarios by hand, and (b) keeps writing a
     scenario that fails every correct target. Rejected.
   - Steer every creation under an addressed subject on the subject's values, not only where the
     plain creation finds no row: also correct, but it can move bytes of every beyond10x/ess#553
     model. Rejected for the fallback.
   - The requester's view-parameter half: arrange a child only under the input value its parent
     was created with whenever a view parameter reads it. Rejected: it narrows the rows the
     specification admits, and removes the spread that catches a target ignoring the filter. The
     creator guard says the rule and is witnessed.
   - (c) now, under `ess/32`: needs a new format (D214, D135). Split out.

## Decisions

**Accept, redesigned** (2026-10-29, batch 3, D214). The unit fixes the two synthesis defects,
(a) and (b) of the fit review. It leaves out the validation half and the view-parameter rule.

1. **(a) Child rows under the addressed parent pass the creator's guard over that parent.**
   - `prepare` (`crates/verify/ess-conformance/src/synthesize/row_set.rs:3074`) gets one more
     attempt after `Arranging::LINKED`, before `guarded_creator` (`:3205`).
   - In that attempt `arrange_row` creates each row the selector reads under the addressed subject
     (`owner.under`, `:2356`) through `super::arrange_toward_held`
     (`crates/verify/ess-conformance/src/synthesize.rs:12488`).
   - The owner relation's `via` is bound to the subject instance, and the subject's state and
     settled values are held. So the creator's input is steered on the parent row as it was created
     (`related_guard::steering_for`, used by the second pass of `arrange_toward_with`,
     `synthesize.rs:12563`), as the linked path of beyond10x/ess#682 already does for a related row
     arranged first.
   - `Owner` (`row_set.rs:2214`) and `Plan.owners` (`:2493`) carry the subject's arrangement from
     `unders` (`:3356`) for it.
   - The attempt runs only where every earlier attempt failed. Every model 0.77.0 synthesizes keeps
     its arrangement and its bytes. The models that change are the ones whose branch it refused
     with the guarded-creator `ESS-SYNTH-001`, and their scenario replaces that refusal.
   - Inferred, not built: that `arrange_toward_held` with the owner's `via` bound and the subject
     held is enough for the creator's predicate. The first acceptance test decides it.
2. **(b) The `affects:` segment sends an input its subject accepts.**
   - `affects_segment_on` (`crates/verify/ess-conformance/src/synthesize/set_effects.rs:2762`) today
     takes the input from `reach` (`:2806`) without looking at the subject `prepare_in` arranged
     (`:2795`).
   - Where a `when_subject:` refusal compares the subject with the input and answers before the
     branch, and that input on the arranged subject takes the refusal, the input is chosen again:
     - The candidates come from the command's guards, grounded on the subject's settled values
       (`subject_fact::grounded`, `input_presence`).
     - This is the steering `row_set::steered_inputs` (`row_set.rs:4061`) already gives a creator
       beside a related row.
   - The first candidate on which the branch is selected is sent. A segment whose input already
     selects the branch keeps its bytes.
   - Where no candidate selects it, the segment is refused as `ESS-SYNTH-001`, and the refusal names
     the `when_subject:` refusal that answers first.
3. **D135 and D214.**
   - No source, IR, generated-code or wire change is made.
   - Suite bytes change only for models 0.77.0 refused for (a), or whose `affects:` segment the
     interpreted target fails for (b).
   - A consumer pinning those bytes pinned a refusal, or a scenario every correct target fails.
     The change is a correction, not a contract change.
   - The one new refusal, the segment of 2 that no subject passes, is a synthesis refusal and not a
     refusal of input:
     - `ess verify conform synthesize` exits 0 with refusals (checked on 0.77.0).
     - The suite records none.
     - 0.77.0 wrote a scenario in its place that the interpreted target fails (repro outside the
       repository: `DeleteChild/outcome/deleted`, 9 passed, 1 failed).
   - It ships as a refusal. Before the release, the batch's adopter replay reports any adopter
     suite that loses such a segment. If one does, that adopter's specification contradicts itself,
     and the release notes say so. Tests: `an_affects_segment_no_subject_takes_is_refused_by_name`, `a_subject_refusal_the_first_input_misses_leaves_the_segment_as_it_was`.
   - Both suite-bytes tables (`SYNTHESIS_SUITE_BYTES_TABLE_WRITE`,
     `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE`) are expected to stay unchanged.
     - No repository model pairs the creator's guard of beyond10x/ess#682 with a reader addressing
       the parent.
     - No repository model has a `when_subject:` beside `affects:` (grep, inferred).
     - The new tests use inline models, which the tables do not walk.
     - A line that moves anyway is re-pinned only with the model named in `CHANGELOG.md`, and only
       if it is one of the two kinds above. Any other move is a defect of the change.
4. **Not in this unit.**
   - (c) `when_subject:` beside a row set stays `ESS-COMMAND-009` under `ess/32`.
     - Lifting it is a source-format change.
     - The design is decided: "ESS-COMMAND-009's one-kind-of-row refusal is lifted with G2",
       `docs/design/generic-language-features.md:470-471`.
     - Its format is G2's (`:552-556`, wave 4 at `:631`), so it needs no operator choice. It goes to
       the draft follow-up `story:when-subject-beside-a-row-set-validates-under-the-g2-format`.
     - Until then the comparison is written in the row set's `where:`. The fit review, question 3,
       has the command output.
   - Two Optional fields compared through a presence disjunction in a row-set `where:` starve
     `found` with or without a subject. That is a separate synthesis defect of the row search
     (`row_set.rs`, `steered_inputs` / `rows_tried`, inferred). It goes to the draft follow-up
     `story:a-row-set-selector-matching-absent-with-absent-is-witnessed`.
5. **Changed from the request.**
   - The added acceptance, "arranges a child row under the tenant its parent row was created under,
     whenever ... a view parameter reads that input", is dropped.
     - A view parameter states which rows a read returns, not which rows a creator accepts.
     - Without a guard saying so, a child under another tenant is a row the specification admits.
     - Spreading tenants by rule 5 (`crates/verify/ess-conformance/src/witness.rs:77-80`) is what
       makes a target that ignores the view's filter fail.
   - Idiom: declare the creator's guard over the parent row, `when_related: {via: input.parent_id,
     predicate: <tenant differs>}`. With it, every child of `found` and `deleted` carries its
     parent's tenant (repro outside the repository, 8 scenarios, 0 refusals, interpreted target
     passes). With (a), the same holds for a reader addressing the parent.
   - The policy test of `docs/design/generic-language-features.md` § Decision 6 holds: the rule
     stays in the specification, as a guard.

## Scope

Primary surface: `ess-conformance` synthesis, in two places: the row-set arrangement of rows under an
addressed subject, and the input of an `affects:` segment. Size: M, one implementor, one sitting.

- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (cited):
  - `prepare` (`:3074`) gets one attempt after `Arranging::LINKED`.
  - `Arranging` (`:3116`) gets one flag.
  - `prepare_with` (`:3304`) and `unders` (`:3356`) hand the subject's arrangement on.
  - `struct Owner` (`:2214`) and `Plan.owners` (`:2493`) carry it.
  - `arrange_row` (`:2297`): under the flag, the `create` closure at `:2356` calls
    `super::arrange_toward_held` in place of `arrange_toward_owned`.
  - `linked` (`:4182`) and `guarded_creator` (`:3205`) are unchanged.
- `crates/verify/ess-conformance/src/synthesize/set_effects.rs` (cited): `affects_segment_on`
  (`:2762`) chooses the input again after `reach` (`:2806`), on the subject `prepare_in` /
  `prepare_steered` arranged (`:2795`), and refuses where none selects the branch.
  `affects_segment` (`:2653`) is unchanged.
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (inferred): read only.
  - `grounded` (`:1061`), `input_presence` (`:1074`) and `row_truth_with` (`:474`) are called from
    `set_effects.rs`.
  - At most one is made visible, if a private helper is needed.
- `crates/verify/ess-conformance/src/synthesize.rs` (inferred): no edit expected.
  - `arrange_toward_held` (`:12488`) and `arrange_toward_with` (`:12543`) are called as they are.
  - If U14 has landed, the new call passes U14's mode off.
- `crates/verify/ess-conformance/tests/parent_row_guard.rs`: new, the six tests of `## Acceptance`.
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` and
  `synthesis-component-suites-table.tsv` (inferred): expected unchanged, re-pinned only as
  `## Acceptance` says.
- `CHANGELOG.md`: one `Fixed` entry.

Collisions with the other batch-3 units:

- U14 (`story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed`):
  - Both units edit `set_effects.rs`.
    - U14 adds a retry at the end of `affects_segment` and passes a mode through
      `affects_segment_on`.
    - U13 edits the input choice inside `affects_segment_on` after `reach`.
    - Same function, different regions.
  - U14 changes the signature of `arrange_toward_with` / `arrange_toward_held` (a mode beside
    `(owner, named)`). U13 adds one caller of `arrange_toward_held` in `row_set.rs`.
  - Run U14 first, or rebase whichever lands second. The rebase is one argument at one call site,
    plus one region of `affects_segment_on`.
- U11 (beyond10x/ess#645, #693): U13 edits neither `synthesize.rs::arranged` nor
  `synthesize/related_guard.rs`; it calls `related_guard::steering_for` indirectly, unchanged. No
  shared region.
- U15 (authored scenario reading): no shared file.
- Follow-ups out of the unit: `story:when-subject-beside-a-row-set-validates-under-the-g2-format`
  (`crates/specify/ess-domain/src/command/row_set.rs`, G2's format) and
  `story:a-row-set-selector-matching-absent-with-absent-is-witnessed` (`row_set.rs` row search).
