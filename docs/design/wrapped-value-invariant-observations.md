# Wrapped value-invariant observations

2026-10-03. Binding semantic design for `optional-value-invariant-observation`.
The complete contract below was independently reviewed as candidate v3, SHA256
`7d68859b7b16edc3ee0e0448229353e45ee06c59e8b79b25e6e65fbbfebddf39`.
Review `consumer-wrapped-invariant-design-v3` approved it with no findings; the review
executed no runtime tests. The design resolves WVI-D2-1/2/3 through canonical semantic
numeric work, one logical charging schedule, and incomplete-inventory serialization guards.

Implementation remains pending, sequenced with the generated-history proof work and the
browser conformance product. The integration owner must reconcile the proposed format
numbers and exact generated carriers before implementation; this document reserves none.
The implementation and browser audit paths below define the proposed change boundaries,
not permission to edit another owner's active work. Existing release publication holds
and the single held-bundle delivery boundary remain in force.

Source inspection used runtime `c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9` and the browser
product's uncommitted source based on `d849ea01c00af9360d574e06ff864d01c2564954`.
The contract from the next heading onward is byte-identical to reviewed candidate v3.

## Contract and evidence

For each nominal invariant-bearing type at each observable schema position, check every
applicable actual value and require at least one actual occurrence. Legal Optional absence,
empty List/Map and other Union variants are not predicate failures, but cannot discharge the
occurrence requirement. False, zero, empty text and present empty Struct are present.
This is global type/view-position coverage, not an implicit row-provenance guarantee.

Add closed `ViewExpectation::ValueInvariants`, wire tag `value_invariants`, usable inside
existing `ExpectView` and `EventuallyView`. Preserve historical `Satisfies` meaning and bytes.
No new target callback, authored ESS grammar, primitive predicate syntax, transport protocol,
or interpreter command semantics are introduced.

The root's retained mixed-position red remains unchanged: required and Optional fields carry
the same bounded type; synthesis emits four scenarios, zero refusals, and only the required
position's invariant scenario. `OPTIONAL_INVARIANT_POSITION_OMITTED_WITH_ZERO_REFUSALS` exits 1.
Source SHA256 `b1e4d4cd70cf265481164ef4a8feb8f06f113f626a098c55985dadb8625a5555`;
suite SHA256 `3fc4056059eb2f9fa920f87491dcb9c4fe3d2673ff274b581a356b1bfd91f60e`;
red log SHA256 `090e63f6c603751b5a4fe99650ba8417c09a32ccc0a1348c4bb42321eeb2db37`.
This is root synthesis execution, not this reviewer's execution or target evidence.

`synthesize.rs:10808–11140` explains the omission: `reaches` stops at wrapped kinds;
`positions_of` drops unprojectable predicates before inventory; `holds_at` proves a row,
not an applicable value. Native `runner.rs:3521` binds sequence presence only. Generated
Go `predicate.go:77` and TS `predicate.ts:131` also bind count/index. Typed input Map projection
uses values in key order (`input.rs:1068`), unlike untyped view-map flattening. These are reasons
for typed observation authority rather than additional untyped predicate rewrites.

## Exact closed DTO

The new expectation is an object with these required members in this order:

`expect, root, declarations, value, position, witness, work_limit`.

`expect` is the literal `value_invariants`. `root` is a Field DTO for one semantic view field.
`value` is the fully qualified nominal type whose nonempty invariant list is checked.
`declarations` is exactly the reachable nominal closure from root and anchor fields, plus no
unreachable declarations. Source predicates refer only to their own type's admitted paths;
they do not introduce arbitrary external declaration references.

Field DTO: required `name,type`, optional `presence`, ordered in that sequence. Names use the
existing semantic field grammar. Type is the existing canonical `TypeRef` string, including
its parser's depth-32 bound. Presence is exactly `null_when_absent` or `omitted_when_absent`.
Omitted presence preserves existing missing/null Optional equivalence. No `naming`, `wire`,
`reading`, unknown metadata or explicit null presence is admitted.

Declaration DTOs, keyed by fully qualified name, have precisely these kind-specific members:

- Newtype: `kind,of,invariants`, then optional `alphabet,prefix` in that order.
- Struct: `kind,fields,invariants`.
- Enum: `kind,variants` (ordered semantic label array).
- Union: `kind,tag,variants` (ordered map of semantic labels to TypeRef).

`kind` uses existing lowercase spellings. Invariant arrays contain objects `statement,predicate`
using the existing compiled statement and Predicate canonical serialization. Empty arrays are
explicit. Statements are diagnostic, not identity. No invented invariants, stripping of declared
constraints or acceptance of inactive kind members. Constraints are source-projected authority;
the observer does not reuse `typed_fields::declarations`, which refuses constrained types
(`typed_fields.rs:40`). Opaque reading transformations are not performed here: values at the
semantic Target boundary are already the declared representation, as on existing view checks.

Position has exactly one of these two forms:

`{"kind":"site","path":[Segment...]}`

`{"kind":"recursive","edge":[Segment...],"site":[Segment...]}`

Each Segment is exactly one closed object:

- `{"op":"field","name":"..."}`
- `{"op":"present"}`
- `{"op":"element"}`
- `{"op":"map_value"}`
- `{"op":"variant","label":"..."}`
- `{"op":"unwrap"}`

These are typed schema edges, not unchecked JSON selectors. The declaration graph determines
which operation can follow; arbitrary Map keys, list indexes and Union payload property names
never occur in a descriptor. A site path ends on entry to the named type in `value`.

