# The two words `delivery:` accepts

The binding design page for `ess_domain::binding::Delivery`. Review F3
([`ess-review-v0.1.md`](ess-review-v0.1.md)) made `delivery:` a required word and shipped one value;
this page says what the second one means, what it obliges, and what it does to the format.

## What each word says

| word | the claim | who carries the risk |
|---|---|---|
| `at_least_once` | the transport may deliver one occurrence more than once | the **handler**: the invoked command must survive a repeat |
| `at_most_once` | one attempt, and nothing redelivers it | the **binding's `on_failure:`**: the attempt may be lost, and what that costs is declared there |

`at_most_once` promises a **bound, not an arrival**. The invocation is made once; if it does not
land, nothing in the specification delivers it again. What happens instead is exactly what
`on_failure:` already says — `retry` puts attempts back, `escalate` publishes the declared event
that says the work was lost, `drop` records that losing it is acceptable. A binding that writes
`at_most_once` and `drop` has written down, in the document, that this event's effect can disappear.
That is F3's own rule one variant along: `drop` should be something an author typed.

**Neither word is "exactly once", and this model will never spell that.** `at_most_once` excludes
duplicates; it does not exclude loss. The shape that made the word necessary — a single HTTP call
whose response nobody reads — cannot tell "did not happen" from "happened, and the acknowledgement
was lost", so the caller is owed no confirmation and the handler is owed no delivery.

The consequence a reader most often wants: **`at_most_once` does not oblige the handler to be
idempotent.** That is the whole difference from `at_least_once`, and every projection that turns the
guarantee into an obligation reads the word rather than assuming it.

## Where the word is read, and what changes

| surface | `at_least_once` | `at_most_once` |
|---|---|---|
| `ess-domain` ([`binding.rs`](../../crates/specify/ess-domain/src/binding.rs)) | unchanged | a second variant of a closed enum; nothing else in validation turns on it |
| OpenAPI ([`openapi.rs`](../../crates/generate/ess-gen/src/openapi.rs)) | `Idempotency-Key` **required** on the invoked command | no header from this binding — a required key would oblige a caller to name an invocation the model says happens once |
| AsyncAPI ([`asyncapi.rs`](../../crates/generate/ess-gen/src/asyncapi.rs)) | the word, and "the handler must be idempotent" | the word, and "the handler is owed no repeat; a lost attempt is what `on_failure` decides" |
| docs / docs-ir ([`docs.rs`](../../crates/generate/ess-gen/src/docs.rs)) | "Delivered **at least once**…" | "Delivered **at most once**…", naming what it does *not* promise |
| graph, `ess inspect` ([`graph.rs`](../../crates/generate/ess-gen/src/graph.rs)) | `at_least_once` | `at_most_once` |
| browser projection ([`web.rs`](../../crates/verify/ess-conformance/src/web.rs), [`web_replay.rs`](../../crates/verify/ess-conformance/src/web_replay.rs), [`catalog.rs`](../../crates/generate/ess-synth/src/web/catalog.rs)) | `at_least_once` | `at_most_once`; the replay mirror admits it, so a paired document is not refused for a word the model can write |
| semantic diff ([`change.rs`](../../crates/verify/ess-diff/src/change.rs)) | — | `binding/<id>/delivery-changed`, which could not previously fire |
| synthesized Rust/Go targets ([`rust/system.rs`](../../crates/generate/ess-synth/src/rust/system.rs), [`go/system.rs`](../../crates/generate/ess-synth/src/go/system.rs)) | unchanged | the same dispatch; only the package doc's sentence about the model changes |

**The synthesized transport does not fork.** The emitted pump delivers each logged occurrence to
each reacting binding exactly once, which is what `at_most_once` requires and what `at_least_once`
permits, so one dispatch satisfies both. The only two things in an emitted tree that deliver an
occurrence again are `on_failure: retry`, which holds the event for the next pump, and the
`redeliver` entry point a caller invokes — the pump never repeats one by itself. `redeliver` is
still emitted for every model with a delivery, because it is a capability of the generated host
rather than something the transport does on its own; making it conditional is a separate change and
is not made here.

## The conformance consequence

§17's binding scenario *is* the redelivery: publish the event, deliver it a second time, and require
the declared consequence anyway. `at_most_once` says the second delivery does not happen, so:

* synthesis emits **no** `RedeliverEvent` step for such a binding, and refuses the `delivery` aspect
  with `BindingGap::DeliverySingleAttempt`
  ([`synthesize.rs`](../../crates/verify/ess-conformance/src/synthesize.rs));
