//! The lowerable subset (beyond10x/ess#231): which ESS constructs lower to Entity Runtime, which
//! are refused, under which [`LoweringCode`], and what lowering each refused one would need.
//!
//! The published page `website/docs/reference/entity-runtime-lowering.md` is [`reference_page`]'s
//! rendering, and `tests/lowerable_subset.rs` holds it to that byte for byte. The same file holds
//! [`LoweringCode::ALL`] to the enum's declaration, every source-construct code to at least one
//! refused row, every construct name a diagnostic carries to a row, and every row to tests that
//! exist. A code added without a row, or a row whose evidence is gone, fails there.

use std::fmt::Write as _;

use crate::{LoweringCode, ENTITY_RUNTIME_REVISION};

/// Where the rendered page lives, relative to the repository root.
pub const REFERENCE_PAGE: &str = "website/docs/reference/entity-runtime-lowering.md";

/// One ESS construct and what lowering does with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Construct {
    /// The name a diagnostic carries in [`LoweringDiagnostic::construct`](crate::LoweringDiagnostic).
    pub name: &'static str,
    /// What lowering does with it.
    pub lowering: Lowering,
    /// The tests that show it, each `<path relative to the crate>::<test function>`.
    pub evidence: &'static [&'static str],
}

/// What lowering does with a construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lowering {
    /// It lowers, to the Entity Runtime form named.
    Lowered {
        /// The Entity Runtime form.
        into: &'static str,
    },
    /// It is refused under `code`.
    Refused {
        /// The refusal's code.
        code: LoweringCode,
        /// What lowering it would need.
        needs: Needs,
    },
}

/// What a refused construct would need before it could lower.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Needs {
    /// A capability entity-core does not have. Lowering waits for it there; ESS cannot add it.
    EntityCore(&'static str),
    /// Only ESS-side work: the host supplies the value as a bound argument, as it already supplies
    /// generated values and external evidence. Not done yet.
    HostValue(&'static str),
    /// No Entity Runtime definition can state it, and none is planned.
    NoTarget(&'static str),
}

pub(crate) const STATELESS: &str = "a command with no entity subject";
pub(crate) const SPANS: &str = "one command over several entities";
pub(crate) const MIXED: &str = "a creation beside an update in one command";
pub(crate) const SECOND_CREATION: &str = "a second command creating one entity";
pub(crate) const AMBIGUOUS: &str = "an instance binding with no single source";
pub(crate) const WITHOUT_SUBJECT: &str = "an accepting outcome with no subject";
pub(crate) const NULLABLE: &str = "`Optional` below a list or a map";
pub(crate) const RECURSIVE: &str = "a recursive type";
pub(crate) const OPTIONAL_UPDATE: &str = "an `Optional` value written by an update";
pub(crate) const IDENTITY_UPDATE: &str = "the identity in an update's `sets:`";
pub(crate) const LITERAL_SHAPE: &str = "a text literal for a structured type";
pub(crate) const CLEARED: &str = "`{cleared: true}` outside a creation's `sets:`";
pub(crate) const SILENT_PRESERVE: &str = "`preserves:` with no response";
pub(crate) const TEXT_ORDERING: &str = "`<`, `<=`, `>`, `>=` over text";
pub(crate) const ALPHABET: &str = "`alphabet:`";
pub(crate) const TEXT_COUNT: &str = "`.count` of a text";
pub(crate) const SUBJECT_VALUE: &str = "`{subject: …}`";
pub(crate) const INCREMENT: &str = "`{increment: …}`";
pub(crate) const FALLBACK: &str = "`{input: …, else: …}`";
pub(crate) const INPUT_PATH: &str = "a value read through an input path (`input.<path>`)";
pub(crate) const STRUCT: &str = "a struct of sources";
pub(crate) const RELATED_VALUE: &str = "`{related: …}`";
pub(crate) const CASE_FOLD: &str = "`equals_ignore_case`, `in_ignore_case`";
pub(crate) const UNKNOWN_REFUSAL: &str = "an `unknown_instance:` refusal";
pub(crate) const DELETES: &str = "`deletes:`";
pub(crate) const INTO: &str = "a creation `into:` a state";
pub(crate) const ACCEPTS_NOTHING: &str = "`accepts: nothing`";
pub(crate) const INPUT_ABSENT: &str = "`input_absent:`";
pub(crate) const NOW: &str = "`now` in a guard";
pub(crate) const EXISTENCE: &str =
    "a branch chosen by existence (`unknown_instance:` on a creation, `existing_instance:`)";
pub(crate) const CALLER: &str = "the caller (`{caller: …}`, `caller.<attribute>`)";
pub(crate) const SET_EFFECT: &str = "a set effect (`instances:`, `affects:`, `{count: changed}`)";
pub(crate) const RELATED_GUARD: &str = "`when_related:`";
pub(crate) const UNIT_VARIANT: &str = "a union variant with no payload (ess/22)";
pub(crate) const OFFSET: &str = "one constant offset (`upper == lower + 5`, `issued_at - 24h`)";
pub(crate) const DISTINCT: &str = "distinct list members (`distinct: {in, as, by}`)";
pub(crate) const UTF8_BYTES: &str = "the UTF-8 byte length of a text (`label.utf8_bytes`)";
pub(crate) const ROW_SET: &str =
    "a row set (`when_related: {entity, where, …}`, `{related: {entity, where, field}}`)";

/// The row a refused value expression is named by.
pub(crate) fn value_expression(value: &ess_compiler::ir::ResolvedPayloadValue) -> &'static str {
    use ess_compiler::ir::ResolvedPayloadValue as Source;
    match value {
        Source::SubjectField { .. } => SUBJECT_VALUE,
        Source::Increment { .. } => INCREMENT,
        Source::InputOrGenerated { .. } => FALLBACK,
        Source::InputField { field, .. } if ess_domain::command::input_path::is_path(field) => {
            INPUT_PATH
        }
        Source::Struct { .. } => STRUCT,
        Source::RelatedField { .. } => RELATED_VALUE,
        Source::RelatedSelection { .. } => ROW_SET,
        Source::ResponseField { .. }
        | Source::Generated
        | Source::InputField { .. }
        | Source::Literal { .. }
        | Source::Cleared
        | Source::CallerAttribute { .. }
        | Source::ChangedCount => LoweringCode::ValueExpressionUnsupported.construct(),
    }
}

const fn refused(
    name: &'static str,
    code: LoweringCode,
    needs: Needs,
    evidence: &'static [&'static str],
) -> Construct {
    Construct {
        name,
        lowering: Lowering::Refused { code, needs },
        evidence,
    }
}