Witness has exactly two outer forms:

1. `{"kind":"any_observed_occurrence"}`. Some returned row must contain an actual applicable
   occurrence for this position. All selected occurrences on all rows must satisfy the
   predicates. This explicitly proves observed-value coverage, not which instance supplied it.
2. `{"kind":"arranged_row","selector":Selector}`. Some row selected below must contain the
   actual occurrence. All selected-position occurrences on all returned rows still satisfy
   the predicates, including rows outside the witness selector.

Selector is a closed sum with two source-proved cases:

- `{"kind":"projected_identity","entity":"...","identity":Field,"expected":ScenarioValue}`.
  Identity is the source entity's declared identity and is projected unchanged by the view;
  the row matches by existing exact ScenarioValue resolution/comparison. Expected must resolve
  to the instance acted on by the arranged source-selected outcome, not an arbitrary constant.
- `{"kind":"identity_query","entity":"...","identity":Field,"parameter":Field,
  "filter":Predicate,"expected":ScenarioValue}`. The view's source is that entity; its full
  source filter contains identity equality to that parameter as a necessary positive conjunct.
  The step's query parameter is exactly the same ScenarioValue. The equality must occur alone
  or under `All`, never under `Any`, `Not`, or an unrelated binder. The two declared types
  must be nominally assignable using the existing identity/parameter rules. Other conjuncts
  remain in the original filter and must be arranged as true. Query cardinality is at most one
  source row because the equality addresses the entity identity, not because a parameter happens
  to be named `id`. All rows of this query are the witness selection; a result with more than
  one row fails the source-proved uniqueness claim.

Prefer projected identity when both proofs exist. Otherwise choose identity query, otherwise
use `any_observed_occurrence`. No blanket refusal for views without a unique selector. Existing
`identifying` at `synthesize.rs:7920` already states that non-identifying observations are weaker
rather than wrong. An identity-only query cannot distinguish an implementation substituting
one unidentifiable row for another; this remains the query contract's observable limit, not
a claim that the suite recovered hidden identity. Never infer physical reset or isolation.

Standalone suite admission validates the closed proof structure, declared types, positive
conjunct and query/reference relationships. As with existing suite expectations, the suite is
the persisted assertion authority; this does not authenticate a forged contract against an IR
the runner does not carry. Model-aware producers must derive and check these facts from the
actual IR and keep the original provenance. Browser bundle admission compiles retained sources
and compares the selected and parent suites' provenance to that IR. It does **not** authenticate
each persisted assertion DTO's declaration closure against the IR. Standalone suite assertion
authority is retained. No stronger closure-to-IR validator is introduced by this design.

`work_limit` is an exact positive JSON integer at most 9007199254740991, chosen by the producer.
Its default is 1048576 units. It is an execution budget, not a type size restriction or input
witness. Raising it changes suite content/digest, never scenario identity. A deployment may
refuse an excessive requested budget before callbacks; it must report policy/resource refusal,
not a conformance success or an unsupported type.

## Finite graph and stable obligation identities

The graph is derived from root and declarations; no independently forgeable graph table is
serialized. Its canonical key for a node is the complete Segment path from root. Empty path
is root. Construct it by this deterministic depth-first algorithm:

1. Enter a TypeRef at path P. Optional, List and Map have one child at P plus respectively
   present, element and map_value. Primitive and Enum have no child. A Struct child adds
   field(name), preserving source field-array order. A Union child adds variant(label), visited
   in UTF-8 lexical label order. A Newtype child adds unwrap. Named types are not extra segments.
2. Maintain the active named-type stack, with each name's entry path. Entering a named type
   not active creates its occurrence node and expands its body. Returning removes it from
   the active stack. Thus shared nominal schemas reached by separate acyclic routes expand
   separately: `a.total` and `b.total` remain distinct.
3. Entering an already active name at P creates a back-edge keyed by P to its earlier entry
   path Q. Do not expand that occurrence again or create a duplicate terminal site there.
   Its TypeRef and the target node's TypeRef must be the identical nominal name.
4. Every expanded nominal node with nonempty declared invariants is an ordinary site. For
   every back-edge E and every invariant site S reachable from E's target in the finite graph,
   add the ordered pair (E,S) as a recursive obligation. Reachability uses visited graph-node
   keys; repeated cycles do not add obligations. Sort sites and pairs by their canonical JSON
   bytes for deterministic output. Distinct variants/field routes remain distinct graph keys.

An ordinary site's predicate is checked on every actual visit to that site, including visits
after following back-edges. Its ordinary witness needs any visit. A recursive (E,S) obligation
checks the same predicate at every visit to S, and additionally requires a single actual value
lineage that traverses E and subsequently reaches S. Keep an edge-history set on the current
descent branch; pop it on ascent. Reaching E in one element and S in a sibling cannot witness
the pair. Any number of repeated E crossings on that one lineage is allowed. Every admitted
finite recursive value is traversed, not merely its shallow representative.

For `Node { score: Bounded, next: Optional<Node> }`, root.score is a site; next.present is a
back-edge to root. There are separate ordinary score and recursive(next.present, score)
obligations. A valid root score with absent next discharges only the ordinary obligation.
A bad grandchild score fails both observers' universal check. The recursive witness needs
at least one descendant score along next, but makes no exhaustive-depth testing claim.

