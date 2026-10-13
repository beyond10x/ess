# A selector reads a value its own outcome publishes (`ess/25`)

Status: design, not implemented (https://github.com/beyond10x/ess/issues/549). Source format
`ess/25`, unreleased at the time of writing (`CHANGELOG.md` `[Unreleased]`, `FormatVersion::V25`,
`crates/specify/ess-domain/src/system.rs:355`). This page opens no format of its own. No new suite
format, no new `ValidationCode` and no new source keyword. It builds on `ordered-related-head.md`
(https://github.com/beyond10x/ess/issues/543, the head of an ordered selection) and on
`minted-value-sources.md` (https://github.com/beyond10x/ess/issues/545, `{payload: <Event>.<field>}`
in `sets:`). It composes with #543 U7 (struct identities in an order, declared-unordered ties),
which lands first.

## The gap

A matching command picks the best slot of a group, ranked by score, then takes the head of each
side **of that slot**, ranked by element score. Both reads are heads of ordered selections. The
inner selector's `where:` must name the slot the outer head chose. A selector reads its candidate
row bare, the input under `input.`, the addressed subject under `subject.` and `now`
(`crates/specify/ess-domain/src/command/row_set.rs:12-16`). It cannot read another value of the
same outcome. `ordered-related-head.md` lists this under "Not in this design" (`:336-339`).

Minimal reproduction, the binary built at `3fdd9479e` (`ess 0.59.0`, `format: ess/25`), a
specification written for this page with entities `Slot`, `Membership` and `Pair`:

```yaml
payload:
  demo.nested.PairMade:
    pair_id: input.pair_id
    slot_id:
      related:
        entity: demo.nested.Slot
        where: {all: [group_id == input.group_id, state == Open]}
        order_by: [score desc, slot_id]
        first: slot_id
    left_key:
      related:
        entity: demo.nested.Membership
        where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Left, state == Present]}
        order_by: [element_score desc, membership_id]
        first: element_key
```

```console
$ ess specify validate --path nested.yaml
nested.yaml was refused:
  - [unobservable_fact] command.demo.nested.MatchInGroup.outcomes.paired.payload.demo.nested.PairMade.left_key: `payload.demo.nested.PairMade.slot_id` reads `payload`, which is not a declared observable root
  - [unobservable_fact] command.demo.nested.MatchInGroup.outcomes.paired.payload.demo.nested.PairMade.right_key: `payload.demo.nested.PairMade.slot_id` reads `payload`, which is not a declared observable root
```

These are the only refusals, so the path parses and only the root is unknown. The workaround is
to make the caller name the slot (`slot_id == input.slot_id`). That validates (`demo v1 — 1
file(s), valid`), and synthesis writes a scenario for it. But the inner rows are then scoped by a
value the scenario author chose, which says nothing about which slot the implementation picked.
That is https://github.com/beyond10x/ess/issues/544's problem.

The requester's proposal, labelled as theirs: a `where:` operand `{payload: PairMade.slot_id}`,
written as a mapping entry `where: {slot_id: <operand>}`, "or a named intermediate".

## The construct

```yaml
format: ess/25
# …
        where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Left, state == Present]}
```

`payload.<Event>.<field>` in a selector's `where:` is the value **this outcome publishes** in
`<field>` of `<Event>`. `<Event>` is the qualified event name, as in `{payload: <Event>.<field>}`
(`minted-value-sources.md`). The path may continue into the members of a struct field:
`payload.<Event>.<field>.<member>…`. It is an operand like `input.<path>` and `subject.<path>`, on
either side of a comparison, typed at the event field's declared type. An `Optional<…>` field
stays `Optional<…>`, and the existing rules for an `Optional` operand apply unchanged: a selector
comparing with an `Optional` input validates today (measured with the same binary).

### Where it stands

| site | admitted | why |
| --- | --- | --- |
| `where:` of `{related: {entity, where, field}}` and of `{related: {entity, where, order_by, first}}`, in a `payload:` field, a `sets:` entry or an `affects[].sets:` entry, at any struct depth | yes | a value read, after the branch is selected |
| `when:`, `when_subject:`, `when_related:` (identity and row-set, `where:` and `forall:`) | no, `unsupported_construct` | a guard decides the branch, and nothing is published until a branch is chosen |
| `affects[].where`, `instances:` | no, `unsupported_construct` | an effect on the head row is the follow-up of `ordered-related-head.md` ("An effect on the head row") and composes with https://github.com/beyond10x/ess/issues/547 (an `affects:` addressing a row by its identity). The operand and the IR field below are built to be reused there. |
| an error payload's sources | no, `unsupported_construct` | an error outcome fills no event payload to read, as `{payload: …}` is refused there (`minted_value.rs:249`) |

### What it may read

The read field must be one this outcome **emits and fills** under `payload:`, with a source
(`minted_value.rs:265-339`, the same lookup). Its source must be a **snapshot value**: one decided
by the request and the store before the outcome.

| read field's source | admitted |
| --- | --- |
| `input.<path>`, a literal, `{subject: …}`, `{caller: …}`, `{related: {via …}}`, `{related: {entity, where, field}}`, `{related: {entity, where, order_by, first}}`, `{input: …, else: <literal or input>}`, a struct of these (reached by the member path) | yes |
| `{generated: true}`, `{generated: true, present: true}`, `{input: …, else: {generated: true}}` | no: a mint is chosen by the implementation, so no evaluator knows it before the outcome and no scenario can arrange rows against it |
| `{response: …}`, `{count: changed}` | no: they are known only after the call or the effects |
| the field `instance:` publishes, on a creating outcome | no: hint `read the input it is published from, \`input.<field>\`` |

### The name `payload`

The candidate row's bare fields are resolved first, as `subject` is
(`crates/specify/ess-domain/src/command/set_effects.rs:1351-1357`). An entity that declares a field
named `payload` keeps reading `payload.<member>` as that field, at every format. A selector over
such an entity cannot read a published value. Where the typed check then fails, its hint says so.
In the IR the operand never depends on that rule (see "IR"): the reads are listed explicitly.

Measured beside this: the interpreter routes `subject.<path>` to the addressed subject even where
the candidate entity declares a field named `subject`
(`crates/verify/ess-conformance/src/interpret/execute/set_effects.rs:460-466`). The domain types the
same path as the candidate's field. This is inferred from the code and was not run. It is a
pre-existing mismatch, not fixed here, and it is the reason the published reads are explicit in the
IR.

## Meaning

- **One value.** The operand is the value the outcome publishes. It is resolved once per outcome,
  and the event, any `{payload: …}` stored copy and every selector that reads it hold that one
  value. This extends `Work::published`
  (`crates/verify/ess-conformance/src/interpret/execute.rs:1838-1841`), which already shares one
  value between the event and `sets:`.
- **One snapshot.** The outer selector and the inner one both read the store immediately before
  the branch is selected (`filtered-related-reads.md`, "One selection, two consumers"). Nothing the
  outcome writes is read.
- **Evaluation order.** The order is the dependency order of the outcome's payload fields, not the
  declaration order. A field is resolved after every field its selectors read. Payload fields that
  depend on nothing keep today's order.
- **No value.** If the read field supplies no value (an empty selection for a head, zero or several
  rows for a `field:` read), the reading source supplies none either. The outcome then cannot
  publish, as for an empty selection (`ordered-related-head.md`, "No row").
- **Absent.** An `Optional` field published absent is an absent operand. The comparison treats it
  as it treats any absent operand.
- **Precedence.** No change. These are value reads, made after the branch is selected.

### Cycles

The payload fields of one outcome form a graph. Each field is a node, and a selector anywhere in a
field's source that reads `payload.E.f` is an edge to `(E, f)`. `sets:` and `affects[].sets:`
entries read nodes but are not nodes themselves, because nothing can read them. The graph must have
no cycle, a self-loop included. Each cycle is reported once, at its first field in declaration
order, as `self_reference` (`crates/specify/ess-primitives/src/error.rs:336-337`, "Something
references itself where that cannot mean anything"). The message names the chain:
"`PairMade.left_key` reads `PairMade.slot_id`, which reads `PairMade.left_key`".

## Validation

Each refusal is reported at the source site (`….payload.<Event>.<field>`, `….sets.<field>`,
`….affects[<i>].sets.<field>`), as the selector's own refusals are.

| condition | code | message names |
| --- | --- | --- |
| below `ess/25`, where the path after `payload` names an event this outcome fills and one of its fields | `unsupported_format_version` | the operand; hint `declare \`format: ess/25\`` (today: `unobservable_fact`, measured above) |
| below `ess/25`, any other `payload.` path | `unobservable_fact` | unchanged |
| the operand in a guard | `unsupported_construct` | that a guard decides the branch before anything is published |
| the operand in `affects[].where` or `instances:` | `unsupported_construct` | that an effect selecting by a published value is not admitted in this cut |
| the operand in an error payload source | `unsupported_construct` | as `{payload: …}` is refused there |
| `<Event>` is not emitted and filled by this outcome | `undeclared_reference` | the events the outcome fills (the wording of `minted_value.rs:277-294`) |
| `<field>` is not a field of `<Event>`, or is given no source | `undeclared_reference` | as `minted_value.rs:296-323` |
| the read field's source is not a snapshot value | `unsupported_construct` | the source kind, and why it is not decided before the outcome |
| the read field is the created identity | `unsupported_construct` | hint `input.<field>` |
| a cycle, including a field reading itself | `self_reference` | the chain |
| a member path that names no member, a type that does not compare | unchanged | the predicate checker's existing messages |

Every code exists already. The new messages move the string-located diagnostic counts that
`docs/design/review-typed-diagnostics.md` pins, so the domain unit runs `task test-xtask` and
`cargo test -p ess-compiler --test typed_diagnostics`.

## Format and compatibility

- **Rides `ess/25`**, while that format is unreleased. If `ess/25` ships in a release before this
  lands, this construct needs the next format. That is the coordinator's call at that point.
- **No document changes meaning.** Below `ess/25` a `payload.` path is either the candidate's own
  `payload` field, unchanged, or refused. At `ess/25` it is the candidate's field wherever the entity
  declares one.
- **Old-reader controls:**
  - the nested example rewritten `format: ess/24` is refused with `unsupported_format_version`
    naming `ess/25`;
  - a selector over an entity with a struct field `payload`, reading `payload.<member>`, compiles
    to the same IR bytes at `ess/24` and at `ess/25`.
- **IR.** `ResolvedRowSelection` (`crates/specify/ess-compiler/src/ir.rs:1379-1385`) gains
  `reads: Vec<ResolvedPublishedRead>`, each `{event, field, type_ref}`, in dependency order. It is
  omitted when empty (`skip_serializing_if = "Vec::is_empty"`), so every existing model keeps its
  bytes. The predicate keeps the path as written (`payload.demo.nested.PairMade.slot_id`). The
  explicit list lets an evaluator route the `payload` namespace without consulting the entity's
  fields, so the mismatch noted under "The name `payload`" cannot recur. A guard's selection never
  carries reads. The IR is serialised and never read back by ESS, so there is no IR reader to fence.

  Putting the list on the selection rather than on `RelatedHead`/`RelatedSelection` is deliberate.
  The reads are a property of the predicate, and the selection is the type an `affects:` selector
  would share in the follow-up. It is constructed in three places only
  (`crates/specify/ess-compiler/src/resolve.rs:3152`, `:3207`, `:5469`).
- **Suite.** No step kind is added, so `ess-conformance/*` keeps its version. Values are asserted
  through the existing event-payload and view-row assertions.
- **Schema.** None. The operand is inside predicate text, so `RawPayloadSource` and
  `schemas/generated/ess.schema.json` do not change.
- **Diff.** `describe` already prints the selector's predicate (`ir.rs:1672-1694`), so changing
  the operand is a visible change of the reading field. Changing the **read** field's source is
  reported on that field. The reader's line is unchanged, which is accepted: the change is visible
  once, never erased.
- **Migration.** None.

## Interpreter and history checker

- `payload_reads` (`execute.rs:2590`) becomes the set of fields read by `sets:`/`affects[].sets:`
  **and** by every selector's `reads` in the outcome. `publish_reads` (`:2615`, called before any
  effect at `:1982`) resolves them in dependency order, recursively, keeping each in
  `Work::published`. Because `emit` already prefers a published value (`:2712-2725`), the event
  carries the same value. A guard against re-entry answers `NotInterpreted` naming the chain; it is
  unreachable after validation.
- `row_set::Reader::new` (`interpret/execute/row_set.rs:51`) takes the published values.
  `set_effects::Facts` gains `published: Option<&dyn FactSource>`. `Facts::source` routes
  `payload.<E…>.<f>[.<member>…]` to it only for a path that matches a listed read, and otherwise
  keeps today's routing.
- A read field with no value fails its own resolution first (`NoValue`, no step), so the reader is
  never evaluated against a missing operand. An abstract or unknown published value (history)
  leaves the reading selector's membership unknown, and `Reader::head` / `Reader::one` already
  answer `Undecidable` for that (`row_set.rs:145-171` and the head's rule). The rule is sound and
  deliberately weak, as in `ordered-related-head.md`.
- History mode recomputes the operand from the model, so a recorded implementation value is
  checked against it, never trusted.

## Synthesis

Today's machinery arranges several selectors per command in first-appearance order
(`synthesize/row_set.rs:166-195`). It requires each selector to be scoped by an equality with the
input or the subject (`:206-243`), and writes the subject's and the input's values into a predicate
before arranging (`closed`, `:558-583`). On the caller-names-the-slot baseline it wrote a three-heads
scenario (3 slots and 15 memberships created before the command; the event's `slot_id`,
`left_key` and `right_key` asserted).

Changes:

1. **Order.** `selections_of` orders the selectors topologically by `reads`, then by first
   appearance.
2. **Scoped.** An equality between a `String` or `Uuid` field, or a member of a struct identity
   (#463), and a `payload.` operand counts as scoped. This holds where the read field is a head or a
   one-row read of a scoped selector, transitively, because the operand then names a row the
   scenario arranged.
3. **Closed.** After the outer selector's rows are walked (`walk`, `:974`), its value is computed
   with the interpreter's comparator, as for any head. It is written into the inner predicate in
   place of the operand. Where the outer value is an identity the scenario captured
   (`capture_instance`), it is written as that instance reference, which the steps already use for
   arranged identities.
4. **The inner head's rows** are arranged as `ordered-related-head.md` arranges any head: 3 then 2
   selected rows, the middle one first, and tie rows where steerable.
5. **Cross decoys.** For each inner selector and each **other selected outer row** (S1 and S3 of
   the outer arrangement), add one row that meets every conjunct except the operand conjunct and
   carries the other outer row's value there. It is steered to rank **above** the inner head on the
   first key, with a `first` value distinct from the head's. A target that scopes the inner by the
   wrong outer row (first-inserted, last-inserted, reversed key) or drops the scope conjunct then
   answers a cross decoy.
6. **Checked, never assumed.** The branch is decided again over the walked rows (`consistent`,
   `:773`). The expected inner value is computed over those rows with the outer head bound.

Named gaps (`RefusalCause::NoWitness`). In each, the scenario stands without the missing part, as
`ordered-related-head.md` keeps a scenario without an unsteerable tie:

| situation | gap |
| --- | --- |
| the read value is a key the target mints and no step captures | "the scope of `<inner>` is a value the target mints, which no scenario names in advance" (the outcome's scenario is refused) |
| no creating input steers a cross decoy above the inner head | "no row of another `<outer entity>` ranks above the head of `<inner>`: a target scoping by the wrong row is not caught" |
| the outer selection admits one row only | the outer's own `UNOBSERVABLE` gap; cross decoys need a second outer row |

Cost on the issue's command: the outer takes 3 rows plus 1 tie. Each side takes 3 + 1 tie + 2
cross decoys + 1 decoy per conjunct. That is about 20 to 25 creation steps (inferred, not
measured).

### With #543 U7

- **Struct identities in an order.** When the outer `first:` is a struct identity, the published
  field is a struct. The inner compares members (`slot_key.group == payload.E.slot.group`), and
  items 2 and 3 above write member literals. Nothing else changes.
- **`ties: unordered`.** No tie rows are arranged for either selector, and cross decoys rank
  strictly above the head, so they never tie it. At run time an outer tie is unsupported in the
  interpreter, which fails the outer's resolution, so the inner is never read. The `author`
  refusal of arranged ties must evaluate an inner selector with the outer value bound. If U7
  implements it through `Reader::head`, it gets that from the interpreter change above. If not,
  unit U3 here adds the published values to U7's check (inferred: U7 is not written yet).

## Targets

| target | disposition |
| --- | --- |
| native interpreter | executes the operand (above) |
| history checker | sound or `Undecidable` (above) |
| synthesis | arranges the nested witness and cross decoys, or a named gap (above) |
| authored scenarios | run through the interpreter, no change of their own |
| Go and TypeScript suite runtimes | no change: they compare published values |
| generated Rust and Go | no change: `RelatedSelection`/`RelatedHead` are already named obligations (`crates/generate/ess-synth/src/determined.rs:508-525`). `story:generated-behaviour-reads-several-related-rows` gains a scope line: resolve payload fields in dependency order |
| web player | none, as for any selector read (`ess-conformance/src/web.rs:262-263`) |
| Entity Runtime | the existing `ROW_SET` limitation (`crates/generate/ess-entity-runtime/src/subset.rs:100`, `:120`) |
| OpenAPI, AsyncAPI, service contract | no change: wire shapes and `type_ref` unchanged |
| linearizer | reads other rows: `true` already (`ess-conformance/src/linearize.rs:391`) |
| generated docs and diff | `describe` (above) |

## Not in this design

- **"The slot can match"**: choosing the best slot among those whose two sides are both non-empty.
  That is a test of a related row set **per candidate** of the outer selector (a correlated
  `exists`), not a read of another value. Measured with the same binary:
  - inside a selector, `{exists: {entity: …, where: …}}` is refused: "a quantifier takes `in`,
    `as` and `that`, and nothing else";
  - the counter idiom (the joining command increments a `left_waiting` on the slot through
    `affects:`) is refused: "outcome `joined` creates its subject and declares `affects:`".

  I know of no other idiom. Without it, a model whose best slot has an empty side cannot publish a
  pair, and it cannot refuse by name either, because a guard cannot read the published head. That
  needs its own construct and a decision.
- **An effect on the head row** (pop the membership): the `affects[].where` site above, with #547.
- **A named intermediate** (an unpublished value read by selectors). See alternative (b).
- **Mutation operators** (scope the inner by the first-inserted outer row). The cross decoys
  already kill these targets.

## Fit review

1. **Need.** A command reads a ranked row and then a ranked row **related to it**, for example the
   best slot of a group and the best element on each side of that slot, and publishes both. ESS
   cannot scope the second read by the first, so the suite cannot check the second choice. Minimal
   reproduction: "The gap" (binary at `3fdd9479e`). The requester's syntax is quoted there,
   labelled as theirs.
2. **Class: gap.** A selector's operands are the row, `input.`, `subject.` and `now`
   (`row_set.rs:12-16`). `ordered-related-head.md:336-339` names this exclusion. It is not a
   defect.
3. **Already expressible? No.** The `payload.` root is refused (measured above). The workaround,
   the caller naming the slot (measured valid), checks nothing about the implementation's choice
   (#544). A second command (pick the slot, then pop each side) changes the domain: two steps where
   the implementation takes one, and the slot can change between them.
4. **Fit.**
   - *Vocabulary.* `payload.<Event>.<field>` reuses #545's `{payload: <Event>.<field>}`
     (`minted_value.rs:64-104`), "the value this outcome publishes", and its event lookup. As a
     root it follows `input.` and `subject.`.
   - *Composition.* Guards refuse it by name (precedence). `sets:` and `affects[].sets:` admit it
     inside their selectors. `affects[].where` refuses it by name until #547. The operand works with
     both `field:` and `first:` reads, and with #545's stored copy. Bindings and views read no
     outcome.
   - *Siblings.* `{related: {via}}` over a published identity is already expressible as a selector
     on the identity (`{related: {entity, where: <id> == payload.E.f, field}}`), so `via:` needs no
     operand.
   - *Targets.* Every target executes, owes or refuses it by name (the targets table). The explicit
     `reads` list makes every evaluator route the operand without guessing.
5. **Second, unrelated adopter.** A support desk takes the highest-priority open queue of a team,
   then the oldest waiting ticket in that queue: `queue_id: {related: {entity: Queue, where:
   team_id == input.team_id, order_by: [priority desc, queue_id], first: queue_id}}` and
   `ticket_id: {related: {entity: Ticket, where: {all: [queue_id ==
   payload.desk.TicketAssigned.queue_id, state == Waiting]}, order_by: [opened_seq, ticket_id],
   first: ticket_id}}`. A warehouse picks the bin holding most of a product, then the oldest lot in
   that bin (FIFO).
6. **Cost.**
   - Format: none of its own; it rides `ess/25`.
   - Keywords: none new. `payload` is already a source key (#545), and it becomes a predicate root.
   - Diagnostics: no new code, about 9 new messages.
   - IR: one optional field, and old bytes are unchanged.
   - Suite, schema, generated API: unchanged. Diff: none new. Migration: none.
7. **Alternatives.**
   - (a) **Change nothing.** The caller names the slot. That is valid today, but the scenario author
     picks the row the implementation should pick.
   - (b) **A named intermediate**: an outcome key such as `reads: {slot: <source>}`, read as
     `{read: slot}` and `read.slot`. Rejected for now. It needs a new keyword, a new namespace, an
     IR section and a diff classification, and an unpublished value can be checked only through its
     consequences. Both of the issue's values are published. It can be added later beside this
     without conflict.
   - (c) **An inline source as an operand**
     (`slot_id == {related: {entity: Slot, …, first: slot_id}}`). Rejected. It states the outer
     head twice, as the published field and inside the inner `where:`, and the two copies can
     drift. It also adds an operand kind to `ess-primitives` that every predicate evaluator (views,
     bindings, guards) would have to refuse.
   - (d) **The requester's spelling**, a mapping operand `where: {slot_id: {payload:
     PairMade.slot_id}}`. Changed. `where:` is a predicate, and a mapping form would be its second
     spelling. The event name is qualified, as in #545.
   - (e) **`payload.<Event>.<field>` as a predicate root**. Chosen.

## Decisive acceptance

Each test is written first and seen red.

- `crates/specify/ess-domain/tests/published_value_operand.rs`:
  - `a_nested_head_reading_the_published_slot_validates_at_ess_25`;
  - `the_operand_below_ess_25_is_refused_naming_ess_25`;
  - `a_selector_over_an_entity_with_a_payload_field_keeps_its_meaning`;
  - `the_operand_in_a_row_set_guard_is_refused`;
  - `the_operand_in_an_affects_where_is_refused`;
  - `an_operand_reading_a_mint_is_refused`;
  - `an_operand_reading_an_event_the_outcome_does_not_fill_is_refused`;
  - `a_cycle_between_two_payload_fields_is_refused_as_self_reference`;
  - `a_field_whose_selector_reads_itself_is_refused`.
- `crates/specify/ess-compiler/tests/published_value_operand_ir.rs`:
  - `the_selection_lists_its_published_reads_in_dependency_order`;
  - `a_model_without_the_operand_keeps_its_ir_bytes`.
- `crates/verify/ess-conformance/tests/nested_ordered_heads.rs` (fixture
  `tests/fixtures/nested-ordered-heads.yaml`):
  - `the_interpreter_takes_each_side_of_the_slot_it_chose`;
  - `an_empty_outer_selection_publishes_nothing`;
  - `the_event_and_the_stored_copy_hold_the_operand_value`;
  - `synthesis_arranges_cross_decoys_in_the_other_slots`;
  - `a_target_scoping_by_the_first_inserted_slot_fails_a_scenario`, and the same for the
    last-inserted slot and for a target that ignores the slot;
  - `a_minted_scope_is_a_named_gap`.

## Implementation units

Each unit is one agent and one branch, based on the integration head **after #543 U7**.

| unit | what | files | depends on | build footprint |
| --- | --- | --- | --- | --- |
| U1 | the root, validation, cycles | new `ess-domain/src/command/published_operand.rs` (the environment's `payload` reads, the snapshot-value rule, the dependency graph, `reads(outcome, selection)` shared with the compiler); `ess-domain/src/command/set_effects.rs` (`WithSubject` gains a published root); `ess-domain/src/command/row_set.rs` (`check` takes the published reads; the guard message); `ess-domain/src/command/value_expression.rs` (`check_selection`, `check_head` pass the outcome and site); `ess-domain/src/command/minted_value.rs` (the filled-event lookup shared, no behaviour change); `ess-domain/src/command.rs` (call the cycle pass); new test `ess-domain/tests/published_value_operand.rs` | #543 U7 | `ess-domain`, `ess-compiler` (`typed_diagnostics`), `ess-cli`; no `ess-conformance` build |
| U2 | IR field, resolver | `ess-compiler/src/ir.rs` (`ResolvedRowSelection.reads`, `ResolvedPublishedRead`); `ess-compiler/src/resolve.rs` (`:3152`, `:3207`, `:5469`); new test `ess-compiler/tests/published_value_operand_ir.rs` | U1 | `ess-compiler`, `ess-diff`, `ess-cli` |
| U3 | interpreter | `ess-conformance/src/interpret/execute.rs` (`payload_reads`, `publish_reads` in dependency order); `interpret/execute/row_set.rs` (`Reader::new`); `interpret/execute/set_effects.rs` (`Facts.published`); U7's author tie check, where it does not go through `Reader` (inferred); new fixture and the interpreter cases of `tests/nested_ordered_heads.rs` | U2 | `ess-conformance`, one `--test`, plus `ess-cli` (about 10G, the #543 story's estimate) |
| U4 | synthesis | `ess-conformance/src/synthesize/row_set.rs` (`selections_of`, `scoped`, `closed`); `ess-conformance/src/synthesize/ordered_head.rs` (cross decoys, gaps); the synthesis and mutant-target cases of `tests/nested_ordered_heads.rs` | U3 | as U3 |
| U5 | documentation | `website/docs/guides/specify/values-and-views.md`; `website/docs/reference/spec-versions.md` (`ess/25` row); `CHANGELOG.md` `[Unreleased]`; `docs/design/ordered-related-head.md` ("Not in this design" points here); this page's status | U4 | `task test-xtask`; `task site-build` |

Order: strictly U1 → U2 → U3 → U4 → U5. Each needs the previous one's types, and U3 and U4 share
the fixture.

Overlap: U1 edits `value_expression.rs` `check_head` and U3 and U4 edit the head's interpreter and
synthesis, all of which #543 U7 touches. Rebase on U7 before starting, and do not run U1 beside any
other unit that edits `value_expression.rs` or `set_effects.rs` in `ess-domain`.

Gates per unit (no full gate):

| unit | gates |
| --- | --- |
| U1 | `cargo test -p ess-domain --test published_value_operand`; `cargo test -p ess-compiler --test typed_diagnostics`; `task test-xtask`; `clippy -p ess-domain`; `fmt --check` |
| U2 | `cargo test -p ess-compiler --test published_value_operand_ir`; `cargo test -p ess-diff`; `clippy -p ess-compiler -p ess-diff` |
| U3, U4 | `cargo test -p ess-conformance --test nested_ordered_heads`, plus `--test ordered_related_source` and `--test row_set_several_selectors` for regression; `clippy -p ess-conformance` |
| U5 | `task test-xtask`; `task site-build` |
