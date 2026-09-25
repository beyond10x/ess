---
format: aep.planning-md/2
id: review-result:compiler-accounting-source-claim-pass-2
kind: review-result
status: active
title: Compiler accounting final source and claim review pass 2
relations:
- reviews: task:consumer-accounting-compiler-core
revision: 1
---
approve

# S4 source-to-claim admission review, pass 2 of 2

Review unit: the final original S4 source and all three 686-row `NON_AUTHORITY` ledgers in the retained managed tree at HEAD `f1af8280338b97d862a6c474ec50f78d5157d71c`. This is the final bounded pass over the whole original S4 unit. No Cargo command, source change, planning change, authority mutation, or new requirement was used.

The pinned correction result is exact: `s4-adoption/correction-1/correction-result.md` has SHA-256 `8b78a0d7f453f26af0cf66cb6aec336fe6e8a7d8a6e7a04d91ecc28d992f07b1`. The final source, ledger, reconciliation, counterpart, and restored `resolve.rs` hashes all match that result. The ledgers name the same 686 distinct model identities and the same case-file hashes as the final source.

## Prior finding dispositions

1. **Aggregate disposition conflict — resolved.** Each profile contains exactly 679 `Supported`, 2 `UnsupportedAtThisEntrypoint`, and 5 `AggregateClosureCandidate` rows. The five candidates are exactly `rust:ess_domain::command::RawOutcome`, `wire:RawSpecFile#`, `wire:RawSpecFile#/definitions`, `wire:RawSpecFile#/definitions/RawOutcome`, and `wire:RawSpecFile#/definitions/RawOutcome/properties`. The reconciliation contains the same five identities for each of the three profiles, 15 rows total, all `ShapeDelta`, with zero aggregate shape mismatches.

2. **Incomplete cross-family parent attribution — resolved.** All nine changed parents carry the complete required family unions in all profiles. `ResolvedBinding` and `BindingSpec` cite periodic, selection, and external-reference observations; `ResolvedMappingValue` and `MappingSource` cite periodic, selection, and accessor observations; `RawBindingSpec` and its two enclosing wire parents cite selection and external-reference observations; the two mapping wire parents additionally cite periodic and accessor observations. The resolution cases exercise these descendants at `crates/specify/ess-compiler/tests/evolution_compiler_resolution_accounting.rs:260`, `:476`, `:685`, `:841`, and the external-reference case; the private-parts cases exercise the same unions at `crates/specify/ess-compiler/tests/evolution_private_parts_transport_accounting.rs:258`, `:326`, `:521`, and `:729`. The semantic-reference ledger uses the matching family-specific no-effect cases as the union rather than borrowing internal-value evidence.

3. **Exact-member assertion gaps — resolved.** Periodic context/read fields and host mapping values are compared as complete values including resolved type trees (`evolution_compiler_resolution_accounting.rs:298-328`; `evolution_private_parts_transport_accounting.rs:296-320`). Selection evidence requires whole domain-plan equality, a complete minted type table, exact selector/projection/type mapping values, exact predicates and computed reads, and an accessor-backed input variant (`evolution_compiler_resolution_accounting.rs:459-566`; `evolution_private_parts_transport_accounting.rs:326-459`). Accessor evidence requires whole `AccessorPlan` equality and complete handle tables, and finite fixtures cover `Missing`, `Newtype`, `Optional`, and `Union` operations plus `Enum`, `Optional`, and `Sequence` shapes (`evolution_compiler_resolution_accounting.rs:685-938`; `evolution_private_parts_transport_accounting.rs:521-725`). Subject-state evidence pins the exact predicate `incoming == Ringing` in both profiles (`evolution_compiler_resolution_accounting.rs:1525-1555`; `evolution_private_parts_transport_accounting.rs:1052-1104`). These are whole-value and exact-member assertions, not container-presence substitutes.

   The five compile-valid shape-preserving controls reach these corrected comparisons: periodic type, selector, selection projection, accessor projection, and predicate mutations each exit 101 with the wrong value printed against the exact expected value; every corresponding restored run exits 0. The final `resolve.rs` SHA-256 is `49d7bd5fd5b896d10d00372f44968395a943ae4adab7f33854a4be0ef904e3fd`. The controls test the corrected values directly and introduce no unrelated fault requirement.

4. **Semantic-reference attribution limit — resolved.** `unchanged_vocabulary` requires different authored input, different canonical IR bytes, exact vocabulary equality, and identical successful resolution for every reference (`crates/specify/ess-compiler/tests/evolution_semantic_reference_accounting.rs:346-373`). Every supported row cites only its matching `the_vocabulary_is_unchanged_by_*` family case as direct evidence. The five generic vocabulary cases and absent-reference case occur only in `control_cases`; supported rows have empty `refusal_cases`, so `a_move_a_lifecycle_could_not_declare_is_refused_by_name` is no longer row-level evidence.

## Retained limits

The two graph identities remain `UnsupportedAtThisEntrypoint` in each profile: six S4 tuples total. `s3-downstream-counterparts.json` has exactly 54 S4 counterparts and defers only those two graph models to S5. No S5 result qualifies an S4 cell. The five aggregate identities per profile remain aggregate-only candidates. The compiler-local private diagnostic remains non-authority evidence cited by no claim.

What I read: the complete final-pass brief, adversary charter, critic rubric, original S4 brief and `READY.md`, immutable first report and reviewer-authored AEP serialization correction, correction brief/result, final source for all three profiles, all three complete ledgers, reconciliation and counterpart files, and the five red/restored control records. Read-only `sha256sum`, `git status`, `git rev-parse`, `rg`, `sed`, `nl`, `jq`, `comm`, and `wc` checks were used.

What I could not establish: native authority and the final full matrix are root-owned and were not claimed here. The supplied final package, Clippy, formatting, case-count, and control exits were inspected rather than rerun, as the final-pass brief requires. No source-to-claim defect remains from the four original findings, and no new finding was found.

```findings
[]
```