Unwrap and Optional-present do not consume a JSON child. Track an active pair (graph-node,
actual-value-location) during evaluation. Re-entering it without consuming a child is a
nonproductive representation cycle, not an infinite loop. Legal Optional absence terminates
before such a revisit. A schema with only absent finite inhabitants may be valid but cannot
supply a required present invariant occurrence; arrangement reports that distinction. Do not
categorically reject productive recursive schemas or discard their sites at depth 32.

New scenario identity is:

`<value>/invariant-wrapped/at/<view>/<root-name>/<selector-digest>`

`selector-digest` is lowercase SHA256 of the canonical compact JSON Position DTO, with no
newline. Root/type/view are separate typed segments. Admission recomputes it and verifies
the id's nominal type/root/view against the expectation. Duplicate ids with different
position authority are invalid. No command, candidate ordinal, witness selector, invariant
statement, work budget or graph allocation number participates. Unrelated declaration order
cannot re-key a semantic route. Existing direct-position ids and assertions stay byte-exact;
they coexist with new wrapped/recursive obligations. No extra wrapped duplicate is emitted
for a nonrecursive route consisting only of field/unwrap that already has the direct id.

## Actual evaluation and typed facts

First resolve ScenarioValue references and query authority; then obtain one actual snapshot.
For each row, validate root presence and representation, traverse the graph, collect the
selected site's values with their lineage, and check every selected invariant. Required
occurrence is a separate boolean over that same snapshot and permitted witness row set.
Nonempty collections containing only absent inner Optionals do not set it. A present Union
with another variant does not set it. Do not retain booleans across polls.

The terminal nominal site and its own representation must not be conflated. Optional edges
on the route before that site can make it inapplicable. Once the nominal site is actually
reached, evaluate its invariants on its complete value, including a null value if its own
declared representation admits null. Do not skip a Newtype's invariant merely because its
representation unwraps to Optional. For example, Optional<T> null never reaches T, whereas
a required T whose own representation admits null reaches T with that null Node. A genuinely
missing nominal value does not witness an occurrence. Include compiled source controls for
nullable Newtypes to pin the source's admitted distinction and prevent accidental flattening.

Visit rows in returned order, fields in declaration order, list elements in index order and
Map values in UTF-8 key order. Accumulate ordinary unsatisfied results while completing the
snapshot's checks; an Unknown/resource error terminates as an error and cannot be masked by
an earlier False. This makes work accounting independent of short-circuit truth optimization
and prevents an early witness from suppressing checks of later applicable occurrences.

Validate complete root representation including unrelated declared struct members, Optional
presence policy, enum label, exact tagged Union shape, and collection value shape. Unknown
Struct/Union members fail; Map members are arbitrary admitted keys. Union payload property
is the existing `ess_gen::schema::union_content_key(tag)` result, including tag `value`.
Never use the current witness builder's unconditional `UNION_VALUE` (`witness.rs:3571`) as
the wire authority. Newtype alphabets/prefixes retain their declared shape constraints;
predicate invariants on sibling sites remain their own inventoried obligations.

Map key decoding follows its admitted primitive key codec; do not narrow all admitted Maps
to String just because a response validator does so. Traverse values in UTF-8 lexical order
of their actual canonical encoded keys. Detect invalid/colliding canonical keys using that
codec. Source admission and actual runtime fixtures must establish each admitted key family.
There is no fact path through a dynamic key, even if that key is `count`, `0`, a dotted name,
`__proto__`, `toString`, or non-ASCII. No JavaScript prototype lookup is permitted.

Build a local typed FactSource for each selected terminal value. Newtype predicates keep
their original `value` root; Struct predicates keep declared field roots. Preserve exact
numeric tokens and existing primitive semantics; do not coerce Integer/Decimal to Number.
Optional missing/null contributes no fact or presence bit; present aggregates contribute
presence even if empty. Lists bind count and indices 0..n-1. Maps bind count and values at
0..n-1 in the order above. Only typed Struct members bind named field segments. Union
aggregates bind presence; payload traversal comes from the observer, not an invented path
that source predicate validation would reject. Json binds only its admitted presence facts.

Type metadata follows those same synthetic positions: Timestamp orders as an instant;
Duration is not ordered by arbitrary text bytes; other admitted ordered text uses UTF-8
ordering. If an admitted predicate reads current time, sample the runner's target-wall clock
once for the snapshot and use it for all rows/terminals in that snapshot. Do not substitute
the witness builder's reference instant. Existing predicate syntax and exact comparison
semantics are reused; generated predicates gain an optional typed evaluation context while
their legacy untyped entry point retains its behavior. Native uses a local typed FactSource,
not changes to the primitive Predicate enum.

Quantifiers retain lexical semantics (`ess-primitives/predicate.rs:655`): resolve their `over`
in the outer context, then bind their name to one indexed child; nested binders shadow that
name only in their body. Free paths still resolve through the surrounding environment.
Presence, cardinality, primitive metadata and any clock context rebind alongside scalar facts.
Do not copy facts into string keys that capture a free outer path or overwrite a sibling
binder. Empty forall is True and empty exists False; missing cardinality is Unknown.

Only True passes. False and missing required occurrence are ordinary unsatisfied observations.
Malformed actual representation is a contract failure. Unknown and exhausted execution
budget are immediate nonpassing observation errors, not successful skips or retryable False.
EventuallyView may retry ordinary unsatisfied observations within its existing deadline,
but each retry rechecks universal truth and occurrence in the same fresh snapshot. Use the
existing VIEW/EVENTUAL-VIEW/SUITE diagnostic families and include exact dynamic position.

