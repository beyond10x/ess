# ESS scope for exact retained-result retries

This is a read-only proposal, not an adopted ESS design or an executed compiler fix. The smallest coherent change is (1) an effect-free default refusal using the common existing selection subject, and (2) a command-local retained-result replay outcome. A general host lookup, arbitrary response path, or caller-supplied identity is unnecessary.

Owners: the existing EKR retry decisions and activation draft belong to the coordinator; the correction worker owns its retained probes and partial declaration patch; the ESS owner must adopt the syntax, allocate formats and implement/review the upstream capability. This scoping pass changed only this scratch report. It ran no compiler, build, installation, worktree, planning command, source mutation or runtime probe.

## Current measured boundary

Source inspected: ESS checkout `16aa8c7617214420d7d7f2108d0a896a5ed14eb0`. The correction report records checksum-verified release 0.28.0 at `68581d70bb47a048cd399e55c68f225b80303977`, with its four inspected command modules identical to this checkout. Its executable measurements are inherited inputs, not executions by this pass:

- `commit-default-error-probe.log`: ordinary default error after Validated move and Committed preservation refuses with COMMAND-003.
- `commit-state-probe.log`: adding legacy wrong_state refuses with COMMAND-004.
- `commit-partition-probe.log`: external refusals do not cover Proposed/Rejected/Stale; COMMAND-005.
- `seed-binding-probe.log`: revision_id is absent from document-only input; COMMAND-001.
- `preserves-positive.yaml` and its generated suite: existing input-bound preservation works; four generated obligations, no synthesis refusal, suite/11. It is not an EKR runtime witness.

The direct sources explain the limit:

| Source | Consequence |
| --- | --- |
| `crates/specify/ess-domain/src/command/subject_state.rs:88`, especially 112–119 | Every ordinary state-selected branch currently needs an input-bound effect subject. |
| `crates/specify/ess-domain/src/command.rs:1457` | An error outcome may not carry an effect subject. Keep this rule. |
| `command.rs:661` | Preserves reads identity exclusively from command input. |
| `crates/verify/ess-conformance/src/synthesize.rs:2194` and 2255 | State arrangement/observation reads the selected outcome's effect subject; merely loosening shape validation is insufficient. |
| `synthesize/subject_fact.rs:348` | Preservation requires an already bound identity and immediate views covering every declared subject field. |
| `crates/verify/ess-conformance/src/runner.rs:1198` | Existing typed response assertions compare a response to an event emitted by that same invocation. |
| `crates/generate/ess-synth/src/rust/items.rs:420`; Go uses the same predicate | A native outcome carries its typed response only when an event maps from that response. Silent replay currently has no native result field. |

## 1. Commit refusal: retain the natural syntax

The correction's default-error candidate is the desired source spelling:

```yaml
outcomes:
  - name: committed
    when_subject_state: Validated
    moves: ekr.kernel.GraphTransaction.commit
    instance: transaction_id
    emits: [ekr.kernel.RevisionCommitted]
    # Existing complete payload declaration remains.
  - name: already-committed
    when_subject_state: Committed
    preserves: ekr.kernel.GraphTransaction
    instance: transaction_id
  - name: wrong-state
    error: ekr.kernel.TransactionStateConflict
```

The last branch has selection authority but no effect. Derive that authority once from the command's ordinary successful state-selected branches: one entity and one input identity field. A subjectless ordinary default is admitted only when it is a named error, a unique common existing subject has been established, and the existing finite state/input proof says it is the genuine complement. Keep external causes outside this partition, disallow contradictory subjects, keep WrongState coexistence forbidden, and keep all error effects/sets/emits forbidden. No synthetic preserves or updates subject is attached to the refusal.

Expose a compiler-owned selection-subject query/value, separate from effect. Both the domain partition checker and conformance arrangement use it. Existing successful branches retain their current effects and bytes. For the refusal, arrange and observe the common subject directly, invoke the public command with its actual identity, assert the exact error, no events and unchanged subject. Do not route it through fault injection.

Generate/execute controls for **each** refused held state, not just the first state for which the default is reachable. Current `prepare_state_input` returns the first witness; a single Proposed witness would not catch an implementation accepting Rejected or Stale. Existing lifecycle synthesis deliberately excludes explicit subject-state partitions from legacy move-complement refusals (`synthesize.rs:3390`); preserve that exclusion so Committed does not regain an invented refusal.

This correction alone makes mixed move/preserve/refuse representable. It still does not prove that the preserved Commit response equals the original result; the second capability supplies that claim.

## 2. Proposed bounded construct: command-local replay

Recommended **new, unimplemented syntax**, to be adopted by the ESS owner:

```yaml
# On the same command that originally produced the result:
- name: already-committed
  when_subject_state: Committed
  replays: committed

# On document-only Seed; no caller identity is added:
- name: same-seed
  external: retained parsed Seed2 input and actual trusted bootstrap context and authority anchor match
  replays: seeded
```

