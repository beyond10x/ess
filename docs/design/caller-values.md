# The caller as a value source and a guard operand

Source format `ess/16`, suite format `ess-conformance/26` and `/27`, beyond10x/ess#168.

## The problem

Many commands take a value from the authenticated caller rather than from the request: a record is
created in the caller's account, or an action is permitted only when the caller is the agent stored
on the record. Before `ess/16` an actor declared `name`, `may` and `naming` and nothing else, and no
value source read it. The only way to write the account was `{generated: true}`, which a suite checks
for shape only, so an implementation that wrote any account passed. Making the account an input
would make the caller its authority, which is exactly what the implementation does not allow.

## The construct

```yaml
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
  - name: demo.notes.EditNote
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
```

| Piece | Rule |
|---|---|
| `attributes:` on an actor | typed fields (`Field`), one declaration per name, types that resolve; source `ess/16` |
| `{caller: <attribute>}` | a `payload:` or `sets:` source, top level or a nested-mapping leaf; the attribute's type must be assignable to the target or cross a declared conversion |
| `caller.<attribute>` | an operand of `==` or `!=` in `when:` (against an input field) or `when_subject:` (against a stored field or `input.<field>`), the other side a field of the same declared type, `Optional` aside |
| which attributes a command can read | the ones **every** actor whose `may` names it declares, at one type |

### Why every actor

Which actor sends a command is not known when the document is read. A command whose actors disagree
— one declares `account_id`, another does not — would read nothing for the second one, so it is
refused (`conflicting_declaration`) where it reads the attribute, naming the actor that lacks it. A
command no actor may invoke has no caller at all (`undeclared_reference`).

### Why only `==` and `!=` against a field

The issue asks for two things: a value copied from the caller, and "the caller is (not) the record's
agent". The second is an equality between the caller and a field. Ordering, text operators and
literals over a caller attribute (`caller.role == admin`) are a different construct — role-based
authorization — that a suite cannot witness from both sides by choosing between two callers, and
they are refused (`unobservable_fact`) rather than admitted half-checked. The expression checker
compares representations (a `Uuid` account and a `String` text are both text), so the declared
types of the two sides are compared as well (`type_mismatch`).

### Compatibility

`caller` is not a source keyword. `{caller: <text>}` is recognised by its exact shape — one key
holding a text — and a document below `ess/16` whose struct happens to have that shape reads it back
as the nested mapping it was (`caller_value::read_below_ess_16`). A `caller.<member>` path keeps
meaning a member of an input (in `when:`) or stored field (in `when_subject:`) named `caller`. Actor
attributes and a `caller.` operand below `ess/16` are refused with `unsupported_format_version`.

## Compilation

