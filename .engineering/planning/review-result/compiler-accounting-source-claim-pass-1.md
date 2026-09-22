---
format: aep.planning-md/1
id: review-result:compiler-accounting-source-claim-pass-1
kind: review-result
status: active
title: Compiler accounting source and claim review pass 1
relations:
- reviews: task:consumer-accounting-compiler-core
revision: 1
---
# S4 source-to-claim admission review, pass 1 of 2

Reviewer-authored serialization correction. The substantive review is unchanged from original SHA-256 `f511aae0e46da8f90e1d4fa5ad284ff9fcaa0522ca691008da4655acec3b58a4`; this copy moves the original detailed finding payload to ordinary YAML and adds an AEP-compatible findings block.

Verdict: **needs-revision**.

Review unit: original S4 source and three 686-row `NON_AUTHORITY` ledgers. Read-only examination; no Cargo, source changes, planning changes, or new requirements.

The reviewed source hashes match `s4-adoption/adopted-source.sha256`:

- `ir.rs`: `5e0b164e…2877`
- private unit diagnostic: `c676a0b4…29a8`
- compiler resolution: `93c32284…2db`
- private parts transport: `6126ce12…68a`
- semantic references: `40e10a6d…31d`

The reported green executions and 19 causal controls are accepted as given. They establish that the cases execute and that constructor parts reach the IR. They do not by themselves establish every descendant value attributed by the ledgers.

## Findings

### 1. Three accepted aggregate identities are incorrectly marked `Supported` in all profiles

The accepted aggregate design names five exact identities. Each S4 ledger marks only two as `AggregateClosureCandidate`; these three are instead attributed to the subject-state family as `Supported`:

| Coordinate | Model |
|---|---|
| C21 | `rust:ess_domain::command::RawOutcome` |
| C55 | `wire:RawSpecFile#/definitions/RawOutcome` |
| C56 | `wire:RawSpecFile#/definitions/RawOutcome/properties` |

This affects nine mappings: the three models under `compiler-resolution`, `semantic-references`, and `compiler-private-parts`.

The cases only observe the subject-state addition. They do not establish the complete changed parent shapes, which also contain payload-source changes. No named full-object equality covers those parents. The generated reconciliation file already treats all three as `ShapeDelta`, making its aggregate classification inconsistent with the profile ledgers.

Required correction: mark all five accepted aggregate identities `AggregateClosureCandidate` in every ledger. The resulting per-profile counts should be:

- `Supported`: 679
- `UnsupportedAtThisEntrypoint`: 2
- `AggregateClosureCandidate`: 5

### 2. Nine changed parent models are attributed to one family although multiple families changed them

These changed-shape parents cannot be established by the single case family currently attached:

- `rust:ess_compiler::ir::ResolvedBinding`
- `rust:ess_compiler::ir::ResolvedMappingValue`
- `rust:ess_domain::binding::BindingSpec`
- `rust:ess_domain::binding::MappingSource`
- `rust:ess_domain::binding::RawBindingSpec`
- `wire:RawSpecFile#/definitions/RawBindingSpec`
- `wire:RawSpecFile#/definitions/RawBindingSpec/properties`
- `wire:RawSpecFile#/definitions/RawBindingSpec/properties/mapping`
- `wire:RawSpecFile#/definitions/RawBindingSpec/properties/mapping/additionalProperties`

The exact missing unions are:

- `ResolvedBinding` and `BindingSpec`: periodic, selection, and external-reference cases.
- `ResolvedMappingValue` and `MappingSource`: periodic host values, selection values, and event-accessor values.
- `RawBindingSpec` and its four wire parents: selection plus external references; the mapping parents additionally need periodic host and event-accessor mapping cases.

This affects 27 mappings across the three profiles. For semantic references, the corresponding per-family no-effect cases may form the union because `EssSemanticRef` reads construct identities rather than these internal values.

### 3. Exact-value evidence is missing for a finite set of resolution/private-parts rows

The same gaps occur in `compiler-resolution` and `compiler-private-parts` unless noted.

Periodic:

- `rust:ess_compiler::ir::ResolvedPeriodic/field/context`
- `rust:ess_compiler::ir::ResolvedPeriodic/field/read`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/HostContext/field/type_ref`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/HostRead/field/type_ref`

The resolution assertions at lines 180–194 and private assertions at lines 182–201 check names and variants but omit the resolved type references.

Selection:

