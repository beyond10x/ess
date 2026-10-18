# A command the system issues itself is awaited, not driven (`system: true`, ess/27, suite /52–/53)

Status: designed, not implemented (https://github.com/beyond10x/ess/issues/542). Stage 1 is
`story:system-issued-command-is-awaited-not-driven`: the declaration, validation, synthesis, the
interpreter and the Rust runner. Stage 2 is
`story:system-issued-command-await-in-go-and-typescript-runners`: the Go and TypeScript runners.
The issue closes when both ship, and both should ship in one release (see *Format versions*).

## The gap

Every transition needs an outcome that causes it (`missing_causation`, `ESS-ENTITY-005`,
`ess-domain` `validate_lifecycle_causes`). So a lifecycle that only the system moves is modelled
as commands granted to an internal actor: a scheduler that assigns a waiting entry to a worker, or
a sweep that expires an entry after its deadline. Nothing says that actor is the system itself.
Synthesis therefore drives these commands like any other.

A minimal specification shows it. An entry waits, and the system's scheduler assigns it:

```yaml
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.Enqueue]
  - name: queueing.entry.Scheduler          # the system's own scheduler
    may: [queueing.entry.Assign]

commands:
  - name: queueing.entry.Assign
    input:
      - {name: entry_id, type: queueing.entry.EntryId}
      - {name: worker, type: queueing.entry.WorkerId}
    outcomes:
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {worker: input.worker}
        payload:
          queueing.entry.Assigned: {entry_id: input.entry_id, worker: input.worker}
      - name: wrong-state
        wrong_state: true
        error: queueing.entry.EntryStateConflict
```

`ess verify conform synthesize` (0.59.0) writes for it, among others:

```text
queueing.entry.Entry/state/Assigned/refuses/queueing.entry.Assign
    execute_command  Enqueue as Caller
    execute_command  Assign as Scheduler  {entry_id: <entry>, worker: "worker"}
    ...
    execute_command  Assign as Scheduler  {entry_id: <entry>, worker: "worker"}
    expect_outcome   wrong-state
    expect_error     EntryStateConflict
```

A real system accepts neither step. It picks the worker itself, so `"worker"` is a value synthesis
invented. Its scheduler does not take an assigned entry, so it raises no `EntryStateConflict`.
A target has two choices, and the specification checks neither:

1. observe the system's own step and report it as the answer to the driven one, which is a
   convention between the target and its reader;
2. translate "nothing happened" into an error the system never produces.

The generated runners already admit this. `go/runtime.go:1167-1169` and `ts/runtime.ts:2005-2006`
tell a target to answer unsupported where "a command whose actor is the implementation itself has
no caller a target can be". A call-routing specification reports 67 of its 129 scenarios
unsupported for this reason, on five such commands. The happy path is one of them.

## Source syntax

```yaml
format: ess/27

actors:
  - name: queueing.entry.Scheduler
    system: true                       # this actor is the system itself
    may: [queueing.entry.Assign]

commands:
  - name: queueing.entry.Assign
    await_within: PT5S                 # optional: the system takes the step within 5 seconds
    input: …
```

- **`system: true` sits on the actor**, beside `may:`, `naming:` and `attributes:`.
  `system: false` is the same document as leaving the key out, as `returns: false` is.
- **A command is system-issued exactly when it has at least one grant and every actor that grants
  it is a system actor.** A command granted to a system actor and to any other actor stays an
  ordinary, driven command. A command granted to no actor is unchanged. So grants stay the one
  place that says who issues a command, and no second key can disagree with them.
- **`await_within:` sits on a system-issued command.** It is a `Period`: positive whole seconds,
  written `PT<seconds>S`, the spelling and type a periodic binding's `every:` already uses
  (`ess-domain/src/binding/periodic.rs:11`). It claims how soon the system takes its step. That is
  a property of the system, not of the machine, in the same way an `Elapsed` is
  (`scenario.rs:2020-2031`, the `ScenarioStep` documentation on §37).

The requester also proposed a flag on the command. That was not taken, because grants already
answer who issues a command.

## Validation

| rule | code | location |
|---|---|---|
| `system: true` in a source below `ess/27` | `unsupported_format_version` | `actor.<a>.system` |
| `system: true` beside `attributes:` | `conflicting_declaration`. The system carries no credential, and a `{caller: …}` read on its commands has nothing to read | `actor.<a>.attributes` |
| `await_within:` below `ess/27` | `unsupported_format_version` | `command.<c>.await_within` |
| `await_within:` on a command that is not system-issued | `conflicting_declaration`, naming the grants that make it driven | `command.<c>.await_within` |
| `await_within: PT0S`, or another spelling | the `Period` parse refusal, unchanged | `command.<c>.await_within` |
| a binding whose `invoke:` names a system-issued command | `conflicting_declaration` (stage 1). A binding's invocation is already the system acting, and conformance observes it through `expect_invocation`. Two causes of one command need their own design | `binding.<id>.invoke` |

No new `ValidationCode` is needed. A system actor whose `may:` names a view is admitted: it is a
view only the system reads, and the read-grant rules (ess/22) apply unchanged.

## Meaning

### What the runtime asks

For a system-issued command the runtime never asks a target to invoke it. It asks the target to
**await** the command's outcome **on an instance**, within a **bound**, after the scenario's
preceding steps. The answer is one of three:

- the outcome the system reached, with the input it issued the command with and what the command
  published;
- nothing within the bound;
- unsupported.

### The instance

The awaited instance is the input named by the outcome's `instance:`, which is the record the
outcome moves or updates. The scenario names it, from a captured instance or a literal identity.
Stage 1 covers system-issued commands whose outcomes address an existing record this way.
A system-issued command with a `creates:` outcome, or with no subject (only `instances:` or
`affects:`), validates. Synthesis refuses its scenarios by name (`ESS-SYNTH-023`, below) and does
not drive them.

### Inputs: decided and minted

- **Every input of a system-issued command is decided by the system.** The await request does not
  carry the scenario's values, so a target cannot echo them back. The answer reports the input the
  system used. The runner holds each declared input field in that answer to its declared type
  (`ESS-CF-PAYLOAD`). It holds the identity input to the awaited instance (`ESS-CF-AWAIT`). It
  compares any input value the scenario states (`ESS-CF-AWAIT`). Synthesis states none, because
  the specification does not determine how the system decides. An authored scenario may state
  values for the arrangement its author built, for example the one free worker.
- **A stated relation is checked without an oracle.** Where the reached outcome writes an input
  into an event payload or a stored field (`payload: {worker: input.worker}`, `sets: {worker:
  input.worker}`), the expectations after the await read the new value `{kind: awaited, field:
  worker}`. That is the input the target's answer carried. So the event the system published, the
  row it stored and the input it reported must agree, and none of them is compared with a value
  synthesis invented.
- **A value the system mints is not an input.** It is written with the existing `{generated: true}`
  source on `sets:` or `payload:`, and the existing payload shape already checks it by type. This
  narrows the story's wording ("`{generated: true}` inputs"): `{generated: true}` is admitted only
  as a value source (`ess-domain/src/command.rs:1065-1076`, `PayloadSource::Generated`), and this design
  adds no input-level marker. When a minted value is stored in an `Optional` field and shared with
  the payload, it is #545's present mint (`story:minted-value-present-and-shared-with-its-payload`).
- **Which row the system picks** ("the best-ranked free worker") becomes a determined value once
  #543 admits the head of an ordered selection as a value source
  (`story:payload-source-reads-the-head-of-an-ordered-selection`). The specification then derives
  it, and it stops being an input. Synthesis and the interpreter compare it exactly. This design
  does not depend on #543.

### Which branches a scenario can claim

| what selects the branch | today | system-issued |
|---|---|---|
| an accepting branch (`moves:`/`updates:` + `instance:`) | `execute_command`, `expect_outcome` | `await_outcome`, then the same expectations |
| `external:`, refusal or not, `compensates:` included | `configure_external_outcome`, `execute_command`, `expect_error` | `configure_external_outcome`, `await_outcome`, `expect_outcome`, `expect_error`. The system did issue the command, and the outcome it reached is the refusal |
| the addressed subject alone: `wrong_state:`, `when_subject_state:`, `when_subject:`, `unknown_instance:` | `execute_command`, `expect_error`, subject unchanged | `expect_not_issued`: **nothing within the bound**, no event the command's outcomes publish, the subject unchanged (absent, for `unknown_instance:`) |
| the input, or rows the input names: `when:` input guards, a refusing default, `input_absent:`, `when_related:` | `execute_command`, `expect_error` | no scenario from synthesis. The system chooses the input, so a refusal that some other choice would avoid is not an observable obligation. Recorded as `ESS-SYNTH-024`, naming the branch. An authored scenario may still claim `expect_not_issued` over an arrangement its author states |

The story's decision reads every refusal of a system-issued command as "does not happen within the
bound". This design narrows that in two places. The row for `external:` keeps an external refusal
as an outcome that was reached: the system tried, and the upstream refused, which can be observed.
The input row claims nothing where the system's own choice decides. Both are reported to the
coordinator as departures from the story.

### The bound

- **With `await_within:`** the step carries `within`, and the claim is measured on the target's own
  clock, as `observe_elapsed` measures (`target.rs:405-427`). An outcome is reached with a reading
  no greater than `within`, or nothing is reached and the reading is at least `within`. A target
  with a controlled clock advances it; an end-to-end target waits. A reading that is missing, late
  for a reached outcome, or short for nothing reached fails `ESS-CF-AWAIT`.
- **Without it** the bound is the runner's own eventual deadline, the one `eventually_event` and
  `expect_no_publication` already use (`runner/no_publication.rs`). A reached outcome passes when
  it is seen. Nothing reached passes only after the runner's clock has passed the deadline. The
  runner asks again until then, so a target that answers "nothing" early is asked again rather
  than believed.

The story says the default is "a default the suite states". This design does not take that. A
default written into the suite would be a deadline carried by the suite, and §37 forbids that
because "a suite that carried a deadline would be a suite whose meaning changed with the machine
it ran on" (`scenario.rs:2023-2024`). An author who wants a stated bound
writes `await_within:`.

### Holding the system until it is awaited

A target that implements `await_outcome` does not let its system issue a system-issued command on a
scenario's rows until the runner awaits that command in that scenario. The test harness owes this
control, as it owes `configure_external_outcome` (`target.rs:274-279`, "a test adapter control,
not a runtime capability"). A controlled scheduler, or a tick the target drives, gives it. A target
that cannot hold its system answers `await_outcome` unsupported. Without the hold, a step the
system took early would satisfy no await. Worse, a state the system leaves on its own, such as
`Waiting` once a worker is free, would be read back after the system had already moved it. The
interpreter holds trivially, because it issues nothing until it is asked.

Scenarios with no await step are unchanged. A target without the hold sees the same race it sees
today.

## IR shape

- `ResolvedActor` gains `system: bool`, omitted from the document when `false`.
- `ResolvedCommand` gains `system_issued: bool`, omitted when `false`, and `await_within:
  Option<Period>`, omitted when absent. `system_issued` is a pure function of the grants. It is
  written anyway, as `compensates` is (`docs/design/refusal-with-effect.md`, *IR shape*), so a
  consumer can refuse it by name without deriving it again.

A model without either key keeps its IR bytes.

## Suite persistence

### Two steps and one value

```json
{"step": "await_outcome", "command": "queueing.entry.Assign",
 "instance": {"kind": "instance", "instance": "entry"},
 "within": "PT5S",
 "input": {"worker": {"kind": "literal", "value": "w-1"}}}

{"step": "expect_not_issued", "command": "queueing.entry.Assign",
 "instance": {"kind": "instance", "instance": "entry"},
 "within": "PT5S",
 "unpublished": ["queueing.entry.Assigned"]}

{"kind": "awaited", "field": "worker"}
```

- **`await_outcome`** requires an outcome of `command` on `instance`. `within` is present exactly
  when the command declares `await_within:`. `input` is partial: the stated values, authored only.
  The answer becomes the scenario's current command result, so every existing expectation that
  reads "the preceding command" reads the awaited one: `expect_outcome`, `expect_error`,
  `expect_event`, `expect_event_values`, `expect_no_event`, `expect_no_events`,
  `expect_direct_response`. Nothing about them changes.
- **`expect_not_issued`** requires that no outcome of `command` is reached on `instance` within the
  bound. `unpublished` has the shape and meaning of `expect_not_granted`'s field of that name
  (`scenario.rs:2188-2201`): after the window closes, each listed event is observed once, and an
  occurrence the scenario had not seen before fails the step. It lists every event any outcome of
  the command emits. The subject is checked with the existing `snapshot_complete_subject` /
  `expect_complete_subject_unchanged` pair around it, or with `expect_subject_absent` for an
  unknown instance.
- **`{kind: awaited, field}`** is a `ScenarioValue`: the value the most recent `await_outcome`'s
  answer carried in that input field. It is admitted wherever a `ScenarioValue` is (event values,
  view rows) and refused as `ESS-CF-SUITE` before any `await_outcome` has run. It is the sibling of
  `{kind: observed, event, field}` (`scenario.rs:1680-1691`), which reads what an earlier event
  published.

The scenario `queueing.entry.Entry/state/Assigned/refuses/queueing.entry.Assign` becomes:

```text
execute_command            Enqueue as Caller
expect_outcome             enqueued
capture_instance           entry
await_outcome              Assign on <entry>                      # the arrangement: the system assigns it
expect_outcome             assigned
query_view / snapshot_complete_subject
expect_not_issued          Assign on <entry>, unpublished [Assigned]
query_view / expect_complete_subject_unchanged
```

Synthesis also turns every step that drives a system-issued command to arrange a state into
`await_outcome`.

### Grants

A grant-refusal scenario keeps being driven when it sends a system-issued command as a non-system
actor and expects `expect_not_granted`. It checks something real: an outside caller must not be able
to issue the system's step. No admitted-grant scenario is written for a system actor, because the
`await_outcome` scenario is its witness.

### How each faulty target fails

| target | fails at |
|---|---|
| never takes the step | `await_outcome`: nothing within the bound |
| takes it with the bound set, too late | `await_outcome`: reading above `within` |
| takes another branch, or acts on another instance | `await_outcome` (instance) or `expect_outcome` |
| reports an input that disagrees with what it published or stored | `expect_event_values` / the view row reading `{kind: awaited}` |
| reports an input of the wrong type | `await_outcome`, `ESS-CF-PAYLOAD` |
| takes the step from a state that refuses it | `expect_not_issued`: an outcome reached |
| reports nothing but publishes the event, or changes the row | `expect_not_issued` `unpublished`, or the unchanged-subject check |
| answers "nothing" at once, without holding the window | without `within`, asked again until the runner's deadline; with it, the short reading fails |

Each row becomes a mode of `Faulty<T>` (`faulty.rs`) for the await answer. The Rust runner fails
each one. Stage 2 makes the Go and TypeScript runners fail each one too, by replaying the same
transcript.

## Target interface

One capability, and its default answers unsupported. A target written before it keeps compiling,
and each scenario that needs the capability is reported unsupported, never passed.

**Rust** (`target.rs`):

```rust
fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
    Err(TargetError::unsupported(
        format!("awaiting `{}` on an instance", request.command),
        "this target cannot observe, or hold, a step its system takes itself",
    ))
}

pub struct AwaitRequest {
    pub command: CommandRef,
    pub instance: Node,              // the identity input, resolved
    pub within: Option<Elapsed>,     // the declared bound, where there is one
    pub deadline: Deadline,          // the runner's, when there is none
    pub compared: BTreeSet<String>,  // input fields the scenario will compare: names only
}

pub enum AwaitAnswer {
    Reached { result: SemanticCommandResult, input: BTreeMap<String, Node>, elapsed_ms: Option<u64> },
    NotIssued { elapsed_ms: Option<u64> },
}
```

`compared` carries names, never values. Real targets ignore it. The interpreter reads it to answer
unsupported where the scenario compares a value the model does not determine (see *Every lane*).
The wrappers that forward a target (`Untraced<T>` in `reference.rs`, `Faulty<T>`) must forward the
method. A wrapper that falls back to the default hides the capability.

**Go** (stage 2): an optional interface, the pattern `EntitySetupTarget`, `AbsentInputTarget` and
`PeriodicTarget` use (`go/runtime.go:1204`, `:1263`, `:7127`):

```go
type AwaitTarget interface {
    AwaitOutcome(request AwaitRequest) (AwaitAnswer, error)
}
```

A target that does not implement it, or that returns `ErrUnsupported`, has the scenario reported
unsupported. The comment at `go/runtime.go:1167-1169` changes: a command whose actor is the
implementation itself is now awaited.

**TypeScript** (stage 2): an optional method, `awaitOutcome?(request: AwaitRequest):
Answer<AwaitAnswer>`, the pattern `deliverEvent?` uses (`ts/runtime.ts:2047`).

The concurrent explorers (`go/explore.go`, `ts/explore.ts`) never pick a system-issued command as a
client operation. `check-history` is unchanged, because a system-issued command is no client's
operation.

## Diagnostics

| code | where | meaning |
|---|---|---|
| `ESS-CF-AWAIT` (new `CheckCode`, `report.rs`) | runner | a system-issued command reaches the declared outcome on its instance within the bound, and reaches none where the specification says it does not happen |
| `ESS-CF-PAYLOAD` (existing) | runner | an awaited answer's input field is missing or of the wrong type |
| `ESS-CF-SUITE` (existing) | runner | `{kind: awaited}` read before any `await_outcome` |
| `ESS-SYNTH-023` (new) | synthesis | a system-issued command whose outcome addresses no existing record (`creates:`, or no `instance:`) has no awaited scenario |
| `ESS-SYNTH-024` (new) | synthesis | a refusal of a system-issued command decided by the input, or by rows the input names, is not witnessed; cover it with an authored scenario |
| `ESS-AUTHOR-044` (new) | authored | an act sends a system-issued command as a system actor. Name no actor, and the act is awaited. Name a non-system actor with `refused: not_granted` to check the grant |
| `ESS-AUTHOR-045` (new) | authored | an awaited act states no value for the identity input, so it names no instance to await |
| `ESS-AUTHOR-046` (new) | authored | an act expecting a refusal of a system-issued command, read as nothing issued, claims what only an issued command answers: `error:`, `response:`, `events:`, `capture:`, or an input value beyond the instance |

The numbers are the next free entries in each catalogue at base `260a3d1e3`. Other units in this
wave also add `ESS-SYNTH` and `ESS-AUTHOR` entries, so the coordinator assigns the final numbers in
merge order. Each entry gets a row in `website/docs/reference/diagnostics.md`.

## Every lane

| lane | obligation | where |
|---|---|---|
| domain, compiler | the keys, the validation table, the IR fields | `ess-domain/src/{actor,command,binding,system}.rs`, `ess-compiler/src/{ir,resolve}.rs`, `schemas/generated/ess.schema.json` |
| synthesis | one rewrite pass over each built scenario: a step that drives a system-issued command becomes `await_outcome` (or `expect_not_issued` for the subject row of the branch table), decided-input literals are dropped, expectations sourced from an input read `{kind: awaited}`; the two refusals; no admitted-grant scenario for a system actor | `synthesize/system_issued.rs` (new), its call where `synthesize.rs` collects scenarios, `synthesize/grant.rs` |
| Rust runner | the two steps and the value, the bound loop, `ESS-CF-AWAIT` | `runner.rs`, `runner/await_outcome.rs` (new), `report.rs` |
| interpreter | issues only when awaited. The identity input is the instance. Every other input is decided by candidates in a fixed order: for an input a `when_related:` guard reads, the identities of held rows of that entity in canonical order; otherwise the value its mint counter gives (`interpret.rs`, *Where the identifiers come from*); for an `Optional` input, absent first. The first candidate under which `execute` selects one accepting branch, or a forced external one, is reached. No candidate reached means `NotIssued`. Candidates reaching different accepting branches mean unsupported ("nothing is chosen here", `interpret.rs`). A `compared` name that the model does not determine also means unsupported. `within` is answered as `observe_elapsed` is | `interpret.rs`, `interpret/await_outcome.rs` (new) |
| authored | an act naming a system-issued command, with no actor, compiles to `await_outcome` (an accepting or external outcome) or `expect_not_issued` (a refusal outcome). Its `input:` is the stated, partial input, and the identity input is required. `events:` compile as after any act; on a refusal act, which nothing issues, `no_events:` joins `unpublished` and the claims only an issued command answers are refused (`ESS-AUTHOR-046`) | `authored.rs` |
| Go runner, TypeScript runner | stage 2: the same steps, value, bound and codes; `AwaitTarget` / `awaitOutcome?`; the majors tables; the explorers skip system-issued commands | `go/{runtime,prerequisites,replay,explore}.go`, `go/mod.rs`; `ts/{runtime,explore}.ts`, `ts/mod.rs` |
| browser replay | refuses suites `/52`–`/53` by version, as it refuses every major it does not run | `web_replay.rs`, `web_execution.rs` (test only, where the version table already refuses) |
| diff | `ess-diff/20`: `actor/<a>/system-changed {before, after}`, classified `[B, C, C]` when the actor becomes the system (it stops being a caller, as `grant-removed` is, `compatibility.rs:684-685`) and `[C, C, C]` when it stops being the system; `command/<c>/await-within-changed {before, after}`, `[C, C, C]`, narrowed or expanded. A command becoming system-issued is reported through those and the grant changes, never twice | `ess-diff/src/{change,compatibility,delta,impact}.rs` |
| generated docs | an actor renders as "the system itself"; a system-issued command renders "issued by the system", with its bound | `ess-gen/src/docs.rs` |
| `OpenAPI`, generated Rust and Go servers, entity runtime, CLI binding | unchanged, by decision: the owning component still accepts the command, because that is where the system's own scheduler sends it. The flag says who issues the command, not whether it is served | — |
| mutation | no new class. A mutant of a system-issued command is scored through the interpreter's answer to the await steps | `mutate.rs` unchanged |
| docs | `spec-versions.md` `ess/27` row, `formats.md` suite `/52`–`/53` and `ess-diff/20` rows, the actors section of `website/docs/guides/specify/commands-and-outcomes.md`, `website/docs/guides/verify/runners.md`, `website/docs/start/runners/{rust,go,typescript}.md`, `CHANGELOG.md` | `website/docs`, `CHANGELOG.md` |

## Format versions

- **Source `ess/27`**, shared with the other `ess/27` constructs. The unit that opens `ess/27`
  (`FormatVersion::V27`, its `FORMAT_HISTORY` row) is a dependency here and is not part of this
  design. `ess/26` documents keep their meaning. `system:` and `await_within:` below `ess/27` are
  refused naming `ess/27`.
- **Suite `ess-conformance/52` (ordinary) and `/53` (coverage)**. A suite gets one of them exactly
  when it carries `await_outcome`, `expect_not_issued` or a `{kind: awaited}` value. They are
  cumulative over every major below. Older readers refuse them by version before any target
  callback. A suite without them keeps its earlier number and bytes. The pattern is
  `undeclared_fields.rs` (`ORDINARY`, `COVERAGE`, `ADMITTED`, floors, `admit`) in a new
  `src/system_issued.rs`. `/50`–`/51` shipped first for a held `now` (beyond10x/ess#586), so this pair took the next
  one, in merge order.
- **`ess-diff/20`** for the two diff changes.
- **No new `ess-scenario/N`.** An authored act's keys do not change. What an act naming a
  system-issued command compiles to follows from the model, and an older `ess` refuses the
  `ess/27` model before it reads the act.
- `report/2` is unchanged. `ESS-CF-AWAIT` is a new value of a field the report already carries.

**Ship stage 1 and stage 2 in one release.** Until stage 2 lands, the Go and TypeScript runtimes
refuse a `/52` suite by version, and `--target go` / `--target typescript` refuse to emit one, so
a specification with a system actor gets no Go or TypeScript package. That is honest, but a
release that shipped only stage 1 would take those packages away from such a specification.

## Migration

- A specification that declares no system actor is untouched: the same validation, IR bytes and
  suite bytes. Each implementing unit carries a test for its part. The suite half is a probe over
  every example and fixture model, like `adversary_e_u6_bytes`
  (`docs/design/refusal-with-effect.md`).
- An adopter moves an internal actor over by writing `format: ess/27` and `system: true`, and
  optionally `await_within:`. It moves a value the system mints from the input to a
  `{generated: true}` source. After #543, it moves "which row" from the input to a selection
  source. Its target implements `await_outcome`, `AwaitTarget` or `awaitOutcome?`, with the hold.
- For such a specification the suite changes from driving to awaiting. A target that drove the
  system's commands, or answered unsupported, now reports each awaited scenario unsupported until it
  implements the capability. After that the scenarios run. For the call-routing specification above
  that is the 67 scenarios, minus any whose refusal row is the input row of the branch table.

## Alternatives considered

| design | why not |
|---|---|
| change nothing; targets observe instead of drive | the suite cannot tell an observed step from a driven one, and a refusal of the system's own step can only be reported as an error nobody raises (the gap above) |
| a flag on the command (`issued_by: system`) | a second place that says who issues a command, which can disagree with `may:` (story fit review, 4) |
| a binding: an event binding, or a periodic one | an event binding maps fields of one event and cannot say "the system picks". A periodic binding (`docs/design/periodic-binding-triggers.md`) is a host-session poll whose inputs come from host reads. It is admitted only as `at_most_once` with `drop`, and it observes ticks, not an outcome on a record |
| authored scenarios only (an author marks steps as awaited) | synthesis would still drive the command, and its refusals would stay unchecked |
| a list of awaited commands in the target's configuration | a convention outside the specification, which the issue exists to remove |

The requester's proposal is taken in substance: a declaration that an actor is the system itself,
a call `AwaitOutcome(command, instance, bound)` answering the outcome, unsupported or nothing within
the bound, decided inputs compared and minted ones checked by type. Four things changed:

- the flag sits on the actor, not on the command;
- an external refusal is reached, not "nothing";
- refusals decided by the input are not witnessed;
- with no `await_within:`, the bound is the runner's deadline.

## Not done

- Commands the system issues to create a record, or over a set of records with no subject
  (`ESS-SYNTH-023`).
- Opening the window at a marked instant (`since: <instant>`) rather than at the step. Every window
  here opens where the await step starts, directly after the step that made the outcome reachable.
- Holding the input the system reported to the reached branch's input guard, by carrying the guard
  as a persisted predicate. Today a target could report a branch whose guard its own reported input
  fails. Follow-up unit U-12 below.
- A binding that invokes a system-issued command (refused in stage 1).

## Implementation units

One agent each. `depends_on` is a merge order. Files are what each unit writes. Tests are named in
the story's acceptance lines. Where several units would append to one test file, each unit gets its
own file and the coordinator renames them into the story's file at merge, so that units which run
in parallel never edit the same file.

| unit | stage | depends_on | does | files |
|---|---|---|---|---|
| U-1 declaration | 1 | the `ess/27` opener | `system:`, `await_within:`, the validation table, the IR fields, schema regeneration | `crates/specify/ess-domain/src/{actor.rs,command.rs,binding.rs}`, `crates/specify/ess-compiler/src/{ir.rs,resolve.rs}`, `schemas/generated/ess.schema.json`, `crates/specify/ess-domain/tests/system_actor.rs` (new) |
| U-2 suite vocabulary | 1 | U-1 | the two steps, `{kind: awaited}`, majors `/52`–`/53`, admission and floors, coverage carriage, browser version refusal test, the tests that pin the newest major | `crates/verify/ess-conformance/src/{scenario.rs,system_issued.rs (new),lib.rs,admission.rs,coverage.rs,coverage_build.rs,count_json.rs,synthesis_seeds.rs}`, `crates/edge/ess-cli/tests/support/coverage_producer.rs`, `crates/edge/ess-xtask/src/docs.rs`, the latest-major pins in `crates/verify/ess-conformance/tests/{count_reports,field_presence,leaf_payloads,subject_guard_input,event_multiplicity,adversary_generated_docs_pass2}.rs` and `tests/support_seeds/mod.rs`, `crates/verify/ess-conformance/tests/system_issued_vocabulary.rs` (new) |
| U-3 Rust target and runner | 1 | U-2 | `await_outcome`, `AwaitRequest`/`AwaitAnswer`, the bound loop, `ESS-CF-AWAIT`, forwarding wrappers, `Faulty` modes | `crates/verify/ess-conformance/src/{target.rs,runner.rs,runner/await_outcome.rs (new),report.rs,reference.rs,faulty.rs}`, `crates/verify/ess-conformance/tests/system_issued_command.rs` (`await_outcome_unsupported_by_default_keeps_old_targets_green`, the faulty-target table) |
| U-4 synthesis | 1 | U-1, U-2 | the rewrite pass, `ESS-SYNTH-023`/`024`, the grant rule | `crates/verify/ess-conformance/src/synthesize/system_issued.rs` (new), `crates/verify/ess-conformance/src/synthesize.rs` (call site and catalogue entries), `crates/verify/ess-conformance/src/synthesize/grant.rs`, `crates/verify/ess-conformance/tests/system_issued_synthesis.rs` (new: `synthesis_awaits_a_system_issued_command_instead_of_driving_it`, `a_refusal_of_a_system_issued_command_expects_nothing_within_the_bound`, `an_old_suite_keeps_its_bytes`) |
| U-5 interpreter | 1 | U-1, U-3 | the interpreter's await answer and candidate order | `crates/verify/ess-conformance/src/interpret.rs`, `crates/verify/ess-conformance/src/interpret/await_outcome.rs` (new), `crates/verify/ess-conformance/tests/system_issued_interpreter.rs` (new: `the_interpreter_answers_an_await_step`) |
| U-6 authored | 1 | U-1, U-2; after u544 merges (`authored.rs`) | act compilation, `ESS-AUTHOR-044` to `046` | `crates/verify/ess-conformance/src/authored.rs`, `crates/verify/ess-conformance/tests/system_issued_authored.rs` (new) |
| U-7 diff | 1 | U-1 | `ess-diff/20` and both changes | `crates/verify/ess-diff/src/{change.rs,compatibility.rs,delta.rs,impact.rs}`, `crates/verify/ess-diff/tests/system_issued.rs` (new) |
| U-8 generated docs and site | 1 | U-1…U-7 | rendering, guides, reference rows, changelog | `crates/generate/ess-gen/src/docs.rs`, `website/docs/reference/{spec-versions,formats,diagnostics}.md`, `website/docs/guides/specify/commands-and-outcomes.md`, `website/docs/guides/verify/{runners,author-scenarios}.md`, `website/docs/start/runners/rust.md`, `CHANGELOG.md` |
| U-9 Go runner | 2 | U-3, U-5 | `AwaitTarget`, the steps, the value, majors, transcript replay, explorer exclusion, emitted README | `crates/verify/ess-conformance/src/go/{runtime.go,prerequisites.go,replay.go,explore.go,mod.rs}`, `crates/verify/ess-conformance/tests/system_issued_go.rs` (new: `the_go_runner_awaits_a_system_issued_command`), `website/docs/start/runners/go.md` |
| U-10 TypeScript runner | 2 | U-3, U-5 | the same for TypeScript | `crates/verify/ess-conformance/src/ts/{runtime.ts,runtime.test.ts,explore.ts,mod.rs}`, `crates/verify/ess-conformance/tests/system_issued_typescript.rs` (new: `the_typescript_runner_awaits_a_system_issued_command`), `website/docs/start/runners/typescript.md` |
| U-11 parity | 2 | U-9, U-10 | three runners, one transcript, the refusal within the bound | `crates/verify/ess-conformance/tests/system_issued_command.rs` (`all_runners_agree_on_an_await_refusal_within_the_bound`) |
| U-12 guard over the reported input (optional, later) | after 2 | U-3, U-9, U-10 | the reached branch's input guard carried on `await_outcome` and evaluated by every runner over the reported input | `scenario.rs`, `runner/await_outcome.rs`, `go/runtime.go`, `ts/runtime.ts`, `synthesize/system_issued.rs` |

U-4 and U-5 can run in parallel after U-3. U-7 can run in parallel with U-2 to U-6. U-2 to U-5 are
on the critical path.