## Arrangement is a separate proof

Inventory before attempting candidates. Reuse `run`/`run_as`, branch selection, caller,
subject/related/external arrangements and actual view filter/parameters. An observation goal
requires an applicable terminal along the selected route and, for recursion, one lineage
crossing the specified edge. A source-selected outcome must satisfy its original guards and
input/type/entity invariants as well as this goal. Never repair the assertion after selecting
an input, borrow another branch's guard truth or read expected assertions into target behavior.

Extend witness search with typed structural goals: present Optional, inhabited nonempty
List/Map, chosen Union variant, and shortest finite recursive route. Compose them at command
input and nested member positions. Repair constrained leaves under all enclosing constraints.
The first-variant-only builder and suppressed inner records at `witness.rs:3564` must change.
Bounded candidate failure is unresolved search, not proof of an empty domain.

Source-state knowledge distinguishes absent, known concrete typed value, guaranteed present
shape with unknown leaves, and unknown. Propagate actual copied inputs/constants/related
facts, clearing and selected source effects. Required generated values can prove presence;
Optional generated values cannot without another source guarantee. Executable source
conversions use their actual modeled result/shape facts; opaque conversions do not establish
presence. Required witness form does not change this proof obligation: even
any_observed_occurrence requires an arrangement licensed to produce an applicable value.

`shows_row`/`bound` (`synthesize.rs:8002,8057`) must prove that resulting row appears under the
actual view/filter/params. Arrangement proves what a healthy target is obliged to make
observable. The runtime separately proves what was actually observed and evaluated. Neither
can substitute for the other. Direct input presence is not final view presence.

Each complete inventory position becomes a scenario or a position-specific refusal:
proved unreachable finite occurrence; unresolved arrangement; unobservable projected value;
or bounded search/resource failure. A field only cleared or a collection forced empty cannot
license a present witness. Explicit refusals remain missing coverage, not completion. Lack of
row identity alone is not a refusal. Existing separately refused Binary64 profiles remain
tracked elsewhere; they cannot be described as supported by this change.

## Resource profiles and incomplete inventory

Separate three different existing limits instead of conflating them:

- Authored TypeRef syntax has depth 32 (`ess-domain/types.rs:162`); it is not the depth of a
  value traversing named recursive declarations.
- Original suite JSON has nesting 128 (`count_json.rs:126`, Go `jsonValue`, browser coverage
  reader). Direct-response expected literals already have a specifically scoped local budget
  for suite28+ (`count_json.rs:Scope`). This is not permission for arbitrary nested keys to
  reset depth. New DTO TypeRefs and paths are flat strings/arrays; keep ordinary outer JSON
  nesting rules and the existing Predicate-depth32 gate. Add 36/37 to the existing direct
  payload profile whitelist so the independent new format does not regress direct responses.
- Existing typed accessor schema authority uses 4096 nodes, 16384 edges and a 1 MiB budget
  (`accessor_types.rs:27–99`). Direct response allows 65536 elements *per collection* and depth
  128 (`selection.rs:575–610`); that does not justify v1's invented 65536 total-node limit.

For this new contract, retain schema bounds of 4096 nominal declarations, 16384 expanded
graph edges, and 1 MiB canonical compact UTF-8 expectation bytes. A single rooted expansion
therefore has at most 16385 nodes; back-edges add edges, not unfolded nodes. These are
explicit bounded authority resources, not model grammar exclusions. Count repeated expanded
acyclic routes; nominal memoization cannot hide their cost. Recursive links cost one edge
and do not unfold schema endlessly. Charge every reachable declaration, even one not selected
for this scenario. Reject contract-resource overflow before any target callback.

Remove both the 65536 total actual-value-node cap and the depth128 alias/observer cap from v1.
Walk actual finite values iteratively, with active value-location detection, under the explicit
work budget below. Do not add new collection cardinality/text-size/type-depth bans to otherwise
admitted view values. Existing transport/parser limits still apply at their own boundaries;
this unit changes none of them and must not report their refusal as conformance success.
Tests separately exercise suite envelope depth, authority size, productive recursion and
execution work. These are four different boundaries.

The work counter is one checked nonnegative integer for one expectation snapshot. Each new
eventually poll starts a fresh counter; polling retains its existing deadline. The following
**logical event stream**, not the number of implementation passes, defines all debits. Fusing
walks, memoizing shape checks, reparsing a value, or reusing a fact buffer cannot change it.
Implementations must be able to expose the same event trace/count in Rust-driven boundary
tests; this is a test seam, not a new target protocol or persisted execution trace.

### Semantic numeric work codec

Work accounting never charges an original numeric token or Number's legacy Serialize spelling.
For an admitted numeric value, first obtain the exact semantic decimal that the current typed
codec assigns it. Integer uses its exact signed value. Decimal uses its exact coefficient and
scale. A finite numeric value inside Json uses the current exact Node numeric semantics, not
its transport spelling. Binary64 remains outside this observer's separately refused profile.
No value is rounded through float merely to compute a debit.

