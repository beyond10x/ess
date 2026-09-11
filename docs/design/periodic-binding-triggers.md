# Periodic binding cause and required host-input authority

Admitted design for story:periodic-binding-trigger-contract. Implementation and consumer adoption must establish the verifiers below; this design itself is not an execution report. Source labels refer to retained private evidence; no private repository identifier is part of the contract.

## Decision

Add one closed periodic binding cause: a session-local, fixed-rate, serial poll, first due one positive period after host activation, with at most one pending tick and permitted loss of intervening ticks. Pair it with a binding-local typed host-input contract. The host supplies an authenticated session instance, samples eligibility at tick receipt, and obtains fresh read-derived inputs before invoking the declared command. No event is synthesized. Required inputs remain required.

This is an executable obligation under the declared active/eligible/available conditions, not merely a schedule label. Delivery `at_most_once` bounds attempts for one accepted occurrence; it does not discharge periodic liveness. The first supported profile permits only `delivery: at_most_once` and `on_failure: drop`, matching the measured poll. Other policies receive an explicit unsupported-profile diagnostic rather than inheriting event retry semantics.

## Evidence and limits of the claim

Consumer evidence C1 shows a two-second ticker created only after at least one backend subscription exists. There is no immediate read. The loop calls the refresh synchronously, and checks stored campaign eligibility at each received tick. C2 establishes that absent/offline/unspecified campaign state is ineligible. C1 also shows read failures return without applying state, read timeouts derive from a fresh background context, and another worker can invoke the same refresh independently. Thus this poll is neither unconditional nor the only cause of that command.

C3 supplies normalized session identity and a private session token. C4 establishes that the modeled refresh command requires identity **and** status, campaign identity/name, last reason, and the already-optional lead/metrics results. Session identity alone cannot fill that command. Required backend fields must not be recast as optional or fabricated from a timer payload.

Local Go source G1 (`time/tick.go:21–47`) documents adjustment or dropping of ticks for a slow receiver and allocates one pending channel slot. G2 (`time/sleep.go:147–159`) performs a nonblocking send with the original scheduled time. G3 (`runtime/time.go:1117–1125`) advances the next deadline on the original period grid past current time. The inspected installation identifies itself as Go 1.27.0; the consumer module declares Go 1.25.0. This is source evidence for the design, not proof of the deployed binary or its runtime settings. The public ticker contract does not promise lossless catch-up or a fixed wall-time lateness bound. The proposal therefore does not require either.

## Typed source and IR shape

Introduce `BindingCause::{Event, Periodic}` and the corresponding resolved enum. Preserve the existing `when: {event: ...}` spelling and exact event-only serialized output through custom compatibility serialization; the new branch must not add empty periodic/host fields to old documents. Require exactly one of event and periodic, with precise paths for both/neither/unknown fields.

`PeriodicCause` contains a `Period` and a closed `SessionPollProfile` with these explicit values: anchor `host_activation`, first `after_period`, cadence `fixed_rate`, overlap `serial_per_instance`, missed `one_pending_drop_excess`, lifetime `host_instance`, cancellation `stop_acknowledged`. Reject other values for this implementation; they are not a general scheduler configuration surface.

`Period` is a nonzero unsigned 32-bit count of whole seconds with canonical spelling `PT<seconds>S`. Accept 1 through 4,294,967,295 seconds; reject zero, leading zeroes, signs, fractions, alternate units, invalid syntax and overflow. Millisecond multiplication uses checked u64 arithmetic; ordinal/deadline addition and multiplication also refuse overflow. Existing `Elapsed` still admits zero and retains its bytes. Put the shared whole-seconds lexical routine below both domain and conformance, in `ess-primitives`; do not introduce a domain dependency on conformance.

The periodic branch requires a `HostInputContract` local to that binding:

- `owner`: an existing resolved component that owns the adapter boundary.
- `authority`: a validated binding-local authority identifier. It names an explicit host interface to implement, not proof that authentication has occurred.
- `context_fields`: ordinary typed fields supplied once by that authenticated host instance, including the existing declared identity type where used.
- `read_fields`: ordinary typed fields acquired afresh for each eligible received occurrence, with their actual declared types.
- `eligibility: host_boolean`: a required Boolean answer from that same host instance before reads begin. Its consumer witness is the actual stored-state predicate, not an invented event or speculative ESS predicate over missing entity state.

