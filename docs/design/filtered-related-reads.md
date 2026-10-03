# Filtered related reads (#299, coordinated ess/22)

Status: binding design for the accepted remaining bundle; implementation and independent review
are pending. This extends the family F row-set guard design shared by #228 and #237, and the
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

The interpreter and history checker execute the shared selector with pre-outcome authority.
Generated Rust and Go behavior must execute it where the target advertises support, or produce
the existing named generation obligation before emitting runnable behavior. Live web behavior
inherits the generated Rust disposition. Go and TypeScript conformance runtimes execute the
resulting suite observations. Legacy browser replay is not behavior evidence. Entity Runtime
lowering must report its named cross-instance limitation until it has the required query and
atomic command authority; that external limitation is not treated as successful conformance.
OpenAPI/AsyncAPI retain the command wire shape. Generated documentation describes selection and
cardinality. Diff treats changing the selector or source field as a behavior change using an
explicit classified rule; it cannot erase the source as an unknown expression. Explorer,
authoring, and synthesis must either handle the selector or return their named unsupported or
no-witness result. Required acceptance scenarios cannot pass by skipping those operations.

## Decisive acceptance

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
