---
format: aep.planning-md/1
id: task:ess-gaps-measured-in-a-consumer-specification
kind: task
status: implemented
title: Eight measured gaps in ESS and its conformance tooling, from a consumer specification
summary: Prioritize declared binding paths and list selection, subject-state outcomes, exhaustive branch reachability, view fixtures, periodic triggers, clock provenance and compact suites from concrete adopter evidence.
tags:
- priority-high
relations:
- serves: vision:O2
- decomposes: initiative:ess-evolution
revision: 4
---
## What this is

Eight gaps in `ess/1` and its conformance tooling, each **measured** in `downstream-adopter/consumer-application`
(directory `consumer-specification`) during a three-unit specification wave on 2026-09-11, with six adversary
passes over the result. Filed as one task at the operator's request: the maintainer decides which become
stories, in what order, and whether some are one story or several.

Nothing here is a proposal the consumer needs. Each entry says what was measured, what it cost that
specification, and what shape would settle it — the design is the maintainer's.

Gap 1 already has story:binding-mapping-bounded-accessor and is actively implemented for PR #29. Gap 2 was offered for drafting but has no existing story in the inspected store; do not infer one from that offer. The other six were drafted on 2026-09-11 and **archived into this
task**; their bodies carry the full argument and can be lifted from
`aep plan artifact show <id>` if a story is wanted.

## The gaps

### 1. A binding mapping cannot read a path — `story:binding-mapping-bounded-accessor` (drafted)

`mapping:` reads one flat field; `event.data.<field>` is `unsupported_construct`. A system whose events are
wire envelopes (`{event, data}`) and whose commands take flat inputs can write **no binding at all**.

**Cost measured: 17 server-internal commands, 0 bindings.** The workaround — adding flat fields to the event
declaration — was tried across 11 events and 77 fields, and reverted: it makes the declaration stop
describing the wire, no gate can see it (the consumer's own payload checker walks the `data` subtree only and
passed a field the wire never sends), and the one row it enabled **stated something the server does not do**.

### 2. No list selection by a predicate (proposed by the specs-authority session, not yet drafted)

Selecting one element of a `List<T>` by a predicate over its fields. The measured case walks a slice of call
legs and picks two different elements — one by `Domain == "internal" || Source == "webrtc"`, the other by
`Domain == "external"` — and derives a third value from both.

**Cost: the `/agent/{id}/calls` binding stays unwritable even after gap 1 ships.** Worth designing
first rather than proposing a shape: a predicate is the expression language gap 1's refusal exists to keep out.

### 3. An outcome cannot be selected by the state the subject is already in

`when:` reads the input; `wrong_state:` needs states the transitions exclude; `external:` says something
outside the system decided it. A branch chosen by the subject's own held state has no spelling.

Measured: two commands report a campaign membership's status, and which outcome applies depends on the state
the membership already holds. `when:` cannot see it, `wrong_state:` is `UnreachableBranch` because the
transitions start from every state, and `external:` is false — it asks a conformance target to inject a fault
for the system's **most travelled path**.

### 4. Exhaustiveness and synthesis reachability disagree

`ESS-COMMAND-005` requires that outcomes cover every input, so over a closed enum that means an unconditional
branch unless every variant is guarded. `ESS-SYNTH-003` then refuses to synthesize a scenario for a branch no
candidate input reaches. **When every variant of a closed enum is guarded, the branch the first rule requires
is exactly the branch the second cannot reach.** No specification satisfies both.

Measured on two commands over a six-variant enum. Dropping the branch: `ESS-COMMAND-005` ×2. Keeping it:
`ESS-SYNTH-003` ×2. Declaring a seventh variant so it becomes reachable: the synthesizer is satisfied and the
specification is **wrong** — the producer sends exactly six, and the entity's lifecycle has no state for the
value. That was tried, caught by an adversary pass, and reverted. That specification now carries the two
refusals permanently and explains them where a reader meets them.

Three candidate resolutions, in the archived story: exhaustiveness knows a closed enum is fully guarded; a
branch may be declared unreachable with a reason (the shape `BindingGap::PolicySilent` already has for
`on_failure: drop`); or gap 3 removes the need for the branch.

