# Binding arrangement and drop observation (#266, #267)

Status: coordinator contract approved by final independent design review on 2026-10-03.
Synthesis, the interpreter's dispatcher and the native, Go and TypeScript runner controls are
implemented (`ess-conformance` `synthesize/binding_effects.rs`; `tests/binding_arrangement*.rs`).
Execution through generated Rust and Go servers and delivery-context destinations are pending. This
clarifies the existing eventual-binding model; it adds no source syntax.

## Execution authority

A binding is part of the running system. Sending a command explicitly from a scenario is not
evidence that an event caused its binding to invoke that command. Required flow/delivery/failure
controls run through the actual component dispatcher and its storage/event ports. The native
interpreter currently executes no bindings; a test fixture may host that interpreter behind an
actual binding dispatcher, but must not synthesize the expected invocation or settled view from
the suite. Generated Rust and Go server paths must also execute the required controls.

The target records invocations at the dispatcher-to-command boundary, before the command returns.
Observation includes refused attempts. A refusal producing no event is still an invocation.
Events, command attempts, state changes and retry counts are different observations; none stands
in for another. Invocation observation remains correlated to the scenario as its existing target
contract requires. Unsupported observation cannot satisfy acceptance.

The old `ConformanceTarget::observe_invocations` comment promises that an untraced target can
prove every flow/delivery/failure policy, with only mapping unsupported. That comment predates the
shipped exact-attempt and every-invocation instructions and is no longer true for those claims.
Update the public target documentation and generated Go/TypeScript adapter contracts accordingly:
positive flow/effect properties can still be proved without tracing where the emitted suite asks
only for effects; mapping, exact retries, drop's no-retry guarantee and conditional zero-invocation
properties require invocation observation. This bundle's required control fixtures implement it.
The method remains optional with its default Unsupported, so untraced adapters still compile and
report their actual missing capability; this does not turn an event observation into a proof.

`observe_invocations` returns a cumulative, non-consuming snapshot of all attempts of the requested
binding/command since the scenario correlation began. Repeated calls and later steps with the same
correlation retain earlier attempts, including indistinguishable repeated inputs. The target does
not filter by expected input or success, deduplicate identical records, drain its log, reset on a
new deadline, or mix another correlation. The deadline bounds the observation wait; it is not a
new observation baseline. Implement this promise in the required real adapters and test it with
repeated reads, two identical attempts, later-step reads, and correlation isolation. An adapter
which cannot provide it returns Unsupported. This makes explicit the cumulative authority already
required by shipped exact-count retry checking; no serialized DTO/step meaning is changed. A wire
adapter that needs a changed persisted envelope must version it before adoption, not silently
reinterpret an older remote reader.

## Arrangement with eventual bindings (#266)

After a setup command emits an event that can trigger a state-moving binding on its arranged row,
synthesis may not race that binding by sending the bound command explicitly as its next route edge.
The route state records both the directly established row and pending binding effects. It advances
past an eventual binding only through the binding's declared invocation, an arranged applicable
outcome, and an observation establishing the resulting state. The observation is eventual within
the existing scenario window; there is no sleep chosen to make a race disappear.

For the minimal Create -> Created -> Start chain, Create arranges a job whose stable reachable state
is Started. A route must not issue Start after that Create, nor assert the transient New state in
an immediate view expectation. It may assert unchanged identity/fields immediately, and Started
eventually. The binding-flow scenario observes the actual Start invocation and resulting event/state.
If every route to New triggers this binding, the direct Start/outcome/started scenario is explicitly
not arrangeable without racing the binding, with its binding-flow witness identified in coverage.
This is a coverage disposition, not an executed direct scenario or a skipped green result.

Follow deterministic binding chains to their stable effects for arrangements. Conditional binding
predicates and mapped identities must be decided from the actual arranged input/event values;
unknown or externally chosen branch effects cannot be guessed. An unresolved chain, nonterminating
cycle or unobservable final state produces a named synthesis limitation at the route and binding,
while other independent scenarios remain available. Keep independent fields as facts only when no
possible pending effect writes them. This does not introduce general binding-completion authority
for arbitrary aggregate cuts; #361/#362 require the separately governed completion contract.

Do not remove all bound commands from lifecycle drivers: callers may legally invoke them on a row
arranged through a route that does not trigger the binding. Preserve that direct witness when it
exists. A target with bindings disabled must fail the binding-flow control; a target executing them
must pass without a setup race.

## Eligible destination state (#267)

`wrong_state` describes ineligible destination states; it is not a second externally chosen success
of a delivery. Select an accepting bound outcome whose input can be produced by the declared
mapping, then arrange the addressed entity in that outcome's admitted source state. Observe the
arrangement before arming a failure injection or publishing the triggering event. Reuse the
existing subject route helpers and the #266 pending-effect checks. Do not simply remove
`wrong_state` from a candidate list and assume the store now holds an eligible state.

For a mapped existing identity, arrange that exact identity and verify its stored state. A creation
needs the opposite existence condition. Nonmoving accepted outcomes remain independent of held
state according to #282. Multiple genuinely undecidable accepting outcomes retain the named
branch-undecided limitation; the fix must not pick a declaration by position and call it determined.
Delivery context and ordinary event bindings use the same eligibility operation.