`replays` names an earlier successful outcome of **this command**. It declares a silent successful return of that originating invocation's complete typed response and preservation of the subject identity captured from that originating invocation. It does not repeat the original effects.

For Commit, the original outcome already supplies GraphTransaction identity from the real transaction_id input. For Seed, the original seeded outcome already publishes Revision identity in Seeded.revision_id. The latter is captured before the retry; it is never inferred from the retry's response or from the latest head.

Bound the initial capability:

- One command-local originating success; no cross-command reference, recursive replay chain, self-reference or cycle.
- The origin declares a nonempty typed command response and an observable subject with an unambiguous identity source. Initially admit origin effects creates or moves; no general historical query or receipt-search DSL.
- Replay declares no creates/moves/updates/preserves/instance/sets/emits/error of its own. Its retained subject is derived from the referenced origin. This keeps one authority instead of two declarations that can disagree.
- A replay state guard is allowed only when the origin supplies an existing input identity, as Commit does. Seed uses its truthful implementation-owned eligibility explanation; no invented input predicate or revision id.
- Missing original result/identity, ambiguous origin, unavailable complete subject observation or unsupported response type produces a named validation/synthesis refusal, never a vacuous successful obligation.
- Raw command input equality is **not** a sufficient runtime admission rule. A SeedDocumentPath may point to changed bytes. EKR must still compare parsed full Seed2, actual BootstrapContext and trusted host AuthorityStateV1. Whitespace equivalence, changed path content and context are not redefined by this construct.

This is smaller than a general `instance: {view: ...}` or host-owned lookup: the earlier successful outcome already names the identity surface and supplies a real observed result. A singleton view alone would still need a second construct to prove retained response equality. A response-only identity selector is weaker: an implementation could return another revision and cause its own new identity to select the row it claims was preserved.

The correction owner checked this proposal against accepted EKR behavior and found no contradiction under the bounds above. That is design coordination, not approval or runtime evidence.

## 3. Conformance and native APIs are part of the capability

Add one explicit strategy for the replay outcome, rather than inheriting `InjectFault` from its external explanation. A generated witness:

1. Arranges and invokes the originating success through ordinary public commands.
2. Requires that actual successful outcome and captures the actual typed response. Captures the original subject identity from its input or emitted event using existing identity semantics.
3. Queries declared immediate views and snapshots that original subject's complete fields.
4. Invokes the public command again with the originating input under unchanged fixture context, **without ConfigureExternalOutcome**.
5. Requires the replay outcome, a complete typed response equal to the original snapshot, no error and an empty direct-event list; queries and compares the original subject again.

A bounded new suite pair such as `CaptureCommandResult` / `ExpectReplayResult` should name the command, originating outcome and snapshot key, and reuse the existing closed typed response admission. Those spellings are proposals, not existing vocabulary. Capture only an actual successful invocation; deep-copy observed values; verify command/outcome association before capture/comparison; reject missing, overwritten or wrong-command snapshots. Do not inject an expected result map into the target. Preserve exact Integer and existing Timestamp/value comparison rules and the response observer's resource/type refusals.

The new assertion must explicitly require no returned direct events, including an unexpected event name. Existing per-event negative assertions can remain, but do not substitute for the replay result's empty-event contract.

The observation snapshot is test evidence only. Runtime retry authority remains durable kernel/provider receipts after restart. An adapter remembering the previous response and answering on behalf of the real handler is invalid.

Native Rust and Go generators must carry the command's typed response on **both** the replay outcome and its referenced origin even if neither maps the response to an event. Extend the existing response-bearing admission specifically for this relation; keep outputs for legacy declarations unchanged. Do not make `response_payload_matches` claim that a silent response was checked against an event.

EKR still needs a complete immediate Revision observation. Its current CurrentRevision declaration exposes only revision_id, number and knowledge_root, while Revision also declares parent, ontology/evidence/agent roots, transaction_id and committed_at plus lifecycle state. A real read surface exposing the original retained revision's full fields is an EKR adoption prerequisite, not a new public CLI verb. The original captured identity selects Root0 after head advance.

The basic synthesized witness proves one immediate replay of an actual result. Authored real-handler tests must additionally cover restart, later head advance, whitespace-equivalent seed documents, changed file contents at the same path, changed trusted anchor/context, no resampled timestamp, no new occurrence/object/event and each Commit refusal state. Those remain required even with a compiler-green replay scenario. Ordinary model observations do not prove zero physical provider writes; EKR's durable acceptance measures that separately.

## 4. Concrete source/test scope

The following are the smallest logical work areas; the implementor must confirm exact internal factoring before editing, not expand into a generic selector facility.

