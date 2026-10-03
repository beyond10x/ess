# Binding completion at an aggregate query boundary

Status: coordinator proposal for the open #361/#362 dependency; not implemented or independently
verified. This is separate from #391's address rendering and from invocation-count observation.
The blocker stays open until the actual adapter controls below execute.

## What the observation must establish

A query over rows that relevant bindings can change needs a complete causal history and a read
aligned to its completed effects. An empty queue, a timeout, a successful local publisher flush,
a count of observed invocations or a plausible aggregate row supplies neither fact. The observer
does not receive expected rows, preferred command outcomes or a synthesized model execution as
authority for what the implementation did.

For one isolated scenario correlation, the runner identifies actual source operations by their
finalized suite step indices. The target records each operation, its directly emitted event
occurrences, every relevant binding obligation caused by those events, actual command-port
attempts/results, and descendants. Completion means every such obligation is terminal and every
included write is visible to the returned query snapshot. Unbounded retries, unresolved effects,
unobserved source-owned values needed for the aggregate, or failed projection synchronization
cannot be described as a complete cut.

The aggregate producer derives relevance transitively from the admitted source model: the query's
entity/fields/filter/group/measure dependencies, commands that may change them (including related
and set effects), and every event/binding path that can reach those commands. Include paths whose
condition or dynamic address cannot be proved irrelevant; do not assume a current false predicate
remains false under an unknown value. A proof of irrelevance is recorded as typed dependency
closure, so unrelated bindings need no completion certificate. A cyclic graph does not itself
refuse the whole model; a relevant cycle must actually terminate before a complete cut is returned.

## Closed optional target capability

Add a separate optional `CausalBindingTarget` capability, not fields silently appended to existing
command results or invocation observations. Rust target methods default to Unsupported without
performing the requested action; Go uses a separate optional interface and TypeScript uses an
explicitly checked optional capability. Existing adapters continue their original operations.
Required aggregate fixtures implement this capability; an adapter lacking it cannot pass a
binding-affected exact aggregate scenario. No capability is demanded when relevance is empty.

The capability has three operations with closed typed requests/results:

1. **Begin observation.** Bind correlation, admitted aggregate-contract digest, finalized program
   digest and the ordered relevant binding identities. It returns an opaque observation-session
   handle. The target verifies that it can observe the declared execution/storage boundary before
   any tested operation runs. A session has one admitted contract/program; handles are not reusable
   across scenarios, model revisions or process restarts.
2. **Execute observed source operation.** Carry session handle, finalized source step index and the
   ordinary typed command request (or the existing explicit fixture/ingress request). Execute the
   real operation once and return its actual ordinary result plus an opaque root receipt. The
   target records the operation/result and direct event occurrences at their real boundaries.
   Duplicate invocation of an operation key is refused; this API is observation, not a new retry
   mechanism. The ordinary command API is not called a second time to obtain a receipt.
3. **Query completed cut.** Carry session, the roots selected by this finalized query step, actual
   view/params/caller/consistency requirements and a deadline. Return one closed response: Complete
   with the cumulative causal inventory and rows from the aligned immutable query snapshot, or
   Pending/Unsupported/Failure with a typed reason and no complete rows. Pending may be retried
   within the existing runner deadline; timeout is a non-passing observation. A deadline is never
   evidence of completion. The response binds the query step, view/params, source roots, contract
   and program digests, so it cannot be attached to a different query or a stale program.

The normal query target callback does not subsequently fetch different live rows for this check.
The rows and causal inventory share one snapshot receipt. A read-your-writes requirement includes
the original caller's token as well as all completed descendant writes. An eventual projection
must catch up to those writes before Complete. A target without a snapshot/transaction authority
for the required cut returns a named missing capability.

Session/root/event/attempt identifiers are opaque allocation identities, never hashes, excerpts
or encodings of payloads, command inputs, one-time fields or credentials. Requests contain actual
inputs only where the existing operation already needs them; no oracle expectation is sent.

## Inventory and actual execution authority

Each event occurrence has a session-unique occurrence id, declared event identity, actual admitted
payload, parent source-operation or command-attempt id, and publication position. Identical event
payloads still have different occurrence ids. The compiler's binding cause determines the set of
relevant obligations registered for that occurrence; false conditional bindings have an explicit
Skipped result tied to the actual event, not a missing record. The runner re-evaluates the admitted
condition against that payload. Unknown is unresolved and prevents Complete.

Each binding obligation names its causing occurrence and binding, and records its ordered
attempts. Each attempt contains a distinct id, ordinal, actual complete mapped input, actual
caller/provider authority where applicable, ordinary command result, direct event occurrence ids,
and committed-effect position or explicit no-write result. It is recorded before the command port
is called; all completed results, including refused and untyped failed attempts, are retained.
Retries have new attempt ids and share one obligation, preserving the #269 total-attempt rules.

Terminal states are closed: Accepted, Dropped, Escalated or Skipped. Dropped records the actual
refusal/failure and applicable drop/final/exhaustion rule; Escalated names its actual emitted
escalation occurrence and complete builder authority; Skipped is only a false condition with zero
attempts. Escalation occurrences participate in descendant binding discovery. A pre-input mapping
obligation, incomplete attempt, uncertain remote effect or unresolved escalation-builder error is
a nonterminal/error observation, not silent completion. A command-port transport failure whose
remote commit is unknown cannot claim no-write merely because local retry stopped.

