---
title: Entity Runtime lowering
description: Which ESS constructs a component lowered to Entity Runtime may use, which are refused and under which code, and what each refusal would need.
---

# Entity Runtime lowering

[ess-lowering-begin]: # (generated from ess_entity_runtime::subset; regenerate with ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --test lowerable_subset)

`ess-entity-runtime` lowers one component of a compiled specification to Entity Runtime definitions and the binding obligations its host fills. It targets entity-core revision `4746bd7cc37d27c7cc5815c44a62a96f3ddc1f44`. `lower_component` takes the compiled model, the component's name and the definition versions, and returns either the lowered component or every refusal at once: each diagnostic carries its code, the path of the refused construct in the model, the construct's name as the tables below write it, and a sentence. A refused construct does not hide the others: a command refused as a whole still has each of its branches checked, and an entity with no definition version still has its fields and rules checked. The exception is a command with no entity subject (`StatelessCommandUnsupported`): its branches have no entity to be lowered against, so only its input, its response and what its guards read are checked.

Nothing is lowered approximately. A construct that has no exact Entity Runtime form is refused under its own code, so a component that lowers means what its specification says.

## What lowers

| Construct | Entity Runtime form |
|---|---|
| an entity: identity, fields, lifecycle | an `EntityDefinition` with its schema, `identity` and `lifecycle` |
| primitive, `List`, `Map`, struct, enum, union and `Json` types; `Optional` members | `FieldDefinition`s; an `Optional` member is not `required` |
| entity and type `invariants:` | named entity rules, one per invariant and per layer of a newtype chain |
| `prefix:` on a newtype | a `starts_with` rule |
| `relations:` | `RelationDefinition`s, with existence, cardinality and ownership binding obligations |
| `creates:` | the entity's `create` entrypoint, its identity observed in an emitted event |
| `updates:` and `moves:` | a named operation; an unmapped field is a host fulfillment obligation |
| `preserves:` with a response | an outcome with no effect that responds |
| `error:` outcomes | `refuses`, with an error-payload binding obligation |
| `when:` over the input | `when`, an input-guarded refusal ordered before the branches it overlaps |
| `when_subject:` a state, a field, or a predicate over the stored row and `input.` | `in_state`, or `when` over `$fields`, `$from_state` and the arguments |
| `when_state_changes:` | membership of `$from_state` |
| the wrong-state branch | `wrong_state`, ordered after every guarded branch |
| `external:` and `external_when:` | a guard over host-supplied evidence, with the input guard beside it |
| orderings over numbers and `Timestamp`s; string operators | `compare`, `before`/`after`, `starts_with` and the other entity-core string conditions |
| `input.<field>`, a literal, `{response: …}`, `{generated: true}`, a declared conversion | an argument template, an escaped literal, or a typed bound slot the host fills |
| `{cleared: true}` in a creation's `sets:` | the member left absent |
| `emits:` and `response:` | `emits` and `responds`, conditional members through `PresentArgument` |

## What is refused

The last column says what lowering the construct would need. *entity-core* means a capability the Entity Runtime kernel does not have; lowering waits for it there. *Host value* means ESS alone could lower it by having the host supply a value as a bound argument, as it already supplies generated values and external evidence; that work is not done yet. *No form* means no Entity Runtime definition can state it.