- `rust:ess_compiler::ir::ResolvedBinding/field/selection`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/Selection`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/Selection/field/projection`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/Selection/field/selector`
- `rust:ess_compiler::ir::ResolvedMappingValue/variant/Selection/field/type_ref`
- `rust:ess_compiler::ir::ResolvedSelectionPlan`
- `rust:ess_compiler::ir::ResolvedSelectionPlan/field/plan`
- `rust:ess_compiler::ir::ResolvedSelectionPlan/field/types`
- `rust:ess_domain::binding::MappingSource/variant/Selection/field/path`
- `rust:ess_domain::binding::MappingSource/variant/Selection/field/selection`
- `rust:ess_domain::selection::First/field/predicate`
- `rust:ess_domain::selection::InputSource/variant/Accessor`
- `rust:ess_domain::selection::InputSource/variant/Accessor/field/plan`
- `rust:ess_domain::selection::SelectionOperation/variant/First/field/predicate`
- `rust:ess_domain::selection::SelectionOperation/variant/First/field/reads`

At resolution lines 355–379 and private lines 273–299, the selector is only checked with `is_number`, projection with non-null, result type by outer `kind`, and type table by key presence. `InputSource::Accessor`, first predicates, and computed reads are not observed.

Accessor:

- `rust:ess_compiler::ir::ResolvedMappingValue/variant/EventAccessor/field/types`
- `rust:ess_domain::accessor::Operation/variant/Missing`
- `rust:ess_domain::accessor::Operation/variant/Newtype`
- `rust:ess_domain::accessor::Operation/variant/Newtype/field/next`
- `rust:ess_domain::accessor::Operation/variant/Optional/field/next`
- `rust:ess_domain::accessor::Operation/variant/Union`
- `rust:ess_domain::accessor::Operation/variant/Union/field/tag`
- `rust:ess_domain::accessor::Operation/variant/Union/field/variants`
- `rust:ess_domain::accessor::ProjectionPlan`
- `rust:ess_domain::accessor::ProjectionPlan/field/0`
- `rust:ess_domain::accessor::ValueShape/variant/Enum`
- `rust:ess_domain::accessor::ValueShape/variant/Enum/field/variants`
- `rust:ess_domain::accessor::ValueShape/variant/Optional/field/of`
- `rust:ess_domain::accessor::ValueShape/variant/Sequence`

The private-parts profile additionally lacks any occurrence of:

- `rust:ess_domain::accessor::Operation/variant/Optional`
- `rust:ess_domain::accessor::ValueShape/variant/Optional`

The resolution accessor cases exercise Field, Leaf, and the existence of Optional, while the private case exercises only Field and Leaf. The type map is checked only for a key. Missing, Newtype, Union, Enum, Sequence, and ProjectionPlan are not observed by the cases attributed to them.

Subject state:

- `rust:ess_compiler::ir::ResolvedCondition/variant/SubjectState/field/predicate`
- `rust:ess_domain::command::OutcomeCondition/variant/SubjectState/field/predicate`

Resolution lines 1215–1222 and private lines 778–785 establish non-null and equality to another unresolved predicate. They do not establish the exact predicate. A compiler returning the same wrong predicate for both branches would pass.

A bounded correction can use named whole-object equality for the periodic fields, selection plan/mapping entries, type maps, and predicate. Additional existing-route fixtures are needed only for the accessor and selection variants currently absent. A measurable fault is to substitute another valid selector index, projection plan, type handle, or predicate while preserving number/non-null/container shape; the present assertions remain green.

### 4. Semantic-reference support is valid as no-effect evidence, but its row attribution is overstated

The nine `the_vocabulary_is_unchanged_by_*` cases are legitimate model-family no-effect observations. Each changes authored input, proves compiled IR bytes changed, then proves exact reference-set equality and identical resolution. `EssIr::resolves` reads construct names, outcome names, and transition names; it does not inspect these internal fields.

The five generic vocabulary cases and `a_move_a_lifecycle_could_not_declare_is_refused_by_name` are not tuple-specific evidence for all 682 rows. The transition-name refusal is unrelated to every admitted S4 model. For the 679 non-aggregate supported rows after finding 1:

- keep the appropriate `the_vocabulary_is_unchanged_by_*` case as direct row evidence;
- retain the generic vocabulary and absent-reference cases as profile controls;
- remove the transition-name refusal from row-level `refusal_cases`.

This is attribution cleanup, not a loss of the legitimate no-effect claim.

## Attribution limits that remain accepted

- Exactly two graph identities remain unsupported at each S4 entrypoint:
  `rust:ess_compiler::graph::DependencyRelation` and its `HostedBy` variant.
- All five accepted aggregate identities remain aggregate-only parent provenance.
- S5 evidence does not qualify any S4 profile cell.
- The compiler-local private diagnostic is useful but is not a native receipt and is cited by no claim.
- `EssIrParts::naming` carries no obligation in this set and creates no prerequisite.
- Constructor-part mutations prove transport through `from_parts`; they do not establish unasserted descendant values.

```yaml
- category: aggregate-disposition-conflict
  severity: blocker
  verdict: CONFIRMED
  profiles: [compiler-resolution, semantic-references, compiler-private-parts]
  models:
    - rust:ess_domain::command::RawOutcome
    - wire:RawSpecFile#/definitions/RawOutcome
    - wire:RawSpecFile#/definitions/RawOutcome/properties
  affected_mappings: 9
  correction: classify these as AggregateClosureCandidate; each ledger becomes 679 Supported, 2 UnsupportedAtThisEntrypoint, 5 AggregateClosureCandidate

