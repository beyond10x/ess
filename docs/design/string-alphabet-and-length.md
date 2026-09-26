# String alphabets, input examples, and `.count` on text and lists

Status: proposed (beyond10x/ess#103, beyond10x/ess#104, `story:count-guards-above-one-are-synthesized`).
This page makes no source, IR or format change. The implementation unit builds exactly what it
says. Every claim about current code cites the tree this page was written against: integration
base `4ffcb537c`, where the maxima are `ess/10`, `ess-conformance/17`, `ess-diff/7` and
`ESS-SYNTH-017`. A line marked *inferred* was not measured.

## Behavior and authority

One consumer specifies a keypad. The keypad sends only `0-9 * # A-D` and refuses a sequence longer
than 64 keys. The specification can state neither rule:

- **#103.** The implementation refuses any character outside the set. The only witness synthesis
  builds for `keys` is the text `"keys"`, so the generated success scenario fails against a
  correct implementation.
- **#104.** The length precondition has no ESS form.

A third gap belongs to the same construct. The 2026-09-25 wave (unit A1, beyond10x/ess#94)
synthesizes a list guard from a list of one element. So `.count > 1` over input or over a stored
list has no witness. The cases that pin this today are
`crates/verify/ess-conformance/tests/stored_field_guards_adversary.rs:275-300` (stored) and
`:410-436` (input). Both assert `ESS-SYNTH-003`.

Measured with the installed `ess 0.32.1` (`ess specify validate`) on #103's keypad model. Only the
line named in each row was changed:

| Written | Result |
|---|---|
| `when: keys.count > 64` | `[unobservable_fact] … \`keys.count\` cannot select \`count\` from \`keypad.dial.KeySequence\` (resolved prefix type \`String\`)` / `error[ESS-COMMAND-003]` |
| `when: keys.length > 64` | the same refusal, naming `length` |
| `alphabet: "0123456789*#ABCD"` on the newtype | `unknown field \`alphabet\`, expected \`of\` or \`invariants\`` |
| `{name: keys, type: …, example: "12#"}` on the input | `unknown field \`example\`` |

The refusal comes from `resolve` (`crates/specify/ess-domain/src/expression.rs:450-532`). A `.count`
selector is admitted only on `Shape::List` / `Shape::Map` (`:492-506`). A scalar with a segment left
over reaches the `cannot select` error (`:517-525`).

The operator decided three things. This page records them and does not reopen them:

1. #103 gets **both** of its options: `alphabet:` on a `String` newtype, which validation,
   invariants and the witness all respect, and `example:` on a command input, which is the witness
   when present.
2. #104: `.count` on `String` and on newtypes of it counts **characters, meaning Unicode scalar
   values**, and is usable in every predicate position. Its witnesses are `N+1` and `N`
   characters, drawn from the alphabet when one is declared.
3. List `.count > N` is witnessed by lists of `N+1` and `N` elements.

## 1. `alphabet:` on a String newtype

```yaml
format: ess/11
types:
  - name: keypad.dial.KeySequence
    kind: newtype
    of: String
    alphabet: "0123456789*#ABCD"
    invariants: [value.count <= 64]
```

**Meaning.** Every character of every value is one of the alphabet's characters. The empty text
satisfies every alphabet. Membership is decided per Unicode scalar value, with no normalization and
no case folding: a composed `é` (U+00E9) is not in an alphabet that holds `e` and U+0301. The
alphabet is a **set** of characters written in a fixed order. The order has one use, the witness
mapping in section 4.

**The alphabet and the invariants are one conjunction.** A value must satisfy the alphabet and
every invariant. The alphabet is not a predicate, and it is deliberately not one. A predicate
would need a per-character quantifier over text, which the grammar does not have. Adding one would
bring back the regular-expression problem the string-operator design refused
(`docs/design/string-predicate-operators.md`, *Out of scope*). A set of characters is the one
character constraint whose legal witness can be written down without a solver: take any member.

**Nested newtypes.** A newtype of a newtype of `String` may declare an alphabet at each layer. The
effective alphabet is the outer alphabet's characters that every inner alphabet also holds, in the
outer alphabet's order. A layer that declares none imposes nothing.

### Parse and representation

- `RawTypeBody::Newtype` (`crates/specify/ess-domain/src/types.rs:836-843`) gains
  `#[serde(default)] alphabet: Option<String>`. `TypeBody::Newtype` (`:591-598`) gains the same
  field, with `skip_serializing_if = "Option::is_none"`. A struct, an enum or a union that writes
  `alphabet:` keeps getting the unknown-field refusal it gets today. That refusal comes from
  `deny_unknown_fields` on the body (`:833`), so it needs no new code.
- `ResolvedBody::Newtype` (`crates/specify/ess-compiler/src/ir.rs:306-311`) gains
  `alphabet: Option<String>` with `skip_serializing_if`, so an IR with no alphabet keeps its bytes
  and its `spec_digest`. `resolve.rs:1247-1256` copies the field through.
- `ResolvedBody` gains `pub fn is_constrained(&self) -> bool`: a newtype or struct with any
  invariant, or a newtype with an alphabet. The sites that today ask "does this type carry
  invariants?" are the class this addition belongs to (section 6). Each of them calls this method
  instead of matching on `invariants`.

### Validation

| Written | Domain code | CLI code | Location |
|---|---|---|---|
| `alphabet:` on a newtype whose `of` does not resolve to `String` through newtypes (a `Uuid`, an enum, a number) | `type_mismatch` | `ESS-TYPE-002` | `types.<T>.alphabet` |
| `alphabet: ""` | `empty_declaration` | `ESS-TYPE-007` | same |
| a character written twice | `duplicate_declaration` | `ESS-TYPE-006` | same. The message names the character and both of its positions |
| nested alphabets that share no character | `conflicting_declaration` | `ESS-TYPE-004` | the outer type's `alphabet` |
| `alphabet:` in a document below `ess/11` | `unsupported_format_version` | `ESS-TYPE-009` | same: "declared alphabets require specification format ess/11" |
| a YAML scalar that is not a string (`alphabet: 123`) | reader error (serde: expected a string) | — | — |

The CLI code is the existing bridge of family and class
(`crates/specify/ess-compiler/src/resolve.rs:155-240`). No new class is added.

Why these rules:

- **Duplicates are refused** so that the written order is the canonical order. Two spellings of one
  set would diff as a change and synthesize different witnesses.
- **An empty intersection is refused** rather than read as "only the empty text". An author who
  writes two alphabets means non-empty values.
- **No length bound on an alphabet.** The witness mapping is one modulo per character, so a
  `1000`-character alphabet costs nothing.

The shape checks run in `NamedType::check_shape` (`types.rs:658-712`). The `of` check and the
nested intersection need the registry, so they run beside `validate_invariants` (`:738-765`). The
format gate sits beside the other type gates in `primitive_admission::system`
(`crates/specify/ess-domain/src/primitive_admission.rs:50-95`).

### Where the alphabet is enforced

| Place | Today, for invariants | The alphabet |
|---|---|---|
| Conformance value admission: `setup_body` (`crates/verify/ess-conformance/src/input.rs:353-376`), behind `validate_typed_value` (`:300-306`), which the witness filter `admitted_inputs` (`crates/verify/ess-conformance/src/witness.rs:398-420`) and authored setup values call | invariants evaluated over `value`, `True` required | checked in the same arm, before the invariants. A character outside the alphabet is `Err("… is not in the alphabet of <T>")` |
| Example validation (section 2) | — | checked |
| Generated Rust and Go types (`crates/generate/ess-synth/src/rust/items.rs:411-423`, `go/items.rs:557-570`) | documented, not enforced ("checking is behaviour, and behaviour is an obligation") | documented the same way: one doc line, "Every character is one of `…`" |
| Entity Runtime nominal rules (`crates/generate/ess-entity-runtime/src/lib.rs:922-945`) | lowered to entity-core rules | **refused**: new `LoweringCode::AlphabetUnsupported`. entity-core has no condition that iterates the characters of a text. Its path walk addresses only arrays and maps (`entity-runtime` `crates/entity-core/src/runtime.rs:2947-2972` at `718a702`) |
| JSON Schema (`crates/generate/ess-gen/src/types.rs:660-663`) | `x-ess-invariants`, an annotation | `x-ess-alphabet: "<alphabet>"`, an annotation. No `pattern`: `Node::pattern` is kept for grammars "small enough to be certainly right" (`:239-242`). An ECMA-262 character class is not certainly right for supplementary-plane characters without the `u` flag |
| schema-contract (`crates/generate/schema-contract/src/realize.rs:520-523`) | `x-ess-invariants` → obligation `model_invariants` | `x-ess-alphabet` → obligation `model_alphabet`. **Without this arm, every model that declares an alphabet fails realization with `unsupported_keyword`** (`:578-581`) |
| View rows (`value_object_invariants`, `crates/verify/ess-conformance/src/synthesize.rs:4434-4492`) | a newtype's invariants are asserted on a view row through `satisfies` | **not asserted on rows.** An alphabet is not a predicate, and a suite has no construct for it. It is enforced where values are built: inputs, the witness and setup |

## 2. `example:` on a command input

```yaml
commands:
  - name: keypad.dial.SendKeys
    input:
      - {name: session_id, type: keypad.dial.SessionId}
      - {name: keys, type: keypad.dial.KeySequence, example: "12#"}
```

**Meaning.** The example is the value synthesis builds this input from. It is not a constraint:
the input still accepts every value of its type. Section 4 says exactly where the example is used.

**Only a command's input takes one.** `Field` (`types.rs:311-322`) is shared by struct, event,
error, entity, view, parameter and response fields. Adding `example` there would parse it in every
one of those positions, and each position would then have to refuse it by hand. One missed position
would be an example silently ignored. So:

- `RawCommandSpec.input` (`crates/specify/ess-domain/src/command.rs:2881`) becomes
  `Vec<InputField>`. `InputField` is a new struct with `Field`'s attributes and serde surface
  (`deserialize_field_name`, `type`, flattened `naming`, `deny_unknown_fields`), plus
  `#[serde(default, skip_serializing_if = "Option::is_none")] example: Option<Node>`.
- `CommandSpec.input` (`:1321`) stays `Vec<Field>`, so every predicate environment built over it
  (`command.rs:1845`, `:1988`, `command/subject_state.rs:228`, `command/subject_fact.rs:308`) is
  unchanged. `CommandSpec` gains `examples: BTreeMap<String, Node>`, keyed by input name.
  The conversion splits each `InputField`. `From<CommandSpec> for RawCommandSpec` merges the
  examples back, so a model round-trips.
- Every other position keeps refusing `example:` as an unknown field, by construction.
- A test asserts that the published schemas of `Field` and `InputField` differ by exactly the
  property `example`. That catches drift between the two mechanically.
- `ResolvedCommand` (`ir.rs:899-918`) gains
  `#[serde(skip_serializing_if = "BTreeMap::is_empty")] examples: BTreeMap<String, Node>`. The IR
  bytes of a command without examples are unchanged.

**Scalar inputs only.** An example is admitted on an input whose type resolves, through newtypes
and `Optional`, to a primitive other than `Binary64`, or to an enum. The witness varies leaves
(`witness.rs:15-33`, rule 3). An example of a struct or a list would fix a whole subtree that no
guard could then vary. The issue's case is scalar.

### Validation

| Written | Domain code | CLI code | Location |
|---|---|---|---|
| `example:` on an input whose type is a list, a map, a struct, a union or `Binary64` | `type_mismatch` | `ESS-COMMAND-002` | `command.<C>.input.<f>.example` |
| a value that is not one of the type: the wrong YAML kind (`example: 0123` over a `String` is the number 123, so quote it), `null`, a name that is not a variant of the enum, a `Uuid`/`Bytes`/`Timestamp` that fails its grammar, a fractional `Integer` | `type_mismatch` | `ESS-COMMAND-002` | same |
| a value that breaks the alphabet, or that some invariant of some newtype layer does not answer `True` for | `conflicting_declaration` | `ESS-COMMAND-004` | same. The message names the layer and the invariant or the character |
| `example:` in a document below `ess/11` | `unsupported_format_version` | `ESS-COMMAND-009` | same: "input examples require specification format ess/11" |
| `example:` anywhere but a command input | reader error, `unknown field \`example\`` (unchanged) | — | — |

**One table for "is this node a value of this primitive".** Conformance already answers that
question in `input::primitive_value` (`crates/verify/ess-conformance/src/input.rs:968-1000`). The
implementation moves that function to `ess-domain`, as `Primitive::admits(&Node) -> Option<FactValue>`
beside `Primitive`, and `ess-conformance` calls it. The alternative, a second copy in the domain
checker, would be a second opinion about whether `1.5` is an `Integer`, which the function's own
comment exists to prevent (`input.rs:962-967`). Invariants are evaluated with the ordinary evaluator
over a `FactStore` holding `value`. `value.count` is decided by the section 3 rule.

## 3. `.count` on `String`

```yaml
when: keys.count > 64                        # a command guard
invariants: [value.count <= 64]              # on a newtype of String
invariants: [name.count >= 1]                # on an entity
filter: forall: {in: tags, as: t, that: t.count <= 8}   # over List<String>
```

**What it counts: Unicode scalar values.** This was decided, and these are the reasons:

- **It is what the evaluator lanes can agree on.** Rust's `str::chars().count()` counts scalar
  values. Go's `utf8.RuneCountInString` counts runes. That is the same number for valid UTF-8,
  which `encoding/json` guarantees by replacing invalid bytes with U+FFFD. TypeScript's `[...s]`
  iterates code points. That is the same number for well-formed strings, and `serde_json` refuses a
  document with a lone surrogate before Rust sees it. Bytes differ by encoding. UTF-16 code units
  (JavaScript's `.length`) count U+1F600 as 2.
- **Graphemes are out.** A grapheme count needs Unicode segmentation tables that differ between the
  lanes' standard libraries and between Unicode versions. That is the same reason case folding is
  out of scope for the string operators.
- **It agrees with the alphabet.** An alphabet is a set of scalar values, so a value made of `N`
  alphabet characters has `.count` `N`. Under graphemes, an alphabet holding U+0301 would break
  that.
- **It agrees with JSON Schema.** `minLength` and `maxLength` count characters as RFC 8259 defines
  them, which are code points (*inferred* from the specification text; this page adds no
  `maxLength` projection).

### Validation

`resolve` (`expression.rs:450-532`) gains one arm beside the collection arm at `:492`. When the
shape is `Shape::Scalar(ScalarKind::Text)`, `environment.is_string(&current)` holds and the segment
is `count`, the arm returns an `Integer` resolution, as `.count` on a list does. It sets a new
`Access::text_length: bool` and leaves `access.collection` false. `is_string` already exists on
both environments: `DomainEnvironment` (`:240-242`) and the compiler's `Environment`
(`crates/specify/ess-compiler/src/expression.rs:40-46`). It is asked of the resolved terminal, so a
newtype of `String` at any depth and an `Optional` of one are admitted. A binder over
`List<String>` resolves through the same walk.

| Written | Code | Message |
|---|---|---|
| `x.count` where `x` resolves to `Uuid`, `Timestamp`, `Duration`, `Bytes`, an enum, a number or a `Boolean` | `unobservable_fact` (unchanged: `cannot select \`count\` …`) | unchanged |
| `keys.count.more` | `unobservable_fact` | `` `keys.count` is a text length of type Integer (Number); `more` selects nothing from it `` |
| `keys.count` in a document below `ess/11` | `unsupported_format_version` | `` `keys.count`: the length of a String requires specification format ess/11 `` |

**The format gate is in the checker, so no position can miss it.** Every predicate position builds
its environment over the one `TypeRegistry` that `Specification::types_with_lifecycles`
(`crates/specify/ess-domain/src/spec.rs:457-466`) returns. That registry gains a
`#[serde(skip)] format: Option<FormatVersion>`, set there from `self.system().format`.
`TypeRegistry` is `serde(transparent)` (`types.rs:1087-1091`), and a skipped field keeps it
transparent. `TypeEnvironment` gains `fn admits_text_length(&self) -> bool { true }`.
`DomainEnvironment` answers `registry.format().map_or(true, |f| f.major() >= 11)`. The IR-level
`Environment` keeps the default. The IR exists only after validation, so it is already admitted. A
syntactic gate in `primitive_admission` could not do this job: whether `x.count` is a text length
depends on `x`'s type.

**Positions.** Because the checker is the one policy, the construct is admitted in every position
the checker serves: command guards, and the input predicate beside `when_subject_state`,
`when_state_changes` and `external`; `when_subject:` predicates; entity, struct and newtype
invariants; view filters; quantifier bodies; and authored `satisfies:`. Two positions refuse it
**after** validation, by existing rules, and this page keeps both:

- **Binding selections.** `resolve_reads` requires a `String`/enum leaf
  (`crates/specify/ess-domain/src/selection.rs:590-603`), and a `.count` read is a number. The
  existing message: "selection predicate requires a declared String/enum item leaf". List `.count`
  gets the same answer today.
- **View-row assertions.** Authored `satisfies:` (`crates/verify/ess-conformance/src/authored.rs:2155-2162`)
  and the invariant witnesses that read a view row (`predicate_projectable`,
  `crates/verify/ess-conformance/src/input.rs:790-802`) both use `projection_target` (`:756-787`).
  That function maps `access.collection` to `Target::Aggregate("a collection")` (`:771-773`). It
  maps the new `access.text_length` to `Target::Aggregate("a text length")`.

**Why a text length never reaches a suite.** It is refused wherever a suite would carry a
predicate: `satisfies`, and a selection plan (`crates/verify/ess-conformance/src/text_match_format.rs:3-6`
names these as the only two carriers). So no suite major moves, and the TypeScript runner and the
browser adapter need nothing. The cost is that an entity or newtype invariant that reads a length
is witnessed on inputs and setup values but not on view rows. It then gets `ESS-SYNTH-011` or
`ESS-SYNTH-013`, as a list-count invariant does today (`website/docs/reference/predicates.md:655-666`).
Lifting both is one later unit: a suite major, a typed suite gate and a Go evaluator case.

**One misattribution is fixed for the whole class.** `unpublished` (`synthesize.rs:4783-4805`) lists
every read that no view resolves to a scalar. So a `.count` read, list or text, is reported as
"published by no view" when the view does publish the field. `InvariantUnobservable` (`:415-427`)
gains `unassertable: Vec<FactPath>`: the reads some view declares and `projection_target` calls a
collection or a text length. `unpublished` keeps only reads no view declares. The hint names
`unassertable` first: "a `.count` is not asserted on a view row in this suite format; the value is
held to it where it is built, on command input and setup". This covers list `.count`, which has the
same defect today. `ValueInvariantUnwitnessed` (`:462-472`) keeps its shape. Its `Display` names a
`.count` read when that is why `at` is `None`.

### Evaluation

The rule is the same in every lane: **a bound fact wins; otherwise, when the last segment is
`count` and the parent path is bound to text, the value is the text's scalar count.** It applies to
leaf reads only. A quantifier's cardinality is untouched, so `forall` over a text stays `Unknown`
in every lane.

| Lane | Where | Change |
|---|---|---|
| Rust (`ess-primitives`, used by validation, synthesis and the Rust runner) | `FactSource` (`crates/specify/ess-primitives/src/facts.rs:1083-1134`) | a provided method `fn observe(&self, path: &FactPath) -> Option<FactValue>` implements the rule over `self.fact`. Every evaluator read uses it: `Operand::resolve` (`predicate.rs:291-297`), the `Truthy`/`Defined`/`AnyOf`/`NoneOf`/`TextMatch` arms (`:617-628`), and the observed/missing lists in the cause walk (`:801-806`). The conformance callers use it too: `InputFacts::resolve` and `explain_leaf` (`input.rs:516-590`, `:656-661`) and the view-filter missing list (`synthesize.rs:3816-3825`). `Element` (`predicate.rs:532-552`) needs nothing, because the provided method calls its rebinding `fact` |
| Go runtime | `src/go/predicate.go` leaf reads at `:604`, `:610`, `:613`, `:665`, `:744` | one `lookup(source, path)` helper implements the rule with `utf8.RuneCountInString`. The cardinality read at `:689` stays a raw map read |
| TypeScript runtime | `src/ts/predicate.ts` leaf reads at `:132`, `:236`, `:243` | one `lookup` helper with `[...text].length`. The cardinality read at `:289` stays raw |

The rule is unconditional in Rust. `ess-primitives` has one non-ESS reader in this workspace,
`infra-spec`. Its predicates may read only the closed `WORKLOAD_FACTS` list
(`crates/infra/infra-spec/src/facts.rs:48-67`, enforced at `raw.rs:504-517`), which holds no path
ending in `.count`, so it cannot observe the change. Outside the workspace, `swarm` depends on
`ess-primitives` (*inferred* from its manifest). For it the change is `Unknown` becoming a number,
and only for an unbound `p.count` over bound text.

Go and TypeScript never receive a suite carrying a text length (above). They change anyway, so that
a forged suite is not read differently by different lanes. That is the property
`crates/verify/ess-conformance/tests/primitive_divergence.rs` exists for.

**One vector table.** `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` gains
`text_lengths`: `{facts, path, expected}`. It is read by `ess-primitives`' tests, by the Go runtime
through `crates/verify/ess-conformance/tests/primitive_corpus.rs` (the `text_matches` pattern at
`:263-350`), and by a new TypeScript case. `typescript_runtime.rs` hands that case the corpus path
in `ESS_PRIMITIVE_VECTORS`, and the case fails when the variable is unset rather than skipping.
Rows, at least:

| Text | Count | Why |
|---|---|---|
| `""` | 0 | empty |
| `abc` | 3 | ASCII |
| `é` (U+00E9) | 1 | 2 bytes |
| `e` + U+0301 | 2 | not a grapheme count |
| U+1F600 | 1 | 4 bytes, 2 UTF-16 units |
| `12#` with `keys.count` also bound to 7 | 7 | a bound fact wins |
| a number at `n`, reading `n.count` | `Unknown` | only text counts |
| an unbound `keys` | `Unknown` | |

## 4. The witness

Rules 1 to 5 (`witness.rs:15-67`) stay. What follows extends rule 2 (the base text) and rule 3 (the
ladders). Every addition is bounded by rule 4's `MAX_CANDIDATES` = 64 (`:106`).

### The base value of a text leaf

`primitive_value` (`witness.rs:1297-1325`) gives a `String` leaf its own path, and `path-n` at a
further `Distinction` (`:1323-1324`). The base is now chosen in this order:

1. **The example**, when the leaf is a command input that declares one and the distinction is
   `PLAIN`. The example is the base for every leaf kind. The ladders are built relative to it:
   `alternatives` excludes the base (`:919-931`), as it does today.
2. **The path text mapped into the effective alphabet**, when the leaf's newtype chain declares
   one. Each character of the path text (or of `path-n`) that is in the alphabet `A` is kept. Every
   other character `c` becomes `A[(c as u32) mod |A|]`.
3. **The path text**, as today.

Examples are used at `PLAIN` only. A further instance falls back to rules 2 and 3, because an
example is one value and rule 5 needs instances that differ. An author whose implementation refuses
more than the type says should declare the constraint (`alphabet:`, an invariant). The type governs
every instance; an example governs one. This is the witness module's own rule: "A witness is only
as good as the type it is built from" (`:69-72`).

**Rule 2 under an alphabet.** Mapping keeps the path's own characters wherever the alphabet allows.
An alphabet that admits the path text leaves the witness unchanged. Two fields of one alphabet type
can map to the same text: `ab` and `ba` under a one-character alphabet both become `xx`. Then a
swapped binding between them is not detectable, and the alphabet is what makes it so. This page
does not try to repair that. A repair would need a second encoding with no better guarantee.

The builder records the effective alphabet of each text leaf in a new
`Builder::alphabets: BTreeMap<FactPath, String>`, written in the newtype arm (`:1206-1215`) beside
`invariants`. It records each declared `String` leaf in a new `Builder::strings: BTreeSet<FactPath>`,
written in the primitive arm. The second set exists because `Leaf::Text` also covers `Uuid`,
`Duration` and `Bytes` (`:1274-1287`), and only a `String` has a length.

### Count lengths: one rule for text and lists

For a read path `p.count` compared with a numeric literal `v`, the lengths tried are `⌊v⌋`,
`⌊v⌋ + 1` and `⌊v⌋ − 1`, in that order. Negative lengths are dropped, and so are duplicates. This
is the number rule `n, n + 1, n − 1` (`:931`), clipped to lengths. For a non-integral `v` such
as `2.5` the rule yields `2, 3, 1`, and `2 < 2.5 < 3` decides both sides. When `p.count` is
compared only with another fact, `v` is `BASE_NUMBER` = 1 (`:1005`), the value every number witness
starts at. That fact's own ladder tries 0 and −1.

Lengths above **`MAX_COUNT_WITNESS` = 1024** are not built. The cap applies to text characters and
to list elements alike. It covers the 64 of the issue and a column limit of 255. A scenario that
carries a 1025-character string costs about a kilobyte.

**Text.** Where `p` is in `builder.strings`, the ladder at `p` gets, after its existing
alternatives, `resize(t, k)` for each length `k` and each `t` taken from the base and then from
every alternative already on that ladder, in ladder order:

- `resize(t, k)` keeps the first `k` scalar values of `t` when `t` has at least `k`.
- Otherwise it appends `t`'s own scalar values, cycled from its start.
- When `t` is empty, it cycles the base from rules 2 and 3, which is never empty.

Every character of `resize(t, k)` is a character of `t` or of that base. So a base inside the
alphabet stays inside it, and a literal-built alternative outside it stays outside it and is
dropped by `admitted_inputs`, as today. Resizing the other alternatives, and not only the base, is
what lets a length guard and a prefix guard on one path be met together. `all: [keys.count > 3,
keys: {starts_with: "#"}]` is met by `resize("#", 4)` = `####`. Truncation and cyclic extension do
not keep a suffix: `all: [keys.count > 3, keys: {ends_with: "#"}]` is met by no candidate. It gets
the existing `ESS-SYNTH-003`, which names the count tried. That bound is stated, not solved.

**Lists.** `Choice::OneElement` (`:383-395`) becomes `Choice::Elements(usize)`, and `OneElement` is
`Elements(1)`. Where a list path `L` is in `builder.lists` and a guard reads `L.count` against a
literal, the ladder at `L` is `Elements(1)`, then `Elements(k)` for each length `k` other than 0
and 1, in order. Length 0 is the base `[]`. For `tags.count > 0` the lengths are `0, 1`, so the
ladder is exactly today's `[OneElement]`, and every suite A1 produced keeps its bytes. `Builder::list`
(`:1126-1160`) builds element 0 at `L.0` as today, varied by its own ladders, and fills elements 1
to `k − 1` with **copies of element 0**. Copies keep a quantifier decided by element 0 alone, as
under A1. `exists t in tags: t == vip` is satisfied by `[vip, vip]` and refuted by
`[tags.0, tags.0]`, and `forall` is decided the same way. Distinct elements would make
`all: [tags.count > 1, forall t in tags: t == vip]` unwitnessable. `MAX_POSITIONAL_ELEMENTS`
(`:999`) is unchanged: it bounds reads by position, not counts.

**Newtype invariants that read a length.** `invariant_ladders` (`:803-837`) builds a ladder for a
leaf whose base its type refuses, from the invariant literals at `value`. It gains the same length
rule for `value.count`. `admits_base` (`:844-859`) also checks the alphabet. For a type with
`value.count >= 8` whose base is `#593`, the ladder tries `resize("#593", 8)` = `#593#593`, which
the type admits.

**Stored fields need nothing more.** A `when_subject:` predicate is translated through the driver's
`sets:` mappings onto the driver's input, and `candidates` is asked over the translation
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:326-365`). So `labels.count > 1`
over a stored list, and `name.count > 64` over a stored string, reach the rules above. The arranged
row is decided through `TypedFacts` (`subject_fact.rs:225`), which reads through `observe`.

**Beyond the cap: `ESS-SYNTH-018`.** A new `RefusalCause::CountUnwitnessed { path, literal, bound }`
takes code 18 (`synthesize.rs:525-550`). It replaces `GuardUnsatisfiable` (`ESS-SYNTH-003`) exactly
when an outcome has no witness and its guard compares some `.count` with a literal whose
`⌊v⌋ + 1` exceeds 1024. The hint: "the guard compares `.count` with a value above the 1024 this
synthesizer builds; cover the branch with an authored scenario (ess-scenario/1), or lower the
bound". The story's third acceptance line is met two ways:

- No count guard at or under the cap is refused any more.
- A guard above it gets this hint, not `NoWitness`'s "drop it from the command's input"
  (`synthesize.rs:556-558`).

### The keypad, worked

`KeySequence` with `alphabet: "0123456789*#ABCD"` (16 characters, indices 0-15). The guard is
`too-long: keys.count > 64`, and `sent` is the default.

| Input | `keys` | Decides |
|---|---|---|
| base | `#593`: `k` = 107 → 107 mod 16 = 11 → `#`, and likewise `e` → `5`, `y` → `9`, `s` → `3` | `sent`. 4 ≤ 64, and every character is sendable, which is #103 |
| length 64 | `#593` repeated 16 times | `sent`, at the boundary |
| length 65 | the same, plus `#` | `too-long` |
| length 63 | the first 63 of those | `sent` |

With `example: "12#"` instead, the base is `12#`, and the 65-character candidate is `12#` repeated
21 times followed by `12`.

## 5. Formats

| Family | Moves? | Why |
|---|---|---|
| `ess/` (authored) | **yes, to `ess/11`**, one number for all three constructs | Each is refused by an older reader: `unknown field alphabet`, `unknown field example`, and `unobservable_fact` for `keys.count`. Each of those errors blames the document for the tool's age. The number turns them into "upgrade the tool". The `alphabet` and `example` gates sit in `primitive_admission` beside the ess/10 gate (`primitive_admission.rs:249-265`). The `.count` gate is in the checker (section 3). |
| `ess-conformance/` (suite) | **no** | No suite carries a text length (section 3). Examples and alphabets change witness **values**, and only for models that declare them, so those are new documents. `select_fresh_format` (`crates/verify/ess-conformance/src/scenario.rs:156-173`) is unchanged. |
| compiled IR | no number exists | `alphabet` and `examples` are skipped when absent. A model that uses none of the three keeps its IR bytes and `spec_digest`. |
| `ess-diff/` | **yes, to `ess-diff/8`** | New vocabulary: `TypeChange::AlphabetChanged` and `CommandChange::InputExampleChanged` (section 6). Without them the residual check (`crates/verify/ess-diff/src/diff.rs:123-128`, `:1860-1915`) reports both as `UnclassifiedChanged`. That is safe, but it says nothing. |
| `ess-scenario/1` | no | An authored scenario writes values, and they are validated through `setup_body`, which gains the alphabet. |
| `infra-spec/1` | no | It cannot reach a `.count` path (section 3). |

**Numbers this unit takes:** `ess/11`, `ess-diff/8`, `ESS-SYNTH-018`. At implementation time, take
the next free ones: `max(SUPPORTED_FORMATS) + 1` (`crates/specify/ess-domain/src/system.rs:53`),
`max(SUPPORTED_DELTA_FORMATS) + 1` (`crates/verify/ess-diff/src/delta.rs:13`), and the next
`RefusalCause` code after `crate::aggregate::UNWITNESSED` = 17
(`crates/verify/ess-conformance/src/aggregate.rs:40-43`).

Tests that use a number above today's maxima to mean "unreadable" move past the new ones. Read each
line before moving it: a line that means "unreadable" moves, and a line that happens to name the
number does not. Today's list:

```console
rg -n --pcre2 'ess/(?:1[1-9]|[2-9][0-9])\b' crates website | grep -v ess-conformance
rg -n --pcre2 'ess-diff/(?:[89]|[1-9][0-9])\b' crates website
```

It finds `crates/specify/ess-domain/src/system.rs:1730` (`format: ess/11` in
`a_specification_reports_every_problem_in_one_run`) and
`crates/verify/ess-diff/tests/canonical.rs:1074` (`ess-diff/8` as a format this build does not
read). The `ess/99` and `ess-diff/99` lines stay.

## 6. Every site that changes

Found by `rg -n 'ResolvedBody::Newtype|TypeBody::Newtype' crates` (tests excluded: 60 sites with
`..`, and 9 that destructure `{ of, invariants }` exactly, which the compiler flags), and
`rg -n 'invariants, \.\.' crates`, which finds the constraint gates listed below. An arm marked
**wildcard** compiles unchanged, so only a test catches it.

| Site | Today | Change |
|---|---|---|
| `ess-domain/src/types.rs` `RawTypeBody`, `TypeBody`, `check_shape`, `validate_invariants` (`:833-852`, `:591-598`, `:658-712`, `:738-765`) | newtype = `of` + `invariants` | `alphabet`, and its validation rows |
| `ess-domain/src/types.rs` `TypeRegistry` (`:1087-1091`) | types only | `#[serde(skip)] format` |
| `ess-domain/src/spec.rs` `types_with_lifecycles` (`:457-466`) | clones the registry | sets its format |
| `ess-domain/src/command.rs` `RawCommandSpec` / `CommandSpec` (`:2876-2900`, `:1317-1335`) | `input: Vec<Field>` | `InputField`, `examples`, round trip, example validation |
| `ess-domain/src/expression.rs` `resolve`, `Access`, `TypeEnvironment` (`:450-532`, `:125-131`, `:77-126`) | `.count` on collections only | the text arm, `text_length`, `admits_text_length` |
| `ess-domain/src/primitive_admission.rs` `system` / `specification` (`:50-95`, `:283`) | gates through ess/10 | the `alphabet` and `example` gates |
| `ess-domain` `Primitive` (new `admits`) | the table lives in `ess-conformance/src/input.rs:968-1000` | moved here; `ess-conformance` calls it |
| `ess-primitives/src/facts.rs` `FactSource` (`:1083`) | `fact` | provided `observe` |
| `ess-primitives/src/predicate.rs` (`:291-297`, `:604-636`, `:801-806`) | `facts.fact(path)` | `facts.observe(path)` |
| `ess-compiler/src/ir.rs` `ResolvedBody::Newtype`, `ResolvedCommand` (`:306-311`, `:899-918`) | — | `alphabet`, `examples`, `is_constrained()` |
| `ess-compiler/src/resolve.rs` (`:1247-1256`, the command resolution) | copies `of`, `invariants` | copies `alphabet` and `examples` |
| `ess-compiler/src/expression.rs` (new) | — | `predicate_sites(ir)`: every predicate the IR holds, with its site and owner fields (type, entity and struct invariants, guards, subject predicates, view filters, selections). A test asserts that its count equals `ess_domain::primitive_admission::predicates(spec)` (`primitive_admission.rs:168-243`) on every model under `examples/` and on one fixture that uses every position |
| `ess-conformance/src/input.rs` `setup_body`, `projection_target`, `resolve`/`explain_leaf` (`:353-376`, `:756-787`, `:516-590`, `:656-661`) | invariants; collection → aggregate | alphabet check; `text_length` → `"a text length"`; `observe` |
| `ess-conformance/src/witness.rs` (`:15-67` doc, `:274-304`, `:383-395`, `:803-859`, `:1126-1160`, `:1206-1215`, `:1297-1325`) | rules as written | section 4. The module doc's rules 2 and 3 say so |
| `ess-conformance/src/synthesize.rs` `RefusalCause` (`:415-427`, `:525-600`), `unpublished` (`:4783`), view-filter missing list (`:3816-3825`) | codes 1-17 | `CountUnwitnessed` = 18; `unassertable`; `observe` |
| constraint gates: `ess-conformance/src/replay.rs:178`, `selection.rs:158`, `response.rs:103`, `synthesize.rs:4442`; `ess-synth/src/selection.rs:123`; `ess-cli-contract/src/resolve.rs:76` | "constrained" means "has invariants" | `is_constrained()`. An alphabet type is refused where an invariant type is refused. `value_object_invariants` (`synthesize.rs:4442`) keeps reading invariants only, because an alphabet is not asserted on rows |
| `ess-conformance/tests/stored_field_guards_adversary.rs:275-300`, `:410-436` | pin `ESS-SYNTH-003` for `.count > 1` | re-pinned, as the story asks: both scenarios exist, and the satisfying input holds two elements. This is the story's second acceptance line |
| `ess-conformance/src/go/predicate.go` (`:604`, `:610`, `:613`, `:665`, `:744`), `ts/predicate.ts` (`:132`, `:236`, `:243`) | raw map reads | `lookup` with the length rule |
| `ess-primitives/tests/vectors/primitive-semantics.json`, `ess-conformance/tests/primitive_corpus.rs`, `tests/typescript_runtime.rs` | `text_matches` read by Rust and Go | `text_lengths` read by Rust, Go and TypeScript |
| `ess-entity-runtime/src/lib.rs` `LoweringCode` (`:391-414`), nominal rules (`:922-945`), `lower` entry | lowers `.count` paths through `$…` references. entity-core resolves `count` on arrays and maps only (`runtime.rs:2955-2958` at `718a702`), so a text length would be `Unknown` for every row | `AlphabetUnsupported` at the newtype arm; `TextLengthUnsupported` for every read that `predicate_sites(ir)` resolves with `text_length`, checked before lowering. Lifting it needs an entity-core length address, which is an entity-runtime change |
| `ess-gen/src/types.rs` (`:283-284`, `:660-663`) and the command-input message | `x-ess-invariants` | `x-ess-alphabet` on the newtype; JSON Schema `examples: [<value>]` on an input property that declares one |
| `ess-gen/src/docs.rs` (`:1310-1328`) | "wraps `String` …" prose | one clause: "its characters are drawn from `…`" |
| `schema-contract/src/realize.rs` (`:509-523`) | refuses unknown keywords (`:578-581`) | `x-ess-alphabet` → obligation `model_alphabet`. `examples` is already an annotation (`:542`) |
| `ess-synth/src/rust/items.rs` (`:26`, `:37-58`), `go/items.rs` (`:151`, `:167-199`) | invariant doc lines | alphabet doc line |
| `ess-synth/src/web/catalog.rs` (`:404-413`) | `invariants` entry | `alphabet` entry when declared. Examples are not catalogued: they are witness inputs, not contract |
| `ess-diff/src/change.rs` `TypeChange`, `CommandChange`, `minimum_format` (`:343-370`) | — | `AlphabetChanged {before, after}` and `InputExampleChanged {field, before, after}`, both at minimum format 8. The alphabet change's relation is **by set membership alone**: `None → Some` is `Narrowed`, `Some → None` is `Expanded`, a strict superset is `Expanded`, a strict subset is `Narrowed`, and anything else, including a reorder, is `Changed`. That is the rule `VariantAdded`/`VariantRemoved` follow (`:873-889`). An example change is `Changed` |
| `ess-diff/src/diff.rs` `body_changes` (`:472-552`), command comparison, `residual_construct` / `residual_command` (`:2069-2075`, `:2139`) | — | compare both; remove `alphabet` and `examples` from the residual |
| `ess-diff/src/delta.rs:13` | `[1..=7]` | adds 8 |
| `schemas/generated/ess.schema.json` | — | regenerated by `cargo xtask schema` (`RawTypeBody`, `InputField`) |
| `website/docs/reference/predicates.md` (`:53`, `:461-495`) | model at `ess/8`. "`.count` on a field that is neither a list nor a map is refused". `when: sku.count > 0` is expected `refused:ESS-COMMAND-003`, and `sku` is a newtype of `String` (`:61-63`) | raise the model to `ess/11`. The sentence says "a list, a map or a String". `sku.count > 0` becomes `ess-expect="synthesizes"`, and the refused example moves to `quantity.count > 0` (`Integer`), which keeps `ESS-COMMAND-003`. The harness is `crates/edge/ess-cli/tests/predicate_reference_page.rs` |
| `website/docs/reference/spec-versions.md`, `website/docs/reference/formats.md` | `ess/` rows to `ess/10`, `ess-diff/` to `/7` | rows for `ess/11` and `ess-diff/8` |
| `docs/design/review-typed-diagnostics.md` inventory block (`:252`) | refusal sites counted per `ess-domain` file; `crates/specify/ess-compiler/tests/design_page_inventory_covers_every_producing_source.rs` and `typed_diagnostics.rs` fail when a count drifts | the new refusal sites in `types.rs`, `command.rs` and `expression.rs` are counted there |
| types and commands reference pages | — | `alphabet:` and `example:`, each with an `ess-check` example |

**Wildcards confirmed to need nothing** (read at this base): `ess-compiler/src/binary64.rs:23`,
`graph.rs:561`; `ess-composition/src/lib.rs:1384`; `ess-service-contract/src/lib.rs:644`;
`ess-gen/src/model_types.rs:112`, `:182` (it reuses `types::body`, so it gets `x-ess-alphabet`);
`ess-diff/src/diff.rs:335`. The other wildcard arms in `ess-conformance` and `ess-synth` read `of`
to walk a representation.

## 7. Conformance strategy and the mutants it kills

Each guard is written red first. Then it is mutation-tested: break the guarded condition, observe
the named failure, restore.

| Mutant | Killed by |
|---|---|
| no text arm in `resolve` | `keys.count > 64` is admitted at `ess/11` in each position: guard, `when_subject:`, newtype `value.count`, struct and entity invariants, view filter, quantifier body over `List<String>`. One case per position |
| the arm admits any text scalar (no `is_string`) | `id.count` refused with `unobservable_fact` for `Uuid`, `Timestamp`, `Duration`, `Bytes` and an enum |
| `keys.count.more` admitted | a case asserting `unobservable_fact` |
| no format gate, or the gate in one position only | an `ess/10` document refused with `unsupported_format_version` in each position above, and for `alphabet:` and `example:` |
| a byte count (`len()`), a UTF-16 count, or a length that overrides a bound fact | the `text_lengths` rows `é`, `e`+U+0301, U+1F600 and "bound wins", in all three lanes |
| the length rule applied to a number, or to cardinality | the `n.count` row; a Rust, Go and TS case that `forall` over `""` stays `Unknown` |
| one evaluator read left on `fact` | an equivalence case per leaf kind, written as an exhaustive `match` with no wildcard: evaluating over `{keys: "abc"}` equals evaluating over `{keys: "abc", keys.count: 3}`, for `Compare` on either side, `Defined`, `Truthy`, `AnyOf`, `NoneOf`, a quantifier body, and the observed list in causes |
| lengths `N` and `N−1` only (no `N+1`) | the keypad suite holds `too-long` with a 65-character `keys` and `sent` with at most 64 |
| a base that ignores the alphabet | #103's reproduction: every character of every `keys` in the suite is in the alphabet, and a `ConformanceTarget` that refuses other characters (the pattern at `stored_field_guards_adversary.rs:86-180`) passes the suite |
| a resize that pads with a constant | the same case, on the 65-character input |
| the example ignored, or used at a further distinction | `sent` carries `keys: "12#"`; an ordered view that arranges two instances carries two different `keys` |
| no alphabet check in `setup_body` | `validate_typed_value` refuses `keyz` for `KeySequence`, and an authored setup value `keyz` is refused |
| list elements built distinct, not copied | `all: [tags.count > 1, forall t in tags: t == vip]` has a witness |
| the list ladder reordered, or 0/1 re-added | a characterization case written first, green on the base: the candidate lists for `tags.count > 0`, `exists t in tags: t == vip` and `tags.0 == vip` are byte-equal before and after |
| the cap removed, or `018` not substituted | `keys.count > 5000` refused with `ESS-SYNTH-018` and its hint; `keys.count > 1023` witnessed |
| stored lists not covered | the two re-pinned adversary cases (section 6) |
| a validation row missing | one case per row of the two validation tables, asserting code and location |
| `Field` and `InputField` drift | the schema-difference case (section 2) |
| a missed `is_constrained` site | a case per site with an alphabet-only newtype, asserting the same refusal an invariant-carrying newtype gets there |
| `x-ess-alphabet` without its schema-contract arm | a realization of a model with an alphabet reports the `model_alphabet` obligation and no `unsupported_keyword` |
| Entity Runtime lowers either construct | `AlphabetUnsupported` and `TextLengthUnsupported`, each asserted by variant |
| `predicate_sites` misses a position | the parity case (section 6) |
| the diff reports `UnclassifiedChanged` or the wrong relation | one case per relation row, and `minimum_format() == 8` |
| IR bytes moved for an unaffected model | the existing IR and `spec_digest` cases under `crates/specify/ess-compiler/tests/`, unchanged and green |

## 8. Out of scope

- **Regular expressions and `pattern:`.** They bring the witness and dialect problems the string
  operators refused. An alphabet is the solver-free subset.
- **Grapheme counts, normalization, case folding.** See section 3.
- **Asserting a length or an alphabet on a view row.** This needs a suite major and a typed suite
  gate. Lists share the gap (section 3).
- **An entity-core length address.** It is an entity-runtime change. Until it lands, lowering
  refuses the construct by name.
- **Examples on aggregate inputs, and example lists.** One scalar value per input.
- **Projecting `value.count` invariants as `maxLength`.** Invariants stay annotations in every
  schema this repository publishes.
- **Deriving an alphabet from an imported OpenAPI `pattern`.** Imports never guess (`AGENTS.md`,
  *Boundaries*).
- **The agent plugin's grammar page** (`agentplugins`). It is updated after the release.

## What the implementor checks before building

Each of these was not read at this base, or rests on something outside it. Confirm each with one
read or one run:

1. `serde(transparent)` accepts `TypeRegistry` with a second, `#[serde(skip)]` field (`cargo check
   -p ess-domain`). If it does not, the format moves to a wrapper that `DomainEnvironment` holds,
   and every environment is still built from it.
2. `InputField` with `deny_unknown_fields` and a flattened `Naming` refuses an unknown key exactly
   as `Field` does. Serde's documented incompatibility between `flatten` and `deny_unknown_fields`
   already applies to `Field`, so mirror its attributes exactly. The schema-difference case decides
   it.
3. `Refusal` bodies are rendered and never persisted, so adding `unassertable` moves no stored
   bytes. Run `rg -n 'InvariantUnobservable' crates --glob '!**/tests/**'`: at this base it finds
   only `synthesize.rs`, `coverage_build.rs:399` (a variant match) and the parked
   `consumer_coverage/entry-classifications.json:16235`, which is keyed by variant name. Check that
   the key does not include field names.
4. Go's and TypeScript's leaf reads are exactly the ones listed in section 3 (`rg -n 'source\['
   src/go/predicate.go`, `rg -n 'source\.get\(' src/ts/predicate.ts`).
5. The command-input message in `ess-gen` builds its properties through one function that can take
   the command's examples. That was *inferred* from `COMMAND_INPUT` (`crates/generate/ess-gen/src/types.rs:760`, used at `:806`).
