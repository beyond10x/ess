# The toolchain, in its own model

ESS retrofitted from this repository: what `ess` accepts, what it establishes, what it writes, what
it refuses, and who may ask. Five bounded contexts — one per command area the CLI is built out of,
plus the release protocol the repository runs on itself.

This is a **retrofit drafted from source**, not a design that was implemented. Every declaration
below was read from code, a workflow file or the published reference; the places where the reading
stopped are marked `UNMAPPED:` in the files and listed again here. Nothing was invented to make the
document validate.

Pinned at **`ess/3`**, and read with ESS **0.23.0**. Every line and file cited below was checked
against `origin/main`, not against a local checkout.

## What it says, and where each part was read from

| Domain | Read from |
|---|---|
| `toolchain.specify` | `crates/specify/ess-compiler/src/resolve.rs:186` (diagnostic families), `:196` (classes), `crates/specify/ess-compiler/src/diagnostic.rs` (severity, structured body), `crates/edge/ess-cli/src/input_discovery.rs` (source selection) |
| `toolchain.generate` | `crates/generate/ess-gen/src/provenance.rs:23` (provenance, field for field), `crates/generate/ess-synth/src/lib.rs:88` (four targets), `crates/generate/ess-synth/src/failure.rs:12` (ten target-failure codes), `crates/edge/ess-cli/src/main.rs` (`enum Projection`) |
| `toolchain.verify` | `crates/verify/ess-conformance/src/report.rs:46` (scenario status), `:105` (run verdict), `:146` (seventeen check codes), `:506` (scenario result), `crates/edge/ess-cli/src/main.rs` (`enum ReferenceTarget`) |
| `toolchain.infra` | `crates/infra/infra-domain/src/coverage.rs:12` (one coverage profile), `crates/infra/infra-analyze/src/code.rs:18` (finding severity), `crates/edge/ess-cli/src/observed_bindings.rs:218` (satisfied/violated/unknown and their exit codes) |
| `toolchain.release` | `.github/workflows/release.yml` (tag trigger, concurrency, jobs), `.github/workflows/release-record.yml` (the `workflow_run` reaction, the daily run, and why 0.5.0 made it exist), `AGENTS.md` § Release completion |

The entity in `toolchain.specify` is **knowledge, not a file**. `ess specify validate` stores
nothing; what moves between `Selected`, `Resolved` and `Compiled` is what the toolchain has
established about one source selection. There is no `Refused` state, because a refusal changes
nothing — and the model would refuse one anyway: an outcome that reports an error cannot also move
an entity.

## What it does not model, deliberately

- **The credential edges.** `execute`, `publish`, `fetch` and `reconcile` invoke Docker, ORAS and
  Helm. `toolchain.generate` stops at the artifact, which is where the area's own boundary is.
- **Output ownership decisions, and `ess generate output recover`.** `models/output-ownership/` owns
  the anchor, the transaction and the `Committed`/`Restored` decisions. Here that protocol appears
  as one refusal, `UnownedDestination`, rather than as a second copy of its states. Recovery cannot
  be modelled in this domain even in outline: what a recovery answers is an *interruption*, and an
  interruption is caused by a process dying rather than by a command, so there is no outcome in this
  system that could take the transition into it. The transaction is the subject that can carry that
  decision, and it is declared where it belongs.
- **Execution recovery.** `models/execution-recovery/` owns the authority, the lock, the journal
  and the refusal codes for a Helm reconciliation.
- **`topology.yaml`.** ESS is a binary somebody runs, not a workload with replicas. A topology
  document here would state a runtime requirement that does not exist.
- **`conversions:`.** A conversion admits a crossing between two contexts, and no binding here
  carries one type into another. `toolchain.specify.ModelDigest` and
  `toolchain.generate.ModelDigest` are the same bytes in two contexts and never meet in a mapping.

## UNMAPPED

Each is a place the reading stopped, not a simplification.

| Where | What is missing |
|---|---|
| `components.yaml`, `record-daily` | The period maps and the phase does not. The workflow runs at `cron "17 6 * * *"`; `every: PT86400S` is a day and `anchor: host_activation` anchors the grid to when the host came up. A wall-clock anchor has no word. |
| `toolchain.specify.Diagnostic.subject` | The real body is a list of `Detail` variants with a different field set each (`crates/specify/ess-compiler/src/diagnostic.rs`). A faithful model is a tagged union this retrofit did not read closely enough to type. `subject: String` stands in for it and says less than the code does. |
| `toolchain.verify.Run.started_at` | `ess/3` adds a clock-reading contract and it cannot be attached here. See below. |
| `toolchain.release.Release.blocked_by` | Real and filled by nothing in this model: the concluding step reads it from the jobs it depends on, and no command input here carries it. `ess/4` adds a payload filled from a command's response, which is the construct that would close it; this document is pinned at `ess/3`. |

## What `ess/3` changed here

Two of the three constructs this pin exists for are used, and the third was tried and does not fit.

**Used: a periodic cause.** `.github/workflows/release-record.yml` runs on a `workflow_run` trigger
*and* daily. Under `ess/1` the daily half was an `UNMAPPED:` marker — the half that catches a tag
pushed while no run was watching, which is the reason the workflow exists. It is now the
`record-daily` binding, and it synthesizes four scenarios (`flow`, `mapping`, `delivery`,
`on-failure`). Its event-triggered sibling declares the same `at_most_once` and `drop` and gets two
refusals for them. The asymmetry is not the delivery words: all four periodic scenarios read
`Periodic PT86400S through required host: <fixture>` — `Ready`, `InitiallyInactive`, `SlowFirstRead`,
`FirstReadFails` — so what they exercise is the host read contract, under names the binding
vocabulary reuses. A periodic cause has a host to put in those states; a cancelled event delivery
has nothing to put anywhere.

