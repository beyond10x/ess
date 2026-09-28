# Outcome shapes beyond `ess/14`

Status: implemented (beyond10x/ess#144, #145, #150, #151, #152; `story:outcome-shapes-beyond-ess-14`).
All five constructs are admitted under source format `ess/15`; below it each is refused with
`unsupported_format_version` at the key written (`unknown_instance`, `deletes`, `into`, `accepts`,
`system.preconditions`). The absence and whole-view steps are ordinary suite `22` and coverage suite
`23`. The open questions took their defaults.

Refusals under `ess/15`, by validation code:

| Code | Refused |
|---|---|
| `conflicting_declaration` | `unknown_instance:` beside another condition, twice on one command, with an effect, or with `refuses: false` and an `error:`; `deletes:` with `sets:` or beside another verb; `into:` without `creates:`; `accepts: nothing` beside a subject, event, error, assignment, replay or a condition other than `when:`/default; a precondition whose input selects a refusing branch or several branches; a precondition literal a struct or newtype invariant is not true of (unknown included, as the setup reader requires), or whose created row a false entity invariant forbids |
| `missing_declaration` | a refusing `unknown_instance:` branch with no `error:`; a precondition leaving a required, non-fixture input out, or a struct literal in one leaving a non-optional field out |
| `unreachable_branch` | `unknown_instance:` on a command with no `moves:`/`updates:`/`deletes:` branch reading `instance:` from input |
| `unknown_state` | `into:` naming a state the lifecycle does not declare |
| `undeclared_reference` | a precondition naming an undeclared command, input field, actor, an actor not granted the command, or a fixture the command does not declare for that input; a struct literal in a precondition naming a field its type does not declare. `{fixture: name}` is a fixture reference only on an input its command declares a fixture input for, and a literal elsewhere |
| `type_mismatch` | a precondition literal that is not a value of its input's type, held to it down to every scalar leaf as an `example:` is; a list, map or struct literal is a value (beyond10x/ess#205), `null` only where the type is optional, and a `Json` or `Binary64` leaf or a union literal never |
| `unobservable_fact` | a precondition whose guard its literal input leaves undecided, or whose command selects by the existing subject |
| `non_exhaustive_branches` | a precondition whose input selects no branch |
| `empty_change` (unchanged) | a subjectless outcome with no event, no error and no `accepts: nothing` |

Known limits: a state reachable only through a creation `into:` it is still refused by the entity's
own reachability check (`unreachable_state`); the generated explorers exclude a command with an
`unknown_instance:` branch or a `deletes:` effect, and refuse a precondition that reads a fixture
input; a command whose input they cannot draw, such as a list, stays out of every sequence, and a
precondition still sends it with its literal input; `ess-diff` reports a changed creation state as `outcome-subject-changed` (no separate
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
  expectation. Both take a new ordinary/coverage suite pair; Rust, Go and TypeScript readers
  evaluate them (Go and TypeScript since beyond10x/ess#188).
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
bytes. The Go and TypeScript runtimes execute the step at `/26` and `/27` (beyond10x/ess#188), and their explorers already exclude a command whose condition kind they do
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

## `ess/16`: selection by existence — create-or-update and create-or-refuse

Status: implemented (beyond10x/ess#164 and its follow-up comment,
`story:upsert-outcome-by-existence`).

### Behaviour and authority

Retrofits onto 0.36.0 found commands that address a record by a caller-supplied identity and
choose their branch by whether a row with that identity is stored: `PUT /items/{id}` updates an
existing item and creates a missing one (#164), and a create with a caller-supplied id answers
"already exists" and changes nothing when the id is taken (the follow-up comment). No condition
could say either. Every condition either reads the input or presupposes the row (`when_subject*`,
`wrong_state:`), and `unknown_instance:` (ess/15) could only refuse or accept a no-op. Two accepted
branches side by side were refused as undetermined by input (ESS-COMMAND-004), so the
specification kept one branch and marked the other `UNMAPPED:`, and the suite never checked that
the second call updates, or refuses, rather than duplicating.

### Construct

One mechanism, two spellings, both beside `unknown_instance:` (ess/15):

```yaml
# create-or-update: the creation is the answer for an identity no record carries
- {name: updated, updates: demo.items.Item, instance: item_id, ...}
- {name: created, unknown_instance: true, creates: demo.items.Item, instance: item_id, ...}

# create-or-refuse: the refusal is the answer for an identity a record carries
- {name: booked, creates: demo.items.Slot, instance: slot_id, ...}
- {name: already-booked, existing_instance: true, error: demo.items.SlotTaken}
```

`unknown_instance:` on a `creates:` branch keeps its meaning — the branch for an identity no record
carries — and gains an effect. `existing_instance:` is the opposite marker, a new
`OutcomeCondition::ExistingInstance` (`crates/specify/ess-domain/src/command/outcome_shapes.rs`
holds both). The one-key-or-two question the story left open took two keys: the design note on
cross-record guards sketched `already_exists:`, and the coordinator named it `existing_instance:`
to read beside `unknown_instance:`. The creating half of create-or-update is not a second key
because it is the ess/15 marker with a creation. In each form the other branch is unconditional,
so the pair counts as exhaustive and ESS-COMMAND-004 has nothing to refuse.

**Precedence (coordinator decision, correction round 1).** An input-guarded refusal (`when:` with
an `error:`) is answered **before** selection by existence, in both forms: the precedence #178
fixed for a refusal overlapping an accepting branch (`input-guard-overlap-precedence.md`). A
request a declared refusal claims by its input is refused whether or not a record carries the
identity; only a request no such refusal claims is answered by the creation, the update or
`existing_instance:`. The creating half is therefore not "the first answer" the ess/15 marker is;
the generated page says so.

| Code | Refused |
|---|---|
| `unsupported_format_version` | either form below `ess/16`, at `unknown_instance` (on a creation) or `existing_instance` |
| `conflicting_declaration` | a creating `unknown_instance:` with an `error:`, `refuses:` or `replays:`; one whose payload takes the created identity from anything but an input field (`{generated: true}` is never named again); one with no sibling `moves:`/`updates:` on the same entity reading `instance:` from that input (a `deletes:` sibling does not count: that is a create-or-delete toggle); `existing_instance:` beside another condition (`when:`, `when_subject*:`, `when_state_changes:`, `external:`, `wrong_state:`, `unknown_instance:`, `input_absent:`), with an effect, twice on one command, or beside a branch acting on the existing record (`moves:`/`updates:`/`deletes:` from input, `wrong_state:`) |
| `missing_declaration` | an `existing_instance:` branch with no `error:` |
| `unreachable_branch` | a creating `unknown_instance:` on a command with no branch acting on an input-named instance (the ess/15 rule); `existing_instance:` on a command with no `creates:` publishing an input-supplied identity — `input.f`, or an optional id `{input: f, else: {generated: true}}` (the #164 follow-up: a caller that sends `f` can send it twice) |
| `unobservable_fact` | a system precondition invoking a command of either form: which branch it takes depends on a record it cannot observe before it runs |

The IR carries the creating half as `condition: {kind: unknown_instance}` with its creation, and
the refusal half as `condition: {kind: existing_instance}` and `test_strategy:
send_existing_identity`.

### Conformance

Both are witnessed by calls that share one identity, with existing steps only, so a suite keeps
the format its other steps select (no round-3 suite pair).

- The creating branch is filed under its own outcome id like any creation, with the input that
  refutes every sibling guard and an identity no other scenario sends, so a target the scenarios
  share cannot already hold it. So is the creation of create-or-refuse. These identities are drawn
  from witnesses of their own (`existence::Fresh`), past the ess/15 unknown-identity witness an
  `unknown_instance:` refusal on the same entity sends, past every arrangement witness, and apart
  from each other; the invocation that leaves out inputs read only through an `else:` literal
  (`Witness::LiteralFallbacks`) creates a second record under a witness of its own.
  `unknown_instances` no longer files a refusal or no-op scenario for such a command: an unknown
  identity creates.
- The updating branch keeps its ordinary scenario. Its arrangement creates the row through the same
  command — the first call — and it sends the captured identity again with other field values, so
  it requires the update branch and the new values in the views. `synthesize/existence.rs` adds,
  after that, a `snapshot_subject` over each immediate, unparameterised view projecting the
  identity whose filter, if any, is decided to admit the row the update leaves: it selects exactly
  one row or fails, so an implementation that stored a second row and answered the update branch
  is caught. Where views project the identity and none can carry that claim, the scenario is
  withdrawn and refused naming them. A model with no view projecting the identity observes no row
  for any update and keeps the scenario without it.
- The `existing_instance:` branch gets a scenario of its own, with one segment per creating branch
  (a creation guarded by `when:` beside the default one included): the creation with a fresh
  identity, the row snapshotted (the refused-subject observation wrong-state refusals use), the
  same identity sent again through that branch's input with other field values, then
  `expect_outcome`, `expect_error` for the declared error, `expect_no_event` for every declared
  event, and the row compared with its snapshot. A target that looks for the stored record on one
  creating path and not another fails it.
- Every input-guarded refusal beside either form keeps its own scenario, which sends the refused
  input for an identity nothing stored, and gains a segment of the same shape: a row stored under a
  fresh identity, the refused input sent for that identity, and the refusal required with no
  event and the row unchanged. Both halves of the precedence are witnessed: a target answering
  existence first (the update, or `already-booked`, for a stored identity) fails the second
  segment. Where the segment cannot be built the scenario is withdrawn and refused.

### Projections

- `OpenAPI` and the generated docs describe `existing_instance:` in their condition sentences; the
  served surface answers it with `409` (`http::CONFLICT`), a conflict with the record that exists.
- Every generated code target (`ess-synth` Rust, Go, Web and Clap, through `synthesize_for` and
  each target's own `workspace`): refused by name with `MissingRepresentation` at
  `commands.<command>.outcomes.<branch>.unknown_instance` / `.existing_instance`. The seams and
  explorers select a branch from the decoded input and the model's state machine, neither of which
  holds whether a record carries the identity.
- Entity Runtime lowering: refused with `ExistenceSelectionUnsupported` — create-or-update at the
  command, before the mixed-entrypoint refusal it would otherwise meet; `existing_instance:` at the
  branch. entity-core answers a missing row itself and selects no branch by it.
- `ess-diff` reports the condition as `existing-instance`; the creating half stays
  `unknown-instance`, with its creation reported as the outcome's subject.

### Known limits

A create-or-update whose updating branches are selected by the held state (`when_subject_state:`)
is witnessed: the creating branch reads no held state, so it is reached by input alone, both in its
own scenario and where an arrangement creates the row through it. The same with `when_subject:` or
`when_state_changes:` guards is not measured. The input-first precedence is witnessed for an
input-guarded refusal (`when:` with an `error:`) only; an input-guarded external or accepting
branch beside either form is not given a stored-row segment.
