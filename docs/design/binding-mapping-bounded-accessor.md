# Bounded event accessors in binding mappings

Status: proposed design; implementation and consumer adoption are not yet evidenced.

## Purpose and boundary

A binding can copy a declared value inside an event envelope into a command input without
adding fictitious fields to that event. The new spelling is, for example,
`mapping: {status: event.data.status}`. It is a projection of a value the model declares.
It does not select a list element, index a map, inspect transport metadata, evaluate a
predicate, supply a default, or invoke a command conditionally.

The implementation must agree across validation, resolved IR, generated documentation,
Rust and Go service synthesis, conformance synthesis and both conformance runners.
Admitting a dotted string while an emitter treats it as one field name is incorrect.

This design retains the existing one-field construct. A new accessor is a distinct typed
construct; it never changes the interpretation of a serialized `event_field` or `observed`.

## Syntax and finite bound

An accessor starts with `event.` and has two or three nonempty declared field-name
segments after that prefix. One segment continues to be the existing flat mapping.
Thus `event.data.status` and `event.data.body.text` are accessors. The prefix is not a
segment. Four segments, brackets, wildcard selectors, predicates and numeric indices
are unsupported. An empty segment is invalid syntax. Segment names obey the existing
field-name rules; dots are separators and have no escape syntax.

The limit is **three segments after `event`**. The draft's two-segment limit counted the
envelope inconsistently: the measured body-text mapping has `data`, `body`, and `text`.
The limit follows that actual nested envelope and remains finite. Newtype unwrapping and
union dispatch are typed operations, not additional author-written path segments.

Paths use declared ESS names. A field's `wire:` name does not become a second source
spelling, even when it contains dots. Wire decoders and wire-facing descriptions use the
declared field's existing wire name. Generated language identifiers use the same naming
and collision allocation as the struct declaration, separately for each segment.

## Resolution and result type

Resolve the first segment against the triggering event's declared fields. Resolve each
remaining segment against the preceding field's type. This is one shared domain
resolution rule used by domain validation and compiler lowering; the compiler does not
invent a more permissive second walk.

The resolver returns a leaf type, an effective result type, a `may_miss` traversal flag,
and a closed access plan.
It also retains the full authored path for diagnostics and display. It records the
declared type/member identities needed for provenance. Ordered maps keep union branch
plans deterministic.

| Type encountered while another segment remains | Meaning |
| --- | --- |
| Declared struct | Resolve the segment against its declared fields. |
| Declared newtype | Unwrap its representation, retaining the wrapper identity in the plan; continue at the same segment. Its invariants do not become mapping expressions. |
| `Optional<T>` | Record that the result may be absent; unwrap `T` and continue. |
| Declared tagged union | Dispatch as described below. |
| `List<T>` or `Map<K,V>` | Refuse traversal. Copying the entire collection at the terminal segment remains allowed. |
| Primitive or enum | Refuse the nonexistent member as an unobservable fact. |

Resolution tracks `(declared type, remaining segment position)` on its active work stack.
Re-entering that pair without consuming a segment is a recursive accessor expansion and
is refused as unsupported. It does not reject unrelated recursive declarations. Cycle
detection is only one boundary: the resource account below also bounds finite expansions.

### Shared plans and resource account

Intern each reachable `(source type reference, remaining segment position)` once per
accessor. The key includes any inline Optional/container structure and the terminal type;
newtypes retain their declared handles. Resolve with an iterative worklist and an explicit
active stack. Memoization survives completion of a branch. The result is a directed
acyclic graph: a node table plus child node references, not recursively embedded copies
of branch plans. Assign stable node ids by the ordered semantic keys after resolution,
and serialize each node once. Admission rejects duplicate ids, dangling edges, cycles,
unreachable nodes and nodes whose operation does not agree with its declared result type.

The same sharing must survive every consumer. A runner visits the selected branch only.
A native generator emits at most one typed helper per plan node plus calls from its
parents, rather than inlining a completed subtree again at every union alternative.
Descriptions print the authored path and refer to shared branch accounts; they never
expand a path into every possible variant sequence. This makes the repeated diamond
`U0={a:U1,b:U1}, ... , Un={status:String}` linear in its declarations and edges.

These implementation limits apply only to the new accessor capability:

| Account | Limit | Reason |
| --- | --- | --- |
| Author-written segments | 3 | The measured nested envelope. |
| Active typed operations on one traversal | 128 | Bounds alias/union chains and native helper-call depth independently of path length; no host call-stack recursion is needed during compilation or interpretation. |
| Unique plan nodes per accessor | 4,096 | Allows branching well beyond the measured two/three-field projections while giving construction and admission a fixed ceiling. |
| Child references per accessor | 16,384 | Bounds wide unions independently of shared node count. Count a repeated reference each time it is stored. |
| Canonical plan bytes per accessor | 1 MiB | Bounds labels, field names and references as well as topology. |
| Aggregate new plan nodes / references / canonical bytes per compiled model | 65,536 / 262,144 / 16 MiB | Bounds many individually small accessors; legacy fields, types and flat mappings are not charged to this account. |
| Serialized accessor-plan bytes per projection or suite | 16 MiB | Bounds repeated plan occurrences across generated scenarios; repeated serialized bytes are charged even when source resolution shared them. |
| Additional generated source per accessor, per native target | 2 MiB | Bounds accessor helper expansion and escaped identifiers; base type/service generation retains its existing separate limits. |
| Aggregate additional accessor source per native target | 32 MiB | Prevents many accepted accessors from creating unbounded new source in one synthesis run. |

