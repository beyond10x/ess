---
format: aep.planning-md/3
id: review-result:adversary-d-relid-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit relid
relations:
- reviews: story:a-relation-can-be-carried-by-the-identity
revision: 1
---
unit: relation-via-identity (beyond10x/ess#230), uncommitted working tree on 4e7c3867e in ess-d-relid
verdict: NEEDS-CHANGE
cases: executed 1038→1041 (ess-domain), red 3; ess-entity-runtime 82 run after adding 1 (green, no before run)
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (see part 6)
needs-coordinator: yes. Finding 1 means a model that validated before this change is now refused, which breaks the premise of the decision that this unit needs no format gate

## 1. Diff stat

`git --no-pager diff --stat` is unchanged from the implementor's: 8 files, 488 insertions, 72 deletions, all theirs. I changed no tracked file. I added two untracked test files:

- `crates/specify/ess-domain/tests/adversary_relid_pass1.rs`
- `crates/generate/ess-entity-runtime/tests/adversary_relid_pass1.rs`

## 2. Cases added (each file was run on its own before any suite run)

| case | asserts | now |
|---|---|---|
| domain `a_guard_decided_by_the_creating_branchs_relation_is_not_undone_by_an_earlier_identity_input` | The sign-in fixture plus a second entity `Quota` keyed by `TenantId`, a `references` relation on `SignIn.tenant`, and a `counted` branch that updates `Quota` by `input.tenant`. It validates with that branch after `initiated` (control, green) and also with it before `initiated`. | red |
| domain `a_creating_branch_cannot_read_the_row_it_is_creating_through_its_own_identity` | `creates: User` with a caller-supplied id and `{related: {via: user_id, field: email}}` is refused. No relation is declared anywhere. | red |
| domain `a_relation_from_an_entity_to_itself_through_its_identity_is_refused` | `User` declaring `{references, target: User, one, via: user_id}` is refused. | red |
| runtime `an_identity_carried_reference_lowers_to_definitions_the_registry_accepts` | A `Mirror` entity referencing `Child` via `child_id` lowers. The lowered relation is `References` with via `child_id`, and entity-core 0.24.1 `Registry::validate_all` accepts it. | green |

Red output, verbatim (`cargo test -p ess-domain --test adversary_relid_pass1 --no-fail-fast`, EXIT=101):

```
test a_relation_from_an_entity_to_itself_through_its_identity_is_refused ... FAILED
test a_creating_branch_cannot_read_the_row_it_is_creating_through_its_own_identity ... FAILED
test a_guard_decided_by_the_creating_branchs_relation_is_not_undone_by_an_earlier_identity_input ... FAILED
thread 'a_guard_decided_by_the_creating_branchs_relation_is_not_undone_by_an_earlier_identity_input' (656213) panicked at crates/specify/ess-domain/tests/adversary_relid_pass1.rs:79:9:
the relation on SignIn.tenant still decides the guard
[conflicting_declaration] command.demo.signin.InitiateSignIn.outcomes.no-configuration.when_related: `input.tenant` is `demo.signin.TenantId`, the identity of `demo.signin.Configuration` and `demo.signin.Quota` (hint: give the entities distinct identity types, or store the input in a field of the created subject that a `references` relation carries)
thread 'a_creating_branch_cannot_read_the_row_it_is_creating_through_its_own_identity' (656212) panicked at crates/specify/ess-domain/tests/adversary_relid_pass1.rs:134:5:
a creating branch reads a field of the row it creates, before it exists:
thread 'a_relation_from_an_entity_to_itself_through_its_identity_is_refused' (656215) panicked at crates/specify/ess-domain/tests/adversary_relid_pass1.rs:146:5:
a `references` from an entity to itself through its own identity is refused:
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Origin run: `git archive 4e7c3867e` into `~/.cache/ess-wave-n2/relid/adv1/base`, built with its own target `~/.cache/b10x-target/ess-d-relid/adv1-base`, EXIT=0:

```
test a_relation_from_an_entity_to_itself_through_its_identity_is_refused ... ok
test a_creating_branch_cannot_read_the_row_it_is_creating_through_its_own_identity ... ok
test a_guard_decided_by_the_creating_branchs_relation_is_not_undone_by_an_earlier_identity_input ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

I ran the base once before that on the shared `ess-d-relid` target. Cargo reused the tree's `ess-domain` library: it produced the same test-binary hash `adversary_relid_pass1-df51f59bf8a5b6cf`, finished in 0.71 s, and all three cases were red. I discarded that run.

## 3. Suite runs (after the cases existed)

- `cargo test -p ess-domain --no-fail-fast`: EXIT=101, `error: 1 target failed: -p ess-domain --test adversary_relid_pass1`. Summed over all targets: executed 1041, failed 3. The implementor's run executed 1038, failed 0 (`relid/test-ess-domain.log`).
- `cargo test -p ess-entity-runtime --no-fail-fast`: EXIT=0. Last line: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`. Executed 82, failed 0. My file adds 1 of those; there was no before run.
- `cargo clippy -p ess-domain -p ess-entity-runtime --test adversary_relid_pass1 -- -D warnings`: `Finished`, clean. `cargo fmt --check` is clean after rustfmt on my two files.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/specify/ess-domain/src/command/related_value.rs:240 | blocker | NEEDS-CHANGE | introduced | `input_carrier` now returns the subject's identity for any branch that names its instance by the input, whether or not the identity carries a relation. `related_guard::related_entity` (related_guard.rs:242) takes the first outcome that returns a carrier. So an earlier `updates:`/`moves:` branch keyed by `input.<via>` hides the `references` relation on a later branch's `sets:` field, and a `when_related:` guard that validated at 4e7c3867e is now refused as ambiguous. The model uses nothing from #230, so a model that validated before changes meaning, which the no-gate decision excludes. | Return the identity from the `.or_else` only when the identity carries a `references`/`one` relation (the same test `referenced_entity` applies), or have `related_entity` prefer a carrier that decides. |
| crates/specify/ess-domain/src/command/related_value.rs:190 | warning | NEEDS-CHANGE | introduced | `subject_field_from_input` admits the subject's identity on `creates:` with no relation. When the identity type names only the subject's entity, `{related: {via: <identity>}}` on a creating branch reads the row the branch is creating, before it exists. The base refused this (a subject `via` on `creates:` needed a `sets:` field). | Admit the identity on `creates:` only when the entity it resolves to is not the subject's own entity, or only through a relation the identity carries. |
| crates/specify/ess-domain/src/entity.rs:1524 | warning | NEEDS-CHANGE | introduced | `carried_by_identity` accepts a `references` from an entity to itself through its own identity. Every row references itself, so the relation states nothing. It is still published as `x-ess-relation` on the entity's own identity, lowered as a runtime relation, and lets a creating branch read its own row through a declared relation. The implementor flagged this as open. | Refuse `target == source` in `carried_by_identity` as `conflicting_declaration`. |

What reaches each finding:
- Finding 1: `Specification::assemble` → `related_guard` validation, on any ess/18 model with a `when_related:` guard whose command also has an update/move branch keyed by the same input, where the identity type is shared. I built this model; the shape comes from the shipped sign-in fixture. I found no adopter model with it in the repo.
- Finding 2: `value_expression::related_via` during validation. I built the model. Caller-supplied ids on `creates:` are a documented pattern: the #230 fixture's `BindIdentity` uses one.
- Finding 3: `validate_relations`, for any author who writes the relation. It is built, not observed, and it is a design call.

## 5. Attacked and could not break

- Type check: an identity type that differs from the target's identity, or a newtype over it, is refused `type_mismatch`, because `carried_types` uses exact type equality.
- Cross-domain target: the fixture is already cross-domain, and `validate_relations` runs over every entity in the population.
- Ownership: `owner_of`, the `service-contract` closure (`Owns` only) and entity-runtime `entity_closure` (`Owns` only) never treat a `references` as an ownership. entity-core accepts the identity as the carrier because the identity is a `$fields` field.
- Generated artifacts: the Rust `…Data` identity line gets the annotation once. The JSON-schema identity property carries a single `x-ess-relation`. There is no duplicate field.
- ess-diff: adding, removing or moving a relation is one generic `relations-changed` class; nothing is identity-specific.
- Byte stability: the implementor's `generate --check` reported "projections are up to date" (EXIT=0). Not re-run.
- The three defect classes: `point_at` on an identity pointing at its own entity returns false. That is the documented payload-shape fallback, not a new silent drop.

## 6. Paths written outside the worktree

- `~/.cache/ess-wave-n2/relid/adv1/`: `review.md`, `red-domain.log`, `base-domain.log`, `runtime.log`, `suite-domain.log`, `suite-runtime.log`, and `base/` (46M base-commit extract).
- `~/.cache/b10x-target/ess-d-relid/adv1-base/`: 301M isolated build for the base run. It can be deleted.
- The first, discarded base run wrote the base-compiled `adversary_relid_pass1` test binary into the shared `~/.cache/b10x-target/ess-d-relid`. The later tree run rebuilt it.

## 7. Findings block

```findings
- file: crates/specify/ess-domain/src/command/related_value.rs
  line: 240
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: input_carrier returns an identity carrier with no relation, so a when_related guard decided by a later branch's references relation is refused as ambiguous when an earlier branch updates by the same input, and a model that validated before now fails
- file: crates/specify/ess-domain/src/command/related_value.rs
  line: 190
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a creates branch may now read a field of the row it is creating through its own identity when no relation is declared
- file: crates/specify/ess-domain/src/entity.rs
  line: 1524
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a references relation from an entity to itself through its own identity is accepted and published although it states nothing
```