const fn lowered(
    name: &'static str,
    into: &'static str,
    evidence: &'static [&'static str],
) -> Construct {
    Construct {
        name,
        lowering: Lowering::Lowered { into },
        evidence,
    }
}

/// Every construct the page lists, lowered ones first, each group in reading order.
pub const CONSTRUCTS: &[Construct] = &[
    lowered(
        "an entity: identity, fields, lifecycle",
        "an `EntityDefinition` with its schema, `identity` and `lifecycle`",
        &["tests/lowering.rs::billing_and_gatepass_lower_completely_deterministically_without_touching_source"],
    ),
    lowered(
        "primitive, `List`, `Map`, struct, enum, union and `Json` types; `Optional` members",
        "`FieldDefinition`s; an `Optional` member is not `required`",
        &[
            "tests/lowering.rs::complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment",
            "tests/lowering.rs::enum_fields_carry_each_variant_name_exactly_as_declared",
            "tests/lowering.rs::a_json_field_lowers_to_the_json_field_kind",
        ],
    ),
    lowered(
        "entity and type `invariants:`",
        "named entity rules, one per invariant and per layer of a newtype chain",
        &["tests/lowering.rs::invariants_at_two_layers_of_a_newtype_chain_lower_to_distinct_rule_names"],
    ),
    lowered(
        "`prefix:` on a newtype",
        "a `starts_with` rule",
        &["tests/lowering.rs::a_prefix_lowers_to_a_starts_with_rule"],
    ),
    lowered(
        "`relations:`",
        "`RelationDefinition`s, with existence, cardinality and ownership binding obligations",
        &["tests/adversary_relid_pass1.rs::an_identity_carried_reference_lowers_to_definitions_the_registry_accepts"],
    ),
    lowered(
        "`creates:`",
        "the entity's `create` entrypoint, its identity observed in an emitted event",
        &[
            "tests/lowering.rs::complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment",
            "tests/selected_creation_identity.rs::selected_creation_outcomes_keep_their_authored_identity_and_event_coordinate",
        ],
    ),
    lowered(
        "`updates:` and `moves:`",
        "a named operation; an unmapped field is a host fulfillment obligation",
        &[
            "tests/lowering.rs::pay_invoice_refuses_before_load_then_selects_exact_subject_before_fulfillment",
            "tests/lowering.rs::accepted_operation_action_algebra_is_enforced_after_branch_selection",
        ],
    ),
    lowered(
        "`preserves:` with a response",
        "an outcome with no effect that responds",
        &["tests/lowering.rs::subject_field_selection_and_a_responding_preserve_lower_and_decide_faithfully"],
    ),
    lowered(
        "`error:` outcomes",
        "`refuses`, with an error-payload binding obligation",
        &["tests/lowering.rs::pay_invoice_refuses_before_load_then_selects_exact_subject_before_fulfillment"],
    ),
    lowered(
        "`when:` over the input",
        "`when`, an input-guarded refusal ordered before the branches it overlaps",
        &[
            "tests/input_guard_overlap.rs::an_input_guarded_refusal_is_selected_before_the_accepting_branch_it_overlaps",
            "tests/input_guard_overlap.rs::the_first_declared_accepting_branch_answers_the_overlap",
        ],
    ),
    lowered(
        "`when_subject:` a state, a field, or a predicate over the stored row and `input.`",
        "`in_state`, or `when` over `$fields`, `$from_state` and the arguments",
        &[
            "tests/lowering.rs::subject_field_selection_and_a_responding_preserve_lower_and_decide_faithfully",
            "tests/lowering.rs::a_stored_field_predicate_on_a_refusal_lowers_to_the_row_fields",
            "tests/lowering.rs::an_input_operand_in_a_stored_field_predicate_lowers_to_the_arguments",
            "tests/lowering.rs::state_in_a_stored_field_predicate_lowers_to_the_held_state",
        ],
    ),
    lowered(
        "`when_state_changes:`",
        "membership of `$from_state`",
        &["tests/lowering.rs::state_change_selection_partitions_the_held_state_exactly"],
    ),
    lowered(
        "the wrong-state branch",
        "`wrong_state`, ordered after every guarded branch",
        &["tests/adversary_state_scoped_pass1.rs::a_state_reading_refusal_in_a_wrong_state_is_selected_before_wrong_state_at_runtime"],
    ),
    lowered(
        "`external:` and `external_when:`",
        "a guard over host-supplied evidence, with the input guard beside it",
        &["tests/lowering.rs::external_when_requires_both_input_eligibility_and_supplied_evidence"],
    ),
    lowered(
        "orderings over numbers and `Timestamp`s; string operators",
        "`compare`, `before`/`after`, `starts_with` and the other entity-core string conditions",
        &[
            "tests/lowering.rs::adv_a_numeric_stored_ordering_is_decided_by_the_lowered_runtime",
            "tests/lowering.rs::an_input_timestamp_ordering_lowers_to_entity_core_instant_operators",
            "tests/lowering.rs::string_operators_lower_to_conditions_the_runtime_decides_byte_for_byte",
            "tests/current_time_guard.rs::a_guard_against_a_fixed_instant_still_lowers",
        ],
    ),
    lowered(
        "`input.<field>`, a literal, `{response: …}`, `{generated: true}`, a declared conversion",
        "an argument template, an escaped literal, or a typed bound slot the host fills",
        &[
            "tests/lowering.rs::ordered_occurrences_exact_literals_and_shared_response_values_survive_lowering",
            "tests/lowering.rs::optional_creation_event_and_response_values_share_presence_without_defaults",
        ],
    ),
    lowered(
        "`{cleared: true}` in a creation's `sets:`",
        "the member left absent",
        &["tests/lowering.rs::a_creation_that_clears_an_optional_field_leaves_it_absent_without_a_host_slot"],
    ),
    lowered(
        "`emits:` and `response:`",
        "`emits` and `responds`, conditional members through `PresentArgument`",
        &["tests/lowering.rs::complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment"],
    ),
    // Refused: the constructs beyond10x/ess#231 names first, then the rest in code order.
    refused(
        NOW,
        LoweringCode::CurrentTimeUnsupported,
        Needs::HostValue("the request's clock reading as a bound argument; entity-core refuses `$now`"),
        &["tests/current_time_guard.rs::a_guard_reading_now_is_refused_by_name"],
    ),
    refused(
        RELATED_VALUE,
        LoweringCode::ValueExpressionUnsupported,
        Needs::HostValue("the related row's field, loaded by the host and bound as an argument"),
        &[
            "tests/related_values.rs::a_related_source_is_refused_by_name",
            "tests/lowerable_subset.rs::a_related_value_is_refused_under_its_own_construct",
        ],
    ),
    refused(
        UNKNOWN_REFUSAL,
        LoweringCode::OutcomeShapeUnsupported,
        Needs::HostValue("the host's own not-found answer, mapped to the declared refusal"),
        &["tests/outcome_shapes_adversary.rs::adversary_an_unknown_instance_branch_is_refused_by_name"],
    ),
    refused(
        EXISTENCE,
        LoweringCode::ExistenceSelectionUnsupported,
        Needs::HostValue("the host's own exists answer, selecting the declared branch"),
        &[
            "tests/upsert_by_existence.rs::create_or_update_is_refused_by_name",
            "tests/upsert_by_existence.rs::create_or_refuse_is_refused_by_name",
        ],
    ),
    refused(
        INCREMENT,
        LoweringCode::ValueExpressionUnsupported,
        Needs::EntityCore("arithmetic over a stored value"),
        &["tests/lowerable_subset.rs::each_value_expression_is_refused_under_its_own_construct"],
    ),
    refused(
        CLEARED,
        LoweringCode::ClearedValueUnsupported,
        Needs::EntityCore("a removal a definition states; `Remove` is a host-selected action only"),
        &["tests/lowering.rs::an_operation_that_clears_a_field_is_refused_rather_than_left_to_the_host"],
    ),
    refused(
        ALPHABET,
        LoweringCode::AlphabetUnsupported,
        Needs::EntityCore("a condition over the characters of a text"),
        &["tests/lowering.rs::an_alphabet_and_a_text_length_are_refused_by_name_by_the_lowering"],
    ),
    refused(
        TEXT_COUNT,
        LoweringCode::TextLengthUnsupported,
        Needs::EntityCore("the length of a text; `count` reads arrays and maps only"),
        &[
            "tests/lowering.rs::an_alphabet_and_a_text_length_are_refused_by_name_by_the_lowering",
            "tests/adversary_guards_lowering.rs::adv_an_input_text_length_in_a_stored_field_predicate_is_refused_by_name",
        ],
    ),
    refused(
        OPTIONAL_UPDATE,
        LoweringCode::OptionalBoundOutputUnsupported,
        Needs::EntityCore("`PresentArgument` on an operation write; it covers creation, event and response members only"),
        &["tests/lowering.rs::operation_identity_and_optional_output_boundaries_are_explicit"],
    ),
    refused(
        STATELESS,
        LoweringCode::StatelessCommandUnsupported,
        Needs::NoTarget("entity-core decides over one entity instance and has no stateless decision"),
        &["tests/lowering.rs::input_diagnostics_accumulate_and_never_return_partial_output"],
    ),
    refused(
        SPANS,
        LoweringCode::CommandSpansEntities,
        Needs::NoTarget("an operation belongs to one entity definition"),
        &["tests/lowering.rs::command_shape_diagnostics_cover_multiple_targets_and_identity_bindings"],
    ),
    refused(
        MIXED,
        LoweringCode::MixedEntrypointUnsupported,
        Needs::NoTarget("creation and operation are separate entity-core entrypoints"),
        &[
            "tests/lowering.rs::command_shape_diagnostics_cover_mixed_multiple_and_subjectless_entrypoints",
            "tests/lowerable_subset.rs::a_mixed_entrypoint_does_not_hide_a_refused_update_value",
        ],
    ),
    refused(
        SECOND_CREATION,
        LoweringCode::MultipleCreationCommands,
        Needs::NoTarget("an entity definition has one unnamed creation entrypoint"),
        &["tests/lowering.rs::command_shape_diagnostics_cover_mixed_multiple_and_subjectless_entrypoints"],
    ),
    refused(
        AMBIGUOUS,
        LoweringCode::AmbiguousInstanceBinding,
        Needs::NoTarget("the host loads one instance, from one input field, before a branch is chosen"),
        &["tests/lowering.rs::command_shape_diagnostics_cover_multiple_targets_and_identity_bindings"],
    ),
    refused(
        WITHOUT_SUBJECT,
        LoweringCode::AcceptingOutcomeWithoutSubject,
        Needs::NoTarget("entity-core would create or revise the instance where ESS changes nothing"),
        &["tests/lowering.rs::command_shape_diagnostics_cover_mixed_multiple_and_subjectless_entrypoints"],
    ),
    refused(
        NULLABLE,
        LoweringCode::NullableElementUnsupported,
        Needs::EntityCore("a JSON `null` element; `required` says only whether a member is present"),
        &["tests/lowering.rs::structural_boundary_diagnostics_name_nullable_elements_and_recursion"],
    ),
    refused(
        RECURSIVE,
        LoweringCode::RecursiveTypeUnsupported,
        Needs::EntityCore("a typed definition reference; field definitions nest by value"),
        &["tests/lowering.rs::structural_boundary_diagnostics_name_nullable_elements_and_recursion"],
    ),
    refused(
        IDENTITY_UPDATE,
        LoweringCode::OperationIdentityMutationUnsupported,
        Needs::NoTarget("the identity mirror is immutable"),
        &["tests/lowering.rs::operation_identity_and_optional_output_boundaries_are_explicit"],
    ),
    refused(
        LITERAL_SHAPE,
        LoweringCode::LiteralShapeUnsupported,
        Needs::NoTarget("no exact structured value can be read from the text without guessing"),
        &["src/lib.rs::scalar_literal_decoding_preserves_exact_source_values"],
    ),
    refused(
        SILENT_PRESERVE,
        LoweringCode::SilentPreserveUnsupported,
        Needs::NoTarget("entity-core refuses a branch with no effect, write, event or response"),
        &["tests/lowering.rs::a_silent_preserving_outcome_is_refused_because_er_cannot_observe_it"],
    ),
    refused(
        TEXT_ORDERING,
        LoweringCode::TextOrderingUnsupported,
        Needs::EntityCore("an ordering of text by its UTF-8 bytes"),
        &["tests/lowering.rs::adv_a_byte_wise_text_stored_guard_is_refused_by_name_by_the_lowering"],
    ),
    refused(
        SUBJECT_VALUE,
        LoweringCode::ValueExpressionUnsupported,
        Needs::EntityCore("a value expression over the stored row"),
        &["tests/lowerable_subset.rs::each_value_expression_is_refused_under_its_own_construct"],
    ),
    refused(
        FALLBACK,
        LoweringCode::ValueExpressionUnsupported,
        Needs::EntityCore("a value expression with a fallback"),
        &["tests/lowerable_subset.rs::each_value_expression_is_refused_under_its_own_construct"],
    ),
    refused(
        INPUT_PATH,
        LoweringCode::ValueExpressionUnsupported,
        Needs::EntityCore("a read of a member of a structured argument, absent where an `Optional` on the way is"),
        &["tests/input_value_paths.rs::a4_a_value_read_through_an_input_path_is_refused_by_name"],
    ),
    refused(
        STRUCT,
        LoweringCode::ValueExpressionUnsupported,
        Needs::EntityCore("a value assembled from several sources"),
        &["tests/lowerable_subset.rs::each_value_expression_is_refused_under_its_own_construct"],
    ),
    refused(
        CASE_FOLD,
        LoweringCode::CaseFoldUnsupported,
        Needs::EntityCore("a condition that folds ASCII case"),
        &[
            "tests/lowering.rs::a_case_insensitive_guard_is_refused_by_name_by_the_lowering",
            "tests/adversary_guards_lowering.rs::adv_a_fold_in_the_input_guard_is_refused_by_name",
        ],
    ),
    refused(
        DELETES,
        LoweringCode::OutcomeShapeUnsupported,
        Needs::EntityCore("removal of an instance"),
        &["tests/outcome_shapes.rs::a_deleting_outcome_is_refused_by_name"],
    ),
    refused(
        INTO,
        LoweringCode::OutcomeShapeUnsupported,
        Needs::EntityCore("creation in a state other than the initial one"),
        &["tests/outcome_shapes.rs::a_creation_into_a_declared_state_is_refused_by_name"],
    ),
    refused(
        ACCEPTS_NOTHING,
        LoweringCode::OutcomeShapeUnsupported,
        Needs::EntityCore("an acceptance with no subject"),
        &["tests/outcome_shapes_adversary.rs::adversary_an_accepts_nothing_branch_is_refused_by_name"],
    ),
    refused(
        INPUT_ABSENT,
        LoweringCode::InputAbsentUnsupported,
        Needs::NoTarget("a request with no input is a transport fact that never reaches entity-core"),
        &["tests/absent_input.rs::an_input_absent_branch_is_refused_by_name"],
    ),
    refused(
        CALLER,
        LoweringCode::CallerUnsupported,
        Needs::EntityCore("an operand for who sent the command"),
        &[
            "tests/caller_values.rs::a_caller_source_is_refused_by_name",
            "tests/caller_values.rs::a_guard_reading_the_caller_is_refused_by_name",
        ],
    ),
    refused(
        SET_EFFECT,
        LoweringCode::SetEffectUnsupported,
        Needs::EntityCore("an operation over more than the one instance its request names"),
        &[
            "tests/set_effects.rs::a_set_update_is_refused_by_name",
            "tests/set_effects.rs::affects_is_refused_by_name",
        ],
    ),
    refused(
        RELATED_GUARD,
        LoweringCode::RelatedGuardUnsupported,
        Needs::EntityCore("a read of another entity's row"),
        &["tests/related_guard.rs::issue_211_a_command_guarded_by_a_related_row_is_refused_by_name"],
    ),
    refused(
        UNIT_VARIANT,
        LoweringCode::UnitVariantUnsupported,
        Needs::EntityCore("a union variant that admits no payload member; every entity-core variant admits one"),
        &["tests/union_unit_variants.rs::a_stored_union_with_a_unit_variant_is_refused_by_name"],
    ),
    refused(
        OFFSET,
        LoweringCode::OffsetUnsupported,
        Needs::EntityCore("an operand that moves a value by a constant"),
        &[
            "tests/offset_guard.rs::a_guard_reading_an_integer_offset_is_refused_by_name",
            "tests/offset_guard.rs::a_guard_reading_a_timestamp_offset_is_refused_by_name",
        ],
    ),
    refused(
        DISTINCT,
        LoweringCode::DistinctUnsupported,
        Needs::EntityCore("a condition that compares keys across a list's elements"),
        &["tests/distinct_guard.rs::a_guard_requiring_distinct_members_is_refused_by_name"],
    ),
    refused(
        UTF8_BYTES,
        LoweringCode::Utf8BytesUnsupported,
        Needs::EntityCore("an address for the UTF-8 byte length of a text"),
        &[
            "tests/utf8_bytes_guard.rs::a_guard_reading_a_byte_length_is_refused_by_name",
            "tests/utf8_bytes_guard.rs::a_guard_comparing_two_byte_lengths_is_refused_by_name",
        ],
    ),
    refused(
        ROW_SET,
        LoweringCode::RowSetUnsupported,
        Needs::EntityCore("a query over the rows of an entity, read atomically in one decision"),
        &["tests/row_sets.rs::a_row_set_guard_and_a_filtered_read_are_refused_by_name"],
    ),
];