Normalize to signed coefficient C and base10 exponent E: strip coefficient leading zeroes,
then trailing zeroes while increasing E. Every zero, including negative zero, is `0`. Render
without exponent: if E is nonnegative, append E zeroes; otherwise place a decimal point -E
digits from the right, adding `0.` and leading fractional zeroes when needed. A nonzero negative
value has one leading minus. The result has no plus sign, exponent, redundant leading zeroes,
decimal point without fractional digits or trailing fractional zeroes. Its UTF-8 byte length
is `numeric_size`. Compute the length with checked integer arithmetic before allocating the
rendered spelling. Native Number::exact_text supplies this value for the admitted exact profile;
if it contains scientific notation for an admitted finite Node representation, expand it by
the same coefficient/exponent rule rather than using its text length. Go/TS decode their held
semantic value with their existing exact numeric codec, then apply this rule. Neither
json.Number nor JsonNumber.raw is itself the normative spelling.

Thus admitted representations of 1, 1.0 and 1e0 all cost one numeric byte; -0 and 0.0 cost one;
0.0100 and 1e-2 cost four; exact adjacent integers above2^53 keep their distinct full decimal
digits. A declared Decimal represented by canonical decimal text is charged numerically after
its typed codec admits it. An ordinary String containing the same characters is charged as
text. This rule changes no equality, source literal admission or historical serialization.
Actual original-byte suite/payload admission remains a separate check over its actual bytes.
Invalid numeric representations fail the existing typed codec; a work codec cannot repair them.

### Canonical logical schedule

Rows follow returned order. Within a row, use the derived typed graph, declaration field order,
list index order and UTF-8 Map key order. Missing declared members have a logical location.
Within an opaque Json value, use its structural array order and UTF-8 object-key order. Physical
object identity/shared buffers do not merge different logical locations. The schedule is:

**S — shape, once per row root.** In preorder, emit `shape_node(location,type)` costing1 on
every typed entry, including nominal aliases, Optional boundaries and missing members. Emit
`shape_edge(location,edge)` costing1 immediately before each actual child entry. An absent
Optional has no child edge; an empty collection has no element edges. At a Struct, visit every
declared member, including missing ones; its required/policy check occurs at that member's
node. At a Union, visit only its actual selected payload after checking the tag. At a present
Map/List, visit every value. A Json leaf is traversed structurally with one node/edge event
per actual JSON node/child; it has no invented typed member facts.

Each actual object key is charged once in S as `shape_key` with its UTF-8 length, including
Union tag/content keys and Struct keys. Duplicate raw keys were rejected at the applicable
reader; undeclared Struct/Union keys fail shape. At a scalar emit one `shape_scalar` costing
its semantic size: numeric_size for numbers or admitted Decimal text; UTF-8 length for other
text;4/5 for true/false;4 for null. This debit includes the one **logical** scalar parsing pass;
there is no additional parser-dependent token debit. For an object whose key order must be
canonicalized (Map or opaque Json object), emit one `sort_keys` costing
`n + ceil(log2(max(1,n))) * sum(UTF8_key_lengths)`. Other phases never emit this sorting event.
An implementation may reuse this order or recalculate it without changing the debit.

**O — observation traversal, once per row root after S.** Traverse the same actual typed graph
(opaque Json is one terminal here). Emit `observe_node` costing1 on each node and `observe_edge`
costing1 before each child entry, with the same absent/empty rules as S. No scalar/key/parse/sort
debits occur in O. Maintain the recursive lineage state on this logical traversal. Every entry
to the selected nominal site emits one `terminal(location,site)` costing1, regardless of its
witness-row eligibility or whether it is the first witness. For a recursive position the same
terminal is checked universally; edge history decides only occurrence eligibility.

**F — local facts, once for each O terminal, reused by all its invariants.** Immediately after
that terminal event, walk its complete typed value in preorder and emit `fact_node` costing1
per typed entry and `fact_edge` costing1 per child, including aliases/Optional boundaries.
Opaque Json binds only its admitted presence at its own path; no internal fact walk. Derive
the unique logical fact records and emit them sorted by (segment-array path,record-kind),
where kind order is scalar,presence,type. Each scalar record costs1 plus its semantic scalar
size; each presence record costs1; each type-metadata record costs1. Emit one scalar and one
type record for every admitted scalar path, including synthetic collection count (Integer).
Emit one presence and one type record for a present aggregate. Duplicate alias/presence records
at the same path collapse to one record of each kind; metadata must agree with the effective
declared type. Optional absence emits none. This record schedule is normative even if a runtime
stores metadata together with the scalar or retains a lazy view instead of materializing a map.
F has no extra key sorting/scalar parse charges; S covered those logical operations.

**P — predicate preflight, once per declared invariant, in declaration order, against F.**
Emit `predicate_node` costing1 for each AST node. Visit every All/Any child and Not child in
the stored order, independent of short-circuit truth. For Forall/Exists, resolve over in the
outer lexical environment, emit `collection_read` costing1, then `iteration` costing1 for each
actual indexed member and recursively preflight the body in its bound environment. Missing
cardinality has no iterations and later evaluates Unknown. A zero collection has no iterations.
For a nonquantifier leaf, emit one `operand_read` costing1 for each syntactically supplied fact
or literal operand, in serialized order; Defined/Truthy have one, comparisons/text matches have
two, AnyOf/NoneOf/FoldMatch have the path followed by every listed literal. Always/Never have
none. For each resolved scalar operand emit `operand_value` costing its semantic scalar size;
missing operands emit none. Defined uses presence only and emits no operand_value. Truthy uses
its scalar when present. Free and shadowed bindings follow the exact lexical context above.

These P operand-value debits include the logical equality/order/text-scan work and are
**additional** to the S parse and F scalar-record debits, even for cached/literal operands.
There are no other parser/comparison charges. String-derived `.count` emits its numeric
operand_value and additionally `text_count` costing the source string's UTF-8 byte length.
Each occurrence of that read emits that event; physical caching cannot remove it.