The numeric ceilings are conservative capability limits, not claims that larger models
are semantically invalid or that the whole compiler now has these memory bounds. Their
ratios reserve four edges per node and bounded presentation overhead; none is a security
scanner or repository gate limit. Count before allocating/enqueuing a node or edge.
Count UTF-8/escaped output bytes through a bounded sink; do not first build an unbounded
string to measure it. Sorted binding names, target-input names and semantic node keys
make the first exceeded account deterministic. Refuse with the accessor path, account,
limit and attempted count; never truncate, drop a branch or silently fall back to a
flat lookup. Native code generators must retain sharing and their own output counter.
Memoized nodes retain their maximum remaining operation depth. Compute DAG depth from
child depths and reject an over-limit root even when memoization shortened the active
work stack; the bound is on actual possible execution, not one compilation traversal.
Suite admission rechecks the same graph, depth and byte limits. A writer that cannot
fit its entire accessor account refuses that projection with an explicit capability
account; it does not publish a silently incomplete suite.

A well-formed accessor whose plan exceeds a construction limit receives an unsupported
accessor-resource refusal, not a type error. A source accepted within those limits may
still receive a target-specific capability refusal if that native representation cannot
be emitted within its budget. Such a target does not claim a generated implementation.
The limits do not change admission or bytes for any existing flat-only specification.

At the terminal segment, the value is copied whole. Struct, union, list, map and Optional
leaves do not entail traversal of their contents. A terminal Optional retains its declared
type but does not, by itself, set `may_miss`: the whole optional value is observable,
including its empty state. This distinguishes copying an Optional value from projecting
through a possibly absent container. Existing flat mappings retain their original
conversion and type rules.

### Union projection

The next segment names a member of a union's **variant payload**, not a variant name,
the discriminator, or the wire content-key wrapper. For example, if `event.data` is a
union of `Ready {status: String}` and `Gone {reason: String}`, `event.data.status` reads
the `Ready` payload's `status`; on `Gone` it produces absence. This admits an Optional
target and refuses a required target. It does not select an arbitrary variant.

Resolve that same remaining suffix in each declared variant. A branch whose payload does
not declare the requested member records `Absent`; a branch reaching a declared member
records its access plan. At least one branch must resolve the complete suffix; otherwise
the path is `UnobservableFact`. All successful branches must have the same terminal type,
including its Optional wrapper. Distinct nominal newtypes remain distinct types;
conversion permissions do not choose among incompatible branch result types. Incompatible
branch leaves are a type-mismatch diagnostic naming the alternatives.

A branch that encounters a list/map traversal, a resolution cycle or malformed path is a
refusal of the entire accessor, not an absent branch. A missing declared member is the
only reason, other than a declared Optional, for a valid branch to produce absence.
Union traversal makes the effective result Optional even when every current variant has
the member, preserving the draft's explicit rule about union projections.

On the wire, dispatch reads the declared discriminator and the adjacent content key
defined by `ess_gen::schema::union_content_key`. It then follows the selected payload's
field names. A malformed discriminator, unknown variant or missing required content
does not mean a valid alternative without the member; it is invalid input. Rust dispatch
matches the actual generated enum. Go dispatch uses the generated union representation
and its existing validity contract. Neither reads an invented flat JSON member.

### Presence, null and invalid input

The effective result type is `Optional<Leaf>` when `may_miss` is true, flattening a
terminal Optional for assignment; otherwise it is exactly the terminal type. The plan
retains `may_miss` separately so traversal absence cannot be confused with an observed
empty Optional value. Repeated Optional layers traversed on the way do not create extra
command-input presence states. Newtype identity at the terminal value is preserved;
unwrapping is only a traversal operation.

The **typed native** result is one of `Projected(typed_leaf)`, `Unavailable`, or an
invalid-input error. None at an Optional container **while further path segments remain**
produces `Unavailable`. At the terminal member, preserve the whole typed value, including
every nested Some/None layer. A terminal outer None produces `Projected(None)`; terminal
`Some(None)` instead produces `Projected(Some(None))`. An explicitly typed host conversion
can consume either whole value. Native mappings accept typed events directly and do not
insert a wire decode/encode normalization boundary.

When an existing wire decoder constructs such an event, it continues to decode a missing
or null Optional member as outer None. That decoder behavior does not authorize a
conformance observer to reconstruct the native value from the same JSON spelling. The
observer's separate rule below retains member presence and recognizes ambiguous wrapper
states. Null at a required position and a missing required struct member are errors. An
unexpected scalar where the plan requires a struct is also an error. An Optional target
is not permission to accept malformed input.

