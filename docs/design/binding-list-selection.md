# Bounded ordered selection for binding inputs

This contract defines event-binding selection over declared ordered records. Generic
consumer examples illustrate host preparation obligations without extending the wire
payload or authenticated context. Native prepared-input helpers accept each explicit
conversion result once and execute generated bounded selection; implementing that
conversion and delivering its result remains a host obligation.

## Admission decisions

Implement selection over an explicitly typed, ordered list of records. Reuse the
existing declared type environment and finite Boolean predicates. Add two selector
operations: `first` and `first_present`. A first selector may exclude occurrences
selected by earlier selectors over the same list. Exclusion compares list identity
and index, never record equality, business identifiers or host pointer addresses.

The measured reducer does not receive an event field containing this list. Its
host builds candidates from the root payload, followed by its present child, then
its present parent. The root and related payloads have different declared nominal
types. A list selector cannot manufacture a wire field or silently pretend those
types are interchangeable. Introduce a typed **local selection input**, with an
explicit existing exact-source/target conversion obligation when necessary. The
local input is not a new field on the event or an authenticated-context record.

Keep wire decoding, list preparation, authenticated delivery, invocation suppression
and multi-leg state transformations explicitly host-owned where they are not
mechanical declared copies. Implement native and conformance selection now for
directly declared list inputs; a host conversion must not turn every selector into
an unwitnessed capability.

## Concrete syntax

The following is new syntax within the coordinator's unreleased source capability.
`example.telephony.CallBody` is an illustrative declared record type, with the same
relevant member names as the measured records: `id: String`, `domain: Optional<Domain>`
and `source: Optional<Source>`. The two actual domain variants are `Internal` and
`External`; no third domain value is introduced. Source literals below must belong
to that record's declared source enum.

```yaml
selection_inputs:
  - name: legs
    from: event.data
    as: List<Optional<example.telephony.CallBody>>
selections:
  - name: first_agent
    first:
      in: legs
      where:
        any:
          - all: [defined(item.domain), item.domain == Internal]
          - all: [defined(item.source), item.source == Webrtc]
  - name: first_external
    first:
      in: legs
      excluding: [first_agent]
      where:
        all: [defined(item.domain), item.domain == External]
  - name: first_identified
    first:
      in: legs
      where: 'item.id != ""'
  - name: agent
    first_present: [first_agent, first_identified]
  - name: external
    first_present: [first_external, agent]
```

These ordered sequences have unique local names. References must point to an earlier
entry; unknown, repeated, self or forward references are errors, not inferred ordering.
All inputs of `first_present` must refer to the same typed selection input. Each
selector has type `Optional<Occurrence<T>>`, where the occurrence internally retains
the local input handle, its original zero-based index and a typed reference to T.
This internal occurrence type is not a producer field or a new user-declared entity.

`from` resolves an existing EventField/EventAccessor and its exact declared type.
`as` names the list type the selector consumes. Equal types require no conversion;
a differing pair must be present in the existing conversion registry, with its
authored reason and executable host implementation. Compile this as the existing
conversion capability at a new typed local-input use site, not a second conversion
registry or a generic transformation language. Evaluate each host input conversion
once per event occurrence, before selection. The measured preparation conversion
copies root fields into the declared body type and appends present child and parent
in that exact order. Its field fidelity and ordering require independent vectors.

For direct-list conformance fixtures, `event.data` itself has the declared list type;
there is no conversion and no fabricated consumer wire field. The example's `legs`
name is a binding-local value, never `event.legs`.

Mapping values gain an object alternative, for example
`chosen: {selection: agent, path: []}` or
`chosen_id: {selection: agent, path: [id]}`. Existing mapping strings retain their
meaning, including literals beginning with `selection.`. An empty path selects the
whole record; a nonempty path reuses bounded accessor field/Optional rules against
the selected record. Absence stays explicit. Required target inputs are refused
unless the existing type/conversion contract can handle the complete optional
source. There is no implicit unwrap, default ID, skipped command or fabricated record.

## Finite predicate and type authority

Initial predicates are the existing `Always`, `Never`, `Defined`, `Compare(Eq|Ne)`
against a typed literal, and `All`, `Any`, `Not`. Reads are rooted only at `item` and
resolve declared String or enum leaves through at most three field segments.
Comparison literals must satisfy the existing enum/newtype/literal checks. No
fact-to-fact comparison, arbitrary function, regular expression, nested list scan,
quantifier, map indexing, sorting, reduction or remote/context lookup enters this
initial selector fragment. Future additions require their own bounded contract.

Boolean composition and Missing/Unknown use the existing predicate evaluator.
Optional field guards explicitly use Defined; absent/null terminal Optional fields
must produce the same Defined/equality truth table in native and conformance paths.
An undecidable predicate is a typed selection refusal, never an unmentioned false.
Malformed required data is refused by input admission, not treated as Optional
absence. Optional list elements are skipped; null elements in `List<T>` are invalid.
The initial selector input must be a required List<T> or List<Optional<T>> (including
transparent declared aliases); Optional whole-list input needs an explicit typed
host conversion and is not silently replaced with an empty list.

