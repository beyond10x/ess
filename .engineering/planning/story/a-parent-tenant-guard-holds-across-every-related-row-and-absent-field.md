---
format: aep.planning-md/3
id: story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field
kind: story
status: draft
title: A parent tenant guard holds across every related row and an absent field
summary: 'U13 adversary: a second related row guarded on tenant still starves found; an absent Optional tenant splits synthesis and interpreter'
tags:
- defect
- ess-next
- severity-high
- zero-open-issues
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
revision: 8
---
## Outcome

A specification whose child rows are created, read and deleted under a tenant-guarded parent gets
scenarios that a correct implementation passes. That holds when the child's creator also guards a
second related row on the tenant, and when the tenant is `Optional` and compared plainly.

## Context

Found by the batch-3 adversary pass over the parent-row guard
(`story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`). Both
findings reproduce on 0.92.0.

Base model: the `MODEL` of `crates/verify/ess-conformance/tests/parent_row_guard.rs`. `Parent {id,
tenant: Optional<String>}` owns `Child {row, child_id, parent_id, tenant}` via `parent_id`.
`CreateChild(tenant, parent_id, child_id)` refuses `parent-not-found` (`when_related exists:
false`) and then `other-tenant` (`when_related via: input.parent_id`, the tenant comparison with
absent matching absent). `GetChild` addresses the parent and reads its child through the row set
`{parent_id == input.parent_id, child_id == input.child_id}`. `DeleteChild` refuses
`other-tenant` with `when_subject:` and deletes the child through `affects:`. The base passes 10 of
10.

## Problem

1. **A second related row guarded on the tenant.** Add `Folder {id, tenant: Optional<String>}`,
   created by `CreateFolder(tenant)`. Give `CreateChild` a `folder_id` input with `folder-not-found`
   (`exists: false`) and `folder-other-tenant` (the same tenant comparison via `input.folder_id`),
   and store `folder_id` on the child.
   - With the parent guard (0.92.0): `GetChild/found` is refused as ESS-SYNTH-001 ("whose
     `parent-not-found` refuses a missing `demo.d.Parent` … no arrangement left a related row"),
     and `DeleteChild/deleted` is refused as ESS-SYNTH-001 ("created only by
     `demo.d.CreateChild/created`, which no arrangement takes"). The held-under fallback binds
     only the parent link.
   - Without the parent `other-tenant` guard (0.92.0): `GetChild/found` and
     `GetChild/child-not-found` are synthesized and fail on `--target interpreted`. The arranged
     `CreateChild` sends `tenant = "tenant-1"` with a folder created without a tenant, and the
     interpreter correctly answers `folder-other-tenant` where the suite expects `created`. The
     suite fails a correct implementation. `CreateChild/created` is also refused as ESS-SYNTH-003.
2. **An absent Optional tenant compared plainly.** In the base model, replace both tenant guards
   with `predicate: tenant != input.tenant`. `DeleteChild/deleted` sends a tenant to a parent
   created without one. The interpreter decides the guard Unknown and reports the scenario
   unsupported ("the guard of `demo.d.DeleteChild/other-tenant` is Unknown over this input"),
   which is correct (`interpret/execute/subject.rs:128`, Unknown is Undecidable). Synthesis treats
   Unknown-by-absence as not taken (`subject_fact.rs` `guard_truth_with` :497, `unknown_by_absence`
   :511), so the two disagree. 7 passed, 1 unsupported; `GetChild/found` and
   `DeleteChild/other-tenant` are also refused.

## Acceptance

- Case 1: with the parent guard, `GetChild/found` is synthesized: every related row that the
  child's creator guards on the input is created under that input's tenant, and the interpreted
  target passes it. Without the parent guard, no synthesized scenario fails on
  `--target interpreted` (each is passed, or refused by name).
- Case 2: with plain `tenant != input.tenant` over Optional tenants, synthesis and the interpreter
  decide an absent tenant the same way. The `deleted` witness either passes or is not written
  (refused by name); one test on each side pins the decision.
- Both models are committed as tests (`tests/parent_row_guard.rs` or a new file), red before the
  fix; the base model keeps 10 of 10 and its suite bytes.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`arrange_row` :2321, `held_subject` :2379, `prepare` :3168, `Arranging::HELD_UNDER` :3301)
- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (`arrange_toward_beside` :13832, `CreatorPass::Beside` :13928-14019, reuse for the held-under path)
- inferred: `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`beside_bound` :646, `present_beside` :682)
- cited: `crates/verify/ess-conformance/src/synthesize/set_effects.rs` (`subject_accepted` :2977)
- cited: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`guard_truth_with` :497, `unknown_by_absence` :511)
- cited: `crates/verify/ess-conformance/tests/parent_row_guard.rs`

confidence: medium (finding 2 medium-high; finding 1 medium)

## Ambiguities

- Finding 2 is close to `story:unknown-by-absence-in-falsified-and-leaves-external` (draft), which
  holds the #234 ruling that Unknown-by-absence counts as false in `falsified`/`leaves_external`.
  The change here is on the `subject_accepted` path. If it needs the same reversal, it waits for
  that ruling (`requires-stakeholder-input`, the ESS maintainer).