- category: incomplete-cross-family-parent-attribution
  severity: blocker
  verdict: CONFIRMED
  profiles: [compiler-resolution, semantic-references, compiler-private-parts]
  models:
    - rust:ess_compiler::ir::ResolvedBinding
    - rust:ess_compiler::ir::ResolvedMappingValue
    - rust:ess_domain::binding::BindingSpec
    - rust:ess_domain::binding::MappingSource
    - rust:ess_domain::binding::RawBindingSpec
    - wire:RawSpecFile#/definitions/RawBindingSpec
    - wire:RawSpecFile#/definitions/RawBindingSpec/properties
    - wire:RawSpecFile#/definitions/RawBindingSpec/properties/mapping
    - wire:RawSpecFile#/definitions/RawBindingSpec/properties/mapping/additionalProperties
  affected_mappings: 27

- category: exact-member-assertion-gap
  severity: blocker
  verdict: CONFIRMED
  profiles:
    compiler-resolution:
      affected_leaf_mappings: 35
      cases:
        - periodic_host_contract_resolves_every_declared_profile_word_and_host_field
        - the_selection_plan_resolves_every_input_selector_and_projection
        - a_bounded_event_accessor_resolves_its_typed_plan_and_minted_types
        - an_optional_accessor_step_reports_that_traversal_may_stop_early
        - a_subject_state_guard_resolves_with_its_state_and_its_input_predicate
    compiler-private-parts:
      affected_leaf_mappings: 37
      cases:
        - the_bindings_part_carries_every_periodic_member_value
        - the_bindings_part_carries_every_selection_member_value
        - the_bindings_part_carries_every_accessor_member_value
        - the_commands_part_carries_every_subject_state_condition_member_value
  message: presence, primitive type, outer variant, equality to another opaque value, and map-key checks do not establish the exact member values claimed

- category: semantic-reference-attribution-limit
  severity: major
  verdict: CONFIRMED
  profile: semantic-references
  affected_rows: all remaining Supported rows after aggregate correction
  direct_evidence: the matching the_vocabulary_is_unchanged_by_* case
  controls_only:
    - all_twelve_reference_kinds_are_minted_and_every_one_resolves
    - a_name_minted_from_a_handle_is_the_same_name_a_document_spells
    - a_name_resolves_against_any_compilation_of_the_same_specification
    - every_reference_parses_back_from_the_spelling_it_prints
    - a_handle_is_not_an_external_identity
    - a_reference_naming_a_construct_the_compilation_lacks_does_not_resolve
  unrelated_refusal: a_move_a_lifecycle_could_not_declare_is_refused_by_name
```

```findings
- file: ~/beyond10x/.ess-evolution/waves/0010-opus-accounting/s4/non-authority-compiler-resolution.json
  line: 1
  category: aggregate-disposition-conflict
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Three accepted aggregate identities are marked Supported in all three S4 profile ledgers, affecting nine mappings; classify them as AggregateClosureCandidate so each ledger has 679 Supported, 2 UnsupportedAtThisEntrypoint, and 5 AggregateClosureCandidate rows.
- file: ~/beyond10x/.ess-evolution/waves/0010-opus-accounting/s4/non-authority-compiler-resolution.json
  line: 1
  category: incomplete-cross-family-parent-attribution
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Nine changed parent models are attributed to only one family although their shapes span periodic, selection, external-reference, and event-accessor behavior, leaving 27 mappings without the required case-family unions.
- file: crates/specify/ess-compiler/tests/evolution_compiler_resolution_accounting.rs
  line: 180
  category: exact-member-assertion-gap
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: Resolution and private-parts cases use presence, primitive-type, outer-variant, opaque-equality, and map-key checks for the detailed periodic, selection, accessor, and subject-state mappings enumerated in the preceding YAML, so those exact member values are not established.
- file: ~/beyond10x/.ess-evolution/waves/0010-opus-accounting/s4/non-authority-semantic-references.json
  line: 1
  category: semantic-reference-attribution-limit
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: The family-specific unchanged-vocabulary cases legitimately establish no-effect behavior, while the generic vocabulary cases are controls and the transition-name refusal is unrelated to the supported S4 rows; narrow row-level attribution without removing the no-effect claims.
```

No report file was written because the assignment was explicitly read-only.