**Tried and reverted: `when_subject_state`.** `RunReleaseChecks` looked like the case for it — the
release checks run for a freshly pushed tag and for no other. Two refusals say it is not:

- `ESS-COMMAND-004` — a subject-state guard cannot sit beside a `wrong_state` branch.
- `ESS-COMMAND-003` — every input-selected branch of a subject-state command must name a `moves` or
  `updates` subject.

Both are right, and together they say what the construct is for: a command legal from every state,
each branch acting on the subject. Starting a release run for a tag that already has a release
behind it changes nothing, so its refusing branch names no subject, and `wrong_state:` remains the
word for it. **No command in this system is selected by the state its subject already holds.**

**Not used: the clock-reading contract.** `reading:` requires a directly String- or Integer-backed
newtype matching its encoding, and the closed set is offset date-time text, local millisecond text
with a literal `Z`, and exact integer Unix *seconds*
(`crates/specify/ess-domain/src/reading.rs:15`). A conformance report writes its instants as
`Timestamp`, which is `#[serde(transparent)]` over `u64` **milliseconds** since the epoch
(`crates/specify/ess-primitives/src/time.rs:41`). Milliseconds-as-integer is not in that set, so the
contract cannot be attached to the one clock reading this system has without saying something false
about its precision. The other candidate does not exist at all: infrastructure observation drops
`creationTimestamp`, condition transition times and `startTime` on purpose, so that a cluster
redeployed identically has the same IR (`crates/infra/infra-domain/src/lib.rs:42`).

## What it measures against the tool that reads it

With ESS 0.23.0:

```console
$ ess specify validate --path models/toolchain
toolchain v1 — 7 file(s), valid

$ ess specify compile --path models/toolchain --out <file>
toolchain v1 — 7 file(s), 98 declaration(s), compiled to <file>

$ ess verify conform author --path models/toolchain --scenarios models/toolchain-scenarios
1 authored scenario(s) from 1 file(s), 0 refusal(s), nothing written

$ ess verify conform synthesize --path models/toolchain \
    --scenarios models/toolchain-scenarios --out <file>
124 scenario(s) (1 authored), 5 refusal(s), written to <file>
```

The suite selects `ess-conformance/6` on its own: the authored setup below is an admitted feature,
and an admitted feature selects the version that carries it. `ess generate --kind schema` writes 120
artifacts and `ess generate project openapi` writes 5 documents from the same model.

The five refusals are the point of keeping them. Two are the synthesizer agreeing with what the
model says:

- `record-on-completion/binding/delivery` — `at_most_once` has no redelivery to observe. The CI
  reaction really can be cancelled, and the word says so.
- `record-on-completion/binding/on-failure` — `drop` is unobservable by design.

Two are the cost of keeping a true branch:

- `check-on-tag/binding/flow` and `.../delivery` (`ESS-SYNTH-010`) — `RunReleaseChecks` declares a
  `wrong_state` branch, and the synthesizer counts a branch the event's own field decides as
  input-decided, so it builds no flow scenario for the binding. A `workflow_dispatch` backfill on an
  already-published tag reaches that branch, so it stays.

One is a gap `ess/3` narrowed and did not close:

- `toolchain.specify.Declarations` (`ESS-SYNTH-013`) — no *outcome* creates a `Declaration`, because
  a declaration comes into existence by being written and the toolchain reads it.
  `models/toolchain-scenarios/a-declaration-is-readable-by-name.yaml` uses `ess-scenario/2` `setup:`
  to establish the row, so the view is no longer vacuous and an adapter without the setup capability
  produces a non-passing result rather than a quiet pass. The synthesis obligation is still refused,
  because it is computed from declared outcomes and an authored setup is not one.

## Re-running it

```console
ess specify validate --path models/toolchain
ess verify conform author --path models/toolchain --scenarios models/toolchain-scenarios
ess verify conform synthesize --path models/toolchain \
  --scenarios models/toolchain-scenarios --out <file>
```

`task example-check` runs the first two, beside the two sibling models. That catches a document
this build cannot read and a scenario naming something the model does not declare.

`ess validate` exits 0 on a stale enum, so neither of those lines can see a copied variant list that
has fallen behind — four of the five in this model were stale when it was first written, all for one
reason: they were read from a checkout four commits behind and nothing re-read them.
`crates/edge/ess-xtask/tests/model_enums.rs` closes that: it re-derives `SUPPORTED_FORMATS`,
`codes::family`, `codes::class`, `CheckCode` and `TargetFailureCode` from the source with `syn` and
compares each against the `variants:` here, naming the difference in both directions.

What is still unchecked is the arithmetic: the 98 declarations, 124 scenarios, 5 refusals, 120 schema
artifacts and 5 OpenAPI documents above are measured by hand and pinned by nothing. **It needs an `ess` that implements `ess/3`**: specification
formats `[1, 2, 3, 4]` arrived in 0.23.0, where 0.22.x carries `[1, 2]`. A 0.22.2 build refuses this
document at `components.yaml` with `unknown field \`periodic\`, expected \`event\`` — the binding
reader reaches the unknown word before the header check does, so the refusal a bare `ess/3` header
gets there (`ESS-SPEC-009`) is not the one this document gets.