Then evaluate that invariant's truth using the existing semantics; evaluation itself emits
no additional events. Reuse F for the next invariant. Finish all selected terminals before
moving to the next row. Accumulate unsatisfied results; immediate Unknown/resource handling
remains as specified above. The complete logical preflight is itself performed incrementally
under the shared counter, not calculated by an unbounded preparatory traversal.

Consume every event's debit before its associated operation; checked overflow or a debit
exceeding work_limit yields resource error. Never reset between S/O/F/P, rows, aliases or
invariants. Default1048576 does not promise that every valid target fits: a larger explicit
budget admits larger finite observations. Exhaustion is not a claim of invalid model/value.
N-1/N/N+1 fixtures must compare exact event traces and totals across native/Go/TS/WASM/browser,
including equivalent numeric representations. Equal far-from-boundary verdicts are insufficient.

Inventory overflow is categorically different from a refused known position. The inventory
builder returns `Incomplete { discovered_positions, frontiers }`, where each frontier records
view/root, current semantic path, active nominal stack/back-edge state, deterministic pending
cursor, exhausted limit and consumed amount. Save/report all queued unexplored routes, or the
unprocessed declaration cursor that losslessly represents them. A frontier stands for an
unknown number of obligations, never exactly one missing position.

Make this condition impossible for the producer to export as a complete generated suite: carry
a private incomplete-inventory marker on synthesized ConformanceSuite, preserved by Clone.
**ConformanceSuite's Serialize implementation itself must refuse incomplete state**, before
serializing any fields. Replace its derived Serialize with a checked implementation using the
historical complete-suite field order. This guards serde_json::to_string, to_vec, to_value,
pretty serializers and non-JSON serializers; merely `serde(skip)` on the marker is insufficient.
Deserialize continues to read standalone assertion documents under existing authority; no valid
serialization of an incomplete synthesized suite may reach that round-trip in the first place.

`admission::suite`, canonical/compact writers, generated emitters, browser bundle creation and
Runner admission also reject the marker before callbacks/output. The complete suite's bytes
remain identical. Synthesis may retain partial scenarios and typed frontier diagnostics for
inspection, but not export them as an admitted partial suite. `coverage_build::finish_inventory`
refuses before extracting/moving fields or producing a known complete inventory.

Audit every repository reconstruction or field-copy path. Checked transformations must accept
the source suite/its completeness state, not only naked provenance and scenario maps. The
mandatory audit list at c2c4 is:

- `synthesize_for` (`synthesize.rs:1961–1984`) currently creates a fresh suite and copies scenarios:
  carry the incomplete marker before any extraction. Component filtering cannot clear it.
- `coverage_build::finish_inventory`, origin merging and retain/filter operations preserve or
  reject the marker before reading counts or rebuilding inventory.
- `coverage::suite_document` and `compact_suite_document` serialize a borrowed-field Document,
  bypassing the suite's Serialize implementation. Keep explicit checked admission before building
  that Document, and make its construction private to those checked routes. Coverage selected/
  parent reconstruction accepts only AdmittedSuite, which cannot contain incomplete state.
- `mutate.rs` synthesis/authoring/analysis paths that take `synthesis.suite`, copy its scenario
  map or publish scenario counts must reject incomplete state before reconstructing/exporting.
- CLI synthesis output and fresh-suite reconstruction/merge paths (`main.rs` near4055,4139,4218)
  must retain the producer state or require checked completeness before copying fields.
- Go/TS emission and browser `emit_product`/bundle building require admitted complete suite
  authority before extracting provenance/scenarios, even when they never call Serialize directly.

Search the final integrated source again for all ConformanceSuite constructors, struct literals,
field copies and borrowed Document serializers; this finite audit list is not permission to
ignore new paths introduced by other merged units. Deliberately assembling a new standalone
assertion from public values remains a distinct authority, not authenticated source synthesis.
No closure-to-IR authorization system is added to prevent intentional standalone authoring.

The CLI prints `inventory incomplete`, the known-position count, `total unknown` and frontiers,
exits nonzero, and writes no success suite/package. Do not add frontier count to known-obligation
totals. The marker needs no persisted suite field or report format. Required negative controls
cover generic Serialize/to_value, raw round-trip attempts, canonical/compact output, clone,
component selection, coverage pretty/compact writers, mutation reconstruction, generated
packages and browser bundle creation. All fail before target callbacks or successful output.

## Admission, encoding and versions

Proposed new pair is ordinary36 / coverage37. This is a proposal, not a reservation. 34/35
remain frozen. Root reports released main supports 1..33 and held runtime supports 1..35.
Old readers must actually reject the new envelope/expectation; preserve a frozen-reader test.
All old parents and direct-only canonical artifacts remain byte-identical. Newly generated
suites select36/37 when carrying the new expectation, wrapped ids or new profile authority;
do not relabel legacy supplied suites. Preserve all existing feature floors in new envelopes.

Canonical expectation encoding is compact UTF-8 JSON with the exact member orders above;
no whitespace or trailing newline for byte/hash measurement. Dictionaries sort keys by UTF-8
bytes. Arrays retain declaration order except the explicitly sorted generated obligation list.
Use [] and {} for empties; omit only declared optional DTO members. Strings use short JSON
escapes for backspace/tab/newline/formfeed/carriage-return, quote and backslash; remaining
controls use lowercase four-digit escapes. No HTML or U+2028/U+2029 escaping, no Unicode
normalization, no unpaired-surrogate acceptance. Numeric predicate/literal spellings follow
the existing exact canonical Predicate/ScenarioValue codecs, never JS reserialization.
Predicate structured-map canonicalization follows that existing codec, with UTF-8 key ordering.

