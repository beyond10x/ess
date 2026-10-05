# A refusal that declares its compensating change (`compensates: true`, ess/22)

Status: implemented (beyond10x/ess#197, `story:feature-request-197`). Decision blocker
`decision-blocker:refusal-may-change-state`, answered by the coordinator on 2026-10-05 with option
B of the fit review: a branch that answers an error may change state only where it is **explicitly
marked** as a failure with effect, and every unmarked branch keeps the rule it has today.

## The gap

A service refuses a command because an upstream rejected it, and before answering it still resets
the record it was asked about: the order goes back to `Offline`, then the caller hears `Refused`.
The specification could declare the refusal or the move, never both: `refusal_mutated_state`
(`ESS-COMMAND-004`, "a refused command changes nothing") refuses a subject on any branch naming an
`error:`. So the reset went unstated and unchecked, and a service that stopped rolling back passed
every scenario.

## The rule this does not change

"A refused command changes nothing" stays the rule for every branch that does not carry the marker,
byte for byte: the same code, the same location, the same message and hint. An unmarked refusal
with `moves:`, `updates:`, `creates:`, `deletes:`, `instances:` or `emits:` is refused exactly as
before. The marker is the only way in, and it admits one shape.

## Source syntax

```yaml
- name: failed
  external: the upstream refuses the join
  error: shop.order.Refused
  compensates: true                 # the marker: this refusal changes its addressed row
  moves: shop.order.Order.reset     # the change, spelled as on any branch
  instance: order_id
  sets:                             # optional, on the same row
    failure: input.reason
```

**The marker is `compensates: true`**, a flag beside the outcome's other flags (`returns: true`,
`when_state_changes: true`). It carries no effect of its own: the change is spelled with the keys
ESS already has for it — `moves:` or `updates:` with `instance:`, and `sets:` — which is what the fit
review required ("ESS already spells the move as `moves:` / `affects:`"; the requester's
`compensates:` *as an effect container* stays refused). `compensates: false` is the same document
as leaving the key out, as `returns: false` is.

## Validation

| rule | code | location |
|---|---|---|
| `compensates: true` under a source below `ess/22` | `unsupported_format_version` (`ESS-COMMAND-009`), the only error the branch earns | `…outcomes.<o>.compensates` |
| `compensates: true` on a branch that names no `error:` | `conflicting_declaration` (`ESS-COMMAND-004`) | `…outcomes.<o>.compensates` |
| `compensates: true` on a refusal that declares no change (no `moves:`/`updates:` + `instance:`), or an `updates:` that sets nothing | `missing_declaration` (`ESS-COMMAND-005`) | `…outcomes.<o>.compensates` |
| `compensates: true` on a refusal not decided by `external:` | `refusal_mutated_state` (`ESS-COMMAND-004`) | `…outcomes.<o>` |
| `compensates: true` beside `creates:` or `deletes:`, or beside `affects:` | `refusal_mutated_state` (`ESS-COMMAND-004`) | `…outcomes.<o>` |
| `compensates: true` beside `emits:` | `refusal_mutated_state` (`ESS-COMMAND-004`), the existing emits rule, unchanged | `…outcomes.<o>` |
| an unmarked refusal with a subject | `refusal_mutated_state` (`ESS-COMMAND-004`), unchanged text | `…outcomes.<o>` |

Every refusal reuses `ESS-COMMAND-004`'s code where the rule is the one it already states ("a
refused command changes nothing" outside the one admitted shape). The two misuses of the marker
itself take the nearest existing code (`conflicting_declaration`, `missing_declaration`), and the
format gate takes `unsupported_format_version`. No new `ValidationCode` is needed. `preserves:`
beside the marker is refused by the existing preserving-outcome rule. `instances:` beside it is
refused by the existing set-effects rule (a refusal changes no row `instances:` selects).

### Which branches may carry it

**`external:` refusals only** — `external:` alone and `external:` with `when:` (input eligibility).

- **Input-guarded refusals (`when:` + `error:`, a refusing default) may not.** They answer before
  the addressed row's existence and held state are read (`docs/design/input-guard-overlap-precedence.md`,
  step 1; `refused_by_input` in the interpreter). An effect there would either move them after
  those reads — reordering the precedence of every other refusal, which this design must not do —
  or act on a row whose existence was never decided.
- **`wrong_state:` / `unknown_instance:`** answer for a row no move starts from, or no row at all.
- **`when_subject_state:`, `when_subject:`, `when_related:`** refusals are decided by stored data;
  the second-adopter cases the fit review names (a sign-in counter, a declined capture) need their
  own precedence argument and a later design. **`input_absent:` / `existing_instance:`** address
  no row this branch could change.

Each is refused as `refusal_mutated_state` with a message naming `external:` as the one admitted
condition.

### One addressed instance, no creation, no events

The effect is one change of the row `instance:` names: a move along a declared transition, field
writes with `sets:`, or both. No `affects:` fan-out, no `instances:` set, no `creates:`, no
`deletes:`, and no `emits:` — no consumer need for an event on a refusal has been shown, and the
existing emits rule refuses one unchanged.

## Meaning

- **The error payload is unchanged.** The error and its `payload:` block are what they are on any
  refusal (ess/19); a `{subject: …}` source in the error payload reads the row as it was before the
  change, as an event payload source does on an accepting branch.
