# The head of an ordered selection as a value source (`ess/27`)

Status: design, not implemented (https://github.com/beyond10x/ess/issues/543,
`story:payload-source-reads-the-head-of-an-ordered-selection`). Source format `ess/27`, which
another unit opens; this construct does not open a format of its own. No new suite format, no new
`ValidationCode`. It builds on `filtered-related-reads.md` (the selector and its snapshot) and on a
view's `order_by:` (`crates/specify/ess-domain/src/view.rs:195-273`, `:963`).

## The gap

A command takes the best-ranked element waiting on a side: the `Present` membership with the
highest score, with ties broken by something. Its event publishes the key of the element it took.
`{related: {entity, where, field}}` (`ess/22`) reads one field of the **one** row a selector
selects. Zero or several rows give no value (`crates/specify/ess-domain/src/command/row_set.rs:26-28`,
`crates/verify/ess-conformance/src/interpret/execute/row_set.rs:145-171`), and a side with two
waiting elements selects two. So the author falls back to `{generated: true}`, and the suite checks
that a key is present and typed, not which element was taken.

A minimal reproduction, `ess` 0.59.0, `ess/24`. The selection read validates. Adding an order is
refused, in either spelling:

```yaml
element_key:
  related:
    entity: demo.ranked.Membership
    where: {all: [slot_id == input.slot_id, state == Present]}
    order_by: [element_score desc, membership_id]
    first: element_key
```

```console
$ ess specify validate --path ordered.yaml
ordered.yaml was refused:
  - ordered.yaml: a list is read only as the `via:` of `{related: {via: [<field>, <field>], field: <field>}}`
$ ess specify validate --path requester.yaml     # order_by: [{field: element_score, direction: desc}, …]
requester.yaml was refused:
  - requester.yaml: invalid type: map, expected a string
```

The requester's proposal, labelled as theirs: `{related: {entity, where, order_by: [{field,
direction}, …], first: <field>}}`, with the order required to be total.

## The construct

```yaml
format: ess/27
# …
      - name: picked
        creates: demo.ranked.Pair
        instance: pair_id
        sets:
          element_key:
            related:
              entity: demo.ranked.Membership
              where: {all: [slot_id == input.slot_id, state == Present]}
              order_by: [element_score desc, membership_id]
              first: element_key
```

`{related: {entity, where, order_by, first}}` reads the field `first` names from the **first** row
of the rows `where` selects, ranked by `order_by`. It stands everywhere
`{related: {entity, where, field}}` stands: an event `payload:`, `sets:`, an error payload and an
`affects[].sets:` entry, at any depth of a nested struct source.

- **`entity` and `where`** are the selector exactly as `filtered-related-reads.md` defines it: a
  declared entity, a typed stored-row predicate whose bare fields name the candidate row (its
  identity, declared fields and `state`), `input.<path>` the command input, `subject.<path>` the
  addressed subject before the outcome. All of its checks apply unchanged, including the refusal of
  a `where` that selects every row.
- **`order_by`** is a non-empty list of ranking keys, spelled as a view spells them
  (`view.rs:242-273`): `<field>` or `<field> asc|ascending|desc|descending`, ascending when no
  direction is written. The first key is the most significant.
- **`first`** names the field read: the entity's identity, a declared field or `state`, at its
  declared type. An `Optional<…>` field stays `Optional<…>`, and an absent value is published as
  absent. Assignment and conversion rules are those of `field:`.

`first:` replaces `field:`. They state different contracts, and the key says which: `field:` needs
exactly one selected row, `first:` needs at least one and takes the head.

### Recognition

The shape is recognised the way `{related: {entity, where, field}}` is, by its exact shape
(`crates/specify/ess-domain/src/command.rs:1629-1683`): one key, `related`, holding exactly
`entity` (a qualified name), `where`, `order_by` (a **list** of texts) and `first` (a text).
Recognition happens before the captured `where:` is settled as a nested-mapping leaf
(`command.rs:1536-1545`), and does not depend on the format.

The list is what makes recognition safe. A list under a nested mapping's key was refused at every
format (`command.rs:1507-1513` reads it; `:1851-1854` refuses it outside `via:`), so no document
any `ess` release accepted holds this shape, and recognising it changes no accepted document's
meaning. A text-valued `order_by: element_score desc` is not the shape: a `related:` mapping holding
`entity`, `where`, `order_by` and `first` as texts stays the nested mapping it always was, at every
format, including `ess/27`.

### Spelling: the view's, not the requester's

The requester wrote each key as a mapping, `{field: element_score, direction: desc}`. ESS already
spells a ranking key, in a view's `order_by:`, as one text, `element_score desc`, and serialises it
that way in the IR (`ResolvedView::order_by`, `crates/specify/ess-compiler/src/ir.rs:2025`). Taking
the mapping spelling would give one concept two spellings, which
`.agents/skills/assessing-external-requests/SKILL.md` lists as a red flag. The source reuses the
view's reader (`Ranking::parse`, made `pub(crate)`) and its `Ranking` type in the IR.

## A total order

The head must be the same row for every evaluator: the interpreter, a target under test, the
synthesized expectation. So the order must be **total over the selected rows**, and the only key
ESS can show to be unique is the entity's identity: an entity declares no unique constraint
(`EntitySpec` holds identity, fields, relations, lifecycle, invariants and naming,
`crates/specify/ess-domain/src/entity.rs:926-951`).

- **The last key is the identity**, in either direction. `order_by: [membership_id]` is total. So
  is `[element_score desc, membership_id]`. `[element_score desc, joined_seq]` is refused, even
  where `joined_seq` happens to be unique in practice, because nothing in the model says so. The
  refusal names the tie-break it lacks: `append \`membership_id\` as the last key`.