* `ConformanceTarget::redeliver_event`
  ([`target.rs`](../../crates/verify/ess-conformance/src/target.rs)) is therefore never reached for
  it, and a system all of whose bindings deliver at most once owes that method nothing;
* an `at_least_once` binding keeps the scenario it has, unchanged.

The refusal is a **declaration, not a defect** — the second of that kind, beside
`BindingGap::PolicySilent` for an `on_failure: drop` whose invoked command acts on no row a scenario
can read before the trigger. Where it does, `drop` is a scenario since beyond10x/ess#267: one forced
refusal, exactly one attempt carrying the mapped input, and the row unchanged
([binding-arrangement-and-drop.md](binding-arrangement-and-drop.md)). A binding still accounts for four aspects: the
`delivery` aspect appears as a named refusal instead of a scenario, which is what
`every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal` already requires.

## The format consequence: `ess/1`, with no version bump

**Decision.** `ess/1` admits `at_most_once`. No new specification format version, and no change to
canonical bytes.

AGENTS.md § Determinism and formats: "A new format version is required when meaning, identity,
references, canonicalization, names, or the persisted envelope changes." None of those moves here:

* **Meaning.** `at_least_once` means exactly what it meant. Every existing document parses to the
  same value, validates the same way and compiles to the same IR, so `source_digest()` is unchanged
  for every specification written before this word existed.
* **The envelope.** No key was added, removed or renamed; the authored document has no raw-source
  canonical hash at all ([`review-format-catalog.md`](review-format-catalog.md), the `format: ess/1`
  row).
* **The old reader.** A pre-0.22 `ess` reading `delivery: at_most_once` fails while the document is
  read, with serde's unknown-variant error naming the words it knows. That is a **refusal**, not a
  silent reinterpretation — the number moves for what an old reader does *wrong*, not for what it
  has not seen. Bumping the format would replace one refusal ("unknown variant") with a different
  refusal ("unsupported format") and cost every existing document a rewrite for a word it does not
  use.

**Precedent, in this repository.** Three additive vocabulary changes took the same decision:

| change | version | what it said |
|---|---|---|
| `refuses: false` on a `wrong_state:` branch | 0.11.0 | a new key under `ess/1`, no bump; the omission keeps meaning what it meant |
| `ScenarioId::Authored` | 0.15.0 | "The format stays `ess-conformance/2`: the id is a new word in a vocabulary that already grew once … the number moves for what an old reader does *wrong*, not for what it has not seen" |
| `--target clap` and its schema growth | 0.14.0 | "The format is still `ess/1`. Both additions serialize out when unset, so every existing document digests exactly as it did" |

The counter-example is in the same history and is the one that had to bump: `ess-conformance/2` and
`/4` grew the **step** vocabulary, and an old Go runner reading a step it did not know reported a
*failed scenario* — a wrong verdict about an implementation caused by the age of the tool. A wrong
answer forces a version; a refusal does not. `Delivery` is read by `serde`'s derived enum reader,
which has no such fall-through.

## A bound that is the component's behaviour (`ess/16`, beyond10x/ess#165)

The line below this section used to end at "No retry count". That stays true for a count that is a
deployment decision — a broker's redelivery policy, an operator's setting. It is not true for a
count that is a **constant in the sender's code**: "up to 3 attempts; a server error is retried, a
client error is final; after the last attempt the event is lost". A receiver can observe that
constant, because it decides whether the receiver sees a second invocation, so it is a claim the
specification can make and a suite can check.

**Decision.** `retry` takes a block under `ess/16`:

```yaml
on_failure:
  retry: {attempts: 3, final: [demo.ledger.Unknown]}
```

| key | meaning |
|---|---|
| `attempts` | invocations in all, the first included; at least 2, because one attempt whose failure is lost is `drop` |
| `final` | optional; refusals of the invoked command that end the retry at once: an outcome that carries `error:`, by its name, or the error, which stands for every outcome of the command that reports it |

The model has outcomes, not transport status codes, so "5xx retried, 4xx final" is written as the
invoked command's declared refusals: the ones listed are final, and every other failure is retried
up to the bound. After the last attempt the event's effect is lost, exactly as under `drop`; there
is no `then: escalate` in this build. Timing — backoff, deadline — stays out: only the count and
the final set, which the sender's code fixes, are claims.