Raw JSON admission precedes map projection and rejects duplicate original keys, unknown/inactive
members, malformed enum labels/types/names and invalid null/empty authority. Reconstruct the
domain type registry, check closure and all nominal references, type-check declared predicates
with their original roots, validate presence on admissible Optional fields, and derive the graph.
Validate position against an actual site/back-edge pair, exact terminal nominal type and nonempty
selected invariants. Check selector/reference dominance and equality with the step query;
`CaptureInstance` or event references must refer to the arranged invocation under existing rules.
Check the id digest. Do all of this before any target callback for raw and in-memory suites.

Every predicate feature visitor must inspect declarations' invariant predicates. Presence,
quoted/text predicates and any associated existing version floors cannot disappear inside the
new object. Validate raw new vocabulary below36 as unsupported even if empty/null. The complete
contract's own validation runs once at admission; snapshot work accounting does not repeatedly
reparse trusted authority. Runtime value traversal uses the admitted contract.

## Exact implementation and integration scope proposed

These paths are concrete proposals for root scope adoption, not permission to edit. Cited
existing seams are identified above; newly named modules/tests are inferred layout decisions.

Core Rust, new module paths fixed by this candidate:

- `crates/verify/ess-conformance/src/value_invariant.rs` — closed DTO, graph, identity,
  schema admission, typed observation and work accounting (new).
- `crates/verify/ess-conformance/src/synthesize/value_invariant.rs` — inventory/arrangement
  goals and frontier handling, extracted from synthesize.rs (new).
- `crates/verify/ess-conformance/src/synthesize.rs` — synthesis result/refusals and integration.
- `crates/verify/ess-conformance/src/witness.rs` — compositional structural goals and candidates.
- `crates/verify/ess-conformance/src/lib.rs` — exports.
- `crates/verify/ess-conformance/src/scenario.rs` — expectation, id, format, internal inventory marker.
- `crates/verify/ess-conformance/src/admission.rs` — raw/in-memory authority and version gates.
- `crates/verify/ess-conformance/src/count_json.rs` —36/37 direct-response payload-profile preservation.
- `crates/verify/ess-conformance/src/runner.rs` — evaluation, reference resolution, snapshot clock.
- `crates/verify/ess-conformance/src/coverage_build.rs` — incomplete inventory refusal and new ids.
- `crates/verify/ess-conformance/src/coverage.rs` — new pair and identity admission/parent preservation.
- `crates/verify/ess-conformance/src/counts.rs` — ensure no complete count after incomplete inventory.
- `crates/verify/ess-conformance/src/mutate.rs` — guard synthesis-derived reconstruction and counts.
- `crates/edge/ess-cli/src/main.rs` — bounded synthesis incomplete diagnostic/exit before output.

Existing embedded Go/TS assets, with Rust-owned generation:

- `crates/verify/ess-conformance/src/go/runtime.go`
- `crates/verify/ess-conformance/src/go/predicate.go`
- `crates/verify/ess-conformance/src/go/mod.rs`
- `crates/verify/ess-conformance/src/ts/runtime.ts`
- `crates/verify/ess-conformance/src/ts/predicate.ts`
- `crates/verify/ess-conformance/src/ts/mod.rs`

The predicate assets need typed-context hooks and lexical metadata propagation, preserving
legacy entry points. No new non-Rust runtime file or standalone executable is proposed.

Existing exhaustive expectation visitors to update if their match/feature scan reaches the new
variant; all are explicit scope rather than an implicit permission to refactor:

- `crates/verify/ess-conformance/src/defined_aggregates.rs`
- `crates/verify/ess-conformance/src/quoted_predicate_format.rs`
- `crates/verify/ess-conformance/src/text_match_format.rs`
- `crates/verify/ess-conformance/src/presence.rs`
- `crates/verify/ess-conformance/src/aggregate_delta.rs`
- `crates/verify/ess-conformance/src/view_paging.rs`
- `crates/verify/ess-conformance/src/now_offset.rs`
- `crates/verify/ess-conformance/src/leaf_payloads.rs`
- `crates/verify/ess-conformance/src/runner/page.rs`
- `crates/verify/ess-conformance/src/authored.rs`
- `crates/verify/ess-conformance/src/fixtures.rs`

Rust tests, with corrected full-feature names replacing the earlier Optional-only proposed paths:

- `crates/verify/ess-conformance/tests/wrapped_value_invariants.rs`
- `crates/verify/ess-conformance/tests/support_wrapped_value_invariants/mod.rs`
- `crates/verify/ess-conformance/tests/support_wrapped_value_invariants/models.rs`
- `crates/verify/ess-conformance/tests/support_wrapped_value_invariants/targets.rs`
- `crates/generate/ess-synth/tests/wrapped_value_invariant_wasm.rs`
- `crates/edge/ess-cli/tests/browser_response_conformance.rs` — reuse existing real CLI/browser
  packaging helpers for the new independent cases; do not create a duplicate test host.
- `crates/edge/ess-cli/tests/fixtures/browser-target/src/lib.rs` — existing Rust concrete
  Installation target, with independently computed wrapped-invariant healthy/mutant behavior.
- `docs/design/wrapped-value-invariant-observations.md` — root-owned binding document if adopted.