An unavailable projection fills no command-input value. Generated typed commands receive
outermost `None` for the declared Optional target, and its wire member is omitted. A
projected value follows an explicit assignment plan, including every Optional wrapper the
declared target needs. A projected empty terminal Optional is preserved as a typed value
unless an explicitly declared conversion consumes it. The accessor itself synthesizes no
empty string, zero, false, empty object or default.

### Target Optional construction

Derive an assignment plan from the exact source type `S` and target type `T` using the
existing assignability rule in its existing order: if `S == T`, use `Identity`; otherwise,
if `T = Optional<U>`, use `WrapSome(assign(S,U))`; otherwise this is not an assignable
copy. Equality comes first, so a terminal Optional is preserved whole rather than
unconditionally unwrapped. Nested Optional targets are supported by this recursive type
rule, implemented as a bounded iterative plan; the existing inline-type-depth limit
bounds the number of wrapper nodes. Required newtype wrappers are never inferred.

For `Projected(v)` without an authored conversion, run `assign(Leaf,T)` directly. The
`may_miss` flag controls the separate `Unavailable` case; it does not add a gratuitous
Some around a successfully projected terminal Optional. Thus two paths can have the same
effective result type while their successful value is lifted according to its actual
terminal type. Rust emits explicit `Some` construction. Go allocates the corresponding
pointer layer at every `WrapSome`, using a temporary when taking an address is required.

| Actual projection | Target | Typed command value | Existing object-member wire representation |
| --- | --- | --- | --- |
| `Projected("ready": String)` | `Optional<Optional<String>>` | `Some(Some("ready"))` | `"ready"` |
| `Projected(None: Optional<String>)` | `Optional<String>` | `None` | Member omitted. |
| `Projected(None: Optional<String>)` | `Optional<Optional<String>>` | `Some(None)` | Member present with JSON `null`. |
| `Projected(Some("ready"): Optional<String>)` | `Optional<Optional<String>>` | `Some(Some("ready"))` | `"ready"` |
| `Projected(Some(None): Optional<Optional<String>>)` | `Optional<Optional<String>>` | `Some(None)` | Member present with JSON `null`. |
| `Projected(Some(None): Optional<Optional<String>>)` | `Optional<Optional<Optional<String>>>` | `Some(Some(None))` | Member present with JSON `null`. |
| `Unavailable` from any traversed Optional/union | `Optional<Optional<String>>` | Outermost `None` | Member omitted. |

This wire behavior is the existing encoder's behavior: outer optional absence omits a
member, while an inner absent Optional encoded as a value emits null. Do not flatten all
target wrappers to make the strings simpler. The existing decoder treats a missing or
null optional member as outer None; consequently some nested in-memory states are not
injective on JSON round trip. This accessor neither changes that codec nor claims exact
recovery of hidden nested Some/None layers from wire observations. Native generated-code
tests assert the typed wrapper construction before serialization. Conformance uses the
separate supported observational subset below and refuses an ambiguous expectation; it
never feeds an invented decoded wrapper stack into the native assignment rule.

### Assignability and conversions

First compute traversal presence over the complete path. A path with `may_miss` cannot
fill a required target, even if a `conversions:` entry permits those nominal types to meet.
That refusal is specific to the new accessor; it does not retroactively change legacy
flat mappings. Its hint offers two model repairs: accept an Optional input, or map a
total whole value and supply an explicitly owned conversion. Neither repair authorizes
adding an invented field to a wire event.

When `may_miss` is false, copying a terminal `Optional<T>` may use an existing exact
`Optional<T> -> RequiredType` conversion. That conversion consumes the whole observable
Optional, including None; it is not an implicit fallback invented by the accessor. Its
authored reason must describe the actual operation and its host implementation must be
verified before adoption is claimed. The draft's blanket reference to an Optional
"segment" is clarified here to distinguish this case from traversal through Optional.

Then apply the existing assignability rule to the effective result and target types.
If they do not agree, require the existing exact `(from, to, because)` conversion for
those effective types. Do not automatically infer `Optional<A> -> Optional<B>` from
`A -> B`, and do not invoke a converter for `Unavailable`. A conversion on a path with
`may_miss` preserves that traversal absence. A terminal `Projected(None)` is a value of
the declared Optional source type and is passed to its exact whole-value conversion.

For a declared conversion, derive its exact source argument with `assign(Leaf, Effective)`
on `Projected(v)` and call the host once; its declared result is already the exact target
type. Do not apply a second automatic target lift around a converter's result. Never call
it for `Unavailable`, even if its source is Optional. A terminal empty Optional is passed
as None, while a required leaf from a possibly unavailable traversal is passed as Some.
Mechanical conversions use the same argument/result typing and may be generated only
where the existing generator knows their implementation. A malformed input calls no
conversion and produces an error. These call/no-call rules are separate from how many
Some wrappers the assignment plan builds.

An authored reason permits a type crossing; it is not executable conversion code.
The existing synthesis distinction remains: representation-preserving conversions that
the generator already knows how to implement may be generated, while other crossings
remain named transformation obligations. This feature must not turn an opaque crossing
into a copy. Conformance must not assert that an opaque converted value equals its raw
source. For new accessors it emits an explicit mapping-conversion coverage refusal unless
the transformation is mechanically determined. Existing flat serialization is unchanged;
any pre-existing flat-conversion conformance defect is reported separately.

