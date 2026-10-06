---
format: aep.planning-md/3
id: story:feature-request-463
kind: story
status: active
title: A row-set selector cannot select rows by a struct identity
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#463
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
- depends_on: story:feature-request-428
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/identity.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_w1_1_pass1.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/row-set-struct-identity.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/row_set_struct_identity.rs
- confidence: cited
  path: docs/design/identity-changing-updates.md
- confidence: cited
  path: website/docs/reference/predicates.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T23:57:37Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-05T23:57:37Z", actor: "human:timo", revision: 3}
---
## Outcome
Resolve beyond10x/ess#463: A row-set selector cannot select rows by a struct identity.

## Origin
beyond10x/ess#463, filed 2026-10-05; an adopter's storage-port specification on ess 0.53.0 (`ess/22`), whose refusal over a namespace keyed `{tenant, namespace}` loses both scenarios. Reproduced minimally and fresh in `<fit-review scratch>/probe-463/`.

## Fit review
1. Need: a command refuses when a stored record with a composite identity, the one the input names, has some property. Minimal reproduction `probe-463/a/`: `demo.vault.Shelf` identified by `at: demo.vault.Place {region, shelf}` (both `String`); `Store {place}` takes `sealed` under `when_related: {entity: Shelf, where: {all: [at.region == input.place.region, at.shelf == input.place.shelf, mode == sealed]}, exists: true}`, else `stored`. On installed ess 0.53.0 `ess specify validate` prints `demo v1 — 2 file(s), valid`. `ess verify conform synthesize` refuses both branches: "selects rows by no equality between a String or Uuid field and the input or the addressed subject" (`probe-463/a.synth.out`). The whole-identity form `at == input.place` is refused at validate: `[type_mismatch] … operator == does not admit demo.vault.Place (aggregate)`, ESS-COMMAND-002 (`probe-463/b/`). The requester proposed (theirs): either member equalities over every `String` member count as the key, or `==` compares two values of one struct type.
2. Class: gap, in synthesis only. Validate accepts the selector, and a row-set selector reads "each candidate row's declared fields, identity and held lifecycle state" (`website/docs/reference/predicates.md:28`). Synthesis requires the equality to be on a one-segment `String`/`Uuid` path (`crates/verify/ess-conformance/src/synthesize/row_set.rs:179-213`, `let [name] = path.segments()` at :186). The documented rule (`predicates.md:1303-1305`) does not say whether a member of an identity counts. The identity is bound for a selector only when the path has one segment, in three places: `subject_fact.rs:484-491`, `row_set.rs:463-467` and `steered_identity` in `synthesize.rs:9832-9842`. All three were added for #429 and run from `ess/23` only (`identity_selectors`, `subject_fact.rs:525-527`). A dotted member of the identity is therefore never bound. The specification is right, and synthesis cannot witness it.
3. Existing idiom: none that witnesses it. Control `probe-463/c/` (scalar `String` identity, `at == input.place, mode == sealed`, `ess/22`, 0.53.0): `stored` synthesizes, but `sealed` is refused ("its own row set is False", `c.synth.out`). The requester's own workaround copies a member into a plain field. Synthesis then cannot tell that the copy is the identity, so its decoys create the selected identity a second time, each step expecting the creating outcome (the issue's text). Duplicate creation is only caught where the creator declares `existing_instance:` (`row_set.rs:571-590`).
4. Fit: the change is synthesis only. It adds no key, and validate and the IR are unchanged.
   - (a) From `ess/23` (the `identity_selectors` gate), `scoped` also counts an equality whose row side is a dotted member of the identity, where that member resolves to `String`/`Uuid` through `Optional` and newtypes. Member typing reuses #428's `row_type` (`synthesize/identity.rs:69-97`).
   - (b) The three one-segment identity gates read any path rooted at the identity, where the row's identity is a literal struct. A member is projected as #428's `project` does (`identity.rs:99-106`). An instance- or observed-valued struct identity has no member projection, so the scenario is refused with that reason named.
   - (c) Decoys. `arrange_rows` arranges a decoy per conjunct before the selected rows (`row_set.rs:951-992`). Where the selector pins every member of the identity, a decoy that refutes a non-identity conjunct (`mode`) carries the selected row's identity: it is that same record. So on a branch that needs a selected row, that decoy is not arranged and counts above one are not tried. It is arranged on the count-0 branch (`stored`), and that branch is what catches a target that ignores `mode`.
   - (d) From `ess/23`, where the selector reads the identity, the walk refuses an arrangement that creates a literal identity a row already carries, whatever the creator declares. This extends `row_set.rs:571-590`.
   - Composes with: `exists`/`count`/`forall` and `copied` reads (the arrangement is unchanged); `subject.<f>` operands (`named`, :193-200); a partial selector such as `at.region` only, which selects many rows under distinct identities through the ordinary decoys.
   - Siblings: #428 made the same member reading for view parameters. The `where:` of `instances:`/`affects:` arranges through the same `misses` and `row_truth_with`, so it gains (b) with the same gate, or the unit says why it does not.
   - Targets: the interpreter already binds the whole key as the identity (`interpret/execute/set_effects.rs:223-234`); that member reads resolve through it is inferred, and the acceptance runs decide it. Generated Rust and Go keep row-set commands hand-written, and Entity Runtime refuses `RowSetUnsupported` (`predicates.md:1305-1306`), so neither changes. Diff and generated docs are unchanged because the source is unchanged.
   - Incidental: the refusal's help line ("give the field a type that has a finite value") does not fit a row-set gap (`a.synth.out`). This goes in the PR, not a new issue.
