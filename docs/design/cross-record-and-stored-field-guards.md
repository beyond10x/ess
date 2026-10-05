# Guards over stored fields and constraints across records

Status: rule 1 implemented in source format `ess/9` (beyond10x/ess#75). Rule 2 — a constraint over a
set of rows — stays out of scope, as below; its one-row case, a guard over **one** row of another
entity named by an identity in the input, is implemented in `ess/18` (beyond10x/ess#211, `when_related:`).

## Behavior and authority

A trial specified two real services from their source. Two rules had no ESS form, and both were
left as `UNMAPPED:` markers — the `ess-specify` skill's convention for a rule the model cannot
state (`plugins/ess-specify/skills/specify/SKILL.md` in `beyond10x/agentplugins`); nothing in this
repository reads the marker:

1. "Express parcels over 20 kg are refused at dispatch."
2. "A room is never double-booked."

In rule 1 the weight and the service are recorded when the parcel is created. The dispatch
command's input carries the parcel identity and nothing else, and `when:` is a predicate over the
command's input only. The refusal depends on two fields of the row the command addresses, one of
them compared by ordering.

In rule 2 no single command input decides the outcome. Whether a booking is admitted depends on
every other booking of the same room whose time range overlaps the new one. The rule is over a set
of records, not over one.

## What exists today

| Construct | What it can say | Why it does not state rule 1 |
|---|---|---|
| `when:` | A predicate over the command's own input fields. | The weight is not input at dispatch. Copying it into the input makes the caller the authority on a stored fact; the guard then checks what the caller says instead of what was recorded. |
| `when_subject_state:` (`subject-state-outcome-guards.md`), `when_state_changes:` (`state-change-outcome-guards.md`) | Select an outcome by the addressed entity's held **lifecycle state**, read immediately before selection. | They read the state name only, and the first page declares no "cross-entity read, view query, field comparison". |
| `when_subject: {field, equals}` (ess/6, `observed-subject-history.md`) | Selects an outcome by equality of one declared **enum** field of the existing subject against one variant, conjunctive with an optional input `when:`. | Enum equality only. `weight_kg > 20` is an ordering over a number, and "Express and over 20 kg" is a conjunction of two fields. It also cannot sit on a refusal: see the table below. |
| `wrong_state:` | Answers in the complement of a move's `from` set, with no subject of its own. | A lifecycle state, not a field value. The *shape* — a refusal decided by the subject the siblings name — is the one rule 1 needs. |
| entity `invariants:` | "What must hold of every instance, at rest" (`crates/specify/ess-domain/src/entity.rs`), checked over the fields plus the `state` pseudo-field. Rule 1's safety property is writable today: `not (state == Dispatched and service == Express and weight_kg > 20)`. Conformance asserts every invariant after each state-changing branch through a view that publishes the fields (`ViewExpectation::Satisfies`, `crates/verify/ess-conformance/src/synthesize.rs`); the Entity Runtime projection lowers it to a runtime rule (`crates/generate/ess-entity-runtime/src/lib.rs`). | It names no command, no outcome and no error: it says the bad row never exists, not what `Dispatch` answers. Its witness is the ordinary scenario of each branch, which never arranges an overweight Express parcel on purpose, so it is only as strong as the rows those scenarios happen to produce. |
| Per-slot lifecycle | Rule 2 can be approximated by modelling each bookable unit (room × fixed slot) as an entity with `Free → Booked`; a second booking becomes a `wrong_state` refusal. | Only exact for fixed, pre-enumerated slots. Free time ranges overlap without being equal, so the entity identity cannot be the slot. It also changes the model the service actually has, which is one booking per request. |

### What today's `ess` accepts for rule 1

Measured with `ess 0.32.0` (`ess specify validate`, `ess verify conform synthesize --target ir`)
on a scratch model: `Parcel` with `service: Service` (enum `Standard | Express`) and
`weight_kg: Integer`, `Create` setting both, `Dispatch` moving `Created → Dispatched`.

| Shape written | Result |
|---|---|
| `when_subject: {field: service, equals: Express}` + `moves:` + `error:` on the refusal | refused, `refusal_mutated_state`: "a refused command changes nothing, so a refusal has no subject" (`command.rs`) |
| the same without `moves:` | refused, `conflicting_declaration`: "a subject-state guard requires an existing moves or updates subject and input identity" — sited at `outcomes.<name>.when_state_changes`, a key the author never wrote (`held_state_key` in `command.rs`) |
| guarded success `dispatched` + effect-free `error:` default; `Create` sets `service: input.service` | validates; `dispatched` gets `ESS-SYNTH-001` "no bounded arrangement establishes the subject fact and eligible input" when the guard names the variant the create witness did not pick |
| the same with a driver writing the literal `sets: {service: Express}` | validates; `dispatched` is witnessed through that driver. The `error:` default is witnessed by sending an identity no row carries (`witness.rs` `uuid`, which "identifies nothing") and requiring `ExpressOverweight` |
| two `when_subject` branches covering both variants, no default | refused, `non_exhaustive_branches` and `unreachable_branch`: the stored fact enters no partition |
| `when_subject` on one branch, `when_subject_state:` on a sibling | refused, `conflicting_declaration`: "subject fact and lifecycle guards cannot be combined in one command" (`command/subject_fact.rs`) |
| `when_subject: {predicate: …}` | reader error `unknown field predicate, expected field or equals` (`RawSubjectField` is `deny_unknown_fields`) |

So the only writable form of rule 1 today is inverted — the guard on the success, an unconditional
`error:` for everything else — and it is reachable only through a driver that writes the guarded
value as a literal. Its refusal witness arranges no parcel: the suite requires `ExpressOverweight`
for an identity no row carries and never dispatches a parcel the guard excludes, so an
implementation that ignores the stored field passes it. That is the gap this page closes.

## Proposal for rule 1: a predicate over the subject's stored fields

`when_subject` takes a second shape, `{predicate}`, beside today's `{field, equals}`; never both in
one branch. It reads the same subject at the same point as `when_subject` and `when_subject_state`
and widens what may be asked of the subject's declared fields. Rule 1, as the service states it:

```yaml
entities:
  - name: shipping.parcel.Parcel
    identity: {name: parcel_id, type: Uuid}
    fields:
      - {name: service, type: shipping.parcel.Service}   # enum: Standard | Express
      - {name: weight_kg, type: Integer}
    lifecycle:
      initial: Created
      states: [Created, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {name: dispatch, from: [Created], to: Dispatched}

commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}

  - name: shipping.parcel.Dispatch
    input: [{name: parcel_id, type: Uuid}]
    outcomes:
      - name: refused-overweight
        when_subject:
          predicate:
            all:
              - service == Express
              - weight_kg > 20
        error: shipping.parcel.ExpressOverweight
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]

views:
  - name: shipping.parcel.Parcels
    source: shipping.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: Uuid}
      - {name: state, type: shipping.parcel.Parcel.State}
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
```

The view is part of the rule: it is the unfiltered view through which the guarded fields are
observed, and without one the witness is refused. An immediate view is read once; where every such
view is `eventual`, the observation waits in an `eventually` block until the view shows the arranged
row (beyond10x/ess#172). That proves the implementation applied the values the arrangement's last
step left, and no step writes the row between that observation and the command. It proves nothing
about a row that did not move, so a refusal's unchanged row is asserted only through an immediate
view, and omitted where there is none.

`weight_kg` is an `Integer` and not a `Decimal` on purpose: `sets:` admits no literal spelling for
`Decimal` or `Binary64` (`crates/verify/ess-conformance/src/input.rs`, `primitive_literal`), so a
`Decimal` field cannot be arranged to a value and cannot be guarded until it can.

### The refusal carries the guard

Two designs were open: (a) admit `error:` on a subject-guarded branch that names no subject of its
own, the subject being the one its siblings name; (b) keep the guard on the success and extend the
ess/7 effect-free default (`has_state_refusal` in `command.rs`, `is_state_refusal` in
`synthesize.rs`) from lifecycle guards to subject facts, so the default is arranged against a real
row. **(a) is chosen**, and (b)'s arrangement follows from it anyway:

- It reads as the rule. Under (b) the author writes the complement, `any: [service != Express,
  weight_kg <= 20]`, and the rule the service states appears nowhere — a second copy of one fact,
  negated, which is what `state-change-outcome-guards.md` refuses for the same reason.
- Every refusal keeps its own error. A command refusing for two stored-field reasons names two
  errors under (a); under (b) both collapse into one default. `command.rs` already argues this for
  `wrong_state:`: "do not generate vague *operation fails* tests if the domain declares a specific
  error".
- It is the shape the model already has. `wrong_state:` and the ess/7 effect-free default are both
  refusals without a subject, decided by the subject the command addresses. `refusal_mutated_state`
  stays: a refusal still carries no `moves:` or `updates:`.
- Both designs need the same new arrangement (below): a row satisfying a predicate for the refusal
  under (a), a row refuting one for the default under (b). Once the default is arranged by refuting
  every guarded sibling over the row, the inverted shape is witnessed correctly as a consequence,
  without being the recommended one.

### Declaration

- **Subject.** The command's common subject: the existing entity and identity field that every
  input-selectable branch with a subject names (`command/subject_fact.rs`: "subject fact
  selection requires one existing subject identity"). A guarded `error:` branch names none and
  takes that one; a guarded `moves:` or `updates:` branch names it itself. A command whose only
  subject-bearing branch is `creates:` has no subject to read, and is refused.
- **Namespace.** The predicate reads the entity's declared `fields` and nothing else: not the
  input, and not `state` (from `ess/15` the input under `input.`, and from `ess/18` the held
  state as `state` — see "The held state in the predicate" below). Input stays in `when:`,
  conjunctive with the subject predicate as it is today for `{field, equals}`. The lifecycle
  guards `when_subject_state:` and `when_state_changes:` stay their own authority, and combining
  those with `when_subject` on sibling branches stays refused (below, "one strategy or two").
- **Literals.** The bare-word rule of `crates/specify/ess-primitives/src/predicate.rs` applies
  unchanged: a right-hand side without a dot is a literal, so `service == Express` compares with
  the text `Express`, and the checker refuses a bare word that names a declared field
  (`text_literal` in `crates/specify/ess-domain/src/expression.rs`).
- **Read point.** The row immediately before command selection, as for `when_subject_state`.
- **Optional fields.** May be read. `evaluate_compare` answers `Unknown` when an operand does not
  resolve (`predicate.rs`), and an unknown fact selects no branch and never the default
  (`observed-subject-history.md`). An author who wants `None` to select something writes
  `not defined(field)`. No new refusal; the finite prover already declines `Optional` paths.
- **Absent subject.** Satisfies no predicate and reaches no declared outcome, as today; it is not
  created and not read as an initial-state row. This is a witness requirement below, because it is
  exactly what the unarranged refusal witness gets wrong today.

### What it refuses

| written | refusal |
|---|---|
| `{field, equals}` and `{predicate}` in one `when_subject` | the reader's own refusal: one shape per branch |
| beside `when_subject_state:`, `when_state_changes:`, `external:` or `wrong_state:` on the same branch | `conflicting_declaration` — "a subject fact has one selection authority" (`command.rs`), as today |
| on a `creates:` branch, or in a command with no subject-bearing sibling | `conflicting_declaration`, sited at `outcomes.<name>.when_subject` — not at `when_state_changes` |
| a field the entity does not declare | `unobservable_fact` at `outcomes.<name>.when_subject`, from the expression checker with the entity's fields as its environment |
| an `input.` path below `ess/15`, or `state` below `ess/18` | `unsupported_format_version` at `outcomes.<name>.when_subject`: the document is not wrong, its header is |
| `state` on a branch whose `moves:` does not start in a state the predicate may select it in (`ess/18`) | `conflicting_declaration` at `outcomes.<name>.when_subject` |
| a literal of the wrong type, a bare word naming a field, an ordering against a non-RFC 3339 text for a `Timestamp` | `type_mismatch` / `undeclared_reference` — the checker's existing diagnostics, unchanged |
| sibling `when_subject` predicates over different entities or identity fields | `conflicting_declaration` at `outcomes` — "subject fact branches must share one entity, identity" (`command/subject_fact.rs`), with the "and enum field" clause dropped |
| `when_subject` beside a lifecycle guard in one command | `conflicting_declaration` at `outcomes` — "cannot be combined in one command", as today |

### Validation: a joint entity-field × input partition

Today the stored fact enters no partition (`finite_coverage` in `command.rs` collects each branch's
*input* predicate only, and `command/subject_state.rs` partitions lifecycle guards only), which is
why two `when_subject` branches over both variants of an enum are refused as non-exhaustive. The
proposal adds a partition over the entity fields the predicates name, crossed with the input fields
the `when:` guards name, in `command/subject_fact.rs`:

- **Evaluator and environment reused.** The expression checker runs over
  `DomainEnvironment::new(types, &entity.fields)` for the subject side — the environment
  `entity.rs` already builds for invariants, minus the `state` pseudo-field — and over the input
  for the `when:` side. `finite::analyze_with_states` generalises: its held side becomes an
  assignment over the guarded entity fields instead of one lifecycle state.
- **Closed domains are what `finite.rs` admits today:** `==`/`!=` and membership
  (`any_of`/`none_of`) against text literals over non-optional, non-collection enum paths.
  Booleans are not among them — a `true` literal parses as `FactValue::Bool`
  (`crates/specify/ess-primitives/src/facts.rs`) and the analyzer declines it — and this page does
  not claim them; admitting a two-variant domain is a separate, small change to `finite.rs`.
- **Open domains** — `weight_kg > 20` — make the prover decline, and the command then needs a
  genuine default, as an open input guard does today. Rule 1's default is `dispatched`.
- **Caps** stay at 64 joint assignments and 128 guard nodes.
- **Diagnostics** are the partition's existing ones, at the existing sites:
  `non_exhaustive_branches` at `<command>.outcomes` ("declare a genuine default"),
  `conflicting_declaration` at `<command>.outcomes` naming the assignment and the branches it
  selects, and the checker's `type_mismatch`, `undeclared_reference` and `unobservable_fact` at
  `<command>.outcomes.<name>.when_subject`.

### Conformance: arranging a row to a goal

The strategy in `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` arranges the
subject at its initial state, advances it through `moves:`/`updates:` drivers with those drivers'
own witness inputs, reads one enum fact as the text literal the arrangement settled (`fact`),
compares it as a string (`reach_fact`, `held == equals`) and keys its search on
`(lifecycle state, that text)`. None of that reaches `weight_kg: 21`. The measured failure above is
this: with `sets: {service: input.service}` the value the row holds is whatever input the create
witness chose, and the search has no way to choose differently. The strategy is rewritten in four
parts, and the `{field, equals}` form runs on it as a one-leaf predicate:

1. **Goal-directed input choice through `sets:` mappings.** The search carries a goal — an
   assignment over the guarded fields — and, for each driver whose `sets:` maps a guarded field
   from an input field (`ResolvedPayloadValue::InputField`, the case `settled` in `synthesize.rs`
   already records), chooses that driver's input so the mapped field carries the goal value,
   subject to the driver's own guards still selecting its outcome. A literal `sets:` is a fixed
   point the search takes as it is. A field written through a declared conversion, or mapped from
   nothing the search can set, is not arrangeable: the branch is refused with `ESS-SYNTH-001`
   naming the field and the reason, as `subject_fact.rs` refuses today.
2. **Goal values from the guard's own literals**, by the rule `alternatives` in `witness.rs`
   already uses for input search: a number compared with `n` is tried at `n`, `n + 1`, `n - 1`,
   `0` and `-1` (an `Integer` only at integral values); an enum at each variant; a `Timestamp` at
   the instant written and a second either side. The first assignment that satisfies the branch's
   predicate — or, for the default, refutes every guarded sibling's — is the goal.
3. **A typed fact source over the arranged row.** The `settled` map (`Determined { value,
   type_ref }`) implements the evaluator's `FactSource`: an `Integer` as a number, an enum as text,
   a `Boolean` as a Boolean, a `Timestamp` as text compared by instant, which `evaluate_compare`
   does for declared timestamps already. This replaces the text-only `fact` and the string
   comparison in `reach_fact`, and it is the same evaluator validation and the runner use.
4. **Visited key = branch-selection vector.** The search node is the lifecycle state crossed with
   the truth (`True`, `False`, `Unknown`) of every guarded branch's predicate over the row — finite
   where the field values are not, and exactly the distinction that matters: two arrangements that
   decide every branch alike are one node. The cap of 64 nodes stays. As implemented the node is
   refined by how each *leaf* of those predicates decides and whether the row sits on the leaf's
   own literal, which is still finite and keeps a boundary row from being merged into a far one.
   Where several rows at one depth reach the branch, the one satisfying the most leaves, then
   sitting on the most literals, is taken — which is what makes the default's witness `Express`
   at `20` rather than the plain `Standard` at `1`.

**Observation.** `observe` in `subject_fact.rs` already requires an immediate, unfiltered,
parameterless view exposing the identity, `state` and the one guarded field, and asserts them
before the command runs. That requirement widens to every guarded field; a specification without
such a view gets the existing typed refusal. Since beyond10x/ess#172 an `eventual` view with the
same projection stands in where no immediate one qualifies, asserted in an `eventually` block. The
absent-subject witness still needs an immediate view: an `eventual` read that shows no row proves
nothing about a row the projection has not caught up with. For the same reason the row after a
command is awaited through an `eventual` view only where the command changed its state or a field
the observation asserts, both in the stored-field observation and in the generic view assertion
such a command's changing branches get (`view_expectations`; other commands keep theirs). A
refusal's row, or one a branch left as it was, is asserted unchanged through an immediate view,
and that check is omitted where no immediate view projects it. The scenario still asserts the
outcome, the error and that no event was published. `unknown_instance:` does not compete for
selection beside the guarded branches.

From `ess/23` (beyond10x/ess#461) a refusal selected by `when_subject: {predicate: …}` that
names no subject of its own compiles with `complete_refusal`, as a named wrong-state refusal has
since `ess/7`. The `{field, equals}` shape gives no such refusal: validation refuses a
`{field, equals}` branch that names no subject (`conflicting_declaration`, "a subject-state guard
requires an existing moves or updates subject"). Its scenario snapshots the complete subject before
the command (`snapshot_complete_subject`) and requires it unchanged after it
(`expect_complete_subject_unchanged`), with no direct event (`expect_no_events`), through
`preserve_refused_subject`: a target that refuses and still writes a field the guard does not
read fails. So that a write is visible, every input an accepting branch's `sets:` writes into the
record is sent with a value other than the one the arranged record stores, where the input still
selects the refusal (`subject_fact::refused_writes_apart`). Where the immediate views cover only part of the subject, the published fields are
observed and the rest named in a `PartialObservation` note (beyond10x/ess#132), and no new
synthesis refusal arises. Below `ess/23` the guarded-field observation above stands, and the
suite keeps its bytes. Changing the format moves the IR header, which `ess verify diff` reports as
`outcome-observation-changed`.

**Wrong state.** A guarded branch is selected in any state, before `wrong_state:` applies: the
stored fields select a branch in a state no move of the command starts from, exactly as they do in
one it does start from. Entity Runtime orders branches that way (input-guarded refusals, the other
guarded branches, the default, `wrong_state` last). So the `state/<S>/refuses/<command>` scenario
misses every sibling, chosen in `subject_fact::refusal_witness` (beyond10x/ess#173, #192):

- a branch guarded by its input alone, a plain `when:` or an input-guarded refusal, is missed only
  through its input;
- a branch guarded by the stored row alone (`held: when_subject: history == Pending`) is missed
  only through the row, which must decide its guard false;
- a branch that needs both, a `when:` beside a `when_subject:`, is missed through either: its input
  half refuted, or its subject half decided false by the row.

A stored-guarded sibling that moves the subject along a transition not starting from `<S>` needs no
refuting while its guard decides true or false on the row. Selected or not, a move from a state it
does not leave is the wrong-state answer, so the row the `rush` move itself left in `Rushed` can
serve even though `rushed`'s guard still holds on it. A guard that is unknown on the row (it reads an
optional field no creation fills) stops selection before the wrong-state answer; that case, and a
moving branch whose own stored guard the row rejects, are not yet handled
(`story:wrong-state-witness-unknown-and-own-stored-guards`).

The input is one the moving branch's own input guard admits. A candidate refuting every input half
is preferred; only where none exists does the witness rely on the row for a mixed branch. For
`already-confirmed: history == Confirmed, token != ""` beside `token-required: token == ""` it sends
`token != ""` to a row whose `history` is not `Confirmed`. The row is the ordinary arrangement for
`<S>` where it serves. Otherwise it is the first row in `<S>` that the declared drivers leave,
found by the search above and steered by the command's own stored guards. Every stored guard the
witness relies on the row for is observed before the command, through the view **Observation**
names. Where no reachable row in `<S>` refutes every such guard, the scenario is refused with
ESS-SYNTH-003 naming the input and stored guards. It is never written to depend on which branch an
implementation decides first.

**The witness for `refused-overweight`**, in the suite's own step vocabulary:

1. `execute_command` `Create` with `service: Express, weight_kg: 21` — chosen through the
   `sets:` mappings; `21` is the guard's `20` plus one.
2. `expect_outcome` `Create/created`; `capture_instance` `parcel` from `Created.parcel_id`.
3. `query_view` + `expect_view` `Parcels` contains `{parcel_id: parcel, state: Created,
   service: Express, weight_kg: 21}` — the arranged facts observed, not assumed.
4. `execute_command` `Dispatch` with `parcel_id: parcel`.
5. `expect_outcome` `Dispatch/refused-overweight`; `expect_error` `ExpressOverweight`.
6. `expect_no_event` for `Dispatched` and every other event.
7. `query_view` + `expect_view` `Parcels` contains the same row as step 3 — a refusal changes
   nothing.

**The witness for `dispatched`** is the same with a row that refutes the sibling — such as
`Express` at `20`, the boundary — then `expect_outcome dispatched`, `expect_no_error`, `expect_event
Dispatched`, and the view holding `state: Dispatched`. An implementation that ignores the stored
weight fails step 5 of the first witness; one that refuses every Express parcel fails the second.

**The witness for an absent subject:** `execute_command` `Dispatch` with an identity no row
carries, then `expect_no_event` for every event and `expect_view` `Parcels` *excludes* that
identity (`ViewExpectation::Excludes` exists in `scenario.rs`). It asserts nothing about the error,
because the specification declares no absent-subject outcome, and a suite that required
`ExpressOverweight` there — today's — would be inventing one. It opens the scenario of every branch
that names no subject of its own — `refused-overweight` here — before that scenario's arrangement,
so it needs no scenario id and no new vocabulary.

**Transition and invariant scenarios** that route through a guarded branch reuse the same
arrangement; a `dispatch` transition witness arranges a row the guard does not refuse.

### One strategy or two

Today synthesis has two disjoint strategies — `subject_fact::prepare` for a fact,
`prepare_state_input` for a lifecycle state (`run` in `synthesize.rs` picks one) — and validation
refuses a command that would need both. **Two remain.** The fact strategy is the one rewritten
above and serves both `when_subject` shapes; the lifecycle strategy and its complete finite
partition (`finite::StateGuard`, every state × every closed input) are untouched; the refusal on
mixing them in one command stands. Folding the two would either give the lifecycle guards the open
partition of the field predicate, losing the completeness proof they have, or bound field
predicates to closed domains, losing rule 1. A command that needs both today has `wrong_state:`
beside `when_subject` for the lifecycle complement, which the ess/6 fixture
(`crates/verify/ess-conformance/tests/fixtures/subject-history.yaml`) already does. From `ess/18`
`state` is also a leaf of the fact strategy's predicate (below); the lifecycle strategy and the
refusal on mixing the two in one command are unchanged.

beyond10x/ess#461 asked for `when_subject:` and `when_subject_state:` on different branches of one
command, with a joint state × stored-field partition. That is declined: it is a second spelling of
`state ==` within one command and reopens this split. The refusal (`conflicting_declaration`,
`ESS-COMMAND-004`) keeps its code and message, and gains a hint naming the form that states the
same command, `when_subject: {predicate: state == Active}` on the update or
`when_subject: {predicate: state != Active}` on a refusal declared before it.

### Projections

The projections render the `{field, equals}` form as prose in docs
(`crates/generate/ess-gen/src/docs.rs`) and with `Debug` formatting of the input predicate in
`OpenAPI`, the native plan and the semantic diff (`openapi.rs` beside it,
`crates/generate/ess-synth/src/plan.rs`, `crates/verify/ess-diff/src/diff.rs`). The predicate
form renders as `SubjectState` does: the
predicate through `Display`, in all four, so a reader of the published contract sees `service ==
Express and weight_kg > 20` and not a Rust structure. The guarded fields become part of the
command's documented contract that way, as the state condition is. The HTTP status is unchanged:
`http.rs` already answers `409` for a refusal decided by the subject (`SubjectField` sits with
`WrongState` and `SubjectState` under `CONFLICT`). Entity Runtime lowers `SubjectField` to
`$fields.<field> == <variant>` and entity predicates through `PathRewrite::Entity` already; the
predicate form lowers through the same rewrite.

### Persisted formats and compatibility

Two precedents: `when_subject: {field, equals}` took ess/6
(`crates/specify/ess-domain/src/primitive_admission.rs`: "subject facts and preservation require
specification format ess/6"); `when_state_changes:` took
no bump, because it was a new key on `RawOutcome`, which denies unknown fields, so an older build
refused it by name (`state-change-outcome-guards.md`). This construct is neither: it is a new shape
of an existing key's value, and an older reader fails it with `unknown field predicate` — a parse
error with no version hint. `AGENTS.md` requires a new format version when meaning changes, and the
meaning of `when_subject` does. **The predicate form requires `ess/9`**, gated beside the ess/6 and
ess/7 gates in `primitive_admission.rs` with `unsupported_format_version`: "subject predicates
require specification format ess/9". `{field, equals}` documents keep ess/6 and their bytes.

- The domain gains `OutcomeCondition::SubjectPredicate` and the IR
  `ResolvedCondition::SubjectPredicate { predicate, input }` beside `SubjectField`, which is
  unchanged so ess/6 models keep their IR bytes. Every exhaustive match over the enum gains an arm
  — there is no wildcard arm by design, and the compile errors are the drift alarm. Its
  `test_strategy` stays `observe_subject_fact`: what the scenario does — arrange, observe through a
  view, run — is the same.
- `RawSpecFile` changes, so `schemas/generated/ess.schema.json` is stale until `cargo xtask schema`
  runs, and `projection-check` is the only thing that sees it.
- The suite needs no new vocabulary: every step above exists. Old-reader refusal of `ess/9`, byte
  preservation of ess/6 and ess/7 models, and the arranged witness against a guard-ignoring mutation
  are the deciding checks. The test in `crates/specify/ess-domain/src/spec.rs` that uses `ess/8` as
  a header this build cannot read moves to `ess/10`, past both `ess/9` and the `ess/8` that
  #95 takes (`docs/design/string-predicate-operators.md`).

### What was rejected

**The inverted default** (design (b) alone): stated above.

**`state` in the predicate namespace**, subsuming `when_subject_state:`. Rejected for rule 1 and
admitted from `ess/18` for a narrower need (beyond10x/ess#204): a branch selected by the held state
*and* a stored field together, which `when_subject_state:` beside `when_subject:` cannot say
without two selection authorities. It does not subsume `when_subject_state:` — a command still uses
one strategy or the other — and it does not express `when_state_changes:`. The move check it
needed is the one below.

### The held state in the predicate (`ess/18`, beyond10x/ess#204)

A `when_subject` predicate may read `state`, the lifecycle state the addressed row holds
immediately before selection: `{all: [state == Ready, hold_note != ""]}`. It is the pseudo-field
entity invariants already read, typed by the lifecycle's own enum, so the finite prover treats it
as a closed domain. No entity field can be named `state`.

- **Format.** Below `ess/18` the path is refused with `unsupported_format_version` at
  `outcomes.<name>.when_subject`; a model without it keeps its bytes and digest.
- **Authority.** One per branch, as before: `when_subject_state:` beside `when_subject:` stays
  `conflicting_declaration`, and so does a lifecycle guard beside `when_subject` in one command.
- **Moves.** A branch reading `state` that takes a move must be able to take it in every state the
  predicate may select it in: each declared state the move does not start from is bound as `state`
  alone, and a predicate not then decided false is refused (`conflicting_declaration`).
- **Precedence.** Guarded branches select before `wrong_state:` applies (the #192 ruling), so a
  predicate may name a state no move of the command starts from — `state == Closed and note != ""`
  answers a closed row carrying a note before `wrong_state:` answers every other closed row.
- **Conformance.** The arranged row's held state is bound as `state` wherever a stored-field
  predicate is evaluated over it (`synthesize/subject_fact.rs`, `row_truth_with`), so the search
  reaches the state and the boundary rows refute each conjunct: a `Ready` row without the note,
  a row in another state with it. A row in a wrong state is answered by the guarded branch only
  where that branch reads `state`; every other wrong-state row stays the wrong-state family's. Each
  wrong state `S` keeps its `<entity>/state/<S>/refuses/<command>` scenario: it first witnesses,
  in declaration order and each on a row of its own, every guarded branch reading `state` whose
  predicate is not false with `state` bound to `S` alone, sent an input selecting it there; then the
  plain wrong-state row, only where some input and row still reach it. Whether the guards answer
  `S` is decided by that search over every branch together, input halves included: where no input
  and row miss them all, their rows are the scenario. A branch that may be taken in `S` and that no
  bounded arrangement reaches refuses the scenario with that cause; no state is dropped.
- **Claimed states.** From `ess/23` (beyond10x/ess#461) the same rule holds for a refusal's own
  scenario: a `complete_refusal` refusal whose predicate reads `state` is witnessed, beside its
  first row, on a further row in each other declared state where the predicate with `state` bound
  alone is not false (`subject_fact::claimed_states`), and where no guarded branch declared before
  it is selected by `state` alone, since declaration order answers there with that branch.
  `state != Active` over `Active`, `Suspended`, `Removed` is witnessed on a suspended and a
  removed row, each with the complete snapshot, so a target that accepts the command on either
  fails; declared after `state == Removed`, it is witnessed on the suspended row only. A state no
  bounded arrangement reaches the refusal in is refused beside the scenario, which stands: the
  refusal reads "has a scenario … that leaves a held state it claims unwitnessed", and a search
  that found no row names the state.
- **Runtimes.** Entity Runtime lowers `state` to `$from_state`: `$state` in an outcome selector
  is the destination, which entity-core refuses. The interpreted target does not evaluate a
  subject guard and reports such a scenario `unsupported`, naming the guard.

**Widening `SubjectField` in place.** It changes the IR of every ess/6 model that uses it, against
the byte-preservation rule, for no gain over a sibling variant.

## Rule 2: out of scope, and the steps that are in

A view-level non-overlap constraint was considered:

```yaml
views:
  - name: booking.room.BookingsByRoom
    source: booking.room.Booking
    constraints:
      - never_overlapping: [starts_at, ends_at]
        per: room_id
```

It is **not proposed**:

- **Not a guard.** It is an invariant over every pair of rows, not a predicate over one command. A
  specification would also have to say *which* command's outcome answers a violation, which the
  constraint above does not.
- **Conformance cannot witness it.** Synthesis would need to arrange a *set* of rows, choose an
  input that overlaps one on a boundary (touching, containment, identical, another room) and prove
  the refusal. A refused booking shows one overlap was caught; "never" needs an adversarial search
  over pairs, and concurrency — two overlapping requests racing — which the single-client runner
  does not model.
- **Ordering across records.** Overlap is `a.starts_at < b.ends_at and b.starts_at < a.ends_at`.
  Validation orders a `Timestamp` against an RFC 3339 literal, and refuses a bare word naming a
  field on the right-hand side — beyond10x/ess#74, `text_literal` in `expression.rs`, which points
  two fields of one struct at `window.starts_at < window.ends_at`. An overlap witness needs an
  ordering between an arranged row's field and a new input's, which no candidate search does.
- **Consistency.** Views already declare `consistency: read_your_writes | eventual`
  (`crates/specify/ess-domain/src/view.rs`), but that decides how a generated scenario *asserts*
  the view (`assertion_style`), not what a command reads at selection time. Nothing today says
  that a command's selection reads a view, and a constraint enforced at command time over an
  `eventual` view would be a promise nobody can keep.

**The cheapest in-scope step is a duplicate-identity outcome on `creates:`.** A create may already
publish an identity the caller supplied — `payload: <Event>: booking_id: input.booking_id`
validates and is witnessed on 0.32.0 — and what an implementation answers to a second create with
the same identity is unspecified and unwitnessed: the witness derives a fresh identity that
"identifies nothing", and no outcome exists to name the collision. Proposed, in the shape of
`wrong_state:`:

```yaml
- name: already-booked
  already_exists: true
  error: booking.room.SlotTaken
```

Admitted only beside a `creates:` sibling whose published identity is taken from input; refused
beside `{generated: true}`, where nothing can collide. Its witness is one duplicate: create, capture
the identity, create again with it, `expect_error`, `expect_no_event`, and `expect_view` still
containing the first row unchanged. It needs no ordering, no set arrangement and no composite
identity — `EntitySpec::identity` is one `Field` — and it reuses the subject-less refusal shape
this page and `wrong_state:` already have. For rule 2 it means: with the booking identity being the
slot key the caller composes, a double booking of a fixed slot is a declared, witnessed refusal,
without pre-creating every slot as a `Free` row. Range overlap, composite keys and the concurrency
argument stay out of scope, and `UNMAPPED:` remains the honest marker for them.

### One row of another entity, by identity (`ess/18`, beyond10x/ess#211)

The second step that is in scope: a guard over **one** row of another entity, the row whose
identity an input field carries. It is not rule 2 — no set of rows, no ordering across rows — but it
is the cross-record case a creating command most often needs: "there is no configuration for this
tenant", "the configuration does not register this client".

```yaml
- name: no-configuration
  when_related: {via: input.tenant, exists: false}
  error: demo.signin.NoConfiguration
- name: no-redirect-entry
  when_related: {via: input.tenant, predicate: redirect_client != input.client}
  error: demo.signin.NoRedirectEntry
- name: initiated
  creates: demo.signin.SignIn
  instance: sign_in_id
```

- **The row.** `via: input.<field>` names an input whose type is exactly one entity's
  identity, resolved as `{related: {via, field}}` resolves it (`related_value::referenced_entity`,
  a `references` relation on the field a creating branch stores the input in breaking a tie). One
  hop, from the input — or, from `ess/22`, from a stored field of the addressed subject (below). A
  lookup by any other field is a query, and stays out of scope.
- **An Optional reference (`ess/22`, beyond10x/ess#304).** The input may be `Optional<…>` of that
  identity; the guard is then checked only when present. The `Optional` wrapper is removed only to
  find the entity; `ResolvedRelatedVia` keeps the declared type. An absent reference reads no row
  and selects no `when_related` branch — it is not a missing row — so the branches that read no
  related row (an input-guarded `when:` branch, the default) must answer it exactly once, or
  validation refuses the command with `non_exhaustive_branches` naming the absent reference. A
  present reference is read as a required one is. Under `ess/21` and earlier the Optional form is
  refused with `unsupported_format_version` naming `ess/22`.
- **A stored reference (`ess/22`, beyond10x/ess#304).** `via` may instead be a bare `<field>`: a
  stored field of the subject the command addresses through its input, read as the subject held
  it just before the branch — "a task is completed only once the task its stored `blocked_by`
  names is `Done`" (`via: blocked_by`, `predicate: state != Done`, beside `exists: false` and a
  default that moves the task). The field is typed as the other entity's identity or
  `Optional<…>` of it; a `references` relation the subject declares on it breaks a tie, and the IR
  carries it as `ResolvedRelatedVia::Subject` at its declared type. An absent stored reference
  reads no row and selects no `when_related` branch, as an absent Optional input does, with the
  same exhaustiveness rule. It is read at step 5 of [the precedence order](#the-precedence-order):
  after the input-guarded refusals, the addressed row's existence and its held state, and before
  every accepting branch — so a present-related refusal answers before an accepting branch it
  overlaps, `wrong_state:` composes with it unless an accepting `when_related:` branch moves the
  subject (then refused `conflicting_declaration`, as for an input `via`, until a precedence for
  it is designed), and a missing row's
  `exists: false` answers only once the addressed row has. It is refused on a command that
  creates its subject (`conflicting_declaration`: no row holds the field before the branch), on
  one that addresses no existing subject through its input, or a field the subject does not
  store (`undeclared_reference`), and beside an input `via` in one command. Under `ess/21` and
  earlier a bare `via` the command could read — a stored field typed as one entity's identity — is
  refused with `unsupported_format_version` naming
  `ess/22`; any other keeps the `type_mismatch` it always had.
- **Two tests.** `exists: false` is taken when no row carries the identity. `predicate:` is taken
  when the row exists and the predicate — over its declared stored fields and, as in a
  `when_subject` predicate, the input under `input.` — holds. A missing row makes the predicate
  `Unknown`, so it selects only `exists: false`: never a predicate branch, never the default. A
  command with a predicate branch therefore declares its `exists: false` branch, or is refused
  (`non_exhaustive_branches`); `exists: true` is refused in favour of the default or a predicate.
- **Precedence.** As [the precedence order](#the-precedence-order) states. An `exists: false`
  branch may not carry `when:` (`conflicting_declaration`), so every missing row has exactly one
  answer, and an accepting `when:` branch or an input-guarded refusal that overlaps it is legal.
- **Any branch.** The guard reads a row the command does not address, so it sits beside a
  `creates:`, on a refusal that names no subject, or on a branch that moves or updates one, and
  composes with `when:` — except on the `exists: false` branch, as above.
- **Authority.** Beside `when_subject`, `when_subject_state`, `when_state_changes`, `external`,
  `wrong_state`, `unknown_instance`, `input_absent`, `existing_instance` or `replays` on one branch
  it is `conflicting_declaration`. In one command it remains refused beside `when_subject*`,
  `unknown_instance` and `input_absent`. From `ess/22`, `wrong_state:` may coexist with a
  `when_related:` predicate refusal: the addressed row's wrong state answers first, then a present
  related row may refuse the request. `existing_instance:` sits beside it, answering first
  ([the precedence order](#the-precedence-order)). Through `ess/21` one command reads one related row, and
  declares at most one `exists: false` branch; from `ess/22` it may read several through its input
  ([below](#several-related-rows-ess22-beyond10xess283)), with at most one `exists: false` branch each.
- **Validation.** The rows that exist are partitioned jointly with the input, as the stored-field
  partition does; where the finite prover declines — a comparison with the input, an open domain —
  the command needs a genuine default.
- **Format.** Below `ess/18` the key is refused with `unsupported_format_version` at
  `outcomes.<name>.when_related`, in YAML and JSON sources alike.
- **Related refusal beside wrong state (`ess/22`, beyond10x/ess#282).** A command may combine a
  present-row `when_related:` predicate refusal with its own `wrong_state:` outcome. Declaration
  order does not decide the answer: after the earlier missing-row and input-refusal steps, the
  addressed row's lifecycle is checked first. A moving acceptance in a wrong state therefore takes
  `wrong_state:` without emitting an event or changing storage; from an allowed state the related
  predicate may refuse it, and a related row that admits it reaches the acceptance. A nonmoving
  acceptance remains independent of the moving sibling's source-state requirement. Through
  `ess/21`, the combination retains its `conflicting_declaration` refusal.
- **Several related rows (`ess/22`, beyond10x/ess#283).** <a id="several-related-rows-ess22-beyond10xess283"></a>
  A command may guard on more than one row, each named by an input `via` of its own, required or
  `Optional<…>`: "a run starts only on a switch that is not paused, under a capability that is not
  revoked" (`no-such-switch` and `switch-paused` through `input.switch`, `no-such-capability` and
  `capability-revoked` through `input.capability`, beside a default that starts the run). Each row is
  validated as a lone row is: its entity resolved from its own `via`, its predicates checked against
  that entity, at most one `exists: false` branch over it, and one wherever a predicate reads it.
  The precedence across rows is declaration order (the story's decision): at step 1 the rows are
  read in the declaration order of their `exists: false` branches and the first missing one
  answers, so any missing row answers before every present row's predicate; at step 5, after the
  addressed row's existence and held state, the first declared present-related predicate refusal
  whose predicate and input guard hold answers, across rows, before every accepting branch —
  whether or not `wrong_state` is declared. Two refusals over one row that both hold stay
  ambiguous (`conflicting_declaration`), as on a command reading one row, and so do two accepting
  branches over different rows: only refusals are ordered across rows. Beside `wrong_state:` every
  present-related branch must refuse, as for one row. Validation partitions every row's fields —
  and an Optional row's absence, which selects none of its branches — crossed with the input, under
  the finite prover's 64-assignment cap; past it, or over an open domain, the command needs a
  genuine default, and beside one every two accepting branches over different rows that can both
  hold — each on its own row, with an input both guards admit, a side the prover declines counting
  as one that can — are still refused `conflicting_declaration`: the partition fails closed rather
  than admitting them unchecked. A stored-field `via` stays alone: a second one is refused, and so
  is one beside an input `via`, since its row is read at step 5 and no order between the two kinds
  is designed. Under `ess/21` and earlier a second row is refused with `unsupported_format_version`
  naming `ess/22`. Synthesis arranges each branch's scenario around the row its guard reads — for a
  branch reading none, the row it copies a value from or files its creation under, else the first —
  with every other row present and arranged so that nothing the order answers before the branch is
  selected there by the input sent: any row beside an `exists: false` branch, no earlier-declared
  refusal beside a predicate refusal, no branch at all beside anything else. The input is searched
  with each such row's comparisons to it as well. A refusal over one row is also sent, before its
  own send, the overlaps the order decides: for an `exists: false` branch, every earlier-declared
  Optional reference left out, every row whose `exists: false` is declared later missing too, and
  each other row selecting one of its own branches; for a predicate refusal, each other row
  selecting an accepting branch or a refusal declared after it. A target reading the rows in another
  order, stopping at an absent reference, or ignoring one row, fails
  (`ess-conformance/tests/related_guard_multiple_vias.rs`, `adversary_283_pass1.rs`). A driver
  running such a command arranges the other rows first, each where no branch answering before the
  driven one can hold whatever the input; a row no such arrangement reaches — a predicate reading
  the input — refuses the run naming `arrange_related_row`, and so does an aggregate view sharing
  one related row between its rows. Generated Rust and Go behaviour (beyond10x/ess#319) reads one
  related row per command: a command reading several stays an obligation naming them, and its
  contract states this order.
- **Held state (`ess/20`, beyond10x/ess#229).** From `ess/20` the predicate also reads the related
  row's held lifecycle state as `state` — "a release needs a candidate in state `Accepted`":
  `predicate: state != Accepted` on the refusal, beside a default that moves the release. It is
  the operand `when_subject` gained at `ess/18` (#204), for another row: the same readable-field
  list (`subject_fact::readable_fields`), typed by the related entity's lifecycle enum, so an
  undeclared state is refused and the path enters the related-row × input partition as a closed
  domain. An entity cannot declare a stored field named `state`, so the root is new and no
  document that validated before changes meaning; still the operand is gated at the next format
  rather than admitted silently at `ess/18` (coordinator decision, 2026-09-30), and under `ess/18`
  and `ess/19` it is refused with `unsupported_format_version` naming `ess/20`, not as
  `unobservable_fact`. Synthesis needs nothing new: the related-row search already drives the row
  along its lifecycle and reads its held state, so the refusal is sent for a row in a refusing
  state between decoys in an accepting one, and the default the other way round; a target that
  checks only that the row exists fails the refusal
  (`ess-conformance/tests/related_guard_state.rs`). Under `ess/20` only, where the move that brings
  the row into the state the guard needs itself reads a related row of the same entity — accepting a candidate
  names a parent candidate — that inner row is arranged one level deep, fresh where its lifecycle
  starts, and the move is sent only where that row selects it; it is never searched for along its
  lifecycle, which would recurse through the same move. Anything deeper is refused, and a branch
  left without a witness by that bound says so in its `ESS-SYNTH-003` refusal. Below `ess/20`
  none of this applies — such a move stops the arrangement, and an `exists: false` creator inside
  one is not driven — so a document without an `ess/20` construct synthesizes the suite it did
  (`EssIr::format`, read by `related_guard::nests`).
- **Conformance.** `synthesize/related_guard.rs` arranges every branch of such a command. The
  `exists: false` branch is sent an identity no row carries, beside two rows of the entity that
  carry others, so an implementation answering "some row exists" fails it; where an accepting
  branch is guarded by the input, the witness sends an input that branch takes, so the precedence
  above is what it holds. Every other branch is sent the identity of a row the scenario creates
  between two decoys. The row is searched for as a `when_subject` branch's is — every creator and
  the moves after it, steered toward the command's predicates over the row — so each side of a
  predicate over a stored field alone (`plan == Basic`) is reached on a row holding it; each
  row/input comparison is grounded on the row's value. Each decoy is, where the search finds one, a
  row on which the same input selects another branch, so reading it in place of the named row
  answers otherwise. The arrangement carries its chain of entities: a creator that would need a
  related row of an entity it is already arranging — a folder inside a folder — stops there, and
  the next creator (the root folder's) arranges the parent; the new row is then bound from the
  branch's own events, not the parent's. `existing_instance:` is witnessed by a creation with the
  related row arranged, then the same identity sent naming a related row that does not exist:
  the identity is checked first, so a target reading the related row first answers
  `exists: false` and fails. A refusal naming no subject, beside a branch that acts on an existing row, is sent for
  a row that branch could act on, so the related row is the only thing that refuses it. A driver
  running such a command to arrange a row for another scenario arranges the related row first, as
  the command's own scenario does, under the witness it arranges: a further row it creates — a
  ranked companion, a second source state (beyond10x/ess#111) — carries its own supplied identity
  and its own related row. Where the related row is the row the creation is owned by, named
  through the same input field (an entry posted into an account that owns it), that row is
  arranged once, as the related row. A predicate with two or more connective children is
  witnessed once more per child on a further related row isolating it — a conjunct refuted with
  every other held, a disjunct held with every other refuted — asserting the branch the command
  answers there, as a `when_subject` conjunct is (#155, #204); a boundary no bounded arrangement
  reaches is refused under the branch's scenario id (`ESS-SYNTH-003`). Every other family that would send the command — a boundary,
  an unknown identity, an illegal move — has no row to point it at and refuses with the strategy
  `arrange_related_row` named. Through an Optional reference (`ess/22`, #304) the branch an absent
  reference selects is witnessed once more in its own scenario, on a further instance sent without
  the reference between two related rows a predicate refusal would select, and an unknown addressed
  identity is sent without the reference; no new scenario id. Through a stored reference the row
  (or its absence) is arranged as above, and the subject's creation is rewritten to name it — the
  act that wrote the field from its input unchanged, the link `{related: …}` reads — so the
  command is sent naming only the subject: a present row between decoys on which the same request
  answers otherwise, an identity no row carries beside rows that carry others, or the field left
  out between rows a predicate refusal would select. The missing row is witnessed only where the
  act that writes the field stores an identity unchecked; where every writer refuses an identity
  no row carries first, no run holds one, and the branch is refused as unreachable
  (`ESS-SYNTH-003`). A move that brings a row along its lifecycle through such a command is sent
  with the row's reference left out where that selects it — so rows of an entity that block each
  other are driven without a second row — and otherwise, for a required reference, naming a row of
  the related entity arranged one level deep that selects it; where neither run exists the route
  is refused naming the stored reference, not an unreachable branch
  (`ess-conformance/tests/related_guard_stored_reference.rs`).
- **Runtimes.** The interpreted target answers a missing related row by its `exists: false`
  branch — of several rows, the first declared missing one (#283) — and on a stored row evaluates
  its related predicates after the addressed row's lifecycle and before accepting or external
  branches ([the precedence order](#the-precedence-order)). An absent Optional reference performs
  no lookup and selects no related branch. A stored reference
  is read from the addressed row once its existence and held state have answered. It
  declines a command with `existing_instance:`, reporting such a scenario `unsupported`.
  Entity Runtime refuses the command with `RelatedGuardUnsupported`: an entity-core operation
  reads its arguments and the one row its request names.

## The precedence order

One order answers every command, whichever branches it declares (coordinator decision,
beyond10x/ess#227 correction 1). Every other design note links here rather than stating an order
of its own:

1. on a command with a `when_related:` branch reading an input, `existing_instance` then `exists: false` (#211 revision); an absent Optional reference (`ess/22`, #304) reads no row, so neither `exists: false` nor step 5 answers it; several rows read through the input (`ess/22`, #283) are read in the declaration order of their `exists: false` branches, and the first missing one answers;
2. input-guarded refusals, the first declared whose guard holds (#209, #227);
3. existence of the addressed row (`unknown_instance`, and `existing_instance` on commands without `when_related`);
4. the held state: `when_subject_state` and `when_subject` select by it; `wrong_state` answers only
   where the branch step 6 selects moves from a state its move does not start from, so an accepting
   branch that moves nothing answers in every state, as Entity Runtime admits it (beyond10x/ess#235);
5. from `ess/22`, where the command also declares `wrong_state`, on a present related row the
   `when_related:` predicate refusal whose predicate and optional input guard hold
   (beyond10x/ess#282); overlapping related refusals remain ambiguous, and commands without this
   composition retain declaration order. On a command reading several rows through its input
   (`ess/22`, #283) this step applies with or without `wrong_state`: the first declared
   present-related predicate refusal whose predicate and input guard hold answers across rows, and
   two that hold over one row remain ambiguous. A `when_related:` reading a stored field of the addressed
   subject (`ess/22`, #304) answers entirely here, whether or not `wrong_state` is declared: its
   field read as the row held it before the branch, absent selecting no related branch, an
   identity no row carries `exists: false`, a stored row its predicate refusal;
6. accepting and external branches in declaration order (#217).

There is no cycle: step 1 applies only to `when_related` commands, which Entity Runtime does not
lower. The kernel half of the order — input refusals decided before any row is loaded, the first
declared answering — is what Entity Runtime's lowering produces, pinned by
`crates/generate/ess-entity-runtime/tests/refusal_beside_state.rs`. Entity Runtime's executor
answering a revision conflict on the addressed row before the input refusals contradicts step 2
against step 3; that is filed as an Entity Runtime defect, and ESS is not modelled on it.

Validation (`ess_domain::command::subject_state`), synthesis (`synthesize::sibling_refusals`,
`synthesize/related_guard.rs`) and the model interpreter (`interpret::execute`,
`related_absent` then `refused_by_input`) follow it.

An input refusal guarded by the held state as well (`when_subject:` beside `when:`) answers at
step 4, not step 2. Synthesis witnesses the step 3/4 boundary for it: the unknown-instance and
wrong-state scenarios also send an input it claims and require their own answer
(beyond10x/ess#454, `outcome-shapes.md`).