## Typed model and compatibility

### Authored specification and compiler IR

Add `MappingSource::EventAccessor` with validated ordered segments; preserve
`MappingSource::EventField { field }` for exactly one segment. The author-facing YAML
remains a string. Display reconstructs the original dotted source spelling.

Add `ResolvedMappingValue::EventAccessor` with its authored segments, effective
`ResolvedTypeRef`, and closed resolved access plan. Plan operations are field selection,
newtype unwrap, Optional unwrap, and union dispatch with ordered child references. Plans
reference declared members and types; arbitrary JSON paths or opaque expression strings
are not admitted. The DAG node table and resource account above are persisted, not expanded
back into a tree during serialization. A branch terminal distinguishes a selected value
from declared absence. The separate assignment plan states every target Optional lift.
Only the new enum variant contains these fields. Flat `event_field` and literal values
serialize byte-for-byte as before.

The language capability is introduced as **`ess/3`**, preserving every previously valid
`ess/1` and `ess/2` document's meaning and canonical bytes. Accessors require `ess/3`;
an accessor in an older format is refused with a format/capability diagnostic pointing
to the source declaration. Flat documents do not get upgraded automatically. An exact
old reader must refuse an accessor document on its unsupported source-format number.

The resolved IR remains the compiler's existing serialized model, with a new closed
mapping alternative. There is no new `ess-ir/2` envelope. The source-format boundary and
the distinct `event_accessor` kind prevent an old consumer from mistaking the new
construct for a flat field. All in-repository Rust enum consumers must handle it
exhaustively; any external IR consumer must explicitly support the new alternative.

### Conformance suite vocabulary

Keep `ScenarioValue::Observed { event, field }` and its flat lookup unchanged. Introduce
`ObservedAccessor` with the originating event and a closed, serializable access plan.
Conformance values are keyed by **declared** field names, as `ObservedEvent.payload`
already specifies; transport adapters remain responsible for normalizing wire names.
The runner must not heuristically try source names and wire names until one succeeds.
The serialized plan records the union and presence operations and the bounded assignment
plan with its exact target type, so a runner does not need the original IR or rediscover
types from arbitrary JSON. The native assignment plan is not itself permission to
reconstruct typed Optionals from observations. The compiler also derives the observational
capability below; admission rechecks its finite type/presence conditions. A runner either
computes the uniquely determined command-member observation or reports an explicit
unsupported capability. It must not guess a typed leaf and compare that guess's encoding.

### Wire-observable Optional projection

Keep three terminal observations distinct: `Missing`, `PresentNull`, and
`PresentValue(node)`. Do not turn Missing and PresentNull into the same Node or decode
either into an invented native Optional. Traversal unavailability is a fourth case:
the path could not reach its terminal member because an earlier Optional was absent or
no selected union variant declared that suffix. It always demands an omitted target
member and calls no converter. Malformed required structure remains an error.

For a copy with no conversion, let `n` be the terminal source's number of outer Optional
wrappers and `k` the number of `WrapSome` operations added by `assign(Leaf,Target)`.
The residual leaf is copied whole. Native source wrappers are not removed. The following
finite observation rule applies when the copied residual leaf has no hidden nested
Optional representation, as checked below:

| Source wrappers / added target wrappers | Missing terminal member | Present null | Present non-null value |
| --- | --- | --- | --- |
| `n = 0` | Required-member error. | Required-value error unless the declared residual type admits null; such a type requires a separate supported observation capability. | Same present value; added Some wrappers do not change its wire spelling. |
| `n = 1, k = 0` | Omitted target. | Omitted target: the only admitted empty native Optional is outer None. | Same present value. |
| `n >= 1, k >= 1` | Present-null target: native outer None is wrapped in Some. | Present-null target: every compatible empty wrapper interpretation is inside at least one new outer Some. | Same present value. |
| `n >= 2, k = 0` | Omitted target would be determinate. | Ambiguous: decoded outer None encodes omission, while native Some(None) encodes present-null. | Same present value would be determinate. |

The last row is an **unsupported mapping-observation capability at synthesis time**,
including its otherwise determinate missing/value observations: a valid source can
produce the ambiguous null case, so this build does not claim a mapping-wide verifier.
Emit a stable `AmbiguousOptionalObservation` capability refusal naming the binding,
target, source type and equal-type assignment. Do not generate a scenario whose verdict
depends on whether the source happens to choose a non-null example. Preserve the model's
binding and native synthesis support; only the unprovable conformance mapping claim is
refused and retained in the coverage account. Its flow/delivery claims remain separately
accounted for.

For example, source and target both `Optional<Optional<String>>` require this refusal.
The native event `Some(None)` still maps by Identity to `Some(None)` and emits null.
The observer cannot instead demand omission by decoding the observed null. With target
`Optional<Optional<Optional<String>>>`, one added Some makes both candidate native
interpretations encode null, so the presence rule is determinate and supported. Missing
at that source also yields target present-null; it is not traversal unavailability.
Tests must keep those two missing cases separate.