`ResolvedActor::attributes` carries each attribute's resolved type; `ResolvedPayloadValue::
CallerAttribute { attribute, type_ref }` is the value source; guards keep their `caller.` paths.
`ResolvedOutcome::decided_by_caller` records that the caller decides a branch: its guard reads the
caller, or it is an unguarded refusal (`otherwise`) after an accepting branch whose guard does, the
way the issue states the rule ("permitted only to the note's agent"). All three are
left out of the IR document when absent, so a model without them keeps its bytes.

## Projections

| Consumer | What it does |
|---|---|
| `ess-gen` HTTP status | a refusal the caller decides (its guard reads the caller, or it is the `otherwise` refusal after an accepting branch guarded on the caller) answers `403` (`http::FORBIDDEN`), ahead of the `409`/`422` its condition would otherwise give |
| `ess-gen` `OpenAPI` | every operation names the caller attributes it reads under `x-ess-caller`, beside `x-ess-may-invoke`; like that one it is an annotation, because the model states what a credential carries and not how a caller proves it |
| `ess-gen` documentation | an actor's section lists what its credential carries |
| Entity Runtime | refuses a caller value or guard with `CallerUnsupported`: entity-core decides from a command's arguments and the stored row and has no operand for who sent the command |
| `ess-synth` | a command's behaviour reads a caller value through the `Context` port. A served surface is handed the authenticated `Caller` and enforces its actor's grant before the port runs, answering `403` `{refused: "not granted", actor}` otherwise (beyond10x/ess#265); without a served surface, actor grants stay refused with `NeedsCallerIdentity` |
| `ess-diff` | reports a changed source through `describe()` ("the caller's account_id"); an added or removed actor attribute is not yet a delta of its own |

## Conformance

### The protocol

A command step carries `caller`: the attribute values of the caller it is sent as, for every
attribute its actor declares, and nothing where it declares none. `SemanticCommandRequest::caller`
and `AbsentInputRequest::caller` hand them to the target (`None` when the step names no caller). The
target sends the command authenticated as a caller carrying exactly these values, or answers
`TargetError::unsupported` — never the command sent as someone else. The reference targets
(`billing`, `oracle-fixture`) declare no attributes and answer `unsupported` to a step that names a
caller.

A suite with a `caller` takes `ess-conformance/26` (ordinary) or `/27` (coverage), the round-3 pair
`leaf_payloads` registered; `caller_values` is the construct module (`used_by`, `admit_format`), and
admission refuses the key below /26 (`UnsupportedVocabulary`). An older reader would ignore the field
and send every command as whoever it is configured to be, asserting the refusal and the account
against the wrong caller. The Go and TypeScript runtimes execute /26–/27
(beyond10x/ess#188).

### Synthesis

Nothing else in synthesis knows what a caller is, and nothing has to (`synthesize/caller.rs`):

1. Two callers, `first` and `second`, are chosen: for each actor, one value per attribute it
   declares, from that actor's type for it, by the witness builder, at a distance (`1 << 18`) no
   input witness reaches. Two actors may declare one name at different types for commands of their
   own, so each command step carries the values of the actor that sends it.
2. Under an **assignment** — which caller sends each command — every caller read is a constant. The
   IR is read with the assigned caller's values written in (`EssIr::with_commands_rewritten`): a
   `{caller: …}` becomes the literal, a guard's `caller.<attribute>` the literal fact at the
   attribute's own type (a `Boolean` is a Boolean fact and a number a number, so it compares equal
   to a stored field or input of that type). That model is synthesized by the ordinary path.
3. Every command step of the result is marked with its assigned caller.

The suite is the assignment that sends everything as `first`. A branch it cannot reach — the refusal
for a caller who is not the record's agent, where `first` made the record — comes from the assignment
that sends the command under test as `second` and everything that arranges it as `first`, and its
refusal is dropped. So the #168 refusal is witnessed with two callers: `first` creates the note,
`second` edits it and must be refused.

Then every scenario that sends a command reading the caller runs again **in the same scenario** with
the callers' roles swapped: `second` creates a note and must record `second`'s account; `first` edits
`second`'s note and must be refused. An implementation that records one fixed account, or admits one
caller by name, fails the half in which that caller's role is the other one. Scenario ids stay the
ones the model obliges; no new id family is needed.

The second run renames every instance and instant it binds (`-swapped`). It also draws a fresh value
for every literal identity it sends, whether the command creates that instance or a branch names the
input as the instance it acts on (#275, #465), and replaces every copy of it: an event payload, a
view row or parameter, an error field. A target may arrange an addressed record itself, for an
`external:` outcome it is forced into, so the first run's literal would name a record the first run
already had arranged. A struct identity is fresh in every member (#430). Where the type has no value
left — the one row of a singleton entity — the scenario takes the one-row route or keeps its first
run with a note, and never sends one identity twice. An `observed` reference is
read by the runner from the **first** occurrence of its event in the scenario — the first run's — so
the second run captures the field itself, right after the step that first expects the event, and
reads the capture instead. A scenario with a fixture prelude, a retained replay or a periodic check
keeps its first run only, because those steps are once per scenario. So does a scenario whose
second run asserts over every row of a view rather than over the rows it made — an expectation on an
aggregate view, a `counts`, `ranked` or `at` expectation, or an `excludes` that names no row:
appended, it would be read over both runs' rows. Such a scenario loses the swapped half: it still
asserts `first`'s values where they are read, but no longer shows they are the sender's rather
than fixed.

### Stated limits

- A caller attribute of a struct or collection type is readable and compiles, but synthesis leaves
  it undetermined: the literal it would be written as does not exist.
- The swapped run is appended to one scenario, so a view expectation of the second run is read over
  the first run's rows as well. The second run is left out where one of its expectations is answered
  by rows it did not make: any expectation on an aggregate view; a `counts`, `ranked` or `at`
  expectation; and an `excludes` that names no row — neither the source's identity field nor an
  instance or observed value — such as "no row with this text", which the first run's row falsifies.
  What remains is `contains` (a floor), `excludes` of one named row, and `satisfies`/`changed_by`,
  which synthesis writes only for predicates every row of the model satisfies.
- Synthesis runs the ordinary path twice, and twice more for each command that reads the caller.