Root owns catalog/version fixture regeneration; enumerate exact generated carriers after36/37
reservation. The browser bridge already exists in its owner's product tree and is the required
integration seam; c2c4's old declaration-only replay is not the execution target. The concrete
agreement is:

- `crates/verify/ess-conformance/src/web_execution/bundle.rs`, Loaded::admit: admits original
  ordinary suites or complete coverage inputs before a Loaded handle. Shared new admission
  checks cover malformed/old-major ValueInvariants for selected and parent suites. Its source
  compilation/check_provenance verifies provenance, not every persisted DTO against IR.
- `crates/verify/ess-conformance/src/web_execution.rs`, Engine execution: constructs Installation
  only after Load/nonce checks and calls the shared Runner, then CountReport/CountRun. The new
  Runner expectation needs no new ABI opcode, ABI version, browser evaluator or transport.
- `crates/verify/ess-conformance/src/web_execution/presentation.rs`: renders original scenario
  DTO bytes through its generic closed display family. Declarations/paths/work_limit must remain
  visible, with no step whitelist toggle. Existing frame/presentation/depth limits are separate
  from observer work; their exhaustion remains an explicit refusal, never dropped fields.
- `crates/verify/ess-conformance/src/web.rs`, `web::emit_product`: use the existing product
  emitter, bundle::SourceDocument, Execution::{Ordinary,Coverage}, exact emitted
  `rust/browser_host.rs`, and concrete independent Installation. The incomplete-state audit
  applies before this producer extracts/serializes any suite fields.

These bridge production paths are coordinated audit scope, not a request to change the target
ABI or add another expectation reader. If ordinary shared admission/Serialize checks suffice,
no new browser production code is needed. Test in existing browser_response_conformance.rs and
fixtures/browser-target/src/lib.rs because their packaging helpers are currently private to
that test; do not copy a hand-written host. The separate WASM test checks the shared Rust Runner
but cannot substitute for this actual product browser matrix. No transport or interpreter
command-evaluator changes are authorized.

## Actual acceptance matrix

Compile sources first and preserve refused-source attempts separately. None of the following
is executed evidence yet. Use the same admitted suite bytes and independent healthy/mutant
targets across native, generated Go, generated TS, actual WASM, and full CLI/product browser.
Check exact diagnostic-code sets symmetrically, callback counts and terminal non-skipped results.

1. Retained mixed required/Optional red; both positions become inventoried. Delete only the
   Optional observer and prove the inventory regression still fails.
2. Optional absence alongside actual present witness; all-absent/no-row mutant; zero/false/empty
   text/empty Struct; nested Optional; declared omission/null policies and named wrappers.
3. List bad later element, empty-only, List<Optional<T>> all absent, nested Lists and Optionals.
   Map bad nonfirst value, empty-only, each admitted key family, hostile names, reordered keys,
   and Unicode keys whose UTF-16 order differs from UTF-8. No first-element-only false green.
4. Every invariant-bearing Union variant, Optional/aggregate payload, other-variant-only mutant,
   wrong/missing tag, undeclared member, tag `value` content collision, two paths to same nominal type.
5. Recursive terminating values, ordinary and back-edge witness separation, mutual recursion,
   constrained grandchild corruption, sibling-path false witness, epsilon-cycle termination,
   and independent schema/value/work limits. Schema changes unrelated to a path preserve its id.
6. Quantified Map/List invariants, nested/shadowing binders and free outer reads, typed timestamp
   ordering, exact large integers/decimals, admitted time-relative predicates with fixed clock.
7. Cleared/projected-out present input; guards/related/external selection; known constant/copy,
   generated required versus Optional unknown, source conversions. Prove actual selected outcome.
8. any_observed_occurrence passes legal unidentified background occurrence and labels that claim;
   arranged projected identity rejects background substitution; identity query enforces its exact
   parameter and source uniqueness; neither selector is inferred from a suggestive field name.
9. Eventual polling cannot combine truth and occurrence from separate snapshots; Unknown/resource
   stops without retry. Compare exact logical S/O/F/P event traces and cumulative debits at
   N-1/N/N+1 across runtimes, including equivalent admitted 1/1.0/1e0 and zero spellings,
   Decimal normalization, exact adjacent integers above2^53, multiple invariant lists reusing
   one fact set, and fused-versus-separate physical implementations. Include values above65536
   total nodes under an explicitly sufficient budget, with no artificial node cap.
10. Forged type/site/back-edge/nominal/selector/reference/predicate authority rejected before every
    callback. UTF-8 byte and canonical digest boundaries. Ordinary36/coverage37 parent chains,
    historical bytes and36/37 direct-response depth preservation; actual frozen old-reader refusal.
11. Inventory resource overflow retains exact frontier, reports total unknown, produces no admitted
    suite/known complete coverage inventory, and invokes no target. Test Serialize to_string/to_vec/
    to_value, raw round-trip, Clone/component scoping, borrowed-field coverage pretty/compact writers,
    mutation/CLI reconstruction and Go/TS/browser packaging. Raising the inventory capacity in a
    controlled test reaches the hidden obligation rather than changing the model. Preserve bytes
    for complete historical suites under the new custom Serialize implementation.

All new committed executable implementation and harness code is Rust; existing embedded Go/TS
assets follow the repository exception. Targets own state/results and never consult suite
expectations. No full-feature claim from a fixture that always observes absence, or from merely
listing an unresolved refusal. Scope/adoption and independent design review precede implementation.
