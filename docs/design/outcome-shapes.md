# Outcome shapes beyond `ess/14`

Status: implemented (beyond10x/ess#144, #145, #150, #151, #152; `story:outcome-shapes-beyond-ess-14`).
All five constructs are admitted under source format `ess/15`; below it each is refused with
`unsupported_format_version` at the key written (`unknown_instance`, `deletes`, `into`, `accepts`,
`system.preconditions`). The absence and whole-view steps are ordinary suite `22` and coverage suite
`23`. The open questions took their defaults.

Refusals under `ess/15`, by validation code:

| Code | Refused |
|---|---|
| `conflicting_declaration` | `unknown_instance:` beside another condition, twice on one command, with an effect, or with `refuses: false` and an `error:`; `deletes:` with `sets:` or beside another verb; `into:` without `creates:`; `accepts: nothing` beside a subject, event, error, assignment, replay or a condition other than `when:`/default; a precondition whose input selects a refusing branch or several branches |
| `missing_declaration` | a refusing `unknown_instance:` branch with no `error:`; a precondition leaving a required, non-fixture input out |
| `unreachable_branch` | `unknown_instance:` on a command with no `moves:`/`updates:`/`deletes:` branch reading `instance:` from input |
| `unknown_state` | `into:` naming a state the lifecycle does not declare |
| `undeclared_reference` | a precondition naming an undeclared command, input field, actor, an actor not granted the command, or a fixture the command does not declare for that input |
| `type_mismatch` | a precondition literal that is not a value of its input's type |
| `unobservable_fact` | a precondition whose guard its literal input leaves undecided, or whose command selects by the existing subject |
| `non_exhaustive_branches` | a precondition whose input selects no branch |
| `empty_change` (unchanged) | a subjectless outcome with no event, no error and no `accepts: nothing` |

Known limits: a state reachable only through a creation `into:` it is still refused by the entity's
own reachability check (`unreachable_state`); the generated explorers exclude a command with an
`unknown_instance:` branch or a `deletes:` effect, and refuse a precondition that reads a fixture
input; `ess-diff` reports a changed creation state as `outcome-subject-changed` (no separate
`CreationStateChanged` kind, so no new delta format) and a changed precondition list as an
unclassified system change; Entity Runtime lowering refuses an `unknown_instance:` branch, a
`deletes:` effect, a creation `into:` a state and `accepts: nothing` with the lowering code
`OutcomeShapeUnsupported`.

## Behavior and authority

The five issues come from retrofits onto 0.35.0 in which the implementation was the source of
truth. Each names a behaviour the implementation has and no outcome can state:

| Issue | Behaviour | What the specification says today | Cost reported |
|---|---|---|---|
| #150 | a record first appears in a state past `initial` (a call announced as already ringing) | creation always lands in `initial`; `creates:` beside `moves:` is refused (`ESS-COMMAND-004`) | the branch marked `UNMAPPED:`; three authored scenarios dropped |
| #151 | a record is removed at the end of its lifecycle | a terminal state the row never holds in the implementation | terminal refusals kept as known refusals; terminal-branch mutants unkillable |
| #145 | an id the system never held answers differently from one in a terminal state | 0.35.1 takes a declared `external:` not-found refusal; otherwise the unknown id answers the `wrong_state` outcome | a scenario expecting an accepted no-op for an id that does not exist |
| #144 | a request is accepted and nothing observable changes, on a command with no subject | `preserves:` (ess/6) covers a command with an existing subject; a subjectless outcome that emits nothing is refused (`ESS-COMMAND-007`) | the catch-all over-claims |
| #152 | every command runs inside an ambient precondition (an open session whose user row exists) | nothing; the explorer starts from an empty model | a disagreement at step 1 of every exploration |

## Constructs

### #150 — create into a declared state: `into:`

```yaml
- name: offered
  creates: example.Call
  instance: call_id
  into: Ringing
  emits: [example.CallOffered]
```

`into:` names a declared state of the created entity's lifecycle and is admitted only beside
`creates:`. Omitted, creation lands in `initial` as today, so every existing document keeps its
meaning and IR bytes. The IR's creation effect carries the state.

Consequences for synthesis: the scenario asserts `into` on the created row wherever a view projects
`state`; route search starts from every state some creation lands in, not only from `initial`, so a
state reachable only from a creation into `Ringing` is arranged through that creation. A state no
creation and no transition reaches stays unreachable and is reported as today.

### #151 — an outcome that deletes its subject: `deletes:`

```yaml
- name: ended
  deletes: example.Call
  instance: call_id
  when_subject_state: Connected     # optional, as on any subject outcome
  emits: [example.CallEnded]
```

`deletes:` is an effect beside `creates:`/`moves:`/`updates:`/`preserves:`, on an existing subject,
and one outcome does one thing to one entity as today. It may emit events and name no error. It may
not `sets:`. Synthesis asserts **absence**: after the branch, every immediate view of the entity has
no row with that identity. A command addressed to a deleted identity afterwards resolves exactly as
for an unknown instance (below), so the scenario after deletion reuses that answer.

### #145 — an unknown instance has its own outcome: `unknown_instance: true`

```yaml
- {name: no-such-call, unknown_instance: true, error: example.CallNotFound}
- {name: already-ended, wrong_state: true, refuses: false}
```

A marker beside `wrong_state:`, at most one per command, admitted on a command whose acting
branches read their instance from input. It names an error, or declares `refuses: false` for an
accepted no-op. Resolution order for an identity no record carries: `unknown_instance:`, then the
0.35.1 declared external not-found refusal, then `wrong_state:` (the 0.34 rule). The generated
seams gain the variant where the marker is declared; models without it keep their generated bytes.

### #144 — an accepted no-op with no subject: `accepts: nothing`

```yaml
- name: acknowledged
  when: flag == true
  accepts: nothing
```

An outcome with no subject, no event and no error, admitted under `when:` or as the default. It is
witnessed by the scenario requiring no error, no direct event of any declared name, and no change
in any immediate view the arrangement populated (the preservation step, over every view rather than
one subject). `preserves:` stays the form for a command with a subject.

### #152 — ambient preconditions: `preconditions:`

```yaml
# system.yaml
preconditions:
  - command: example.OpenSession
    as: example.Agent
    input: {user_id: '{fixture: session-user}'}
```

A system-level, ordered list of command invocations that synthesis runs before every scenario's
arrangement, and that the generated explorer runs before every sequence. Their outcomes must be
the declared success branch; a precondition that is refused fails the scenario as setup, not as a
finding. The model the explorer compares against starts from the state they leave. Values that a
deterministic generator cannot choose come from `fixture_inputs` (ess/13). Commands used as
preconditions are still synthesized as commands.

## Formats

- Source `ess/15`: `into:`, `deletes:`, `unknown_instance:`, `accepts: nothing`,
  `preconditions:`. Each refused below `ess/15`.
- Suite: `deletes:` needs an absence expectation and `accepts: nothing` an all-views-unchanged
  expectation. Both take a new ordinary/coverage suite pair; Rust, Go and TypeScript readers refuse
  them by version until they evaluate them.
- `ess-diff`: effect changes gain `CreationStateChanged`, and the new effect and markers are
  reported as outcome changes.

## Open questions

1. `into:` a terminal state: admitted (a record born finished) or refused? Default: admitted.
2. `deletes:` and eventual views: absence is asserted on immediate views only, as preservation is.
3. `preconditions:` and multi-tenant systems: one list per system, or per actor? Default: per
   system.
4. Whether `accepts: nothing` may carry `when_subject_state` on a command that does have a subject,
   or `preserves:` remains the only form there. Default: `preserves:` only.

## Order

`unknown_instance:` (#145) first, since `deletes:` reuses its resolution; then `deletes:` (#151),
`into:` (#150), `accepts: nothing` (#144), `preconditions:` (#152). The story may split along that
line.

## `ess/16`: an absent input as a whole — `input_absent: true`

Status: implemented (beyond10x/ess#170, `story:absent-command-input-outcome`).

### Behaviour and authority

A retrofit onto 0.36.0 found a command whose implementation answers a request that arrives with
**no body at all** with a declared error (400, "body is null") before any field is validated. That
is a different request from `{}` and from a body lacking a field. The specification had no way to
say it: the closest form, `when: not defined(text)` over a required `text`, validated and could
never be witnessed (`ESS-SYNTH-003`), and the only witnessed form made every field `Optional`,
changing the contract of every other branch.

### Construct

```yaml
- {name: body-missing, input_absent: true, error: demo.notes.BodyMissing}
```

A marker beside `wrong_state:` and `unknown_instance:`, following the same pattern
(`crates/specify/ess-domain/src/command/absent_input.rs`, beside `outcome_shapes.rs`). Admitted
under `ess/16`; below it refused with `unsupported_format_version` at `input_absent`. The branch is
decided before any input field is read, so the command's fields keep their types.

| Code | Refused |
|---|---|
| `conflicting_declaration` | `input_absent:` beside another condition (`when:`, `when_subject*:`, `when_state_changes:`, `external:`, `wrong_state:`, `unknown_instance:`), twice on one command, with an effect (`creates:`…`deletes:`, `sets:`, `emits:`, `payload:`, `replays:`, `accepts:`), or with `refuses:` |
| `missing_declaration` | an `input_absent:` branch with no `error:` |
| `unreachable_branch` | `input_absent:` on a command that declares no input |
| `type_mismatch` | under `ess/16`, a guard that as a whole cannot hold because every way it could hold needs a required path to be absent (`not defined(f)`, `missing(f)`): an input that is not `Optional`, or a field reached from one through struct fields none of which is `Optional` (`body.text`); the hint names `input_absent:` |

The last row closes the validate/synthesize disagreement #170 reports (the class of #74, #94,
#112). Negation is pushed inward by De Morgan (under `not`, `all` reads as `any` and `any` as
`all`); a conjunction cannot hold when one conjunct cannot, and a disjunction only when every
disjunct cannot. Quantifier bodies are not entered. So `missing(text)` and `all: [text == "x",
missing(text)]` are refused, while `any: [text == "x", missing(text)]` holds whenever `text ==
"x"` and is admitted with its dead disjunct, and `not: {all: [defined(text), text == "x"]}` reads
`text != "x"` and is admitted. The refusal applies from `ess/16`; below it such a guard keeps the
meaning it had and validates, as before. A positive `defined(f)` over a required input is always
true and stays admitted.

The IR carries `condition: {kind: input_absent}` and `test_strategy: send_no_input`.

### Conformance

Synthesis files one scenario under the branch's own outcome id: an
`execute_command_without_input` step (the command and its actor, no `input` key), then
`expect_outcome`, `expect_error` for the declared error, and `expect_no_event` for every declared
event. It arranges nothing. The step is a step of its own rather than an `input_absent: true` field
on `execute_command`, for two reasons: a reader that does not know it refuses the step tag instead
of ignoring an unknown field and sending `{}`, and adding a field to `execute_command` would have
touched every construction site of that step in synthesis. A suite carrying it takes the round-3
pair, `ess-conformance/26` (ordinary) and `/27` (coverage)
(`crates/verify/ess-conformance/src/absent_input.rs`); a suite without it keeps its format and
bytes. The Go and TypeScript runtimes refuse `/26` and `/27` by version, so they need no execution
support for the step, and their explorers already exclude a command whose condition kind they do
not know.

A target answers the step through `ConformanceTarget::execute_command_without_input`, which takes
an `AbsentInputRequest` (command, actor, correlation; no input). Its default body answers
`Unsupported`, so a target that cannot send a request without a body reports that one scenario
`unsupported`, never passed. Retained-replay admission counts the step as an invocation: it ends the
previous invocation's command, outcome and input binding, and it is refused while a capture is
active or a subject comparison is open, so it can stand in for neither the original nor the retry.

### Projections

- `OpenAPI`: the request body of a command declaring the marker is `required: false`; the branch
  answers `400` (`http::NO_INPUT`), described as the request with no input. Other commands keep
  their body required.
- Every generated code target (`ess-synth` Rust, Go, Web and Clap, through `synthesize_for` and
  each target's own `workspace`): refused by name with `MissingRepresentation` at
  `commands.<command>.outcomes.<branch>.input_absent`, as `Json` is — the seams decode a request
  into the command's input before any branch is selected, so they have no place to answer it.
- Entity Runtime lowering: refused with `InputAbsentUnsupported`; an absent body never reaches
  entity-core.
- `ess-diff` reports the condition as `input-absent`.