Case insensitivity is a precise host boundary here. The existing consumer reducer
compares raw strings with Unicode simple case folding. Its ESS declarations instead
use closed enum types, with wire spellings differing in case from canonical variant
names. Native selectors compare those admitted enum values. Do not introduce an
ASCII-only substitute for the raw decoder or quietly add arbitrary strings to the
enum. The host wire codec must map the existing wire aliases/case variants to those
declared variants, preserving the reducer's behavior, and must be tested separately
with mixed-case vectors. Missing/null optional domain/source maps to declared absence.
Unknown raw enum strings remain outside the declared producer contract and require
an explicit refusal/host obligation; they are not a new enum variant. This design
does not claim equivalence to the permissive raw reducer on undeclared producer data.

## Ordering, no match and multiple matches

`first` scans the admitted list in original order and returns the lowest eligible
matching index. Empty/all-absent lists or no matching record return None. Multiple
matching records are valid and resolved by order; they are not an error and are not
deduplicated. `first_present` returns the first present prior selection. In the
example, computing the ID candidate even when unused is pure selection; it does
not perform an extra host conversion, command invocation or state mutation.

The measured switch is equivalent to first primary match, followed by first external
match excluding that primary occurrence, followed by ID fallback for a missing
primary. Excluding the *final* agent is wrong: when there was no primary match, ID
fallback may legitimately select the same occurrence as the external match.
Excluding every primary-looking record is also wrong: after the primary slot is
filled, a later record matching both predicates can fill the external slot.

## Actual decision table

Indices refer to the ordered list. `I` means Internal domain; `E` External;
`W` means source Webrtc; `-` is an absent Optional domain/source, not a third enum
variant. `x`, `y`, `z` are distinct nonempty IDs and `empty` is the empty string.
Every record has the rest of its required declared fields; this is a selection
table, not permission to omit those fields from an admitted fixture.

| Candidates in order | first_agent | first_external | agent | external | Existing host action |
|---|---:|---:|---:|---:|---|
| [] | none | none | none | none | no call update |
| nil, (x,-,-) | none | none | 1 | 1 | update x |
| (x,E,-), (y,I,-) | 1 | 0 | 1 | 0 | update y, external values from x |
| (x,E,W) | 0 | none | 0 | 0 | external fallback uses x |
| (x,E,W), (y,E,-) | 0 | 1 | 0 | 1 | first switch arm consumes x |
| (x,I,-), (y,E,W) | 0 | 1 | 0 | 1 | filled agent slot lets y fill external |
| (x,E,W), (y,E,W) | 0 | 1 | 0 | 1 | later dual match remains eligible externally |
| (x,E,-), (y,E,-) | none | 0 | 0 | 0 | ID fallback may alias external |
| nil, (empty,-,-), (z,-,-) | none | none | 2 | 2 | ID fallback skips empty ID |
| (empty,I,-), (y,E,-) | 0 | 1 | 0 | 1 | no update: chosen agent has empty ID |
| (empty,E,-), (y,-,-) | none | 0 | 1 | 0 | update y; external ID need not be nonempty |
| (x,I,-), (y,I,-), (z,E,-) | 0 | 2 | 0 | 2 | first agent match wins |
| mixed-case wire equivalents of either dual-match row | same after codec | same | same | same | independent codec vector required |

The nil case is supported by the helper's contract; the actual host assembly already
omits missing child/parent pointers. Empty agent ID after a primary match does NOT
trigger the ID fallback. Two distinct records sharing an ID remain distinct
occurrences. Mutation cases must also use those equal-ID/different-other-field pairs.

## Deterministic resource and failure contract

Fixed initial implementation limits: four local selection inputs, eight selectors,
64 elements per input and 256 total elements, four prior exclusions or first-present
references per node, 64 predicate AST nodes per first selector and 256 total, three
authored field segments, and the already admitted accessor resolution/plan bounds.
No truncation or sampled match search is permitted. Validate all input cardinalities,
references and required predicate paths before any target command is invoked; a
first match at index zero cannot hide a later invalid required read or an over-limit
list. Predicate text values examined by this feature are limited to 4 KiB each;
the aggregate examined path/value bytes are capped at 1 MiB per binding occurrence.
These are selection-work limits, not a new size limit on unrelated event fields.

Compile and admission failures identify the binding, input/selector name, declared
path and violated account. Runtime failures retain binding/event occurrence and the
first offending input/index/path in declared-input, list-index, selector order.
Use closed causes distinguishing invalid input, undecidable predicate, unsupported
construct and resource exhaustion. Follow the existing binding failure strategy;
do not reinterpret these as no match, partial invocation or successful omission.
Native selectors borrow typed candidate records and retain indices rather than
cloning the whole list or expanding a combinatorial match space. Preflight reads
and resource accounting must have the same ordering in Rust, Go and conformance.