All actual committed command effects have one monotonically ordered commit position in the
observation session's supported consistency domain. The inventory includes that order, not just
tree order or wall-clock timestamps. The required generated/native adapters serialize observed
command effects in their real store transaction/lock boundary and observe that position. A
distributed adapter that cannot establish such an order and aligned query snapshot is explicitly
unsupported for this capability; it must not invent an order from completion notifications.
The source observer applies actual inputs/results in committed order through the separate shared
execution adapter described in component-design:aggregate-observation-integration-boundary.
Failed/no-write attempts constrain outcomes without publishing fictitious effects. Source-invalid
actual outcomes, mappings, traces or effects fail conformance; they are not used to rewrite the
source model to fit the result.

## Race-free completion bookkeeping

The real dispatcher registers every relevant child obligation before making its parent terminal.
Publication and child registration share the dispatcher/store synchronization boundary, including
events generated by a refused command or escalation. An in-flight attempt remains pending until
its result and all direct events are recorded. Scheduled retries remain pending even while no
worker is running. A retry decision and its next pending attempt cannot leave an observable gap
where the obligation looks complete. Cancellation or worker failure does not erase obligations.

Query-completion observation first seals the requested root set against additions of further source
operations for that cut. It then checks terminal transitive closure and captures inventory plus
view snapshot under the same transaction/lock or an equivalent retained snapshot-token protocol.
The target may continue unrelated work; it cannot report a live mutable view as the sealed result.
Later source operations get later roots/cuts. Complete inventories are immutable and repeatable for
the same cut, and a later cut includes earlier committed effects even if its roots are a superset.
Scope/correlation isolation prevents untracked writers from changing the queried scenario rows;
if an adapter cannot guarantee that isolation or account for those writes, Complete is unavailable.

This is maintained execution metadata at the real dispatcher, not a runner instruction that
drains a queue or synthesizes missing results. The completion inventory cannot be filled from the
aggregate program's predicted values. Healthy/fault adapter tests must exercise the race seams.

## Admission, versions and disclosure

Use ordinary suite/38 and inventory suite/39 for the aggregate observation program and completed-cut
association. Suite/36–37 are reserved for conditional binding observation, and held /34–35 retain
their meaning. New readers reject this metadata in earlier versions and earlier readers reject
/38–39. Select the minimum format needed by the emitted vocabulary; unchanged old suites are not
rewritten. The native closed Contract reader still reconstructs checked source and reprojects IR,
and foreign readers validate the bounded semantic profile; this capability does not weaken either.

Where an adapter serializes these new requests/results, the closed envelope is
`ess-binding-observation/1`, with a tagged operation/result, exact identifiers/digests, no unknown
fields and lossless typed numbers. It is a new explicit protocol, not an extension of a released
command/event JSON response. Original-byte tests reject unknown tags/fields, duplicate identities,
missing referenced events/attempts, cycles in parent links, inconsistent ordinals, absent roots,
wrong query/session/model digests, nonterminal Complete and malformed values before using rows.
The existing higher-level report retains only its approved diagnostics/accounting fields; any new
persisted report field needs a separate version decision before emission.

Causal inputs, responses, events and generated values are private observation state. Apply the
existing one-time disclosure authority to every path, including malformed DTO errors, timeout,
unsupported, overflow and budget refusal. Diagnostics name semantic paths and stable codes without
raw values, hashes or excerpts of private data. Retain only actual source-admitted observations
required by the private execution adapter; wipe session state at scenario teardown. A result that
needs an unavailable private value remains undetermined rather than printing or guessing it.

Resource limits extend the original aggregate proposal's bounds: at most4MiB contract/program
metadata,4096 live rows/groups/captures of each kind,32MiB total private state including causal
payloads, depth32 and16million evaluation work units. Add at most4096 roots and16384 total causal
event/obligation/attempt records per session. These are proposed finite engineering limits, not
measured capacity claims. Reaching any limit is a named non-passing obligation; it cannot truncate
the inventory and still return Complete. Implement deterministic boundary/one-over tests and
measure the complete required fixtures against them before admitting the capability, preserving
the reviewed exact-number and complete-domain requirements.

## Required independent adapter proof

Use a real two-component chain: source command creates a row and event; the first binding updates
it and emits a second event; a second binding creates a related row contributing to a grouped
aggregate. Include a bounded retry with an initial refusal and final success, a final/drop path,
an escalation feeding a descendant, and one conditionally skipped binding. Actual dispatcher/store
execution supplies receipts; the query independently reads stored rows.

Run healthy controls in native, actual generated Rust/Go services, generated Go/TypeScript
conformance runtimes, WASM and the full browser product. These controls must include unrelated
binding activity that does not block the query. Named faults each produce failed/non-passing
conformance through the real observation path:

- `omitted_descendant`: drop a relevant child registration or inventory entry.
- `retry_gap_complete`: expose Complete between a refused attempt and its scheduled retry.
- `early_projection_cut`: return rows before the last included effect reaches the projection.
- `wrong_root_or_query`: splice a valid receipt into a different root/query/model context.
- `duplicate_or_wrong_order_effect`: repeat or reorder actual command effects.
- `unresolved_remote_effect`: return Complete despite an attempt whose committed effect is unknown.
- `private_value_in_failure`: force refusal at each parser/executor/timeout path and verify redaction.
- `irrelevant_binding_positive`: prove an unrelated pending binding does not prevent Complete.

Also verify actual completion after retry, timeout for a deliberately pending relevant obligation,
identical repeated reads of one cut, later-cut inclusion, identical-payload distinct occurrences,
and old adapter Unsupported without invoking an operation twice. Independent review must locate
each observed datum's actual producing call site. Interface agreement or fabricated trace fixtures
alone do not clear dependency-blocker:aggregate-binding-cut-authority.