### 5. An authored scenario cannot arrange a view row

`arrange:` names instances and their entity and carries **no fields**, so the only way a row reaches a view is
a command in the timeline that creates it. An entity **no command creates** therefore has no reachable view,
and every assertion over that view is vacuous — an implementation returning nothing passes.

Measured: five entities are created by no command, and between them they source nineteen views. **A
conformance suite for that system cannot check a single one of its backend-sourced reads.** The acceptance
rows that would have covered them were recorded as unexpressible rather than written vacuously. This is not
one repository's shape: any system whose reads are populated by an upstream it does not own has the same
hole, and those reads are precisely what a consumer wants checked.

### 6. A binding cannot be triggered by a period

`when:` admits only an event. A system that reconciles on a timer cannot say what causes the work, and
synthesis builds no scenario for it, because a scenario comes from a binding's `delivery:`/`on_failure:`.

Measured: a campaign membership is re-read every two seconds. The push-triggered path has a home once gap 1
ships; the timer-triggered path has none. The same shape recurs in any liveness sweep, retry drain or cache
refresh. `story:elapsed-time-claims` is the neighbour: a scenario for a periodic binding needs the duration
claims it added.

### 7. A timestamp does not name the clock it was read from

`Timestamp` is one universal line. In a distributed system it is a reading from a named clock in a named
representation, and where those differ the model is silent — the assumption lives in a code comment, if
anywhere.

Measured, all four live in the consumer and none typable: the wire carries a local-time string with a literal
`Z` stamped on the **producing JVM's default timezone**, and the consumer parses it as UTC — correct only
because the producer pins its timezone at boot, which the consumer cannot see and does not check; the
producer's timezone **before** that pin is unknown, and carries an `UNMAPPED:` for exactly that; a timestamp
from the consumer's own clock is compared with one off the wire, and nothing declares them comparable;
durations are derived across two producers' clocks.

Shape: a declared clock with a representation, a timestamp type bound to one, and a refusal when two cross
without a declared conversion — the same argument `conversions:` already makes for types, because a crossing
nobody explained is a silent widening. Distinct from `story:elapsed-time-claims`, which is implemented and
covers **how long** something took; this is what a **point in time** is.

### 8. A synthesized suite has no compact form

`--target ir` writes pretty-printed and there is no other option.

Measured: two committed suites, **13 MB**, **24,961 bytes per scenario**; of 28,534 steps, **26,527 are
`expect_no_event`** — the same negative obligation repeated per unexpected event. Compact JSON measured 40%
smaller. The size is carried by every consumer that commits the suite as a drift-checked artifact, which is
the pattern the format invites. Worth measuring whether the negative set belongs on the suite rather than on
each scenario before choosing between a writer flag and a format change.

## Not a gap — recorded so it is not filed as one

**The conformance runtime exists.** `ess verify conform synthesize --target go` was run against that
specification on 2026-09-11 and wrote a package that `go vet`s clean — the suite, the predicate evaluator and
a runner behind a nine-method `Target` interface, for 326 scenarios of which 43 are authored. What that
consumer lacks is its own adapter implementing those nine methods, which is its work and not this project's.

## Provenance

Every measurement above is from `downstream-adopter/consumer-application` wave 3, 2026-09-11, and is recorded in that
repository's planning store: `review-result:adversary-backend-bindings-pass-{1,2}`,
`review-result:adversary-conformance-suite-pass-{1,2}`, `review-result:adversary-authored-scenarios-pass-{1,2}`,
and the findings inventory `docs/plans/2026-09-11-wave-3-findings.md`. Gap 7 was requested by the operator;
the rest were found by the wave.


## Maintainer planning admission

The operator explicitly prioritized this task on 2026-09-11 and requested AEP decomposition and scheduling. There are eight measured gaps, with six additional archived arguments beyond bounded accessors and list selection. The archived primary stories are historical source, not resumable lifecycle entries; new implementation stories will cite them and preserve their source snapshots. The active accessor story is reused rather than duplicated. Crosswalk remains excluded, and the current deliverable remains remote ESS PR #29 against main.