| Construct | Code | Lowering needs |
|---|---|---|
| `now` in a guard | `CurrentTimeUnsupported` | Host value: the request's clock reading as a bound argument; entity-core refuses `$now` |
| `{related: …}` | `ValueExpressionUnsupported` | Host value: the related row's field, loaded by the host and bound as an argument |
| an `unknown_instance:` refusal | `OutcomeShapeUnsupported` | Host value: the host's own not-found answer, mapped to the declared refusal |
| a branch chosen by existence (`unknown_instance:` on a creation, `existing_instance:`) | `ExistenceSelectionUnsupported` | Host value: the host's own exists answer, selecting the declared branch |
| `{increment: …}` | `ValueExpressionUnsupported` | entity-core: arithmetic over a stored value |
| `{cleared: true}` outside a creation's `sets:` | `ClearedValueUnsupported` | entity-core: a removal a definition states; `Remove` is a host-selected action only |
| `alphabet:` | `AlphabetUnsupported` | entity-core: a condition over the characters of a text |
| `.count` of a text | `TextLengthUnsupported` | entity-core: the length of a text; `count` reads arrays and maps only |
| an `Optional` value written by an update | `OptionalBoundOutputUnsupported` | entity-core: `PresentArgument` on an operation write; it covers creation, event and response members only |
| a command with no entity subject | `StatelessCommandUnsupported` | No form: entity-core decides over one entity instance and has no stateless decision |
| one command over several entities | `CommandSpansEntities` | No form: an operation belongs to one entity definition |
| a creation beside an update in one command | `MixedEntrypointUnsupported` | No form: creation and operation are separate entity-core entrypoints |
| a second command creating one entity | `MultipleCreationCommands` | No form: an entity definition has one unnamed creation entrypoint |
| an instance binding with no single source | `AmbiguousInstanceBinding` | No form: the host loads one instance, from one input field, before a branch is chosen |
| an accepting outcome with no subject | `AcceptingOutcomeWithoutSubject` | No form: entity-core would create or revise the instance where ESS changes nothing |
| `Optional` below a list or a map | `NullableElementUnsupported` | entity-core: a JSON `null` element; `required` says only whether a member is present |
| a recursive type | `RecursiveTypeUnsupported` | entity-core: a typed definition reference; field definitions nest by value |
| the identity in an update's `sets:` | `OperationIdentityMutationUnsupported` | No form: the identity mirror is immutable |
| a text literal for a structured type | `LiteralShapeUnsupported` | No form: no exact structured value can be read from the text without guessing |
| `preserves:` with no response | `SilentPreserveUnsupported` | No form: entity-core refuses a branch with no effect, write, event or response |
| `<`, `<=`, `>`, `>=` over text | `TextOrderingUnsupported` | entity-core: an ordering of text by its UTF-8 bytes |
| `{subject: …}` | `ValueExpressionUnsupported` | entity-core: a value expression over the stored row |
| `{input: …, else: …}` | `ValueExpressionUnsupported` | entity-core: a value expression with a fallback |
| a struct of sources | `ValueExpressionUnsupported` | entity-core: a value assembled from several sources |
| `equals_ignore_case`, `in_ignore_case` | `CaseFoldUnsupported` | entity-core: a condition that folds ASCII case |
| `deletes:` | `OutcomeShapeUnsupported` | entity-core: removal of an instance |
| a creation `into:` a state | `OutcomeShapeUnsupported` | entity-core: creation in a state other than the initial one |
| `accepts: nothing` | `OutcomeShapeUnsupported` | entity-core: an acceptance with no subject |
| `input_absent:` | `InputAbsentUnsupported` | No form: a request with no input is a transport fact that never reaches entity-core |
| the caller (`{caller: …}`, `caller.<attribute>`) | `CallerUnsupported` | entity-core: an operand for who sent the command |
| a set effect (`instances:`, `affects:`, `{count: changed}`) | `SetEffectUnsupported` | entity-core: an operation over more than the one instance its request names |
| `when_related:` | `RelatedGuardUnsupported` | entity-core: a read of another entity's row |
| a union variant with no payload (ess/22) | `UnitVariantUnsupported` | entity-core: a union variant that admits no payload member; every entity-core variant admits one |
| one constant offset (`upper == lower + 5`, `issued_at - 24h`) | `OffsetUnsupported` | entity-core: an operand that moves a value by a constant |

## Every code

A harness matches on the code. The construct is the one a diagnostic under that code names when its site names no narrower row above.