impl LoweringCode {
    /// Every code, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::MissingDefinitionVersion,
        Self::UnknownDefinitionVersion,
        Self::UnknownScaleEntity,
        Self::StatelessCommandUnsupported,
        Self::CommandSpansEntities,
        Self::MixedEntrypointUnsupported,
        Self::MultipleCreationCommands,
        Self::AmbiguousInstanceBinding,
        Self::AcceptingOutcomeWithoutSubject,
        Self::NullableElementUnsupported,
        Self::RecursiveTypeUnsupported,
        Self::OptionalBoundOutputUnsupported,
        Self::OperationFieldFulfillmentUnsupported,
        Self::OperationIdentityMutationUnsupported,
        Self::LiteralShapeUnsupported,
        Self::ClearedValueUnsupported,
        Self::SilentPreserveUnsupported,
        Self::TargetDefinitionRefused,
        Self::TextOrderingUnsupported,
        Self::AlphabetUnsupported,
        Self::TextLengthUnsupported,
        Self::ValueExpressionUnsupported,
        Self::CaseFoldUnsupported,
        Self::OutcomeShapeUnsupported,
        Self::InputAbsentUnsupported,
        Self::CurrentTimeUnsupported,
        Self::ExistenceSelectionUnsupported,
        Self::CallerUnsupported,
        Self::SetEffectUnsupported,
        Self::RelatedGuardUnsupported,
        Self::UnitVariantUnsupported,
        Self::OffsetUnsupported,
        Self::DistinctUnsupported,
        Self::Utf8BytesUnsupported,
        Self::RowSetUnsupported,
    ];

    /// The code as a harness matches on it: the variant's name.
    pub const fn name(self) -> &'static str {
        self.entry().0
    }

    /// The construct a diagnostic under this code names when its site names no narrower one.
    pub const fn construct(self) -> &'static str {
        self.entry().1
    }

    /// What the code means, in one sentence.
    pub const fn meaning(self) -> &'static str {
        self.entry().2
    }

    /// Whether the code refuses something a specification says, rather than an option the caller
    /// passed or a definition Entity Runtime rejected.
    pub const fn is_source_construct(self) -> bool {
        !matches!(
            self,
            Self::MissingDefinitionVersion
                | Self::UnknownDefinitionVersion
                | Self::UnknownScaleEntity
                | Self::OperationFieldFulfillmentUnsupported
                | Self::TargetDefinitionRefused
        )
    }

    /// For a code that is not a source construct, the tests that show it; a source construct's
    /// tests are its rows' in [`CONSTRUCTS`].
    pub const fn evidence(self) -> &'static [&'static str] {
        match self {
            Self::MissingDefinitionVersion
            | Self::UnknownDefinitionVersion
            | Self::UnknownScaleEntity => {
                &["tests/lowering.rs::input_diagnostics_accumulate_and_never_return_partial_output"]
            }
            Self::OperationFieldFulfillmentUnsupported => &[
                "tests/lowering.rs::complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment",
            ],
            Self::TargetDefinitionRefused => {
                &["tests/lowering.rs::target_validation_failures_are_returned_without_approximation"]
            }
            _ => &[],
        }
    }

    #[allow(clippy::too_many_lines)]
    const fn entry(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::MissingDefinitionVersion => (
                "MissingDefinitionVersion",
                "`LoweringOptions::definition_versions`",
                "An entity of the component's closure has no Entity Runtime definition version.",
            ),
            Self::UnknownDefinitionVersion => (
                "UnknownDefinitionVersion",
                "`LoweringOptions::definition_versions`",
                "A definition version names an entity outside the component's closure.",
            ),
            Self::UnknownScaleEntity => (
                "UnknownScaleEntity",
                "`LoweringOptions::scales`",
                "A scale declaration names an entity outside the component's closure.",
            ),
            Self::StatelessCommandUnsupported => (
                "StatelessCommandUnsupported",
                STATELESS,
                "The command has no outcome with an entity subject.",
            ),
            Self::CommandSpansEntities => (
                "CommandSpansEntities",
                SPANS,
                "The command's outcomes act on more than one entity.",
            ),
            Self::MixedEntrypointUnsupported => (
                "MixedEntrypointUnsupported",
                MIXED,
                "The command both creates its entity and changes an existing instance.",
            ),
            Self::MultipleCreationCommands => (
                "MultipleCreationCommands",
                SECOND_CREATION,
                "Two commands of the component create the same entity.",
            ),
            Self::AmbiguousInstanceBinding => (
                "AmbiguousInstanceBinding",
                AMBIGUOUS,
                "The instance a command acts on is not named by one input field, or a creation does \
                 not observe its identity in an emitted event.",
            ),
            Self::AcceptingOutcomeWithoutSubject => (
                "AcceptingOutcomeWithoutSubject",
                WITHOUT_SUBJECT,
                "An accepting outcome of an entity command names no subject.",
            ),
            Self::NullableElementUnsupported => (
                "NullableElementUnsupported",
                NULLABLE,
                "A list item or a map value is `Optional`.",
            ),
            Self::RecursiveTypeUnsupported => (
                "RecursiveTypeUnsupported",
                RECURSIVE,
                "A lowered field's type reaches itself.",
            ),
            Self::OptionalBoundOutputUnsupported => (
                "OptionalBoundOutputUnsupported",
                OPTIONAL_UPDATE,
                "An update writes a value that may be absent on one call and present on the next.",
            ),
            Self::OperationFieldFulfillmentUnsupported => (
                "OperationFieldFulfillmentUnsupported",
                "operation-field fulfillment",
                "Historical: never emitted at this target, which fulfills operation fields.",
            ),
            Self::OperationIdentityMutationUnsupported => (
                "OperationIdentityMutationUnsupported",
                IDENTITY_UPDATE,
                "An update sets the entity's identity.",
            ),
            Self::LiteralShapeUnsupported => (
                "LiteralShapeUnsupported",
                LITERAL_SHAPE,
                "A text literal targets a type that is not a scalar.",
            ),
            Self::ClearedValueUnsupported => (
                "ClearedValueUnsupported",
                CLEARED,
                "An update, an event payload or an identity is cleared.",
            ),
            Self::SilentPreserveUnsupported => (
                "SilentPreserveUnsupported",
                SILENT_PRESERVE,
                "An accepting outcome keeps its row and has nothing to show for it.",
            ),
            Self::TargetDefinitionRefused => (
                "TargetDefinitionRefused",
                "Entity Runtime definition validation",
                "Entity Runtime refused a lowered definition, or two lowerings of one input differ.",
            ),
            Self::TextOrderingUnsupported => (
                "TextOrderingUnsupported",
                TEXT_ORDERING,
                "A guard orders text.",
            ),
            Self::AlphabetUnsupported => (
                "AlphabetUnsupported",
                ALPHABET,
                "A newtype a lowered field reaches declares an alphabet.",
            ),
            Self::TextLengthUnsupported => (
                "TextLengthUnsupported",
                TEXT_COUNT,
                "A predicate reads the length of a text.",
            ),
            Self::ValueExpressionUnsupported => (
                "ValueExpressionUnsupported",
                "a value expression",
                "A `sets:` or `payload:` source reads the subject or another row, increments, \
                 falls back, or nests.",
            ),
            Self::CaseFoldUnsupported => (
                "CaseFoldUnsupported",
                CASE_FOLD,
                "A guard compares text without ASCII case.",
            ),
            Self::OutcomeShapeUnsupported => (
                "OutcomeShapeUnsupported",
                "an ess/15 outcome shape",
                "An outcome is an `unknown_instance:` refusal, deletes, creates `into:` a state, or \
                 accepts nothing.",
            ),
            Self::InputAbsentUnsupported => (
                "InputAbsentUnsupported",
                INPUT_ABSENT,
                "An outcome answers a request with no input.",
            ),
            Self::CurrentTimeUnsupported => (
                "CurrentTimeUnsupported",
                NOW,
                "A guard orders a `Timestamp` against the current time.",
            ),
            Self::ExistenceSelectionUnsupported => (
                "ExistenceSelectionUnsupported",
                EXISTENCE,
                "A branch is chosen by whether a record carries the addressed identity.",
            ),
            Self::CallerUnsupported => (
                "CallerUnsupported",
                CALLER,
                "A value or a guard reads the authenticated caller.",
            ),
            Self::SetEffectUnsupported => (
                "SetEffectUnsupported",
                SET_EFFECT,
                "An outcome changes rows beside, or instead of, the one its request names.",
            ),
            Self::RelatedGuardUnsupported => (
                "RelatedGuardUnsupported",
                RELATED_GUARD,
                "A branch is guarded by a row of another entity.",
            ),
            Self::UnitVariantUnsupported => (
                "UnitVariantUnsupported",
                UNIT_VARIANT,
                "A union lowered as a field declares a variant that carries nothing.",
            ),
            Self::OffsetUnsupported => (
                "OffsetUnsupported",
                OFFSET,
                "A predicate compares a fact with one constant offset of another.",
            ),
            Self::DistinctUnsupported => (
                "DistinctUnsupported",
                DISTINCT,
                "A predicate requires that no two elements of a list share a key.",
            ),
            Self::Utf8BytesUnsupported => (
                "Utf8BytesUnsupported",
                UTF8_BYTES,
                "A predicate compares the UTF-8 byte length of a text.",
            ),
            Self::RowSetUnsupported => (
                "RowSetUnsupported",
                ROW_SET,
                "A branch reads the rows a selector selects, or one value of the one row it selects.",
            ),
        }
    }
}