- **No key follows the identity.** It could never decide anything. This is the same rule a view
  applies to a repeated key (`view.rs:1703-1712`).
- **A struct identity ends a total order** like any other identity. It ranks by its members
  ("Struct identities", below).
- **`ties: unordered` replaces the identity tie-break** where the model leaves ties to the
  implementation ("Declared-unordered ties", below).

When a unique-field construct exists, a key it declares unique may end the order. That is a later
change and does not alter this rule for the identity.

### Struct identities

A struct identity is unique by construction: two rows with equal members are one row. So it ends a
total order. This replaces the earlier refusal, which an adopter's matching specification met as
its first use of the source (`[unsupported_construct] …order_by: \`…SlotMembership\` is identified
by a struct, which has no order`).

- **Rank.** Members in declaration order, the first most significant. Each member ranks by its own
  type's order (the table under "Orderable keys"). A member that is itself a struct ranks by its
  members, recursively. The key's direction applies to the whole identity.
- **Not by name.** A struct's wire form is a mapping. Its keys sort by name, and that order is not
  the declaration order. The interpreter's comparator (`interpret/views.rs::compare`) reads the
  declared members, so `{seq: 9, lane: z}` ranks before `{seq: 10, lane: a}` where `seq` is declared
  first.
- **Members must rank.** An `Optional<…>` member, or one of an unorderable type, is refused at the
  key with the type rules of any key, naming the member path:
  `` `membership_id.waited` is `Duration`, whose values have no rank `` (`type_mismatch`).
- **Only the identity.** A struct field that is not the identity stays refused as a key. It is not
  unique, and no adopter needs it as an earlier key.

### Declared-unordered ties

The identity tie-break states an order. An implementation whose ranking breaks no ties — a heap
that compares the score only, a slot set that keeps equal scores in insertion order — does not have
that order, and a suite synthesized from it fails a conforming implementation. `ties: unordered`
states the model's real contract instead: rows tying on every key are in no order the model fixes.

```yaml
element_key:
  related:
    entity: demo.ranked.Membership
    where: {all: [slot_id == input.slot_id, state == Present]}
    order_by: [element_score desc]
    ties: unordered
    first: element_key
```

The keys are spelled as a view spells them, and the flag sits beside them inside `related:`. It
takes one value. Without it the total-order rule above stands.

| layer | rule |
| --- | --- |
| validation | the order need not end in the identity; every other key rule holds. A key naming the identity is refused (`conflicting_declaration` at `related.ties`): the identity breaks every tie. A value other than `unordered` is refused (`type_mismatch` at `related.ties`). Below `ess/27` the shape is refused naming `ess/27`, as without the flag |
| IR | `RelatedHead` carries `"ties": "unordered"` after `order_by`; a total order writes no `ties` key, so its bytes are unchanged. `describe` adds "rows tying on every key in no order the model fixes" |
| interpreter | rank as before. Where the first two selected rows tie on every key, the answer is `Undetermined::UnorderedTie`, a capability gap the target reports `unsupported`, and no row is picked. A tie behind a head that ranks first alone decides nothing, and the head is read |
| synthesis | the arrangement is unchanged (S2 the head, neither first nor last inserted), and the reading refuses an arrangement in which two selected rows tie on every key, so no scenario states an order for tying rows |
| author | `ess verify conform author` replays the file's literals through the interpreter, `setup:` rows and acts in order. An act whose head read the replay answers `UnorderedTie` is refused, `ESS-AUTHOR-043`, naming the command, the entity and the keys. The replay stops, refusing nothing more, at the file's first refusal, an input that is not wholly literal, a branch it does not decide or one that mints a value, an answer other than the branch the act names, and from the start where the model declares a binding |
| generated targets, diff, schema | as for the total order; the schema admits `ties` with the pattern `^unordered$` |

### Orderable keys