The mapped destination identity must be derivable before the trigger from an admitted literal,
known trigger input, or a captured row/event value already independently observed in arrangement.
Trace the source outcome's payload expression and binding mapping/accessor together; retain every
conversion and Optional-presence obligation. A field merely named by the future event is not yet an
observation. A newly generated source field, a response-only value available only after the trigger,
or an unresolved accessor/conversion cannot supply a setup identity. In that case return the named
`BindingGap::DestinationIdentityUnavailable` under the existing binding-synthesis diagnostic family,
citing the binding and mapped input, rather than guessing the next identity or arranging after the
binding has started. The required fixture maps an already-known caller-supplied identity and must
not take this refusal. An additional source-generated-identity fixture must take it. A capture made
before the trigger is valid authority; synthesis's generator witness is not a capture.

## What drop can prove

The earlier #267 acceptance said “force delivery to fail and assert no command is invoked
(`ExpectQuiet`)”. The current force mechanism selects an external refusal **on the next invocation**,
and `ExpectQuiet` observes an event, not command attempts. Zero attempts would accept a broken
dispatcher that never delivered anything. The accepted intent is therefore bound concretely as:

1. Arrange an eligible bound subject without triggering the tested binding, then `QueryView` its
   declared immediate row view and `SnapshotSubject` selected by the actual captured identity.
2. Arm the declared external refusal after arrangement, as `ConfigureExternalOutcome` already does.
3. Publish/deliver the source event through the actual source command or ingress.
4. Check `ExpectEveryInvocation` with empty `selecting` and the full expected mapped input, then
   `ExpectInvocation` with empty `input` and `count: 1`. Each uses its existing whole eventual
   window. The latter counts every attempt, so missing delivery and any retry fail.
5. After the count window, `QueryView` that same immediate view and run `ExpectSubjectUnchanged`
   against the pre-trigger snapshot of the complete visible row. A missing/ambiguous snapshot or
   changed identity/state/field fails; no expected model row substitutes for the snapshot. Where a
   specified timed no-event property exists, retain its marked-instant
   `ExpectQuiet` check as an additional observation; it never substitutes for invocation counting.

The every-invocation expectation includes the complete mapped input, including Optional absence, and
the adapter must return every attempt of that binding/command under the scenario correlation.
The conformance implementation must additionally reject attempts with incorrect mapped input;
counting only the correct attempts and ignoring extra wrong-input attempts is insufficient for
the drop guarantee. One correct plus one malformed retry is still a failure. This is an explicit
check of the returned invocation set, not an assumption about the filtering adapter.

The forced outcome must leave the subject unchanged under its specification. Its own declared
error/response is not an escalation event, and the source event remains expected. If no external
failure can be forced or no observation can distinguish the claimed behavior, retain the precise
existing synthesis/target limitation; do not claim that an empty log proves drop.

This corrects the observation wording without changing drop's policy or the released meaning of
`ExpectQuiet` or `ExpectInvocation`. The generated drop scenario uses existing suite instructions.
If implementation needs an additional persisted observation to enforce the complete invocation set,
it must return that concrete need for version allocation before changing an old instruction's meaning.
The empty selector/count input above deliberately includes every invocation, using the existing
matching rules; the first step requires at least one correctly mapped attempt, and the final count
step catches even a later malformed retry. No invocation-log baseline is inferred from an event.

## Required controls

Use one neutral Job fixture with New/Started/Stopped states, Create, Start, an observed row view,
a Created -> Start binding and a declared injectable temporary refusal. A second creation route
does not trigger the binding, so direct Start coverage can be compared with the unavoidable case.

Named tests/scenarios:

- `binding_arrangement_waits_for_started`: healthy running binding passes; immediate-New and
  explicit-second-Start synthesis controls fail their regression assertions.
- `binding_disabled_fails_flow`: the same synthesized flow fails against a dispatcher with only
  that binding disabled, with missing invocation/result evidence.
- `binding_direct_route_is_preserved`: the nontriggering route still witnesses direct Start.
- `binding_wrong_state_destination_is_arranged`: flow/delivery run in New and reach Started;
  a target supplied Stopped fails the expected outcome/state, with no ESS-SYNTH-010 in the healthy case.
- `binding_drop_observes_one_failed_attempt`: one forced refusal passes with unchanged state;
  zero attempts, two attempts, one correct plus one wrong-input attempt, and forced-success state
  mutation each fail an independent control.
- `binding_drop_late_retry_fails`: expose an extra attempt near the end of the same deadline;
  the runner must not pass immediately after seeing count one.
- `binding_invocation_history_is_cumulative`: repeated reads and a later step retain two identical
  attempts; wrong-correlation, draining, expected-input filtering and deadline-reset adapters fail.
- `binding_generated_destination_identity_is_not_guessed`: a post-trigger-only mapped identity
  gets DestinationIdentityUnavailable while the input-derived and earlier-captured controls arrange.
- `binding_chain_arrangement_settles`: a two-binding deterministic chain reaches its final state;
  an undecidable/cyclic chain is named in coverage rather than invented as a settled arrangement.

Run the actual healthy/faulty binding fixtures in native, generated Rust and generated Go paths;
run their emitted suites through native, Go and TypeScript conformance runners. All required
scenarios execute with zero skipped/unsupported checks. These are independent behavior controls,
not prewritten reports. Preserve old unaffected suite bytes; owning generators regenerate changed
oracle/example suites. Affected tests, strict lint, formatting and independent source review are
required before integration. Final browser composition and release checks remain bundle gates.