## Native and conformance delivery

Compile one closed selection plan with typed input handles, backward selector
references and accessor-node references. Native Rust and Go emit typed helpers
executing this plan, sharing bounded accessor helpers rather than generating a
general interpreted JSON query API. Both receive the same host-prepared typed input
once; neither synthesizes event metadata or consults ambient session state.

Conformance must support direct declared-list inputs with executable binding
witnesses for the decision table, including explicit Optional selection absence.
The expected selections are independently tabulated input indices/typed values;
the target does not return its own expected result. Reuse accessor observation
admission for projected values and reject observationally ambiguous nested Optional
cases explicitly rather than inventing native states. Observe actual command input
values to catch changed selected records; use distinguishable existing record fields
when duplicate IDs would otherwise hide a wrong occurrence.

Required native Rust/Go and conformance mutants: last instead of first; swapped input
order; ignored exclusion; exclusion by ID/value; exclusion of all primary-like records;
exclusion of final fallback agent; missing ID fallback; missing external fallback;
fallback after an empty-ID primary; nil treated as a zero record; Unknown treated as
false; a hidden invalid/over-limit tail. Retain ordinary selector success cases: a
blanket conversion/selector refusal cannot satisfy these tests.

For a host-converted list, retain a typed `selection-input-conversion` obligation
identifying the binding-local input, exact from/to type pair and conversion reason.
Its independent examples must compare the host's actual preparation with the stated
root/child/parent ordering and field preservation. This is a tagged obligation at
the existing conversion seam, not a host-provided expected-value callback. An absent
conversion witness affects that input-dependent coverage claim; it must not erase
direct-list selector coverage or turn target-only self-consistency into adoption.

## Remaining host authority and transformations

The actual call command takes a selected call identity, not an invented session-ID
payload field. Its session-scoped store is reached through the authenticated host
handler. Actor permissions alone do not prove that delivery context. Preserve that
handler authority and event-occurrence correlation; any future declared context
source must follow the separately designed authenticated-context contract.

Still host-owned: raw enum decoding and homogeneous candidate preparation; return
without an update when no agent or an empty agent ID is selected; direction from the
external leg's first nonempty type then the agent's; state conversion from the agent;
source from agent first then external; queue from external first then agent; external
from/to/anonymous; recording from nonempty recordings on either selected leg. The
store sets recording true when observed but does not clear an existing true flag
merely because this event has no recording. It also preserves state against a late
regression, conditionally preserves nonempty fields, stamps a clock observation,
derives transfer permission and removes terminal calls. Selection alone implements
none of those stateful/clock/conversion behaviors. No full call binding is shown as
complete until those exact obligations and authenticated delivery are reviewed.

## Compatibility and implementation surfaces

Coordinator decision: this is one unreleased coordinated change. Retain source
`ess/3`, ordinary suite `ess-conformance/6`, coverage suite `ess-conformance/7`,
report/2 and target-failure/3 for the additional selection vocabulary. Do not allocate
suite/8 or /9. Preserve legacy sources, mapping strings and serialized bytes. The
coordinator owns conditional feature/version routing across accessor, selection and
other unreleased capabilities. Exact released 0.22.2 readers must refuse new source,
suite and target-failure vocabulary at admission before target effects; current
readers must admit it with the correct conditional version and report selection.
IR keeps its existing envelope with a closed typed variant, as in the reviewed
accessor migration; no generic IR property bag or new top-level IR envelope is added.

Implementation scope: domain binding parsing/typed selector validation and existing
expression adapters; compiler resolved plan/source serialization; native Rust/Go
binding emission; synthesis descriptions/target-failure routing; conformance suite
and Go admission, selection witnesses and report/input lineage; semantic diff for
changed predicate/order/exclusion/fallback/input conversion; CLI formats and public
schemas/guides via the coordinator. Verify descriptions do not mislabel a host
conversion as a mechanical selection or successful downstream adoption.

Dispatch the core selector implementation with the direct-list native/conformance
table as its deciding cases. Track the measured consumer's exact preparation,
wire-codec and host-update obligations separately and explicitly. No source, build,
test execution or planning mutation was performed while preparing this proposal.

## Supported constraint boundary

Native emission and observation construction refuse selection input types with
reachable struct or newtype invariants or reading contracts, including list aliases and nested members.
The selector does not have executable authority to validate those constraints.
Native refusal is `selection-constraint` under `ess-target-failure/3`, before any
artifacts are emitted. Host-prepared helpers obey the same static boundary. This
capability refusal is distinct from malformed runtime values, Unknown predicates,
and resource failures. Ordinary unconstrained direct and prepared records execute.