5. Second adopter: a parcel locker keyed `{bank, door}` that refuses a deposit while that door is out of service. Also a settings store keyed `{tenant, key}` that refuses a write to a key frozen for that tenant. Both are composite natural keys, not one adopter's convention.
6. Cost: no source format (it rides `ess/23`, which #429 introduced, `website/docs/reference/spec-versions.md:65`), no suite format, no keyword, no diagnostic. The scenarios use the existing row-set steps. `ess/22` documents keep their bytes, because every new reading sits behind `identity_selectors`. The cost is one sentence on the `ess/23` row, one clause at `predicates.md:1303-1305`, and the #429 note's "not in this cut" line, which narrows (`docs/design/identity-changing-updates.md:103-108, :121-125`). Measure with before/after synthesis of every repository model.
7. Alternatives:
   - (a) Change nothing and document a single-field copy of the identity. Synthesis still re-creates identities, as in Q3, so this fails.
   - (b) The requester's second form, `==` over two structs. It changes predicate typing in every format and site (`ess-domain/src/expression.rs:1849-1855`), plus every evaluator (interpreter, Go and TypeScript runners, generated filters). That is new surface for a fact the member conjunction already states. Declined: `at == input.place` stays ESS-COMMAND-002, and the idiom is the conjunction over its members.
   - (c) Chosen: the requester's first form, implemented as synthesis reading the member paths it already validates.

## Decisions
accept, redesigned. From `ess/23` only, synthesis treats a top-level equality between a `String`/`Uuid` member of a struct identity and the input or subject as scoping the selector. It binds the identity's members from the literal identity, and it arranges decoys under other identities. A non-identity conjunct's decoy goes only on the empty branch when every member is pinned. It never re-creates a carried literal identity. Whole-struct `==` is declined, and the idiom is the member conjunction. No new source or suite format: this rides `ess/23` (#429, already at the integration head), and `ess-conformance` is unchanged. Out of scope: lifting #429's refusal of a struct-identity re-key (`identity-changing-updates.md:36-38, :75`). This change makes that possible, but it gets its own request.

## Acceptance
- struct_identity_member_selector_witnesses_both_branches: probe `a` at `ess/23` synthesizes `Store/outcome/sealed` and `Store/outcome/stored` with no ESS-SYNTH-001, and the interpreter passes both.
- struct_identity_decoys_carry_other_identities: every decoy for `at.region`/`at.shelf` is created under an identity other than the sent one, and no step creates an identity a row already carries.
- pinned_identity_nonkey_decoy_on_empty_branch: `stored` arranges a `mode`-differing record under the sent identity, and a target ignoring `mode == sealed` fails it.
- ignored_identity_member_mutant_fails: a target that drops `at.region` (or `at.shelf`) from the selector fails a decisive scenario.
- partial_member_selector_selects_many: a selector on `at.region` alone witnesses a two-row count under two distinct identities.
- scalar_identity_with_extra_conjunct_synthesizes: probe `c` at `ess/23` synthesizes `sealed`.
- instance_struct_identity_member_refused_by_name: a selector over a member of an observed struct identity is refused, naming the missing member projection.
- ess22_struct_identity_selector_keeps_bytes: probes `a` and `c` at `ess/22` keep their 0.53.0 refusals byte for byte, and `adv_an_ess22_identity_selector_keeps_its_suite_bytes` stays green.
- whole_struct_equality_stays_refused: `at == input.place` at `ess/23` stays ESS-COMMAND-002.
- repository_model_suites_unchanged: before/after synthesis of every repository model differs only where a refusal became a scenario.

## Scope
- crates/verify/ess-conformance/src/synthesize/row_set.rs  cited — `scoped` :179-213; `Reading::truth` :463-467; decoys :951-992; duplicate creation :571-590
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs  cited — identity bind :484-491; `identity_selectors` :525-527
- crates/verify/ess-conformance/src/synthesize.rs  cited — `steered_identity` :9832-9842
- crates/verify/ess-conformance/src/synthesize/identity.rs  cited — `row_type` :69-97 and `project` :99-106, shared rather than copied
- crates/verify/ess-conformance/tests/adversary_w1_1_pass1.rs  cited — the `ess/22` byte guard :417-441 must stay green
- crates/verify/ess-conformance/tests/row_set_struct_identity.rs  inferred — new cases
- crates/verify/ess-conformance/tests/fixtures/row-set-struct-identity.yaml  inferred — brand-free fixture
- docs/design/identity-changing-updates.md  cited — the identity-selector paragraph :103-108 and "not in this cut" :121-125
- website/docs/reference/predicates.md  cited — the scoping sentence :1303-1305
- crates/specify/ess-domain/src/system.rs  cited — `FORMAT_HISTORY` row 23 gains the sentence (#460: the `ess/23` table row is generated from it)
- website/docs/reference/spec-versions.md  cited — regenerated with `cargo xtask format-history`, never hand-edited