The supported subset is mechanically checked on the already bounded type/plan DAG:
strip only the terminal's outer Optional chain, derive `k` from the exact assignment
plan, and inspect the residual copied type. Any nested Optional chain hidden inside a
newtype, struct member, union payload, list item or map value makes this initial
observation capability unsupported for that whole-value leaf. The walker memoizes
typed nodes and refuses cycles or resource exhaustion using the existing accessor
accounts; it does not enumerate values. The predicate is conservative because those
hidden native states can alter nested member presence while one Node hides their
construction. It does not change the native copy or the ordinary structural codecs.
Newtypes whose transparent representation itself admits null likewise need a separately
proved observation capability; the scalar `n`/`k` table is not applied through a newtype
boundary. Existing nonmechanical-conversion observation refusals still apply.

For a mechanically supported conversion, conformance must additionally prove that its
result observation is independent of every wrapper ambiguity at its exact source type.
This feature supplies no generic proof engine or host-provided expected-value callback:
an existing representation-preserving conversion can reuse the table only when its
established codec contract preserves its premises. Otherwise emit the same explicit
mapping-observation capability refusal. Native conversion call/no-call rules stay as
specified above. No target under test supplies its own expected value, and no silent
input normalization or new execution boundary is introduced.

The new observed value is admitted only in binding invocation expectations, where an
absent result has the concrete meaning that the target input must be absent. It is not
accepted in value-only positions elsewhere in an authored suite. Admission checks the
closed plan, path bound, event reference, and allowed position before execution.

Ordinary suites that actually contain this vocabulary emit **`ess-conformance/6`**.
Coverage-bearing suites that contain it emit **`ess-conformance/7`**. An accessor-specific
refusal in the retained coverage account also selects version 7, even when no accessor
mapping scenario could be emitted; refusing a new capability must not hide its new
coverage vocabulary inside version 5. Version 6 forbids
the coverage envelope; version 7 requires the same coverage account and identity checks
as version 5. This retains the existing distinction between ordinary and coverage-bearing
suites rather than silently making coverage optional. Ordinary suites without accessors
continue to emit version 4, and freshly built coverage-bearing suites without accessors
version 5; explicit filtered descendants follow the lineage rule below.
Existing versions 1 through 5 retain their old admission and canonicalization rules.
The vocabulary is refused under those old versions; relabeling it as version 4 or 5 is
not a supported downgrade. A freshly synthesized component with no accessor vocabulary
does not need a new version merely because some other component declares a path. An
explicitly filtered descendant of a version-7 coverage suite retains version 7 even if
its selected scenarios contain no accessors: its parent lineage still uses that
vocabulary. Version-5 parents have only version-5 descendants. No version-5 child is
allowed to hide a version-7 parent.

Update the Rust reader and generated Go reader before enabling the new writers. Both
must reject unknown versions before executing scenarios. Coverage construction must
select 5 or 7 from the actual retained suite instead of unconditionally stamping 5.
Provenance comparisons and authored-batch admission must not conflate a source model's
identity with the suite vocabulary selected after synthesis.

For a new invocation expectation, preserve the distinction between expected present
inputs and expected absent inputs. Removing an absent field from the expected-value map
and retaining today's subset comparison is insufficient: that would accept a spurious
nonempty value. Compare each absent accessor's target explicitly against input absence.
No previous ordinary `Observed` expectation gains this new omission interpretation.

### Report production, admission and exact selection lineage

No new report format is needed. `ess-conformance-report/1` continues to describe only
suite majors 1 through 4, with its existing fields, validation, canonical bytes and
default selection. Do not add suite/6 to its reader or relax the producer's legacy-only
contract. New ordinary suite/6 and coverage suite/7 runs use the existing explicit
**`ess-conformance-report/2`** contract, as suite/5 runs already do.

| Suite admitted for execution | Requested report mode | Routing before target construction |
| --- | --- | --- |
| 1–4 | Default or explicit 1 | Existing ordinary execution and report/1 output. |
| 1–4 | Explicit 2 | Existing report/2 execution, exact-suite accounting, coverage unknown. |
| 5 | Default or explicit 1 | Existing refusal requiring explicit report/2. |
| 6 or 7 | Default or explicit 1 | Refuse with the actual suite version and instruction to select report/2. No target is constructed or invoked. |
| 5, 6 or 7 | Explicit 2 | Execute exact admitted suite and produce report/2; enforce its unchanged outcome/count/coverage policy. |

The Rust CLI performs this check in the existing `coverage::execute` wrapper before the
closure that creates a target, irrespective of whether `--report-out` was supplied.
Explicit `--report-format 2` writes `CountReport` to `--report-out`, uses the existing
`CountRun` detailed JSON/YAML stdout path, and retains existing text and strictness
semantics. Do not enter `ConformanceReport::standalone` for 5, 6 or 7. Library callers may
run an admitted suite directly, but must use `CountReport::from_run` for new suites;
the legacy standalone producer remains deliberately inapplicable and documents that
precondition. The CLI must not discover incompatibility through its assertion afterward.