Use explicit mapping variants `host_context.<field>` and `host_read.<field>`, initially one field only. The compiler resolves both field tables, prohibits duplicate declarations and namespace ambiguity, and applies existing assignability/conversion rules to every required command input. A periodic binding cannot refer to `event.*`; an event binding cannot refer to the periodic host. Missing authority, missing mapping, unknown field, incompatible type, unknown owner, or unadmitted conversion is a diagnostic. Literal mappings retain their existing limitations; a literal cannot impersonate authentication or a required fresh read.

The host instance handle is opaque and runtime-only. It is scoped to one scenario/lifetime, cannot be serialized into source or suites, and must never put credentials into generated reports. The typed host result is `ContextUnavailable` at activation, or `Ineligible | Inputs(typed fields) | ReadFailed` per received tick. An absent adapter/capability is `Unsupported`; a configured required authority that cannot be acquired is an explicit arrangement failure, not an empty successful context. A runtime read failure is an observed occurrence disposition permitted by `drop`; it is not a reason to declare every test unsupported.

Context fields remain constant during a lifetime; changing session identity/authentication requires a new lifetime. Read fields are fresh per occurrence. All non-optional fields must be present and typed before command invocation; an Optional field must carry the representation already admitted for that type. Failure to obtain a required result refuses that occurrence before invoking. On the measured consumer, the session handle retains the token privately, context supplies identity, and the read provider performs status plus conditional metrics reads. A previous view snapshot is not a substitute for those fresh reads.

## Exact temporal and lifetime semantics

Let A be the target-reported activation time, P the period, and nominal deadlines D(n) = A + nP for n >= 1. These are elapsed offsets on one target clock, not epoch timestamps or a cross-clock synchronization claim.

1. **Activation and anchor.** The host activates only after its session prerequisite succeeds. A is actual poll/ticker activation, not session construction, suite start, or the runner's wall time. No periodic occurrence is due at A. Reopening creates a fresh lifetime and anchor even with the same binding and identity.
2. **Eligibility.** Sample at received-tick dispatch, including delivery of an old pending tick. An ineligible tick makes no read/command attempt and does not reset A. Becoming eligible does not cause an immediate call; the next received tick remains the cause.
3. **Rate and busy work.** Deadline phase remains A + nP independently of read or command completion. The whole eligibility/read/apply operation is serial per lifetime. Other causes and other lifetimes may overlap it; this contract does not promise a global command mutex.
4. **Missed periods.** Keep at most one pending tick while the loop cannot receive; excess due ticks may be dropped. Once idle, a retained tick can be consumed immediately, before the next future grid deadline. Its scheduled ordinal stays the old ordinal; do not relabel it as a new deadline or manufacture all missed occurrences. The profile intentionally does not require which excess ticks were serviced by a particular Go runtime. No loss is permitted as an excuse during an idle, eligible, controlled-time witness whose scheduler has been settled through the deadline.
5. **Cancellation.** A cancellation request and a ready tick can race in the actual select. Do not promise zero new calls at request time, or even a deterministic one-call race allowance. Already-started work is allowed to complete and apply its result, because the actual reads are not parent-cancelled. The lifetime ends at an explicit stop acknowledgement after the poll loop has exited and its work has quiesced. After that acknowledgement, no occurrence/read/command may originate from that lifetime. Reuse requires a fresh lifetime. A target that cannot establish stop acknowledgement refuses that capability; it cannot infer it merely from requesting cancellation.
6. **Failure.** Each received occurrence has at most one read/apply attempt. Required read failure ends it with `ReadFailed` and no command; the next scheduled poll remains possible, not a retry of the failed ordinal. Command failure follows the declared drop consequence. Neither failure changes the period phase.
7. **Lateness.** There is no numerical production wall-clock SLA in the observed code, so none is invented. Executable timeliness means dispatch at each due opportunity while idle/eligible in a controlled target that has settled that time. That target can fail early, omitted, or deferred-to-a-later-controlled-time dispatch. A passive timestamp-only target cannot prove bounded lateness or absence and must refuse the complete periodic observation capability.

