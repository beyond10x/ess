# Running a suite

**Status: binding implementation decision, 2026-09-16.**

One question, asked three ways by an adopter: how do you run the same conformance suite at different
levels of a test pyramid; can scenarios become integration or end-to-end tests with different parts
of the system substituted; and should the generated runner be visible in version control or fetched.

They are one question — **what is allowed to vary between levels, and what is not** — and this
repository has never answered it in one place. `grep -ri "pyramid"` over the whole tree returns
nothing. What exists is scattered across a design sketch, a plan document, two trait doc comments and
an epic, and at least one part of it contradicts another. This record settles it.

## The decision, in one line

**A test level is a property of the target. It is never a property of the suite.**

Three sources already say this, and none of them is in a place an implementor reads:

> The reference implementation may be in-process. A later real implementation may use HTTP + Kafka.
> The canonical suite must remain unchanged. Only the target adapter changes.
>
> — `docs/design/ess-closed-loop-execution-conformance-design-v0.1.md` §41

> A duration claim can be checked three ways, and each is wrong as a *requirement*. Waiting on a wall
> clock is real and makes every suite slow and flaky. Advancing a logical clock is deterministic and
> most systems have none to advance. Reporting an elapsed measurement after the fact is honest and,
> on its own, cannot make the twenty seconds […] actually happen.
>
> So the interface asks for the third and permits the first two to produce it. […] **an end-to-end
> target sleeps, an in-memory target advances a clock it owns and returns immediately, and both
> answer the same question truthfully.**
>
> — `crates/verify/ess-conformance/src/target.rs:250-261`

> A target reports what it observed; the runner decides whether the specification is satisfied.
>
> — `crates/verify/ess-conformance/src/target.rs:53-54`

Everything below follows from taking that seriously.

## What this forbids: there is no skip

The constraint that makes the decision sharp, and the one most likely to be worked around by
somebody who has not read it:

> An unsupported required scenario fails the run. Every scenario a suite holds is required — the IR
> has no way to mark one optional — so there is no exception to apply here, and adding one would be
> adding the silent skip §28 forbids.
>
> — `crates/verify/ess-conformance/src/report.rs:563-565`

**A lighter target does not earn a lighter verdict.** `TargetError::Unsupported` is an honest answer
to "can you show me this", and it is still a failed run. It exists so that a target cannot be read as
agreeing with a claim it never checked — not so that a level can opt out of a scenario.

So running at a lower level means **narrowing the suite to what that level can answer**, using
`ess verify conform synthesize --component <name>` or `ess verify conform select --ids`. It does not
mean handing a partial target a whole suite and reading past the reds.

### A defect this exposes, named here rather than documented as behaviour

The two shipped runners disagree about the sentence above. The Rust report fails the run on an
unsupported scenario. The Go runner maps unsupported to `t.Skipf`, so `go test` exits 0 over a run
that answered almost nothing, and the report says the run failed.

An adopter has already had to write *"read the report document, not the exit code"* into its own
contributor instructions to survive this. That is a defect in the Go runner's exit behaviour, not a
feature of the Go runner, and it should be filed rather than relied upon.

## The four delivery modes

How the runner reaches the implementation is orthogonal to what is substituted. Four modes, of which
one is shipping.

| mode | state | what it buys | what it costs |
|---|---|---|---|
| **emitted** into the consumer's tree | shipping — `--target rust\|go\|web\|clap` | the runner is in version control: greppable, diffable, steppable in a debugger, no network at test time | one copy per consumer, and a runner fix means re-copying everywhere |
| **imported** as a library | not possible today | one versioned runner, bumped with a dependency | `ess-conformance` is unpublished, so a Rust adopter needs a checkout of this repository at a path — which makes their workspace unbuildable for anyone without one |
| **out-of-process adapter** | the named open gap | one runner, any language, no per-language emitter | a wire contract to version, and a slower, harder-to-debug loop than an in-process call |
| **interpreted** | declared, refuses every method | no implementation needed at all | decides nothing yet |

The second and third are one story: `story:a-conformance-suite-runs-against-any-realization`, whose
acceptance is *"Either `ess verify conform run` accepts a realization an adopter supplies — a process
speaking the JSON wire the `web` target already speaks would be enough — or `ess-conformance` is
published so the Rust route is open to somebody without this repository."*

The third is stated plainly in the plan document that built the current shape:

> **An out-of-process implementation cannot be run.** `ConformanceTarget` is a Rust trait, and the
> binary can only reach an implementation it was compiled with. Nothing here speaks to a target over
> a socket. […] The suite document is the same either way, and it holds no handle into any particular
> compilation, so a runner in another language can read it. **That is the half of the portability
> claim that holds today; a transport-level target is not.**
>
> — `docs/plan/ess-wave-4-the-oracle.md:240-248`