Generated Go retains default `ESS_REPORT_FORMAT=1`. `Run` requires explicit
`ESS_REPORT_FORMAT=2` for 5, 6 or 7 before `newTarget`, execution adaptation or any scenario
call. `ESS_REPORT_OUT` then writes the existing report/2 document. Environment strictness
options keep their current interaction and no incompatible request silently switches
format. Update generated README instructions when its embedded input requires report/2.
Execution adaptation must retain the originally admitted suite bytes and identity;
reporting a rewritten or reserialized suite's digest is incorrect.

`CountReport::from_run` already checks the executed suite digest and provenance against
the admitted input. Its reader requires the exact admitted suite, then checks report
format and policy, source identity, exact suite version and `sha256-json-bytes/1` digest,
sorted disjoint complete outcome ids, checked counts and the expected coverage account.
Those requirements remain unchanged for 6/7. `CountRun` wraps the same admitted summary
and adds the existing detailed results; it requires the same original suite input.
Ordinary suite/6 has coverage knowledge `unknown`, so a successful execution remains
inconclusive under report/2's complete-selection qualification. Do not invent a complete
coverage inventory because the new accessor cases passed.

Coverage suite/7 keeps the same `ess-conformance-input/1` carrier as suite/5: exact
selected suite JSON plus exact parent suite JSON, not a regenerated model or just their
claimed hashes. Generalize the suite/5-only parent-version check to accept known coverage
versions 5 and 7 under the lineage rules above, then admit every referenced parent and
verify its exact digest, source/component identity, selected ids and recomputed coverage
relationship. A version-7 child references the actual version of its parent; relabeling
a parent, dropping it, reserializing its bytes or substituting a same-sized scenario set
must fail. Direct fresh /5 and /7 selections use their original admitted suite bytes.
Filtered selections retain the parent vocabulary version, so filtering cannot downgrade
new semantics or produce a /5 carrier that names an unreadable /7 parent.

Exact old readers reject the new source format and suite versions before they can admit
an exact input for report/2. Report/1 readers independently continue to reject a document
claiming suite/6 or /7. An old report/2 implementation is not required to reject the
report's outer format: it is the same report contract. It cannot qualify that report
against a newly admitted suite it does not understand. The previous-reader test must
exercise this actual report-plus-exact-input path, not claim that every generic JSON
parser rejects a report/2 envelope. New readers must reject the same report when paired
with different suite bytes even if the scenario ids and counts happen to match.

The propagation surface includes suite admission; input/1 selection and lineage; ordinary
CLI reference-run, persisted-suite and `--report-out` paths; generated Go admission,
execution and report writer; report/1 producer/reader guards; report/2 summary and detailed
readers; CLI release-evidence qualification against independently supplied exact selection;
and browser replay/input emission that uses the coverage carrier. Any external evidence
adaptor must use the existing report/2 contract with its exact input or refuse it; this
feature does not weaken an external tool's report/1-only admission. No report/3 or changed
report/2 field meaning is introduced.

## Consumers and their obligations

| Consumer | Required work |
| --- | --- |
| `ess-domain/src/binding.rs`, `types.rs`, `system.rs` | Parse the distinct source, resolve declared paths and presence, expose specific diagnostics, gate the capability on `ess/3`. |
| `ess-primitives` validation codes and compiler diagnostic mapping | Give partial-to-required, bounded-depth and unsupported-traversal refusals stable, distinct identities; retain `UnobservableFact` for missing declared members. |
| `ess-compiler/src/resolve.rs`, `ir.rs` | Lower the checked path to a typed plan; account for each touched declared type/member; preserve old variant bytes. |
| `ess-gen/src/docs.rs` | Print the full source path, effective type, presence and conversion obligation in interaction documentation. |
| `ess-gen/src/asyncapi.rs` | Emit a distinct accessor source kind with ordered segments; do not serialize it as a dotted `event_field`. |
| `ess-gen/src/openapi.rs` | Add a binding-accessor extension on affected operations only, including source event, target input, path and presence. Keep unrelated and flat-only documents unchanged. |
| `ess-gen/src/graph.rs` | Annotate accessor binding edges with target/path mappings; preserve existing flat edge labels and bytes. |
| `ess-gen` provenance traversal | Include selected members, newtype wrappers, union variants and their wire names so a relevant change invalidates the projection. |
| `ess-synth/src/plan.rs` | Represent accessor inputs distinctly; determine copy/lifting/mechanical conversion versus an explicit transformation obligation. |
| `ess-synth/src/rust/system.rs`, `rust/feasibility.rs` | Emit each typed field access and unwrap/match operation, allocate actual member identifiers, and produce the target Optional representation. |
| `ess-synth/src/go/system.rs` | Emit equivalent nil checks, union dispatch, member names and Optional construction; preserve the existing conversion obligation boundary. |
| `ess-conformance/src/scenario.rs`, `synthesize.rs`, `runner.rs` | Emit/admit/evaluate `ObservedAccessor`, preserve required-member failures and verify absence explicitly. Refuse invented conversion expectations. |
| `ess-conformance/src/input.rs`, `coverage.rs`, `coverage_build.rs` and suite admission | Select versions from retained vocabulary, maintain exact coverage accounts and provenance, reject downgraded new vocabulary. |
| `ess-conformance/src/go/runtime.go`, `go/mod.rs` | Implement the same plan evaluator, absence comparisons and 6/7 admission in the generated Go runner. |
| `ess-conformance/src/evidence.rs`, `counts.rs`, `report.rs` | Preserve report/1's suite 1–4 boundary; route new admitted runs through report/2 summary and detailed output with unchanged exact-input checks. |
| `ess-cli/src/main.rs`, `coverage.rs` | Refuse unsupported report/suite pairings before target creation on every run path; preserve legacy defaults and use explicit report/2 for 6/7. |
| `ess-cli/src/release_evidence.rs` | Admit original /7 input and its full lineage for report/2 qualification, retaining independent expected selection and model-identity checks. |
| `ess-conformance/src/web.rs`, `web_replay.rs` | Preserve coverage-input bytes and suite identity through browser input/replay; require supported nested suite vocabulary rather than rewriting its version. |
| `ess-xtask/src/support.rs`, CLI report/coverage/lineage and generated-Go tests | Preserve legacy default-report observations and verify the new explicit-mode/report compatibility paths; no gate-policy changes. |
| `ess-diff/src/diff.rs` | Compare ordered path segments and result/presence semantics; report a changed mapping without flattening the path to a legacy field. |
| CLI inspection, CLI consumer tests, schema-contract consumers | Display/serialize the new variant and check exhaustiveness; do not change command selection or infer extra CLI inputs. |

