# Filtered related reads (#299, coordinated ess/22)

Status: revised binding design for the accepted remaining bundle; second independent design review
and implementation are pending. This defines the family F row-set guard shared by #228 and #237, and the
existing related-value source described in value-expressions.md E8. It does not add a view join.

## One selection, two consumers

A selector names `entity` and `where`, using the stored-row predicate grammar already used by
`affects: {entity, where, sets}`. Bare fields name the candidate row; `input.<path>` names command
input, and `subject.<path>` names the existing addressed subject before the outcome. A creation
has no prior subject, so subject operands there are refused. Dotted operands and time predicates
are admitted only with their coordinated family F semantics and type checks.

The predicate selects rows from the immutable store immediately before branch selection. All
guards and values of that command use that same snapshot. Rows newly created or changed by the
outcome cannot become candidates for its own reads. Candidate iteration order never decides which
value is read. There is no implicit latest row, state preference, or same-identity exclusion.

The guard consumer is the shared row-set shape:

```yaml
when_related:
  entity: demo.jobs.Attempt
  where:
    all: [worker_id == input.worker_id, batch_id == input.batch_id]
  count: {eq: 1}
```

`exists`, `count`, and `forall` are the family F alternatives over that same selector, not new
spellings owned by this feature. They keep the family's branch-overlap, exhaustiveness, typing,
and precedence rules. A matching row whose predicate cannot be decided is not silently omitted.
Unknown membership or cardinality remains undetermined unless the existing proof machinery can
decide the particular guard from every possible completion.

### Shared row-set guard contract (#228, #237)

The mapping requires `entity`, `where`, and exactly one of `exists`, `count`, or `forall`.
It is an alternative to the identity-addressed `when_related` mapping: `via` cannot occur in it.
`where` is a nonempty, typed stored-row predicate using `all`, `any`, `not`, definedness and
the admitted comparisons. There is no SQL text, ordering, limit or view name. Candidate fields
are bare paths; input and subject paths retain their explicit namespaces. `forall` is a second
predicate in that same environment, tested only over selected rows. `exists` is Boolean.
`count` is a one-entry comparison mapping (`eq`, `ne`, `lt`, `lte`, `gt`, `gte`) to a nonnegative
Integer literal. Parameter operands are a separately sequenced #200 extension, not an implicit
part of this grammar. Offset/time/dotted-path operands reuse the already reviewed family F
expression types and evaluation; this construct admits no second expression implementation.

| Selected rows | `exists: true` | `exists: false` | `count: {op: n}` | `forall: P` |
| --- | --- | --- | --- | --- |
| Empty | false | true | compare 0 to n | true |
| One or more, known membership | true | false | compare exact cardinality to n | all P true; false if any P false; otherwise Unknown |
| Unknown membership | true if any certain member; false only if no possible member; otherwise Unknown | logical negation of exists | definite only if comparison agrees for every possible cardinality | false if a certain member falsifies P; true only if every possible member certainly satisfies P; otherwise Unknown |

All count arithmetic is exact. Unknown rows are retained as possible members, never dropped.
The table gives sound sufficient decisions; a stronger shared proof may decide additional cases
only by proving the same result for every completion, preserving correlations between repeated
operands. A lower/upper cardinality interval may conservatively yield Unknown. An incomplete
store observation has possible unseen rows and cannot prove absence, exact count or a universal
claim merely by enumerating observed rows. Native execution has the complete pre-outcome store;
history checking needs its existing complete-domain authority to make those proofs.

The optional ordinary input `when` is conjunctive with the row-set test. Validation checks each
predicate's types and the joint guard partition. Known overlap between deterministic branches
is refused. A complementary default covers only valuations where all its preceding applicable
guards are definitely false; Unknown never selects it. Synthesis may return the existing named
no-witness result when a partition is outside its proof/search capability; it cannot invent a
finite enumeration and claim completeness. Bounds above the existing 10,000-live-cursor synthesis
cap are valid declarations but cannot silently truncate a witness. Shared-target arrangements
require a scoped String/Uuid equality from input as described in `aggregate-views.md`; a missing
scope yields a named synthesis refusal, not a global query that happens to pass on an empty store.

### Subject borrowing and precedence

An input-only row-set selector is legal on a creation. A selector using `subject` needs the one
existing supplied subject shared by the command's subject-bearing outcomes, reusing
`subject_fact::common_subject` and its validation. A refusal may borrow that subject. All such
outcomes must name the same entity and input identity field; multiple different candidate
subjects, creation-only subjects, and set subjects are refused at the subject operand. A missing
addressed row never supplies a fabricated subject or turns a subject-dependent selector empty.