## Observation seam: reuse elapsed time, add cause correlation

Existing `target.rs:173–219,555–605` supplies mark/hold/reported-milliseconds semantics. Existing `ObservedInvocation` at :737 has binding, command and inputs, but no lifetime, occurrence, timing or origin. It cannot distinguish a periodic refresh from the same command invoked by the independent push/command worker. Do not infer periodic origin from a matching payload or time window.

Add default-Unsupported target methods for opening a periodic host instance, observing a periodic window, and closing it. Opening refers to an existing marked `InstantName`, returns the runtime lifetime handle and activation offset A on that mark, and registers the complete capture scope before enabling the poll. The scope is the declared binding plus the actual host instance and scenario correlation. Opening does not directly invoke the command.

The periodic-window request reuses the `ElapsedObservationRequest` hold semantics with `watching: None`. Its response includes the existing elapsed reading, a complete-through watermark, an append-only capture cursor, and typed periodic records. One target operation holds/advances and captures that same window; there is no separate “sleep then collect” race. This requires refactoring/reusing elapsed handling internally, not changing the meaning of old elapsed responses or fabricating a watched event. The implementation fixtures advance a clock the target owns and settle work; no wall sleep is necessary.

Every record identifies `(scenario correlation, binding, lifetime, nominal ordinal)` plus a distinct invocation id where an invocation occurs, the scheduled offset, received/start/completion offsets as applicable, and a closed cause/disposition: periodic-invoked, ineligible, read-failed, dropped-range, or independently-invoked. Return observed facts, never `passed: true`. The observer captures all invocations of the declared command for the scoped host instance, including independently caused ones, so a false cause label cannot make them disappear from the comparison. Periodic operation identity continues from eligibility through reads to the actual command call; an adapter cannot stamp whatever matching command it observes after the fact.

The runner checks monotonically increasing cursors/times, reference integrity, checked D(n), unique ordinals/attempts, origin and host isolation, typed actual inputs, serial intervals, no early start and no post-ack origin. It compares the settled window to expected eligible/idle opportunities. A missing invocation is not excused by an empty record list or by an unsupported response. During blocked work, dropped ranges are permitted only where the recorded busy/receiver-unavailable interval explains them. A pending occurrence can account for at most one replay after that interval. A malformed response fails observation; incomplete observation is unsupported/inconclusive, never passing.

The watermark is an explicit target completeness obligation. Reported elapsed time alone does not prove the timer ran. The consumer adapter must instrument the real loop and distinguish the other worker, or use the same loop under an injected ticker/clock in its controlled conformance target. A target that merely implements a second model of the desired scheduler does not establish consumer adoption.

Bound retained observation material: at most 4,096 explicit occurrence records across a generated check; dropped ordinal ranges are checked, nonoverlapping closed intervals bounded to the five-period fixture. There is no expansion of arbitrary long missed ranges. Refuse an unrepresentable five-period window as a synthesis witness gap before constructing a persisted check. The baseline generated witness uses a handful of periods and synthetic host fixture values; arbitrary long-run monitoring is out of scope.

## Deciding controlled witnesses

The smallest generated supported scenario opens a real target host fixture, checks no invocation through A+P-1 second, checks one correctly attributed invocation through settled A+P, checks the second at A+2P, then closes and proves no new invocation through another settled period. For P=1, the pre-first check uses zero elapsed hold. The target reports actual A and elapsed values; the runner never treats its own polling timeout as timer evidence.

Narrow Rust and generated Go verifiers also configure the actual host fixture to test:

