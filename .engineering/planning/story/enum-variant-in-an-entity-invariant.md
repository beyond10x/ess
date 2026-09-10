---
format: aep.planning-md/1
id: story:enum-variant-in-an-entity-invariant
kind: story
status: active
title: An entity invariant may name an enum variant that does not exist, and validate accepts it
relations:
- informed_by: story:review-expression-typechecking
- informed_by: initiative:ess-evolution
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/billing.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/enum_invariants.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/typed_diagnostics.rs
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 14
---
## What is wrong

An entity invariant may compare an enum-typed field to a variant the enum does not declare, and
`ess specify validate` accepts it.

Reproduced 2026-09-04 against `ess 0.15.0`, on a copy of the ACD model in `acd/specs`:

```console
$ sed -i '242s/line_status == selected/line_status == NOT_A_VARIANT_AT_ALL/' domains/routing.yaml
$ ess specify validate --path .
acd v3 — 4 file(s), valid
$ echo $?
0
```

`line_status` is `acd.routing.LineStatus`, an enum with nine declared variants.
`NOT_A_VARIANT_AT_ALL` is not one of them.

**View filters are checked.** The same comparison in a view's `filter:` is refused. So the machinery
exists and the entity-invariant path does not use it.

## Why it is worth fixing rather than noting

The invariants most worth lifting into a model are the ones about lifecycle state — *an agent in an
active-call status is not available*, *Selected implies an agent*. Every one of them compares a field
to a variant, and every one is a place a typo turns into an invariant that is never violated because
nothing can match it. It reads as a passing constraint and it constrains nothing.

This was found while modelling ACD's routing core, where the whole point of the exercise was to lift
invariants out of Go regression tests named after outages. An invariant that silently checks nothing
is worse than the test it replaced.

## Acceptance

1. An entity invariant naming an undeclared enum variant is refused by `ess specify validate`, with
   a stable refusal code, the file it was read from, and the variants that are declared.
2. The refusal names the enum, not only the field, so the fix is one lookup away.
3. A test covers it — and a second covers the case that already works, the same comparison in a view
   filter, so a future refactor cannot lose one while keeping the other.

## What this does not cover

Only equality against a variant. Whether every other predicate form over an enum resolves its
right-hand side is not established here and should be checked while fixing this, not assumed.


## Integration Provenance

Reconciled through AEP from wt-e46db550dce9 at original revision 1 and status draft. Source artifact SHA-256: 3a90e2d966fdab5e49f7f4655b3fbe80d294a78d181c9c31d0db8e7039850cf9. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.

## Scope

Derived 2026-09-06 by `aep-drive:story-scoper` against clean ESS `dcb84be861d2f906b3dd95254f03701cb264faa2` — cited.

- **Primary surface:** `crates/specify/ess-domain` — cited; entity invariant admission and the existing view enum-literal checker both live here.
- **Entity implementation and unit tests:** `crates/specify/ess-domain/src/entity.rs` — cited; `EntitySpec::validate` at line 826 checks fact paths but not their compared literals, and the existing invariant regressions are inline.
- **Shared checking and view regression:** `crates/specify/ess-domain/src/view.rs` — inferred; reuse or extract its existing `validate_filter_values`, `enumeration`, and `compared_values` machinery so entity and view equality checks share enum membership behavior, while retaining the view's existing negative and positive cases.
- **Diagnostic integration tests:** `crates/specify/ess-compiler/tests/billing.rs` — inferred; extend its existing assembly-to-diagnostic test pattern to assert the entity refusal code, source filename, enum name and declared variants.
- **Symbols:** `EntitySpec::validate`, `observable_fields`, `state_type`, `ViewSpec::validate_filter_values`, `enumeration`, `compared_values`, and `collect_compared_values` — cited; these provide the entity environment, enum registry lookup and existing predicate traversal.
- **Bounded change:** reject equality between an enum-typed entity field and an undeclared literal; preserve declared literals, optional enum fields, lifecycle state access and the equivalent view-filter behavior — inferred; these are focused controls around the demonstrated defect.
- **Diagnostics:** the existing `UndeclaredReference` bridge can yield `ESS-ENTITY-001` for entity invariants and `ESS-VIEW-001` for view filters, retaining the domain message and hint — cited; no new refusal code or persisted format is established as necessary.
- **Documents:** none independently required by this imported defect story — inferred; the related expression-typechecking design must govern any broader resolver extraction.
- **Confidence:** high for the validation defect and existing reuse surface — cited; source-attribution completeness and the eventual shared helper placement remain unresolved.
- **Would collide with:** entity/view predicate validation and their inline tests, compiler billing diagnostic tests, and broader expression-typechecking work in ess-domain — inferred; this story remains outside the immediate wave.