**Decision.** The emitted mode stays the supported default, and its visibility in version control is
a feature rather than a cost to be engineered away — a generated runner somebody can read is a
runner somebody can debug when a scenario goes red for a reason the report does not explain. The
other three are pursued through the epic's stories and are not alternatives an adopter can choose
today. No document should suggest otherwise.

## Substitution is expressed by linking

The adopter's question — "run the scenarios with A real and B and C substituted" — has an answer, and
it is not a flag.

**`--component` does not do this.** It is on `ess verify conform synthesize` and it narrows the
*suite*; `ess generate synthesize` takes only an input, a target, an output and a format. There is no
emission mode that swaps one component for a double.

What does it is the shape of the emitted system. The generated `System` is generic over one type
parameter per component plus one for the system's own obligations, and each domain module carries its
own refusing `Unimplemented`. "A real, B and C refusing" is a call to `System::new` and it compiles;
every B and C command answers the typed refusal naming what it owes.

That this is exercised at the granularity of a single obligation, not just a whole component, is
proved by a worked example in this repository: a deliberately corrupted realization *"reuses the
honest realization's acceptance — same store, same mint, same event — and skips only the guard"*,
replacing exactly one of eight obligations while the other seven keep their state
(`examples/billing-realization/src/corrupted.rs`).

**The limit, and it is deliberate.** The linker does not choose. Gap register D-2 states that rule
and its two refusals, and it is stated once: [linker-never-chooses.md](linker-never-chooses.md).
Restating it here would make a second home for a constraint that has one, so this record only takes
what follows from it — there is no priority, no default and no "first wins", and the only accepted
state is exactly one implementation per obligation the plan owes
(`examples/billing-realization/src/linker.rs:3-7`).

**Decision.** Obligations are the substitution seam, and substitution is a linking decision the
adopter makes in their own code. ESS will not grow a `--fake` flag, a stub component kind or a
registry that picks an implementation, because each of those is the machinery choosing between two
realizations, which D-2 forbids. A level is assembled, not configured.

## What each level owes

A target's level is decided by which methods it can answer honestly. The interface is built so that
this is additive: eight methods are the floor and the rest cost exactly the scenarios that reach
them.

| capability | owed when | a target without it |
|---|---|---|
| command execution, view reads, event observation | always | cannot be a target |
| external outcome injection | only where an outcome is declared `external:` | never reached by a specification that declares none |
| redelivery | only where a binding declares `at_least_once` | *"A system all of whose bindings deliver at most once therefore owes this method nothing at all, and it has no default body because a system with one `at_least_once` binding owes it everything"* (`target.rs:190-197`) |
| binding invocations | only under a binding's mapping aspect | reports unsupported for *exactly one* scenario per binding while still proving the flow, the delivery and the failure policy |
| instants and elapsed windows | only where a scenario states a duration | *"A window nobody held is never a window that passed."* |
| ordered scan and halting | only where a view declares an order | *"A target whose only ordered read is 'give me a `Vec`' answers `produced = rows.len()` and `halted = false` truthfully — which is a red scenario, and it is the finding."* |
| fixture establishment | only in authored scenarios that arrange entity state | the scenario cannot run; setup is an adapter capability, and inability to validate is unsupported, never an acknowledgement |

**Decision.** This table is the contract between a level and a suite. An in-process target that
answers the floor plus the clock is a component-level target and its suite is narrowed to match; a
target driving a deployed system answers more and earns a wider suite. Neither is a different suite
document, and neither is achieved by tolerating reds.

## Projecting scenarios into a foreign test framework is refused

An adopter asked whether scenarios could be turned into tests in a harness they already own, written
in that harness's own DSL. They cannot, and the reason is the same sentence the whole design rests
on:

> The moment a method answers a *question the suite is supposed to ask*, the suite has stopped
> checking the implementation and started asking it for its own verdict.
>
> — `target.rs:55-56`

A projection into a foreign DSL hands the verdict to that DSL's assertion library. Every emitter that
exists or is planned goes the other way: it ships **this repository's runner** in a new language, and
the suite document is untouched. `story:java-conformance-target` states the invariant and prices it —
*"A target re-implements the runner, never the suite"* — at one emitter module plus roughly 66 KB of
hand-written runner.

**Decision.** The sanctioned route for an existing harness is the inverse of projection: **make that
harness a target.** It already owns a running system, a way to drive it and a way to observe it,
which is exactly the eight-method floor. What it must give up is deciding whether a scenario passed.

## What this record does not decide

It does not schedule the out-of-process adapter, publish `ess-conformance`, specify a wire, or say
which of the epic's gaps is closed first. Those belong to
`epic:specification-runs-as-a-fake-backend` and its stories, and duplicating them here would create
a second place for the same plan to drift.