| Area | Existing paths and required correction |
| --- | --- |
| Design/version admission | `docs/design/subject-state-outcome-guards.md`, `docs/design/observed-subject-history.md`, a bounded new replay design page; `crates/specify/ess-domain/src/system.rs`, `primitive_admission.rs`. |
| Authored/model semantics | `crates/specify/ess-domain/src/command.rs`, `command/subject_state.rs`: optional command-local replay reference, strict exclusivity/reference checks, common selection authority, per-state proof. |
| Resolved authority/provenance | `crates/specify/ess-compiler/src/ir.rs`, `resolve.rs`, `graph.rs`: resolved replay origin, retained identity source, response dependencies and explicit replay strategy. Use total handle lookups; no property bag. |
| Schema projection | `schemas/generated/ess.schema.json`, regenerated by `cargo xtask schema` because RawOutcome/RawSpecFile's reachable shape changes. |
| Synthesis | `crates/verify/ess-conformance/src/synthesize.rs`, `synthesize/subject_fact.rs`: effect-free refusal arrangement, replay setup/capture/comparison and complete subject snapshots; reuse existing arrangement. |
| Suite/runtime | `scenario.rs`, `runner.rs`, `response.rs`, `coverage_build.rs`, `admission.rs`, `authored.rs` under ess-conformance; native Go runtime and output module under `src/go/`; TypeScript/browser admission must implement or explicitly refuse the new envelope before callbacks. |
| Native API/projections | `crates/generate/ess-synth/src/rust/items.rs`, `src/go/items.rs`, `src/plan.rs`; `crates/generate/ess-gen/src/docs.rs`, `openapi.rs`, `asyncapi.rs` where subject semantics are rendered. Preserve or explicitly expose replay relation, never silently flatten it to an original mutation. |
| Semantic diff | `crates/verify/ess-diff/src/diff.rs`, `change.rs`, `delta.rs`, `raw.rs` and impact handling: changing the replay origin changes meaning even when input, response and original entity types are identical. |
| Focused tests | Extend domain `tests/subject_state.rs`, `subject_state_adversary.rs`, `subject_state_open_default_adversary.rs`; new retained-replay domain/compiler fixtures; conformance `tests/subject_state.rs`, `subject_history.rs`, `response_admission.rs` plus a dedicated retained-replay Rust/Go fixture; ess-synth `tests/response_payload.rs` and native compile/run fixtures; diff and legacy byte fixtures. |

Do not implement a permissive parser-only change. It would leave effect-free refusal synthesis without a subject, silent native outcomes without a response, and replay observations without any earlier response authority.

## 5. Deciding tests and mutations

- Five-state Commit model: Validated moves, Committed replays, Proposed/Rejected/Stale each return the named error without changes. Detect wrong-state broad success, wrong error, mutation on refusal and a rejected-state hole missed by a first-default-only witness.
- Document-only create/replay model: original emitted id is captured, replay has no input identity, and actual response timestamp/id remain exactly original.
- Fail a target that returns current head or a newly generated response while preserving the original entity; subject snapshots alone must not let this pass.
- Fail a target that returns an equal response plus an error/event, or mutates a preserved field unknown to synthesis.
- Fail wrong-command/missing/aliased response snapshots and mutable Go/JavaScript response maps.
- Reject replay cycles, cross-command references, error/replay origins, origin without response/subject, duplicate identity authority, and unavailable complete observation.
- Assert no ConfigureExternalOutcome occurs in the synthesized replay sequence.
- New declarations refused by old readers; unchanged old source, IR and generated native/schema/suite bytes remain stable. New result steps mislabeled with an old suite version refuse before target callbacks.

## 6. Format decision

Local source supports ess/1–6 and suite/1–11. Reserve **ess/7** for the new replay construct and the newly admitted state-default refusal semantics, subject to checking allocations when the upstream unit opens. Keep old-source acceptance and canonical bytes stable.

The new result snapshot/assertion vocabulary needs a fresh suite format: **ess-conformance/12**, and **/13** with coverage, if still unallocated. Do not label new assertions suite/11 or change old ExpectOutcome/ExpectResponsePayload meanings. Reports need no new version if their existing count/check representation suffices; decide from the actual serialized change.

No ess-ir/2 is warranted: the repository explicitly prohibits that plan. Add concrete optional resolved semantics omitted for unchanged legacy inputs, bind them into canonical provenance, and make downstream readers/generators account for them.

Semantic diff currently supports through ess-diff/5. A new typed replay delta would require **ess-diff/6** if still free. Do not silently ignore origin changes, disguise them as an unchanged preserves effect, or invent a format allocation without checking the upstream tree at implementation time.

## Remaining decision

The coordinator/ESS owner must choose and adopt the command-local replay contract before dispatch. This report recommends it over a general retained-view selector because it supplies both independent pre-retry identity and exact prior-result observation using the existing originating outcome. The recorded compiler refusals are real; the proposed repair is not yet implemented or validated.