## ESS evolution continuation

This existing validation defect is selected under initiative:ess-evolution before the migration's semantic crosswalk and ER lowering. Verify the current shared expression checker first: the older story's view-specific helper names may have moved. Preserve existing predicate semantics and reuse the common resolver; do not add a second enum checker or change persisted formats. Verify entity and view diagnostics plus valid enums, optional fields and lifecycle state using focused domain/compiler tests. No full gate is authorized for this continuation.

## Current reproduction and selected correction

The original semantic admission defect is already fixed by the shared expression checker introduced in f03ecdaf. The new owner-level regression confirms that valid optional enums, membership predicates and lifecycle state compile, but the first negative case fails the source-file assertion: the diagnostic span is <document> instead of domains/work.yaml. The fixture declares sample.work.Work and sample.work.WorkView. Locator treats the former declaration needle as a substring of both names and loses its otherwise unique source.

Correct only declaration-needle token boundaries in the existing Locator. Preserve its uniqueness requirement across all input files and its fallback for genuinely ambiguous locations. Keep the common predicate checker unchanged. Test entity and view diagnostics end to end, including vocabulary, source, membership and optional/lifecycle cases; separately prove prefix names do not count as exact declarations and duplicate exact declarations remain unresolved. No new persisted format or enum semantics is introduced.

## Implementation and evidence

Semantic enum admission already exists in the shared checker: it validates both sides of comparisons, membership lists, optional/nested types and bound paths. No duplicate enum walker was added. The remaining source-attribution defect is fixed in Locator: name/id/component declaration needles reject matches that continue with a name character. Other substring needles and the cross-file uniqueness requirement retain their behavior. No model, IR or refusal-code format changed.

The initial owner-level regression failed because the entity diagnostic named <document> rather than domains/work.yaml. After the fix, three focused tests pass: ten negative owner/predicate combinations retain ESS-ENTITY-001 or ESS-VIEW-001, source and declared enum vocabulary; five valid combinations compile for both owners; prefix-only declarations never impersonate an absent exact name, and duplicate exact declarations remain unresolved. Existing diagnostic fixtures now pin File and Order to their actual declaration lines despite FileTwo/Filed and OrderId/OrderPlaced. The adversarial ambiguity test still requires every diagnostic to remain unresolved when exact declarations occur in two source files. No fixture source or semantic assertion was deleted.

Verification against this candidate: 52 compiler library, billing, enum and diagnostic-adversary tests passed. The typed-diagnostics target initially exposed another old prefix-collision location assertion, which was corrected to the actual entity declaration; its final eight tests passed. Thus 60 focused tests passed across the affected targets, with no ignored cases. Strict ess-compiler all-target Clippy passed. Rust 1.85 compiled the compiler and new enum integration target; two existing const-helper dead-code warnings remain on that compiler. Formatting and git diff --check passed. No full workspace/consumer/persistence gate was run.

Evidence: local-evidence:ess-evolution-20260910/ess-enum-owner-diagnostics.log (initial source-attribution failure); ess-enum-owner-diagnostics-fixed.log (three passing new cases); ess-enum-diagnostics-final.log (52 passes and the old nested-location expectation failure); ess-enum-typed-diagnostics-final.log (all eight corrected diagnostic cases pass); ess-enum-diagnostics-clippy-final.log; ess-enum-diagnostics-msrv.log. The feature remains a source candidate until integration. This closes the code gap described above, not ER lowering or application adoption.