A key is a bare name: the identity, a declared field or `state`. It is never a dotted path. Its
type, through newtypes, ranks exactly as a view's key ranks in the interpreter
(`crates/verify/ess-conformance/src/interpret/views.rs:180-230`) and in the suite runner
(`crates/verify/ess-conformance/src/runner.rs:3491-3556`):

| type | ranks by |
| --- | --- |
| `Integer`, `Decimal`, `Binary64` | numeric value (signed zeros tie) |
| `String`, `Uuid` | UTF-8 bytes of the text (a `Uuid`'s canonical lower-case text orders as its bytes) |
| `Boolean` | `false` first |
| an enum, `state` | the UTF-8 bytes of the wire spelling, not declaration order, as in views |
| `Timestamp` | the instant, not the text |

The source refuses `Optional<…>` keys: absence has no rank, and the runner calls a null beside a
value undecidable (`runner.rs:3504-3513`). It also refuses `Duration` (its wire text does not order
as its length), `Bytes`, `Json`, structs, unions, `List` and `Map`.

## Meaning

- **Snapshot.** The rows are the store immediately before the branch is selected, the same
  snapshot every guard and value of the command reads (`filtered-related-reads.md`, "One
  selection, two consumers"). A row the outcome creates or changes is never a candidate, including
  the head row itself when an `affects:` of the same outcome moves it.
- **Precedence.** This is a value read. It happens after the branch is selected and changes no
  branch precedence.
- **The head.** Rank the selected rows by the keys in order, each in its direction. The first row
  supplies the value. The order is total, so exactly one row is first.
- **No row.** An empty selection supplies no value, as a selector matching no row does today. The
  outcome then cannot publish a transition, a store change or an event (`filtered-related-reads.md`,
  "Cardinality is explicit"). A model that answers "nothing waiting" says so with its own branch:
  `when_related: {entity, where, exists: false}` with an error, before the reading branch.
- **One row** is its own head. A branch whose guard admits at most one selected row (`count: {eq:
  1}`) validates, but its order is never observed. Synthesis says so (below).

## Validation

Each refusal is reported at the source site, `command.<C>.outcomes.<o>.payload.<Event>.<field>`,
`….sets.<field>`, `….affects[<i>].sets.<field>` or the error-payload path, as the filtered read's
are. A key refusal is reported at `….related.order_by[<i>]`.

| condition | code | message names |
| --- | --- | --- |
| the shape below `ess/27` | `unsupported_format_version` | `ess/27`; hint `declare \`format: ess/27\`` |
| `first:` beside `field:` | `conflicting_declaration` | that `field:` reads exactly one row and `first:` the head; write one |
| `order_by:` with `field:` and no `first:` | `conflicting_declaration` | that an ordered selection is read with `first:` |
| `first:` with no `order_by:` | `missing_declaration` | that the first of an unordered selection is no particular row |
| `order_by:` or `first:` beside `via:` | `conflicting_declaration` | the two alternatives, as the mixed `via`/selector refusal does |
| `order_by: []` | `empty_declaration` | that an empty order ranks nothing |
| a key word that does not parse (`priority upward`, three words) | `type_mismatch` | the `Ranking` reader's message |
| a key that is no field of the entity, or a dotted path | `undeclared_reference` | the key; hint lists identity, fields and `state` |
| a key written twice | `duplicate_declaration` | the view's wording |
| an `Optional<…>` key | `type_mismatch` | that absence has no rank |
| a key of an unorderable type | `type_mismatch` | the type, and the orderable ones |
| the last key is not the identity, without `ties: unordered` | `missing_declaration` | the missing tie-break: `append \`<identity>\` as the last key`; the hint names `ties: unordered` |
| a key after the identity | `unsupported_construct` | that keys after the identity never decide |
| a member of a struct identity without a rank | `type_mismatch` | the member path and its type, as a key of that type is refused |
| the identity as a key beside `ties: unordered` | `conflicting_declaration` | at `related.ties`: the identity breaks every tie |
| `ties:` other than `unordered` | `type_mismatch` | at `related.ties`: `ties:` takes `unordered` |
| `first:` names no field | `undeclared_reference` | as `field:` does (`value_expression.rs:1760`) |
| `first:`'s type does not fill the target | `type_mismatch` | as `field:` does |
| every selector refusal of `{entity, where, field}` | unchanged | unchanged (`value_expression.rs:1685-1775`) |

Every code exists already (`crates/specify/ess-primitives/src/error.rs:242`). The new messages move
the string-located diagnostic counts that `docs/design/review-typed-diagnostics.md` pins, so the
domain unit runs `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics`.

A row-set guard keeps refusing `order_by:`: `exists`, `count` and `forall` test a set, and an order
cannot change their answer.

## Format and compatibility

- **Source format `ess/27`.** Below it the shape is refused naming `ess/27`. Before `ess/27` no
  reading of the shape was ever accepted (above), so the change is to a refusal's message, not to a
  document's meaning. A document below `ess/22`, read with filtered reads off
  (`row_set.rs:540-555`), may keep the refusal it has today, provided it stays refused.
- **Old-reader controls.**
  - The `ess/27` example, rewritten `format: ess/26`, is refused with `unsupported_format_version`
    naming `ess/27`.
  - Rewritten `format: ess/21`, it is still refused (any code).
  - A struct target whose fields are `related: {entity, where, order_by, first}`, all texts,
    compiles to the same IR bytes at `ess/26` and at `ess/27`.
- **IR.** A new `ResolvedPayloadValue::RelatedHead`, tag `related_head`:
  `{selection: {entity, where}, order_by: ["element_score desc", "membership_id asc"], field,
  type_ref}`. `order_by` reuses `Ranking`, written as a view's is. `RelatedSelection` and its bytes
  do not change. A specification without the source keeps its canonical IR bytes. `EssIr::format`
  already records the source format (`ir.rs`). The IR is serialised, never read back by ESS, so
  there is no IR reader to fence.

  A new variant, rather than an `order_by` field on `RelatedSelection`, is deliberate. Thirteen source
  files match `ResolvedPayloadValue` exhaustively (listed under the units). A field on the existing
  variant would let every `RelatedSelection { .. }` arm take a head read for an exactly-one read
  without anyone deciding it. A new variant makes each one decide.
- **Suite.** The head value is asserted through the existing event-payload and row assertions. No
  step kind is added, so `ess-conformance/*` keeps its version.
- **Schema.** `RawPayloadSource` gains one alternative, so `cargo xtask schema` regenerates
  `schemas/generated/ess.schema.json` and `task projection-check` holds it.
- **Diff.** `ess verify diff` compares payload sources through `ResolvedPayloadValue::describe`
  (`crates/verify/ess-diff/src/diff.rs:1211`, `:2231`). The head's description names the entity,
  the field, the order and the predicate: "`E.f` of the first row by `k1 desc, id asc` whose stored
  fields satisfy `P` before the outcome". Changing the order, the field or the selector is therefore
  a visible behaviour change, never an erased one.
- **Migration.** None. No existing document changes meaning.

## Interpreter and history checker

The interpreter reads the members exactly as the filtered read does (`Reader::members`,
`interpret/execute/row_set.rs:69`), then ranks them with the interpreter's view comparator
(`interpret/views.rs::compare`, made `pub(crate)`). It is type-directed, ranks `Timestamp` by
instant and is the function views already use, so the head and a view's order cannot disagree.

| members | result |
| --- | --- |
| none selected | `Undetermined::NoValue`: "a field of the first `<E>` the selector selects: none is" |
| every possible member's membership and every key value known | the first by the order; `first` read as `values::selected` reads `field` (identity, stored field, absence) |
| a possible member whose membership is unknown, or whose key value is abstract (history) | `Undetermined::Undecidable`, naming the selector and the order |

The history rule is sound and deliberately weak. A stronger proof may decide more cases only by
showing the same head for every completion, as `filtered-related-reads.md` requires of row sets.

## Synthesis

A scenario for a branch that reads a head arranges the selector's rows through the declared
creating commands, as `synthesize/row_set.rs` does today (decoys per conjunct before and after the
selected rows, the branch decided again over the walked rows). It then needs more than one
selected row:

- **Count.** Try three selected rows, then two. Only then fall back to the counts the row-set
  strategy uses (`COUNTS = [1, 2, 0, 3]`, `row_set.rs:60`) for branches that do not read a head.
- **The middle head.** Create selected rows S1, S2 and S3 in that order, steering the first key so
  that S2 ranks first, S1 second and S3 last. The head is then not the first-inserted row, not the
  last-inserted row, not the head under the first key's reversed direction (S3), and not the first
  or last by identity alone: identities are minted in creation order, `next()`, `row_set.rs:1029`.
  One scenario kills five wrong targets.
- **The tie-break.** Where the order has a key before the identity and the identity is chosen by
  the scenario's input, add a row T that ties S2 on every key before the identity and loses on the
  identity. Create it before S2, with an identity that ranks after S2's in the tie-break's
  direction. A target that breaks ties by insertion order, or in the wrong direction, then answers
  T.
- **Later keys.** For each further key before the identity, add a tie row that agrees with the
  head on the keys before it and loses on that key, where the creating inputs can steer it. A key
  that cannot be steered is reported as a named witness gap for that key. It is not silently
  unobserved, and the scenario still stands.
- **Rows already selected.** Where the steps before S1 already leave rows the selector selects —
  the related rows another selector's `exists` test placed in the scope of a head that reads a
  published value (beyond10x/ess#618) — S2 is steered above every one of them, and no arranged row
  ties one on every key. Otherwise the head is the first such row, which is the first created, or
  under `ties: unordered` there is no head at all. Where no creating input ranks S2 above them,
  the rows arranged without regard to them stand with the named gap below, and a branch that then
  has no arrangement of several rows is refused with that gap rather than as a branch admitting one
  row.
- **A declared range.** Where a key's declared range leaves no value beyond S1 on the side S2 must
  take, S1 is dealt the distinction at the other end of the pool and the rows are steered from
  there. Only an arrangement that otherwise fails is dealt again.
- **Owned rows.** Where the selected entity is owned (`kind: owns`) and the selector's filter, with
  a published value written in, compares the owner link with the literal identity of an owner row
  the steps already hold — the head row of another selector — each row is created naming that
  owner as the steering chose it, and no owner of its own is arranged, which would give it a link
  no such filter selects (beyond10x/ess#621). The same holds for the related rows an `exists` test
  places and for the rows ranked above the head under another selected owner. A decoy refuting
  the owner link is left out, as no owner the steps hold is steered for it.
- **Distinct values.** The `first` value of the head differs from every other selected row's, from
  every decoy's and from zero, so copying any other row is visible.
- **Checked, never assumed.** The head is computed over the rows the steps leave (`walk`), with the
  interpreter's comparator, and the expected value is that row's.

Named gaps (`RefusalCause::NoWitness`), each with its reason:

| situation | gap |
| --- | --- |
| no creating command's input steers the first key | "the ranked head is not arranged: no creating command writes `<key>` from its input" |
| the reading branch admits at most one selected row | "the ranked head is not observable: the branch reading it admits at most one selected row" |
| no creating input ranks the head above the rows another selector's test already placed in its scope | "the ranked head is not arranged above the rows another selector's test places in its scope: no creating input ranks it above them without a tie" (a standing gap where the scenario stands without it; the branch's refusal where no arrangement of several rows stands) |
| the head would be decided by an identity the target mints | "the ranked head is decided by an identity the target mints, which no scenario orders in advance" (arranged without ties instead, where the earlier keys decide) |

### Several selectors in one command

The issue's own command takes the head of **two** sides, left and right, in one outcome: two
selectors that differ by `side == Left` and `side == Right`. Synthesis today arranges one selector
per command and refuses every branch of such a command (`selection_of`, `row_set.rs:142-168`).
Measured with `ess` 0.59.0 on the two-sided example written with `{related: {entity, where,
field}}`:

```console
refused: refusal[ESS-SYNTH-001]: outcome demo.ranked.Pick/picked has no scenario `demo.ranked.Pick/outcome/picked`
  no witness: `demo.ranked.Pick` is `row set`, which reads more than one row set: a scenario arranges the rows of one selector per command in this cut
```

So the head alone does not close https://github.com/beyond10x/ess/issues/543. Unit U5 below lifts
the limit for every selector form:

- arrange the selectors in the order they first appear in the command;
- decide every branch again over all walked rows, so a decoy of one selector that another selects
  counts as that selector's row;
- keep copied values distinct across all selectors' selected rows and decoys;
- for an `exists: false` refusal per selector, arrange the other selectors non-empty, so the
  declared refusal, and no earlier one, answers.

## Targets

| target | disposition in this story |
| --- | --- |
| native interpreter | executes the head (above) |
| history checker | sound head or `Undecidable` (above) |
| synthesis | arranges the witness, or a named gap (above) |
| authored scenarios | run through the interpreter, with no change of their own |
| Go and TypeScript suite runtimes | no change: they compare published values |
| generated Rust and Go | **named obligation**: "a read of the first row of an ordered selection (`{related: {entity, where, order_by, first}}`) for `<field>`" in `ess-synth/src/determined.rs`, beside the filtered read's (`:507-513`). Rust and Go behaviour emitters get unreachable arms only. |
| web player | reads no state, so it returns `None`, as the filtered read does (`ess-conformance/src/web.rs:262`) |
| Entity Runtime | the named row-set limitation, `ROW_SET` (`ess-entity-runtime/src/subset.rs:101`, `:120`) |
| OpenAPI and AsyncAPI | no change: the event's wire shape is unchanged |
| service contract | the value's `type_ref`, as for the filtered read (`ess-service-contract/src/lib.rs:467`) |
| generated docs and diff | `describe` (above) |
| linearizer | reads other rows: `true` (`ess-conformance/src/linearize.rs:390`) |

Generated Rust and Go stay owed here, which departs from the story's acceptance (see
"Disagreements"). Generated storage enumerates no rows by a selector, for guards or values
(`determined.rs:202-209`, `:507-513`). The issue's command also carries an `exists: false` row-set
guard, which keeps it owed whatever the value read does. Measured on the single-side example,
`ess` 0.59.0:

```console
| command behaviour | `demo.ranked.Pick` | kept an obligation by a guard over the rows a selector selects (`when_related: {entity, where, exists | count | forall}`), in `empty` | …
```

Generating only the head read would generate no additional command. Generated selector enumeration
belongs to one story that covers row-set guards, filtered reads and the head together. The storage
port already enumerates "every stored instance, in the order the store keeps them" for view
queries, and `view_query.rs` already renders an order. That story is proposed, not written, here.

## Not in this design

- **An effect on the head row.** Removing or moving the popped membership needs an `affects:` or
  `instances:` that selects the head, and `where:` cannot read a value source. The requester did
  not ask for it. It is a candidate follow-up, with the same order and totality rules.
- **A guard over the head** ("the head's score is above 10"). Row-set guards test sets. No need has
  been stated.
- **Mutation operators** for the mutation audit (flip a key's direction, drop the tie-break, take
  insertion order). The synthesis controls above already kill these targets. Operators are a
  separate, optional unit.
- **A `Timestamp` key in generated targets.** Generated view queries do not rank instants
  (`ess-synth/src/view_query.rs:333-362`). It is moot while generated targets owe the source.

## Fit review

The story carries questions 1-4. All seven are answered here so the store can take them whole.

1. **Need.** A command takes the best-ranked of several stored rows, by a stated order with a
   stated tie-break, and publishes or stores a field of that row. ESS cannot say which row, so the
   suite cannot check it. Minimal reproduction: "The gap", above (`ess` 0.59.0 output). The
   requester's syntax is quoted there, labelled as theirs.
2. **Class: gap.** No construct reads a ranked row. The `ess/22` filtered read is documented to
   refuse several rows (`row_set.rs:26-28`), so this is not a defect of it.
3. **Already expressible? No.** `order_by` exists only on views (`view.rs:963`). In a value source
   a list is refused (`command.rs:1851-1854`; output above). A view read is not a value source. A
   command input naming the chosen key makes the scenario author pick the row the implementation
   picks. That is https://github.com/beyond10x/ess/issues/544's problem, and it checks nothing
   about the choice.
4. **Fit.**
   - *Vocabulary.* The selector is the filtered read's, and the ranking keys are the view's,
     spelled the same way (the requester's mapping spelling is changed, above).
   - *Composition.* Guards are unchanged: a row-set guard still refuses `order_by:`, because an
     order cannot change a set test. Bindings read no store. `affects[].sets` and error payloads
     admit the source wherever the filtered read stands.
   - *Siblings.* Ordering now exists where several rows can be read: views, and this source.
     `{related: {via}}` reads one row by identity, where an order means nothing. Effects and guards
     over the head are named under "Not in this design".
   - *Targets.* Every target handles the head or refuses it by name (the targets table). A new IR
     variant forces each exhaustive match to decide.
5. **Second, unrelated adopter.** A price list reads the price in force for an order line, the
   latest effective price of the product:
   `{related: {entity: Price, where: {all: [product_id == input.product_id, effective_from <= now]},
   order_by: [effective_from desc, price_id], first: amount}}`. A support desk assigning the oldest
   ticket of the highest priority writes `order_by: [priority desc, opened_seq, ticket_id], first:
   ticket_id`.
6. **Cost.**
   - Format: no bump of its own; it rides `ess/27`.
   - Keys: `order_by` (already a view keyword) and `first`, both inside `related:` only.
   - Diagnostics: no new `ValidationCode`, about twelve new messages.
   - IR: one additive variant, `related_head`, and old bytes stay unchanged. Suite: no new version.
   - Schema: one alternative. Diff: through `describe`. Generated API: unchanged. Migration: none.
7. **Alternatives.**
   - (a) **Change nothing.** The fields stay `{generated: true}`, and the adopter keeps 3 unmapped
     fields and 2 known-failure scenarios.
   - (b) **`order_by:` beside `field:`, no new key.** Rejected: a sibling key would change
     `field:`'s exactly-one contract, and every existing `RelatedSelection { .. }` arm would
     silently treat a head as an exactly-one read.
   - (c) **`{max_by: <field>}` / `{min_by: <field>}`.** Rejected: one key, and no way to state a
     tie-break, so the head would not be total.
   - (d) **A view read as a value source** (`{view: V, first: f}`). Rejected: views take `param.`
     operands rather than `input.` and `subject.`, and this would couple views to command
     snapshots for one consumer.
   - (e) **The requester's shape** (chosen, with the view's spelling of a key, the identity as the
     required tie-break, and `first:` as proposed).

## Disagreements with the story's decisions

| story says | this design | why |
| --- | --- | --- |
| "Syntax as the requester proposed (`order_by` list of `{field, direction}`)" | the view's spelling, `[element_score desc, membership_id]` | one concept, one spelling: `view.rs:233-273`, `ir.rs:2025`; the skill's first red flag |
| totality by "identity fields, or a unique constraint", "the last key, or a prefix of keys" | the identity, last; nothing after it | ESS declares no unique constraint (`entity.rs:926-951`); a prefix ending in the identity is the same rule; keys after it never decide |
| acceptance `generated_behaviour_reads_the_ranked_head` (Rust, Go) | a named obligation, tested as `generated_behaviour_names_the_ranked_head_owed` (Rust, Go); generated execution is a separate story for every selector form | generated storage enumerates no rows by a selector (`determined.rs:202-209`, `:507-513`); the issue's command is owed by its guard anyway (measured above) |
| scope does not mention several selectors | U5: synthesis arranges several selectors per command | the issue's command has two heads; `row_set.rs:142-168` refuses it (measured above) |
| scope: `wire.rs`; `synthesize/related*`, `interpret/execute/related.rs`; `{rust,go}/behaviour.rs` | `command.rs`; `synthesize/row_set.rs`, `interpret/execute/row_set.rs`, `values.rs`; `determined.rs` | `wire.rs` checks wire namespaces (`wire.rs:1`); `related*` is the identity-addressed `{via}` read; the behaviour emitters only get unreachable arms |

## Decisive acceptance

Each test is written first and seen red.

- `crates/specify/ess-domain/tests/ordered_related_source.rs`:
  - `an_ordered_head_source_validates_at_ess_27`, `an_order_without_a_unique_tie_break_is_refused`
    and `first_beside_field_is_refused` (the story's three);
  - `the_head_shape_below_ess_27_is_refused_naming_ess_27`,
    `a_text_order_by_keeps_its_nested_mapping_meaning`, `an_optional_order_key_is_refused`,
    `a_key_after_the_identity_is_refused`, `first_without_order_by_is_refused` and
    `an_order_key_the_entity_does_not_hold_is_refused`.
- `crates/specify/ess-compiler/tests/ordered_head_ir.rs`:
  - `the_head_compiles_to_related_head_with_its_rankings`;
  - `a_model_without_the_head_keeps_its_ir_bytes`.
- `crates/verify/ess-conformance/tests/ordered_related_source.rs`:
  - `the_interpreter_publishes_the_ranked_head`, `an_empty_selection_publishes_nothing` and
    `an_abstract_key_leaves_the_head_undecidable`;
  - `synthesis_arranges_a_witness_whose_head_is_not_the_first_inserted_row`;
  - `a_target_taking_the_first_inserted_row_fails_a_scenario`, and the same against targets taking
    the last-inserted row, the reversed first key, and ties by insertion order;
  - `a_head_over_at_most_one_row_is_a_named_gap`.
- `crates/verify/ess-conformance/tests/row_set_several_selectors.rs`:
  - `a_two_sided_pick_is_witnessed_on_each_branch` (the `{entity, where, field}` form);
  - `a_two_sided_ranked_pick_is_witnessed` (after U4).
- `crates/generate/ess-synth/tests/ordered_head_owed.rs`: `generated_behaviour_names_the_ranked_head_owed`
  (Rust and Go).
- `crates/generate/ess-entity-runtime/tests/row_sets.rs`: a head read is refused with the
  row-set limitation.
- U7, struct identities and declared-unordered ties:
  - `ess-domain/tests/ordered_related_source.rs`: `a_struct_identity_ends_a_total_order`,
    `a_struct_identity_member_without_a_rank_is_refused`,
    `ties_unordered_admits_an_order_without_the_identity`,
    `ties_unordered_beside_the_identity_is_refused`, `a_ties_value_other_than_unordered_is_refused`,
    `ties_unordered_below_ess_27_is_refused_naming_ess_27`;
  - `ess-compiler/tests/ordered_head_ir.rs`: `ties_unordered_is_recorded_beside_the_rankings`;
  - `ess-conformance/tests/ordered_related_source.rs`:
    `a_struct_identity_breaks_a_tie_by_its_members_in_declaration_order`,
    `the_interpreter_answers_a_declared_unordered_tie_unsupported`,
    `synthesis_arranges_no_tie_under_ties_unordered` (a target breaking ties by insertion order
    conforms, one taking the first inserted row does not) and
    `author_refuses_a_scenario_arranging_a_tie_the_order_leaves_unordered`.

## Implementation units

Each unit is one agent and one branch. "Overlap with u545" is the value-expression lane
(https://github.com/beyond10x/ess/issues/545: `{generated: true, present: true}` and `{payload:
<Event>.<field>}`), which edits the same payload-source code.

| unit | what | files | depends on | overlap with u545 |
| --- | --- | --- | --- | --- |
| U0 | open `ess/27` | `ess-domain/src/system.rs` (`V27`), format list, CHANGELOG stub | — | **shared**: one unit opens it for #542, #543 and #545 |
| U1 | shape, recognition, validation | `ess-domain/src/command.rs` (`PayloadSource::RelatedHead`, `RawPayloadSource::Head`, the visitor, `TryFrom`, the raw back-conversion near `:2072`, the arms at `:1250`, `:1295`, `:3846`); new `ess-domain/src/command/ordered_head.rs` (key, totality and type rules); `ess-domain/src/command/value_expression.rs` (dispatch, labels `:406`, `:455`, `:746`); `ess-domain/src/view.rs` (`Ranking::parse` visibility only); `schemas/generated/ess.schema.json` (`cargo xtask schema`); new test `ess-domain/tests/ordered_related_source.rs` | U0 | **high**: `command.rs` raw source, visitor and `TryFrom` and `value_expression.rs` are u545's surfaces too; `ess.schema.json` regenerated by both |
| U2 | IR variant, resolver, every consumer's named arm | `ess-compiler/src/ir.rs` (variant, `describe`); `ess-compiler/src/resolve.rs` (`head_field`, arms `:3029`, `:5440`); `ess-diff/src/diff.rs`; `ess-conformance/src/{web.rs,linearize.rs,synthesize.rs,synthesize/aggregate.rs,synthesize/set_effects.rs}` (named gap until U4); `ess-conformance/src/interpret/execute.rs` (a named `Undetermined` until U3); `ess-service-contract/src/lib.rs`; `ess-entity-runtime/src/{lib.rs,subset.rs}`; `ess-synth/src/{determined.rs,rust/behaviour.rs,go/behaviour.rs}`; new tests `ess-compiler/tests/ordered_head_ir.rs`, `ess-synth/tests/ordered_head_owed.rs`, a case in `ess-entity-runtime/tests/row_sets.rs` | U1 | **high**: `ir.rs` `ResolvedPayloadValue` and `describe`, `resolve.rs`, and every exhaustive-arm file above if u545 adds a variant (likely, for `{payload: …}`); `determined.rs` and the behaviour emitters, which u545 edits for the present mint |
| U3 | interpreter | `ess-conformance/src/interpret/execute.rs` (the real `value_at` arm); `interpret/execute/row_set.rs` (`Reader::head`); `interpret/execute/values.rs` (`selected` for both variants); `interpret/views.rs` (`compare` visibility); new fixture `ess-conformance/tests/fixtures/ordered-related-head.yaml`; new test `ess-conformance/tests/ordered_related_source.rs` (interpreter cases) | U2 | **medium**: `value_at` in `execute.rs` sits beside the `Generated` arm u545 changes; `values.rs` low |
| U4 | synthesis of the head | `ess-conformance/src/synthesize/row_set.rs` (`reads`, `selected_values`, counts for head branches, `selection_of` including the head); new `ess-conformance/src/synthesize/ordered_head.rs` (arrangement, tie rows, gaps); `ess-conformance/src/synthesize.rs` (replaces U2's gap arm); the synthesis cases in `ordered_related_source.rs` | U3, U5 | **low**: one arm in `synthesize.rs` |
| U5 | synthesis: several selectors in one command, every form | `ess-conformance/src/synthesize/row_set.rs` (`selection_of` to a list, arrangement per selector, joint consistency); new test `ess-conformance/tests/row_set_several_selectors.rs` | — (works on `ess/22`) | none |
| U6 | documentation | `website/docs/guides/specify/values-and-views.md`; `website/docs/reference/spec-versions.md` (the `ess/27` row); `website/docs/reference/entity-runtime-lowering.md`; `CHANGELOG.md`; this page's status | U4 | **medium**: the same `ess/27` row and CHANGELOG section |

Order:

- U5 can start at once, beside U0-U3: it touches synthesis only and works on the `ess/22` form.
- U1 → U2 → U3 is strictly serial, because each needs the previous one's types.
- U4 follows U3 and U5. Both touch `synthesize/row_set.rs`, so U5 lands first.
- U6 closes the story.

U1 and U2 should not run beside u545's domain and IR units. Land one lane's U1 and U2
equivalents, then rebase the other. Otherwise the raw-source visitor and the
`ResolvedPayloadValue` arms conflict in every file listed.

Gates per unit (no full gate):

| unit | gates |
| --- | --- |
| U1 | `ess-domain` and `ess-cli` `test -p`/`clippy -p`; `task test-xtask`; `cargo test -p ess-compiler --test typed_diagnostics`; `cargo xtask schema` then `task projection-check` |
| U2 | `ess-compiler`, `ess-cli`, `ess-diff`, `ess-synth`, `ess-entity-runtime`, `ess-service-contract`; `ess-conformance` built, with one `--test` binary run |
| U3, U4, U5 | `ess-conformance` one `--test` at a time, plus `ess-cli` |
| U6 | `task test-xtask`; `task site-build` if the website changed |

Disk: each `ess-conformance` unit builds the largest test target in the workspace. The story's own
estimate is 10G per unit.