impl std::fmt::Display for LoweringCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// `|` would end a Markdown table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// The published page, rendered from [`CONSTRUCTS`] and [`LoweringCode::ALL`].
pub fn reference_page() -> String {
    let mut page = String::new();
    page.push_str(
        "---\ntitle: Entity Runtime lowering\ndescription: Which ESS constructs a component \
         lowered to Entity Runtime may use, which are refused and under which code, and what \
         each refusal would need.\n---\n\n# Entity Runtime lowering\n\n",
    );
    page.push_str(
        "[ess-lowering-begin]: # (generated from ess_entity_runtime::subset; regenerate with \
         ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --test lowerable_subset)\n\n",
    );
    let _ = write!(
        page,
        "`ess-entity-runtime` lowers one component of a compiled specification to Entity Runtime \
         definitions and the binding obligations its host fills. It targets entity-core revision \
         `{ENTITY_RUNTIME_REVISION}`. `lower_component` takes the compiled model, the component's \
         name and the definition versions, and returns either the lowered component or every \
         refusal at once: each diagnostic carries its code, the path of the refused construct in \
         the model, the construct's name as the tables below write it, and a sentence. A refused \
         construct does not hide the others: a command refused as a whole still has each of its \
         branches checked, and an entity with no definition version still has its fields and \
         rules checked. The exception is a command with no entity subject \
         (`StatelessCommandUnsupported`): its branches have no entity to be lowered against, so \
         only its input, its response and what its guards read are checked.\n\n"
    );
    page.push_str(
        "Nothing is lowered approximately. A construct that has no exact Entity Runtime form is \
         refused under its own code, so a component that lowers means what its specification \
         says.\n\n## What lowers\n\n| Construct | Entity Runtime form |\n|---|---|\n",
    );
    for construct in CONSTRUCTS {
        if let Lowering::Lowered { into } = construct.lowering {
            let _ = writeln!(page, "| {} | {} |", cell(construct.name), cell(into));
        }
    }
    page.push_str(
        "\n## What is refused\n\nThe last column says what lowering the construct would need. \
         *entity-core* means a capability the Entity Runtime kernel does not have; lowering waits \
         for it there. *Host value* means ESS alone could lower it by having the host supply a \
         value as a bound argument, as it already supplies generated values and external \
         evidence; that work is not done yet. *No form* means no Entity Runtime definition can \
         state it.\n\n| Construct | Code | Lowering needs |\n|---|---|---|\n",
    );
    for construct in CONSTRUCTS {
        if let Lowering::Refused { code, needs } = construct.lowering {
            let needs = match needs {
                Needs::EntityCore(what) => format!("entity-core: {what}"),
                Needs::HostValue(what) => format!("Host value: {what}"),
                Needs::NoTarget(why) => format!("No form: {why}"),
            };
            let _ = writeln!(
                page,
                "| {} | `{}` | {} |",
                cell(construct.name),
                code.name(),
                cell(&needs)
            );
        }
    }
    page.push_str(
        "\n## Every code\n\nA harness matches on the code. The construct is the one a diagnostic \
         under that code names when its site names no narrower row above.\n\n\
         | Code | Construct | Meaning |\n|---|---|---|\n",
    );
    for code in LoweringCode::ALL {
        let _ = writeln!(
            page,
            "| `{}` | {} | {} |",
            code.name(),
            cell(code.construct()),
            cell(code.meaning())
        );
    }
    page.push_str("\n[ess-lowering-end]: #\n");
    page
}
