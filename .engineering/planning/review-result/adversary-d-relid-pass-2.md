---
format: aep.planning-md/3
id: review-result:adversary-d-relid-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit relid
relations:
- reviews: story:a-relation-can-be-carried-by-the-identity
revision: 1
---
unit: relation-via-identity (beyond10x/ess#230) after correction 1, uncommitted working tree on 4e7c3867e in ess-d-relid
verdict: NEEDS-CHANGE
cases: executed 1617→1620 (ess-conformance), red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory, ~/.cache/ess-wave-n2/relid/adv2/ (base copy and base target deleted)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat`: `8 files changed, 566 insertions(+), 73 deletions(-)`. All 8 files are the implementor's. I changed no tracked file. I added one untracked test file:
`crates/verify/ess-conformance/tests/adversary_relid_pass2.rs`.

## 2. Cases added (run alone before the suite)

| case | asserts | now |
|---|---|---|
| `a_read_through_a_generated_identity_is_arranged_or_refused_never_dropped` | Fixture plus `User` declaring `{identity, references, target: Identity, one, via: user_id}` (the other side, as the changelog allows) and `TouchUser` (updates User) emitting `subject: {related: {via: user_id, field: subject}}`. The model validates. The scenario either arranges a `BindIdentity` keyed by the user and asserts its subject, or is refused by name. | red |
| `an_identity_carrier_and_a_field_carrier_to_one_entity_are_each_witnessed_apart` | `Identity` carries `user` via its identity and `backup_user` via field `backup`, both to `User`. The rebind reads email through each. The two bound users differ, their emails differ, and each is asserted on its own field (classes 1 and 3). | green |
| `a_guard_on_the_creating_branch_does_not_stop_the_update_reading_through_the_identity` | Only `BindIdentity` is guarded by `when_related`. `RebindIdentity` is synthesised and asserts the keyed user's email between decoys. | green |

Red output, verbatim (`cargo test -p ess-conformance --test adversary_relid_pass2 --no-fail-fast`, EXIT=101):

```
test a_read_through_a_generated_identity_is_arranged_or_refused_never_dropped ... FAILED
thread 'a_read_through_a_generated_identity_is_arranged_or_refused_never_dropped' (1821320) panicked at crates/verify/ess-conformance/tests/adversary_relid_pass2.rs:324:9:
`demo.provisioning.TouchUser/outcome/touched` reads the identity row keyed by the user and arranges none: commands ["demo.provisioning.RegisterUser", "demo.provisioning.TouchUser"], asserted subject None, refusals []
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The line moved to :335 after rustfmt. The assertion is unchanged.

Origin: the same file on a `git archive 4e7c3867e` copy (own target `ess-d-relid-base`). All 3 cases fail at assembly:
`[missing_declaration] entity demo.provisioning.User.relations.identity: relation `identity` of `demo.provisioning.User` is carried by `user_id`, which `demo.provisioning.User` does not declare`.
The base refuses the model, so the path is reachable only through this unit.

## 3. Suite (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`: EXIT=101, `error: 1 target failed: -p ess-conformance --test adversary_relid_pass2`. Summed over all targets: executed 1620, failed 1. The implementor's c1 run executed 1617, failed 0. My file adds 3.
Clippy (`-p ess-conformance --test adversary_relid_pass2 -- -D warnings`) is clean. rustfmt `--check` is clean.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/related.rs:456 (`stored`, via `point_at` :397; dropped at `arrange` :150 `continue`) | warning | NEEDS-CHANGE | introduced | A related read through an existing subject's identity whose creating act does not fill the identity from input (generated, as `User` in the issue's own fixture) finds no input to rewrite. `stored` returns false and `arrange` `continue`s. The scenario runs `RegisterUser`, `TouchUser` with no `Identity` row, asserts no `subject`, and emits no refusal: a silent drop (class 2). | Arrange the referenced row keyed by the subject (create `Identity` with `user_id` = the subject after it exists), or refuse under `arrange_related_row` naming the generated identity. |

What reaches it: `synthesize` on any model that declares the identity-carried `references` on the side whose identity is generated and reads through it. The changelog advertises "the same relation may be declared from the other side". I built the model and found no adopter model with it.

## 5. Attacked and could not break

- Base-validity: `identity_reference` requires `relation.via == identity.name`. At base that was always refused: `validate_relations` runs unconditionally (`spec.rs:366`), and a field may not share the identity's name (`entity.rs:1753`). So no base-valid model reaches any gated branch. Synth `…Data` output is byte-identical for an identity without a relation. `resolve.rs` is unchanged.
- Gate coverage: every identity-as-carrier site asks `identity_reference`. These are `input_carrier`, `subject_field_from_input`, and conformance `input_read`/`point_at`/`stored`. `referenced_entity`, `relations_carried_by`, docs, ess-gen and service-contract are generic over `via`.
- Classes 1 and 3: an identity carrier plus a field carrier to one entity get distinct users and emails (green).
- A guard on the creating branch only: the update read is still witnessed (green).
- Three entities sharing `UserId`, subject without a relation: still ambiguous, as at base.

## 6. Paths written outside the worktree

- `~/.cache/ess-wave-n2/relid/adv2/`: `review.md`, `red-conformance.log`, `red-conformance-2.log`, `red-conformance-3.log`, `base-conformance.log`, `suite-conformance.log`, `clippy.log`.
- `~/.cache/ess-wave-n2/relid/adv2/base/` (base extract) and `~/.cache/b10x-target/ess-d-relid-base/` (base build): both deleted.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/related.rs
  line: 456
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a related read through an existing subject's identity whose creating act generates the identity arranges no referenced row and is neither witnessed nor refused
```