`on_failure: retry` written bare keeps its meaning (no count) and its bytes: `BindingSpec::retry`
and `ResolvedBinding::retry` serialize only when present. The block is refused below `ess/16` with
`unsupported_format_version`; a `final` name that is no refusal of the invoked command is
`undeclared_reference`, a repeated one `duplicate_declaration`, fewer than two attempts
`conflicting_declaration` ([`binding/retry.rs`](../../crates/specify/ess-domain/src/binding/retry.rs)).
The published schema states the two limits it can: `attempts` has `minimum: 2` and `final` has
`uniqueItems: true`.

| surface | a bounded retry |
|---|---|
| IR ([`ir.rs`](../../crates/specify/ess-compiler/src/ir.rs)) | `ResolvedRetryBound { attempts, final }`, `final` resolved to the invoked command's outcome names; `ResolvedFailure::BoundedRetry`, a policy of its own so no projection renders it as an unbounded retry by leaving an arm out |
| docs, AsyncAPI ([`docs.rs`](../../crates/generate/ess-gen/src/docs.rs), [`asyncapi.rs`](../../crates/generate/ess-gen/src/asyncapi.rs)) | "retried up to N attempts in all … ends it at once; after the last attempt the work is lost" |
| semantic diff ([`diff.rs`](../../crates/verify/ess-diff/src/diff.rs)) | a changed count or final set is `binding/<id>/failure-changed`, worded `retry, 3 attempts, final \`rejected\`` |
| synthesized Rust/Go/Web targets ([`failure.rs`](../../crates/generate/ess-synth/src/failure.rs)) | **refused by name** (`missing-representation`, `bindings.<id>.on_failure.retry`): the generated retry holds the event for the next pump and counts no attempts, so it would retry forever where the specification says three times. The command-line target delivers no bindings and has nothing to refuse |

### What conformance proves

Two scenarios, both forced by injection on an `external:` branch as every §18 failure is, and both
observed through the invocation count — the one thing a bound changes:

| scenario | forced | required |
|---|---|---|
| `<binding>/binding/on-failure` | the first retried refusal (an `external:` branch that carries `error:` and is not in `final`; an external success branch is never forced), on the next `attempts` invocations | exactly `attempts` invocations |
| `<binding>/binding/final-failure` | the first `final` refusal that is `external:`, on the next invocation | exactly one invocation |

Neither requires the invoked command's success event, so `on-failure` is a scenario, not
`ESS-SYNTH-010`; a plain `drop` is refused only where its destination cannot be read unchanged
(see above). `final-failure` is not in `BindingAspect::ALL`:
only a bound that names `final` refusals makes that claim, so every other binding still accounts
for exactly four aspects. Where no retried refusal can be forced (every `external:` branch is
final) `on-failure` is refused with `BindingGap::RetriedUnforcible`; where no final refusal can be
forced, `final-failure` is refused with `BindingGap::FinalUnforcible`.

The count must see only the attempts the bound made, so the arrangement before the trigger may
not itself publish the binding's event — directly, or through a chain of event bindings whose
invoked commands, on any branch, lead to it (each command followed once, so a cycle ends). Where
the trigger synthesis picks needs such an
arrangement, the first other publisher of the event whose arrangement does not is used; where
every publisher needs one, both scenarios are refused with `BindingGap::ArrangementSetsOff`.

The suite carries two new step fields: `configure_external_outcome.times` (force on the next
`times` invocations) and `expect_invocation.count` (exactly that many matching invocations). Both,
and a `final-failure` id, take `ess-conformance/26` (`/27` for coverage,
[`bounded_retry.rs`](../../crates/verify/ess-conformance/src/bounded_retry.rs)); an older reader
would force one failure and read a count as "at least one", passing a sender that retries forever.
The Go and TypeScript runtimes execute /26–/27 (beyond10x/ess#188).
`ConformanceTarget::configure_external_outcome_repeatedly` defaults to `unsupported`, so a target
written before it reports that one scenario `unsupported` rather than passing it.

The count is observed for the step's whole eventual window: asked of the target until the run's
deadline passes, it passes only when the matching invocations reach exactly the count and are
still exactly the count at the deadline, and a count above it at any observation fails at once. A
sender that makes fewer attempts fails at the deadline; one that makes more fails as soon as the
extra attempt is visible within the window, including one a target makes visible late. The price
is that a passing count always waits out the window.

## Boundaries

No third word. No retry count where the count is a deployment decision — how often and for how
long is not something this specification takes; a count that is the component's own behaviour is
the bounded `retry` above. No timing, no `then:` after the bound. No `exactly_once`, ever.