No change to entity-runtime lowering is assumed: this design starts from main's existing
binding/compiler/synthesis surfaces. Feature-preservation accounts and public generated
schemas must be updated through their repository-owned tooling where the new vocabulary
changes an enumerated surface; existing baselines are not waived.

## Diagnostics

| Condition | Classification and required content |
| --- | --- |
| Empty/malformed segment | Invalid mapping syntax, full source location and spelling. |
| Missing event or command | Existing undeclared-reference diagnostic, once at the missing end. |
| No declared field resolves the complete path | `UnobservableFact`, first unresolved segment and available declared members. |
| More than three source segments | Unsupported bounded accessor, actual and maximum depth. |
| List/map traversal or cyclic transparent resolution | Unsupported accessor operation, offending type and segment. |
| Traversal through Optional/union to required input | Dedicated partial-accessor diagnostic, source path, effective result, target and the two repairs above. A terminal Optional copied whole follows the declared conversion rule. |
| Incompatible union leaf types | Type mismatch, variant names and their distinct leaf types. |
| Incompatible effective result and target | Existing conversion/type mismatch contract with complete source path and exact effective types. |
| Accessor in `ess/1` or `ess/2` | Unsupported capability for that format; explicitly require `ess/3`. |
| Malformed runtime value | Input/observation error; never a successful absent projection. |
| Terminal nested-Optional observation cannot determine the target encoding | `AmbiguousOptionalObservation` conformance capability refusal; retain native binding support and the explicit coverage gap. |

The implementation must allocate stable diagnostic identifiers through the existing
catalog and test them end to end, rather than reusing an unrelated code because the text
looks similar. A typo is invalid; exceeding this build's deliberate construct boundary
is unsupported. Both are explicit refusals and neither becomes a partial compiled binding.

## Consumer adoption findings and remaining typed contracts

The measured adopter has four candidate rows: status refresh, notification message,
lead arrival, and campaign status. A fifth call-state row additionally selects different
list elements and remains outside this construct. The four-row promise is **not yet
implementable solely by adding bounded accessors**:

1. All four command inputs require session identity sourced from the subscription
   context. The wire events declare only their envelope marker and payload. One status
   payload also declares an `id`, but equivalence to the reducer's authoritative session
   identity is unproven. The other three payloads do not carry the recipient identity.
   The existing binding model has no context source; actor grants declare permission,
   not an identity value. A payload accessor cannot recover that context. Substituting
   the status payload's id without an explicit verified identity relation would change
   the reducer's source of authority.
2. Two status rows convert a **terminal** Optional payload value to required command
   inputs. Their reducers explicitly handle absence/null. Under the distinction above,
   these can be declared through the existing exact whole-Optional conversion contract:
   no Optional container is traversed to reach them. The conversion reason must preserve
   the reducer's actual empty-value behavior, and synthesis must expose the host-owned
   transformation obligation. A declaration alone does not execute or verify it.
3. The notification row requires three segments after `event`, which this design now
   admits. Its map-to-JSON-text crossing still needs an executable, testable conversion;
   retaining its raw byte spelling is not the same operation as canonical serialization.
4. A lead row transforms one whole payload struct into a different nominal command
   struct. That is a legitimate authored conversion obligation, not a mechanical copy.

The follow-up context contract must introduce an explicitly declared, typed binding
context record separate from the event payload. A source such as `context.session_id`
must resolve against that declaration. The host must bind it from the authenticated
subscription/session, validate its declared type, and supply it with the triggering
event occurrence. The model must identify the source of authority; naming a field does
not establish authentication. Generated handler signatures, invocation attribution and
conformance observations must carry the same context. Missing or mismatched context is
a refusal, not a lookup in event data. No `event.<channel ...>` pseudo-field is added.