Read-only scoping is checking the claims against current source before scheduling. The initial delivery order favors observable consumer blockers and a bounded lossless compact-writer improvement, with semantic clock/list/periodic additions designed before implementation. No additional runtime capability is claimed by this planning import.

Original primary task revision 1 and six archived argument files are retained under local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/source with manifest.json hashes. This managed-tree import uses the AEP CLI and generalizes the private organization identifier before any public journal entry. It preserves the staged primary and its existing history; no direct planning-file rewrite or unarchive is attempted.


## Assessed decomposition and priority

All eight gaps now have distinct planned owners. The existing accessor story is reused; seven new stories carry the missing list-selection unit and the six archived arguments. Each new story has acceptance, cited/inferred machine-readable scope, high-priority tag and a decomposes edge to this task. This task decomposes initiative:ess-evolution. Original archived stories and staged primary files remain untouched.

| Priority/stage | Gap | Owner | Current disposition |
| --- | --- | --- | --- |
| Current source PR | 1: bounded paths | story:binding-mapping-bounded-accessor | Active; source implementation in its managed unit for PR #29; downstream context/conversion acceptance still open. |
| Next correctness work | 4: closed-enum coverage | story:closed-enum-outcome-coverage | Proposed; bounded proof and real-variant synthesis witnesses. |
| Next populated-read work | 5: authored setup | story:authored-entity-state-arrangement | Proposed; typed real-target setup and non-vacuous positive view tests. |
| Next small writer delivery | 8: compact suites | story:conformance-compact-json-output | Proposed; lossless opt-in formatting, exact-byte evidence renewed. |
| High-priority design | 7: clock provenance | story:timestamp-clock-provenance-contract | Proposed; resolve authority/representation from the actual producer paths before model fields. |
| High-priority design | 2: list selection | story:binding-list-selection-contract | Proposed; actual first-match/fallback behavior, no inferred general expression language. |
| High-priority design | 6: periodic causes | story:periodic-binding-trigger-contract | Proposed; occurrence/input authority and observable timing, no scheduler service. |
| High-priority behavior/design | 3: subject-state outcomes | story:subject-state-outcome-guards | Proposed; actual held-state witness first, then shared finite-domain coverage. |

Design discovery can proceed without modifying another unit's source. Implementation starts only after rechecking the active accessor's ownership and updated scopes. The current PR does not silently absorb implementation of these seven additional gaps. This is the operator-requested prioritization and planning result, not an assertion that the gaps are fixed.

## Corrections established by source review

- Closed-enum coverage currently requires an unconditional branch even when equality guards cover every real variant. The initial repair needs no invented seventh variant or broad unreachable exemption. Unknown/candidate exhaustion must not become proof.
- Existing Contains and positive-count view assertions already fail empty reads. The missing capability is typed, isolated setup of upstream-owned rows in the actual target. One concrete no-creator CallRecord/CallHistory witness is identified; the aggregate nineteen-view count was not recomputed.
- Current consumer reducers directly assign reported state. The specification's held-state branch distinction therefore requires a behavior witness or model correction; removing a refusal alone would not prove it faithful.
- Compact JSON preserves model digests and meaning but changes the exact suite-byte digest. Reports/lineage must bind newly emitted bytes. Negative-obligation deduplication is a separate possible representation change, not a condition for the small writer option; the historical percentage needs a clear per-file denominator.
- List selection must account for case-insensitive first matches, switch precedence, nil entries, id-bearing fallback and separate derivations from both selected legs. The summary's two predicates alone do not describe the real reducer.
- Periodic input/timer authority and clock provenance remain explicit design questions. Duration observation already exists; it neither supplies invocation causation nor identifies a producer clock. No guessed timezone or fake event is admitted.

## Computed scheduling and evidence

The exact command aep plan artifact waves --kind story --status proposed --format json was run after writing scopes and dependencies. Its unfiltered output is retained at local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/proposed-waves.json.

Computed groups (scope-safe scheduling information, not an implementation approval):

