# Authored entity state arrangement

An authored scenario can establish upstream-owned entity rows in the implementation under test,
then read a populated view without inventing a command that creates those rows. Setup is a test
adapter capability. It does not assert that the model has an import operation or manufacture a
lifecycle transition, command invocation, event, or history.

## Source and persisted contract

`ess-scenario/2` adds an optional `setup` to each arrangement:

```yaml
type: ess-scenario/2
domain: calls.history
scenario: recent-calls
summary: Backend call records appear in descending-time order.
arrange:
  - instance: earlier
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: earlier}, duration_seconds: 12}
```

Setup identity and fields are literal typed values, not instance or observed-event expressions.
Every non-optional field must be supplied; unknown fields, invalid types, undeclared states,
duplicate instance names and duplicate `(qualified entity, identity)` pairs refuse compilation.
The same identity in two different entity types is valid. Missing Optional fields remain absent;
an explicit null remains present null, and only a declared Optional admits it. The entity's
identity declaration supplies the identity type. All entity invariants must evaluate True against
the complete supplied entity facts (including its declared state); False and Unknown both refuse.
Setup may establish any declared lifecycle state; it does not claim a path to that state.

Setup validates collection members, struct members, selected union payloads, and nested newtype or
struct invariants, rather than relying on the existing scalar fact binder's outer-shape checks.
Union payloads use the projection's adjacent tag/value representation (`content` when the tag is
itself `value`). Type expansion is limited to the existing 32-level bound, following only the
selected union branch. Integer map keys require canonical signed decimal text; Boolean keys are
`true` or `false`; textual primitive keys use their declared grammar. Nonempty Decimal-keyed maps
refuse because this setup contract does not define a decimal property-name codec. An invariant
whose needed collection or optional facts are unavailable refuses as Unknown.

Source version 1 retains its existing arrangement and compiled bytes and refuses setup fields.
Version 2 supports legacy captures as well as setup. A version 2 document without a timeline must
both establish state and assert something; setup alone does not establish a conformance result.

The compiled `establish_entity` step carries the instance, qualified entity, identity, literal
fields and state. It belongs to the coordinated, still unreleased ordinary suite 6 / coverage
suite 7 vocabulary, shared with bounded accessors; it requires explicit report 2. Older suite and
source readers refuse the new contract. Existing source and suite bytes do not change. Canonical
serialization remains ordered and deterministic; source dependencies include the entity and all
reachable identity/field types. Exact suite-byte provenance follows existing admission rules.

## Target authority and visibility

Rust exposes a typed setup request through a default-Unsupported `ConformanceTarget` method.
Generated Go exposes an optional setup capability interface with the same request semantics;
existing targets remain source compatible. Unsupported is a non-passing scenario outcome.

Typed request structure is not proof that arbitrary suite JSON obeys the model. Standalone readers
have no IR: their admission checks closed request shape, names, finite literal payloads, duplicate
instances and duplicate qualified identities. The adapter must validate identity, all fields,
lifecycle state and invariants against the actual declared entity contract before establishing
state. Invalid or inconclusive setup is an error; inability to provide this validation is
Unsupported. Source compilation provides early IR validation, not a certificate that readers trust
from a hand-written suite. No unverified schema or invariant claim supplied in a request grants
authority to bypass the adapter's validation.

Setup literals admit at most 120 nested map/sequence levels, leaving room for the suite envelope
inside the existing JSON reader's 128-level limit. Both writers and readers reject deeper setup
values and nonfinite numbers. A suite with setup requires an assertion after its final setup step;
an earlier assertion cannot stand in for checking newly established state.

The target establishes the actual backend entity within the context opened by `begin_scenario`.
Success means the row is durable for that scenario and visible to subsequent view reads at the
view's declared consistency. An adapter must complete or await its setup before acknowledging;
it cannot acknowledge only a queued write. No runner-side row cache is a substitute. The runner
binds only the instance's identity, only after success. Subsequent query parameters resolve that
identity normally. Setup produces no command consistency token and immediate reads use Current;
eventual view assertions retain their existing bounded observation semantics. A later real command
retains its usual consistency-token behavior.

Every scenario is bracketed by begin/end isolation. Setup must reject existing conflicting state,
must not update another scenario's rows, and end must retire the scenario's fixture state. The
compiler rejects duplicate setup identities and the runner rejects duplicate instance bindings and
duplicate qualified identities in hand-constructed suites. Adapter isolation is a capability obligation, tested with consecutive
scenarios using distinct fixture data and an empty-state witness; ESS cannot inspect a remote
database to prove a dishonest adapter's acknowledgement.

## Deciding evidence

The first witness declares backend-sourced CallRecord with no creator command and a descending-time
CallHistory view. Two rows establish identities, times and values. Assertions check both membership,
values, count and explicit order, with no timeline. Working Rust and generated Go adapters establish
their actual queried storage. Empty, wrong-value and wrong-order adapters fail; unsupported setup
does not pass. Consecutive scenario cases detect borrowed prior state. Source cases cover every
typed-field/state/invariant/duplicate refusal, Optional absence versus null, setup-only refusal,
legacy arrangement behavior, deterministic serialization and old-reader rejection.

Implementation and real consumer-adapter adoption remain proof obligations after design review.
This unit does not automatically populate synthesized scenarios or implement a storage backend.