- **The change is part of the decision, so precedence is untouched.** The branch is selected
  exactly where an external refusal is (the forced external answer). Before it acts, the addressed
  row is read as it is for an accepting `moves:` of the same command: a row nobody holds is the
  command's not-found / `unknown_instance:` answer, a row resting outside the transition's `from`
  states is its `wrong_state:` answer
  (`docs/design/cross-record-and-stored-field-guards.md#the-precedence-order`). No other refusal
  moves.
- **The move is a cause.** A transition only a compensating refusal performs is not reported as
  `missing_causation`: the refusal is its cause (`validate_lifecycle_causes`).
- **Invariants hold at rest.** The row after the change is held to its entity's invariants, as on
  any acting branch.

## IR shape

`ResolvedOutcome` gains `compensates: bool`, left out of the document when `false`, beside the
`subject`, `sets` and `error` it already carries. A model without the marker keeps its IR bytes.
The flag is a pure function of the other two (a branch with both an `error` and a `subject` is
admitted only when marked), and is written anyway so a consumer can refuse it by name without
re-deriving it.

## Suite persistence

**No new suite major.** The scenario uses steps every reader already executes:

1. the arrangement: the addressed row brought into a state the move starts from;
2. `configure_external_outcome` forcing the branch, `execute_command`, `expect_outcome`;
3. `expect_error` with the declared error (and its determined payload);
4. `expect_no_event` for every declared event;
5. the read-back: `query_view` / `eventually_view` over an immediate view of the entity, requiring
   the addressed row in the transition's target state with the `sets:` values.

The transition's own scenario (`<Entity>/transition/<move>/by/<Command>/<outcome>`) repeats steps
2–5 from every further state the move starts from. Models without the marker keep their suite
bytes: 259 model variants (136 as written, 123 relabelled `ess/22`) hash identically against base
`7ea664ac6` (the `adversary_e_u6_bytes` probe), the one new line being this design's fixture.

How each faulty target fails it:

| target | fails at |
|---|---|
| answers the error and does not change the row | step 5: the row is still in its arranged state |
| changes another row instead | step 5: the addressed row is unchanged |
| changes the row and answers success | steps 2–3: the branch and the error |
| answers the error after an extra change (an event, another state) | step 4 for an event; step 5 for a state other than the target |
| answers correctly from one `from` state and with success from another | the transition's scenario, step 3 from that state |

Each row is a mode of the hand-written target in
`crates/verify/ess-conformance/tests/refusal_with_effect.rs`, failed by the Rust runner and by the
Go and TypeScript runtimes replaying its transcript, while the interpreter passes the suite.

## Every lane

| lane | obligation | where |
|---|---|---|
| interpreter | apply the move and `sets:` to the addressed row, then answer the error | `interpret/execute.rs` `take` (unchanged: it already applies a subject and reports an error) |
| Rust runner | reads the steps above | no change |
| Go runner | the same | no change |
| TypeScript runner | the same | no change |
| synthesis | the scenario above, from the outcome's own arrangement; the transition's scenario forces the branch from every further `from` state (`from_source`) and there too requires the error and no event, so a target answering success from one source alone fails; a branch whose row no immediate view shows (identity plus the arrival state, or a written field for an update) is refused by name (`no_witness`) rather than synthesized without its read-back. A read-back counts only where it differs from what the arranged row already held: an arrival state the row was not arranged in, or a written value other than the arranged one — so a compensating update that only clears a field the arrangement left empty, or writes a `{generated: true}` value no scenario can name, is refused the same way (adversary F1) | `synthesize.rs` |
| diff | `outcome-compensates-changed` (`ess-diff/14`, unreleased, beside `refusal-policy-changed`): adding or removing the marker, and with it the effect, is **breaking for callers and readers** — a caller retrying after the refusal, and a reader of the row, see a different state; history is compatible (no stored shape changes). The effect itself is reported beside it as `outcome-subject-changed` | `ess-diff` |
| generated Rust and Go | a named obligation: the command's behaviour stays owed, "kept an obligation by a refusal that compensates (`compensates: true`), on `<outcome>`" | `ess-synth/src/determined.rs` |
| Entity Runtime | refused by name, `CompensatingRefusalUnsupported`, before entity-core's own `TargetDefinitionRefused` ("a refusal produces nothing durable") is reached | `ess-entity-runtime` |
| mutation | no new class: `error-swap` on its error is killed by the scenario's `expect_error`; `transition-to` on its move is killed or, where the arrival state becomes unreachable, stillborn; `sets-drop` / `sets-retarget` reach a required `sets:` field. `from-drop` on its move survives where another move of the same command starts from the dropped state — the same survivor an accepting `external:` move has (measured on this design's fixture), not one this construct adds | `mutate.rs` (unchanged) |
| docs | the outcome sentence renders the change beside "It reports …"; `spec-versions.md` `ess/22` row; `formats.md` `ess-diff/14` row; the command guide | `website/docs` |

A change of the effect while the marker stays (another transition, other `sets:`) is reported as
the ordinary `outcome-subject-changed` / `outcome-sets-changed`, classified as before.

## Not done

- Guarded, held-state and related-row refusals with an effect (see *Which branches may carry it*).
- An event on a compensating refusal.
- AEP's `AuditRecord::validate` (a refused command changes nothing) is AEP's; ESS has no AEP
  dependency, and a consumer projecting a compensating branch into an AEP audit record owes that
  mapping on the AEP side.