The existing conversion model is sufficient to **declare** the measured whole-value
crossings: its key is the exact source/target type pair and its reason names the real
reducer operation. Restore those reasons without implying ESS executes them. Verify the
host implementations independently with examples covering empty terminal Optionals and
every relevant source variant, as well as JSON-text fidelity and whole-struct conversion.
If the same type pair later needs two different operations, that exceeds the existing
registry and warrants a separately designed conversion identity; this accessor does not
invent that machinery preemptively. Conformance for nonmechanical crossings still needs
an independent conversion oracle or explicit expected-value vectors, not the target
under test supplying its own expected result.

Typed context remains a follow-up contract, not admitted syntax in this accessor
implementation. Keep the four-row acceptance open until that context is supplied honestly,
the real conversion obligations are verified, and the actual consumer has written and
verified all four rows against a published ESS release.
Do not alter source Optional types, make required command inputs Optional, or invent
event fields to manufacture that count. Private consumer identifiers and exact source
citations belong in the retained review evidence, not this public-source design.

## Verification required before implementation is called complete

Write and observe failing cases before implementation, then run the changed package
lanes once after each meaningful correction. This design stage runs no code or builds.

The deciding matrix includes two- and three-segment paths; a four-segment refusal; renamed
wire fields and generated identifier collisions; nested newtypes; required and Optional
intermediate and terminal values; union members in all/some/no variants; heterogeneous
union leaves; transparent cycles; maps and lists copied whole but not traversed; exact
declared conversions, including an empty terminal Optional consumed whole; and partial
traversals that cannot be made total by a conversion permission. Exercise missing required
members, malformed union tags and null at required
positions separately from valid Optional absence.

Resource tests cover repeated acyclic union diamonds, wide unions, a long transparent
newtype chain, and many independent accessors. Assert shared node/edge counts and bounded
serialized and generated-source size, not merely successful completion. Exercise every
account exactly at its limit and on the next attempted unit, including escaped-name
bytes and repeated suite-plan occurrences; bounded-sink unit cases can cover large byte
counts without compiling large generated programs. Assert deterministic diagnostics and
no partially published projection. A mutation that embeds every completed union subtree
again must fail the size/growth assertion. Cycles, long acyclic depth and acyclic width
are separate cases.

Compile and execute generated Rust and Go examples for present/absent/union cases, and
execute both conformance runners against matching and deliberately incorrect invocation
observations. A mutant that treats the path as one key, that chooses the wrong union
variant, or that ignores an expected absent input must fail. Generated docs, OpenAPI,
AsyncAPI, graph and synthesis artifacts must expose the same declared path.

Nested-Optional cases include every row in the target-construction table and an additional
third target wrapper. Check typed native values before encoding and omission/present-null
after encoding. Spy conversions must show one call for `Projected(None)` with a terminal
Optional source, zero for traversal `Unavailable`, and zero for malformed input. A
mutation that replaces `Some(None)` with outer `None` must fail the present-null case.

For terminal source `Optional<Optional<String>>` and the equal target, native tests must
prove Identity on outer None, Some(None) and Some(Some(value)), while conformance synthesis
must retain `AmbiguousOptionalObservation` instead of generating a guessed mapping case.
For the same source and a deeper Optional target, test Missing → present-null,
PresentNull → present-null and PresentValue → that value. Test an unavailable intermediate
Optional separately: it still requires an omitted target. Repeat with a third source
wrapper and equal/deeper targets. Hidden nested-Optional leaves in newtypes, structs,
unions, lists and maps must receive the declared observation refusal without changing
their native whole-value copy. A mutation that decodes PresentNull to invented outer None
and then emits an equal-type mapping expectation must fail the refusal/accounting case;
the native source Some(None) is the concrete counterexample. No test may claim to recover
the exact hidden wrapper stack by JSON round trip.

Use exact previous-reader fixtures/probes to establish rejection of `ess/3` and suite
6/7, and preserve representative flat specification/IR/projection/suite bytes at their
existing versions. Test new vocabulary relabeled with an old suite version, as well as
coverage construction, mixed retained suites and component selection. Existing flat
mapping tests remain enabled.

Run the complete report-routing table through Rust CLI and generated Go, including
`--report-out`/`ESS_REPORT_OUT`, detailed report/2 stdout and strict/incomplete policies.
A spy target must record zero construction/execution calls for default or explicit
report/1 paired with suite/6 or /7. Admit produced report/2 bytes against the exact
original executed suite and reject changed-byte, missing-parent, forged-count and
wrong-selection variants. Compare legacy /4 default report/1 and /5 explicit report/2
bytes with previous outputs. An ordinary /6 passing run must retain unknown coverage and
inconclusive qualification. A filtered /7 child without accessor scenarios must retain
its /7 parent lineage and pass only with the exact supplied parent bytes. Exercise CLI
release-evidence qualification and browser input/replay retention without releasing or
deploying from a test. Previous-reader probes must confirm the actual nested-suite
refusal for report/2 and the independent suite-version refusal in report/1.

The operator's requested release authorizes required remote GitHub merge and release
checks. Full local workspace and ownership gates remain prohibited. Use targeted local
verification and record its exact coverage; never label an unrun local gate successful.
Crosswalk work is excluded from this feature and release scope.