The accepted #282/#304 precedence governs: early input-reference missing-row refusals, ordinary
input refusals, addressed-row existence and held-state checks keep their priority; stored-reference
and present-related predicates follow them. Row-set tests join this latter stored-read phase,
before selecting the accepting/default branch. `exists: false` on a row set is an empty-set test,
not the early identity-reference absence branch. A nonmoving outcome remains independent of held
state under #282's rule. If a command combines multiple related conditions under #283, identity
absence and Optional absence retain #283/#304's explicit rules; no row-set test promotes itself
to their earlier phase. Value reads occur only after that branch is selected and still read the
same pre-outcome snapshot. A refusal that answers before subject existence cannot copy a subject
value through this source.

The value consumer extends the existing source:

```yaml
delay:
  related:
    entity: demo.jobs.Attempt
    where:
      all: [worker_id == input.worker_id, batch_id == input.batch_id]
    field: delay
```

The alternatives are exactly `{via, field}` (including #285's Optional-aware chained `via`) or
`{entity, where, field}`. Mixing `via` with `entity`/`where`, incomplete selectors, and extra keys
are refused. `field` has the selected entity's declared type, including Optional when the stored
field itself is Optional. Existing assignment/conversion rules apply in payloads, sets, and
nested struct leaves. No new coalesce/default source is added.

## Cardinality is explicit

A value read requires exactly one matching row. Zero or multiple matches supply no value and
cannot publish a transition, partial store update, or event. An Optional target does not turn an
ambiguous match into absence. Unknown membership or a generated unobserved selector operand does
not authorize a guessed row or sampled value in history checking.

An application that creates a new attempt with delay zero when none exists declares a zero-count
branch with literal zero; its one-count branch copies the selected row's delay. It declares a
separate refusal for counts greater than one if multiple stored matches are possible. The normal
guard rules check these branches; this source itself does not choose an application outcome.
An unguarded selector is legal in the same sense as an existing related read: its runtime target
must report the existing no-value/unmet-obligation result when the required row cannot be read.
No implementation may choose the first match to turn that obligation into success.

## Compatibility and targets

The coordinated source allocation is ess/22; ess/21 remains allocated to one-time responses.
Older sources retain their existing nested-mapping interpretation wherever it was valid. A new
filtered-source shape over a non-struct target is refused below ess/22 with the existing
unsupported-format diagnostic. Add old-reader and old-struct-meaning controls before admitting it.

Parsing precedes source-version admission. Capture the potential related selection as a closed,
format-neutral raw alternative that can retain predicate mappings and sequences without eagerly
turning it into a value source. At source22, recognize and type-check only the exact new shape.
Below22, back-convert to the original nested payload representation and validate the original
target type wherever that representation was legal. A raw sequence newly retained for a predicate
must not become a generally accepted old payload sequence: replay the prior refusal when old
conversion cannot represent it. Preserve original key/source locations, duplicate-key refusal and
extra-key diagnostics. Tests must distinguish an old struct with fields named `related`, `entity`,
`where`, `field` from a new filtered source, and include sequence-bearing predicates and malformed
mixed `via`/selector mappings. Do not change the old parser's acceptance by running new-shape
recognition unconditionally before the format is known.

Represent selection as a new typed `related_selection` payload-value alternative, with resolved
entity, typed predicate, selected field, and result type. Keep existing `related_field` bytes
unchanged; never overload its `via` string with a query. The compiled specification already
carries the validated source `format` (`ess-compiler/src/ir.rs`, `EssIr::format`) and is serialized,
not deserialized by ESS. Source ess/22 is the coordinated version for this new resolved variant;
do not introduce a generic IR envelope. An older compiler refuses source22 before resolving the
new construct. Specifications without filtered sources keep their canonical bytes. Synthesis
should express observations through existing capture,
execute, event and row assertions; a new suite instruction requires its own explicit version and
old-reader test instead of silently expanding an old envelope.

Add a distinct typed `ResolvedCondition::RelatedSet` (`kind: related_set`), containing a resolved
entity handle, typed `where` predicate, tagged test (`exists` with Boolean, `count` with the closed
comparison operator and Integer bound, or `forall` with typed predicate), and optional input
predicate omitted when absent. Use the same typed selector structure in `related_selection`;
never serialize a free-form JSON predicate bag. Existing `ResolvedCondition::Related`, its `via`,
and every unaffected condition retain their exact bytes. Source22 is the explicit version fence
for both new resolved alternatives. If a suite embeds a new predicate or expression variant, its
minimum suite version must be allocated and old-reader refusal tested before that variant is
emitted; using existing execute/capture/view steps alone needs no new suite version.

The interpreter and history checker execute the shared selector with pre-outcome authority.
The required generated profile is Boolean/String/Integer/Uuid/enum row fields, their Optional
forms and typed struct paths, with conjunctions, disjunctions, negation, equality, ordering where
the scalar permits it, and definedness. Every `exists`, `count`, `forall` alternative and value
selection must execute in this profile. A target may not owe the selector itself. Decimal/time
operations compose with the separate accepted expression slices: an existing representation
obligation stays explicit until that slice is integrated, and does not satisfy its final acceptance.

| Target | Required disposition before bundle closure |
| --- | --- |
| Native interpreter | Execute full admitted selector and value semantics, exact values and no partial effects |
| History checker | Execute sound complete/partial-store and generated-value reasoning; Unknown remains undetermined |
| Generated Rust and Go | Execute the required profile through actual storage enumeration; compile and run the generated application |
| Live web | Execute generated Rust behavior in the real browser/WASM host; browser replay is insufficient |
| Native, Go and TypeScript suite runtimes | Execute every emitted observation against healthy and faulty targets, zero skipped/unsupported required scenarios |
| Synthesis | Arrange required-profile branches and decoys through declared commands and observe their independent values |
| Explorer and authored scenarios | Execute the required profile after their accepted integration slices; preserve named refusal outside witnessable bounds |
| Entity Runtime | Named cross-instance query/atomic-authority limitation until the external dependency is available |

Legacy browser replay is not behavior evidence. Entity Runtime
lowering must report its named cross-instance limitation until it has the required query and
atomic command authority; that external limitation is not treated as successful conformance.
OpenAPI/AsyncAPI retain the command wire shape. Generated documentation describes selection and
cardinality. Diff treats changing the selector or source field as a behavior change using an
explicit classified rule; it cannot erase the source as an unknown expression. Explorer,
authoring, and synthesis must either handle the selector or return their named unsupported or
no-witness result. Required acceptance scenarios cannot pass by skipping those operations.

## Decisive acceptance

The conformance fixture `filtered_related_reads` uses two independent scope conjuncts, an addressed
subject, a selected value 17, distinct decoy values 31 and 47, a creation/update that would change
membership, and explicitly absent Optional fields. Its named scenarios are
`one_match_reads_value`, `zero_match_branch`, `many_matches_refuse`,
`unguarded_zero_and_many_do_not_mutate`, `empty_forall_is_true`, `forall_has_counterexample`,
`count_boundaries`, `optional_selected_value`, `subject_borrowing_and_precedence`, and
`reads_pre_outcome_snapshot`. Each observes outcome, events and all affected rows; the setup reads
the arranged values back independently before the tested command. Run each scenario against the
healthy implementation in every executable row of the target table. History has matching named
complete-domain, partial-domain and unobserved-generated-operand controls.

For every executable target lane, controls `drop_first_conjunct` and `drop_second_conjunct` each
introduce their corresponding decoy, `copy_wrong_row` keeps correct guard selection but copies 31,
and `read_after_effects` applies the outcome before selecting. Each independently yields failed
conformance with a named outcome/event/row mismatch; healthy controls pass. Generation tests patch
the generated behavior at the actual selection/copy seam and compile it; native tests alter the
target's behavior through its test fixture; Go/TypeScript suite runners use these faulty execution
targets through the actual adapter, not a prewritten report. The browser controls run in the real
host. Also run `reverse_row_order` and `change_only_decoy` as passing metamorphic controls.
No compiler support is inferred from just a matching golden file, and no target is covered by
another target's reported test result.

- Exactly one match copies its independently arranged value; two decoys each violate a different
  conjunct. Changing only a decoy must not change the copied value.
- Zero matches follows the authored zero-count branch; two matches follow the authored ambiguity
  refusal. An unguarded zero/multiple read produces no transition or event.
- Reverse insertion order and change row identities without changing selector facts: results stay
  equal. A first-row or latest-row implementation fails.
- Guard and value consume the same pre-outcome snapshot, including a creation whose fields would
  otherwise match and a secondary effect changing selector fields.
- Optional copied fields retain absence; wrong entity, missing field, incompatible types, mixed
  selectors and subject reads on creation are refused at their source locations.
- History checks preserve generated-value uncertainty and actual observed authority; no witness
  manufactured by synthesis becomes observed evidence.
- The selected value differs from each decoy and from zero. Native, generated-target and suite
  runtime controls kill a dropped-conjunct selector and a wrong-row copy independently.
- Old formats and struct-shaped `related` fields retain their meaning. Unaffected IR and suite
  bytes remain identical, and every new persisted shape has an explicit version/read-refusal test.

Implement with family F row sets after #285's related-source semantics have composed. Source and
runtime changes integrate serially on the one bundle branch. A named unsupported target is honest
coverage, not closure of a required supported-target acceptance scenario.