- Wave 1: story:authored-entity-state-arrangement, story:closed-enum-outcome-coverage.
- Wave 2: story:conformance-compact-json-output, story:timestamp-clock-provenance-contract.
- Wave 3: story:periodic-binding-trigger-contract.
- Wave 4: story:binding-list-selection-contract.
- Wave 5: story:subject-state-outcome-guards.

The output records 41 exact-path collisions, no dependency cycles, and the existing unassessed story:binding-delivery-at-most-once outside this selected task. SHA256 6f3bd1d15d96955b7ebaa9fbc55cf8d78693baa64ed58c6aff2cadbdad5ef5b1. Active accessor ownership is excluded by the proposed-only query and remains a separate preflight constraint; these groups cannot be dispatched over its live edits.


The subject-state unit depends on closed-enum coverage for its input/state proof. Periodic work reuses implemented elapsed-time claims. List selection follows compact output as an explicit integration-order constraint on the shared scenario surface, not a semantic dependence on whitespace. Other collisions are scheduling constraints, not invented dependencies on finishing downstream adoption.

Scoping reports: local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/state-views.md and time-size.md. These are source inspections only; no tests/builds or new consumer acceptance results were produced for this planning task. Private exact list-reducer source citations remain in list-selection-private.md. Validation and publication evidence follow in the wave record.


## Acceptance

This planning task is complete when all eight measured gaps have a distinct scoped owner, observable acceptance and an explicit priority/dependency decision, the independent critique and every finding outcome are recorded, and the validated plan is published in the ESS pull request; implementation completion remains with the child stories.


## Independent planning review

The four installed roles aep-plan:plan-critic-acceptance, aep-plan:plan-critic-design, aep-plan:plan-critic-scope and aep-plan:plan-critic-parallel-safety reviewed all eight children and this parent. Round one recorded nine acceptance findings and zero findings in the other three lanes. The nine closing assertions were clarified without dropping the verification obligations; each finding has a fixed review_outcome. Round two approved all four perspectives with no remaining findings. Exact immutable reports are review-result:gaps-<perspective>-round-<1|2>-20260911, including explicit empty findings blocks.

The runtime has no plugin-native agent type or Sonnet model option. Two existing general agents followed the exact installed role charters, with no cross-agent findings shared. With the accessor implementor occupying the third agent slot, perspectives ran in two batches and each agent handled two lanes; this deviates from four simultaneously independent agents and the skill's model pin. The revised plan was held stable between the final-round readings. No tests or builds ran for the critique.

The original decomposition is published at ac369db9 in PR #29. These review records and clarified acceptance remain in the managed integration tree for the next source/evidence publication batch. The task remains active until that reviewed plan is published; child capability outcomes remain active/proposed regardless of the planning task's later completion.


## Reviewed plan delivered

The reviewed plan and all eight immutable panel reports are published in PR #29 at 592c6fce01dd0bf86fd5d4c95a27cf4a97a1757b. Fresh remote branch advertisement and PR head agree; main remains 6b666e58. Both author and committer are b10x-bot[bot]. Signed common checks passed and publication returned https://github.com/beyond10x/ess/runs/103106297970; the existing App authority admitted this feature-branch update without a rule change. AEP validation reports 270 artifacts and valid, exit 0, with the known unassessed unrelated scope and review-block advisories retained in gap-scoping/critic-final-validation.log. The CLI also labels explicit empty findings blocks as absent; no findings were fabricated to suppress it.

This satisfies the planning-only acceptance. Marking this task implemented does not mark any gap capability complete: the accessor stays active and the seven follow-up stories stay proposed. The lifecycle/publication receipt is retained for the next source commit. The accessor public-guide drafts remain uncommitted and unverified pending source integration; no main merge or release is claimed.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 673cfbfd603308099e9e68e78b4fe06def4a395bfc53f172139942e9e4f1b806, retained as local-evidence:runtime-gaps/publication-replay/snapshots/673cfbfd603308099e9e68e78b4fe06def4a395bfc53f172139942e9e4f1b806.md. Source creation recorded at 2026-09-11T00:19:09Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