| Code | Construct | Meaning |
|---|---|---|
| `MissingDefinitionVersion` | `LoweringOptions::definition_versions` | An entity of the component's closure has no Entity Runtime definition version. |
| `UnknownDefinitionVersion` | `LoweringOptions::definition_versions` | A definition version names an entity outside the component's closure. |
| `UnknownScaleEntity` | `LoweringOptions::scales` | A scale declaration names an entity outside the component's closure. |
| `StatelessCommandUnsupported` | a command with no entity subject | The command has no outcome with an entity subject. |
| `CommandSpansEntities` | one command over several entities | The command's outcomes act on more than one entity. |
| `MixedEntrypointUnsupported` | a creation beside an update in one command | The command both creates its entity and changes an existing instance. |
| `MultipleCreationCommands` | a second command creating one entity | Two commands of the component create the same entity. |
| `AmbiguousInstanceBinding` | an instance binding with no single source | The instance a command acts on is not named by one input field, or a creation does not observe its identity in an emitted event. |
| `AcceptingOutcomeWithoutSubject` | an accepting outcome with no subject | An accepting outcome of an entity command names no subject. |
| `NullableElementUnsupported` | `Optional` below a list or a map | A list item or a map value is `Optional`. |
| `RecursiveTypeUnsupported` | a recursive type | A lowered field's type reaches itself. |
| `OptionalBoundOutputUnsupported` | an `Optional` value written by an update | An update writes a value that may be absent on one call and present on the next. |
| `OperationFieldFulfillmentUnsupported` | operation-field fulfillment | Historical: never emitted at this target, which fulfills operation fields. |
| `OperationIdentityMutationUnsupported` | the identity in an update's `sets:` | An update sets the entity's identity. |
| `LiteralShapeUnsupported` | a text literal for a structured type | A text literal targets a type that is not a scalar. |
| `ClearedValueUnsupported` | `{cleared: true}` outside a creation's `sets:` | An update, an event payload or an identity is cleared. |
| `SilentPreserveUnsupported` | `preserves:` with no response | An accepting outcome keeps its row and has nothing to show for it. |
| `TargetDefinitionRefused` | Entity Runtime definition validation | Entity Runtime refused a lowered definition, or two lowerings of one input differ. |
| `TextOrderingUnsupported` | `<`, `<=`, `>`, `>=` over text | A guard orders text. |
| `AlphabetUnsupported` | `alphabet:` | A newtype a lowered field reaches declares an alphabet. |
| `TextLengthUnsupported` | `.count` of a text | A predicate reads the length of a text. |
| `ValueExpressionUnsupported` | a value expression | A `sets:` or `payload:` source reads the subject or another row, increments, falls back, or nests. |
| `CaseFoldUnsupported` | `equals_ignore_case`, `in_ignore_case` | A guard compares text without ASCII case. |
| `OutcomeShapeUnsupported` | an ess/15 outcome shape | An outcome is an `unknown_instance:` refusal, deletes, creates `into:` a state, or accepts nothing. |
| `InputAbsentUnsupported` | `input_absent:` | An outcome answers a request with no input. |
| `CurrentTimeUnsupported` | `now` in a guard | A guard orders a `Timestamp` against the current time. |
| `ExistenceSelectionUnsupported` | a branch chosen by existence (`unknown_instance:` on a creation, `existing_instance:`) | A branch is chosen by whether a record carries the addressed identity. |
| `CallerUnsupported` | the caller (`{caller: …}`, `caller.<attribute>`) | A value or a guard reads the authenticated caller. |
| `SetEffectUnsupported` | a set effect (`instances:`, `affects:`, `{count: changed}`) | An outcome changes rows beside, or instead of, the one its request names. |
| `RelatedGuardUnsupported` | `when_related:` | A branch is guarded by a row of another entity. |
| `UnitVariantUnsupported` | a union variant with no payload (ess/22) | A union lowered as a field declares a variant that carries nothing. |
| `OffsetUnsupported` | one constant offset (`upper == lower + 5`, `issued_at - 24h`) | A predicate compares a fact with one constant offset of another. |

[ess-lowering-end]: #