- Ineligible at one received tick, eligible at the next: no hidden read on the first, one call on the second, unchanged phase.
- Read work starting at 2 seconds and blocked past 4 and 6: no overlap/backlog, at most one pending dispatch on release, then return to the original 8-second grid. These integer-controlled milestones require no subsecond authored syntax.
- One read failure then success: no command for the failure, fresh inputs for the next ordinal, no immediate retry.
- An independently caused invocation with identical command/input: it cannot satisfy the missing periodic occurrence. Wrong binding, old lifetime, wrong scenario, forged duplicate ordinal, early timestamp, later controlled dispatch after an idle settled deadline, and missing record each remain nonpassing.
- Cancellation with blocked work: allow its completion before acknowledgement, forbid every occurrence after acknowledgement; reset/reopen cannot leak old records.
- Missing host authority, wrong host field type, unsupported clock/causal capture, and incomplete watermarks: explicit nonpassing refusal/error in Rust and Go, using the explicit report format and report counts rather than diagnostic process exit alone.

The controlled host fixture may be configured in the test harness to block/release its real read stub; no production scheduler control service or arbitrary mutation step is introduced into ESS. Synthesis emits an explicit required host-fixture reference and its typed contract; offline synthesis does not claim the target has bound it. Opening resolves that reference through the configured target, or returns Unsupported. Baseline success fixtures require successful reads, so substituting ReadFailed for every due invocation cannot make that witness pass. Generating illustrative values cannot silently satisfy host authority.

## Integration scope and formats

Directly implicated: `ess-domain/src/binding.rs`, `ess-compiler/src/{ir,resolve}.rs`, `ess-conformance/src/{scenario,target,runner,synthesize,admission,input}.rs` and `src/go/runtime.go`. Add a focused conformance periodic module for window validation rather than expanding event-only binding helpers with dummy events. The shared elapsed lexical extraction also touches `ess-primitives/src/{time,lib}.rs` and the existing `Elapsed` parser, retaining all old representations.

Audit every event-only caller. Current source reads identify `ess-gen/src/{docs,graph,asyncapi,openapi}.rs` and native/web `ess-synth` plan/layout/system/refusal/catalog/bridge code. Documentation and graph projection must say periodic cause and host authority; AsyncAPI must not invent a channel/event. Native owners must emit an explicit required host adapter seam or a precise unsupported projection, never silently omit the binding. These additional source paths require coordinator admission before implementation.

The new cause/host mappings and persisted periodic steps require explicit format vocabulary. Use the coordinated unreleased source ess/3 and ordinary suite/6 or coverage suite/7 vocabulary, shared with the accessor/setup wave; report/2 and target-failure/3 remain the explicit observation envelopes. The coordinator owns central source and suite selection. Implement custom event-only serialization to retain exact legacy bytes. Previous-reader tests must show refusal of the new vocabulary; current Rust/Go standalone admission must validate its typed fields, references, arithmetic and observation limits. Existing report/2 nonpassing unsupported semantics can be reused without inventing a passing legacy fallback.

Implementation order: admit this source/authority contract; implement typed cause and projections; implement elapsed-backed target observation and bounded synthesis; demonstrate controlled Rust/Go success and mutations; finally declare and exercise the actual consumer host authority. The consumer timer UNMAPPED remains until the real periodic branch and its required read/context mapping are admitted and observed. Other causes of that same command remain separate unresolved rows if they need other trigger/view constructs. Neither Flutter work, generic SDK design, cron, calendars, jitter/backoff, scheduler services nor clock synchronization belongs to this unit.

## Typed source example

This `ess/3` binding requires the named authenticated authority before activation.

```yaml
  - id: poll-status
    when:
      periodic:
        every: PT2S
        anchor: host_activation
        first: after_period
        cadence: fixed_rate
        overlap: serial_per_instance
        missed: one_pending_drop_excess
        lifetime: host_instance
        cancellation: stop_acknowledged
        host:
          owner: poll-service
          authority: authenticated-session-status
          eligibility: host_boolean
          context_fields:
            - name: agent_id
              type: String
          read_fields:
            - name: status
              type: String
    invoke:
      command: example.poll.Refresh
    mapping:
      agent_id: host_context.agent_id
      status: host_read.status
    delivery: at_most_once
    on_failure: drop
```

Native Rust, Go, Web and Clap plans retain `PeriodicHostRequired` for delivery and
transformation until that host integration is supplied. The conformance capability is
executable through the explicit Rust and Go target interfaces; an absent interface is
nonpassing. Controlled adapter evidence does not by itself establish consumer adoption.
