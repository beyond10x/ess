# String guards: prefix, suffix and substring

Status: proposed (beyond10x/ess#95, `story:string-prefix-suffix-substring-operators`). No source,
IR or format change is made by this page. Unit B1b implements what it says, after unit A1
(beyond10x/ess#93, #94) has merged. Every claim about current code cites the tree this page was
written against (integration base `bb7718889`). A line marked *inferred* was not measured.

## Behavior and authority

A trial specified a telephony backend whose routing rules are written in a small condition
language. Three of its rules have no ESS form: "caller number starts with +44", "dialled number ends
with 0", "subject contains 'urgent'". Each is recorded as `UNMAPPED:` today, and no conformance
scenario checks it.

Measured with `ess 0.32.1` (`ess specify validate`) on the model of the predicate reference page,
with the `placed` guard replaced:

| Guard written | Result |
|---|---|
| `when: {sku: {starts_with: "+44"}}` | reader error, no code: `cannot parse predicate "sku: {starts_with: …}": unknown operator "starts_with"; expected one of eq, ne, lt, lte, gt, gte, any_of, none_of, exists, truthy` (`crates/specify/ess-primitives/src/predicate.rs:1060-1066`) |
| `when: {sku: {eq: "+44"}}` | `type_mismatch`: `` `sku == 44`: operator `==` does not admit `shop.order.Sku` (Text) and `Number literal `44`` (Number) `` |

The second row matters to this design. YAML's double quotes are gone by the time the predicate
reader sees the text, and a map-form comparison operand is read through `Operand::parse`
(`predicate.rs:1016`, `:259-268`), which reads any text that parses as a number as a number
(`FactValue::parse_literal`, `crates/specify/ess-primitives/src/facts.rs:695-712`). A quoted
`"+44"` is therefore the number 44 under `eq:`. A string operator read that way could never take a
phone prefix.

## Syntax

Three operators, **map form only**, each with one operand:

```yaml
when:
  all:
    - caller: {starts_with: "+44"}
    - not: {subject: {contains: "spam"}}
    - dialled: {ends_with: "0"}
```

- **No compact form.** `caller starts_with "+44"` is not an expression. It stays the reader error it
  is today (`predicate.rs:1134-1142`: a bare string that is not a comparison must be a fact path).
  The compact grammar stays one comparison. This is the same reasoning that gave quantifiers no
  string form (`predicate.rs:61-64`).
- **Negation is `not:`** (or `none:`). There is no `not_starts_with`, no alias and no second
  spelling of any of the three keys.
- **Several operators in one mapping are conjoined.** `sku: {starts_with: "A", ends_with: "0"}` is
  `all` of the two, as every operator mapping already is (`from_constraint`,
  `predicate.rs:998-1003`).
- **The operand is text, read verbatim.** A YAML string becomes the literal byte for byte. It is not
  read through `Operand::parse`, so it is never a number, never a Boolean and never a fact path, and
  surrounding quote characters are part of the literal (`{starts_with: '"x"'}` tests for a leading
  `"`). There is no compact form to delimit, so there is no quoting layer to undo. `"+44"`,
  `"0"`, `"a.b"` and `"true"` are the texts they spell.
- **The left-hand side is a fact path; the operand is a literal.** No fact-to-fact form
  (`{starts_with: input.prefix}` tests for the text `input.prefix`). Witness rule 3 draws its
  candidates from the literals a guard writes (`crates/verify/ess-conformance/src/witness.rs:24-27`),
  and a fact operand gives it none.
- **A literal that names a declared field is refused, as for `==`.** `{starts_with: gift}`, where
  `gift` is a field of the same owner, is refused with `type_mismatch` (`ESS-COMMAND-002` in the
  CLI rendering), exactly as `text_literal` refuses `ends_at > starts_at` today
  (`crates/specify/ess-domain/src/expression.rs:703-739`, beyond10x/ess#74). Quoting does not
  bypass it, just as it does not for equality today. Measured on `ess 0.32.1`:
  `sku: {eq: '"quantity"'}` is refused with `type_mismatch` / `ESS-COMMAND-002`, "reads `quantity`
  as the text literal "quantity", not the field `quantity`". The message says that a string operator compares with a literal only. It
  does not suggest the `window.a op window.b` spelling, which has no string-operator form. An enum
  variant spelled like a field is not affected, because string operators never apply to enums.
- **A YAML scalar that is not a string** (`{starts_with: +44}` unquoted is the number 44,
  `{starts_with: true}`) is parsed and kept as that value, and **refused at validate** with
  `type_mismatch` (below). entity-core refuses the same literal at registration
  (`entity-runtime` `crates/entity-core/src/validation.rs:1656-1672` at `718a702`), and the refusal
  gets a code and a site rather than a reader error. A list, a mapping or `null` as the operand is a
  reader error (`ParseError::predicate`, "a comparison operand must be a scalar", which is how
  `from_operator` already answers a non-scalar comparison operand, `predicate.rs:1019-1024`).

### The parsed value

One variant, so every exhaustive walk gains one arm, not three:

```rust
Predicate::TextMatch { path: FactPath, op: TextOp, value: FactValue }
enum TextOp { StartsWith, EndsWith, Contains }   // keyword(): "starts_with" | "ends_with" | "contains"
```

- **Parse.** `from_operator` (`predicate.rs:1013-1067`) gains the three keys: `Node::Text(t)` →
  `FactValue::Text(t)` verbatim; `Node::Number`/`Node::Bool` → that value; anything else → reader
  error. The `unknown operator` message's list gains the three names.
- **Render.** `to_node` (`predicate.rs:1196-1258`) gains an explicit arm writing
  `{path: {<keyword>: <value>}}`, with a text value as `Node::Text` and no added quotes.
  **This arm is not compiler-enforced:** `to_node` ends in `leaf => Node::Text(leaf.to_string())`
  (`:1256`), which would render the new leaf as a compact string that no reader parses back. A
  round-trip case for `"+44"`, `"0"`, `"a.b"`, `"true"`, `'"q"'` and a number value decides it.
- **Display** (`predicate.rs:1431-1447`): `caller starts_with "+44"`, with a text value rendered
  quoted by Rust `Debug` and a number or Boolean value bare. Display is what the semantic diff
  compares and prints (`crates/verify/ess-diff/src/diff.rs:920-935`), what `ess-gen` publishes as a
  structured invariant's statement, and it is never read back. `Invariant::serialize` already falls
  back to the structured form when the rendered text does not reparse
  (`crates/specify/ess-domain/src/entity.rs:536-548`).
- **Walks.** `visit_free_paths` reads `path` (`predicate.rs:732-770`), so `fact_paths`, the
  checker, `read_paths` in the witness and infra-spec's projection check all see it.
  `visit_quantified` treats it as a leaf.

## Semantics

With `v` the value the fact source holds at `path` and `L` the literal:

| `v` | `starts_with: L` | `ends_with: L` | `contains: L` |
|---|---|---|---|
| unobserved, or `null` | `Unknown` | `Unknown` | `Unknown` |
| text | `v` begins with the bytes of `L` | `v` ends with the bytes of `L` | the bytes of `L` occur in `v` |
| a number or Boolean | `False` | `False` | `False` |

- **Byte-wise and case-sensitive.** No case folding, no Unicode normalisation: a decomposed
  `e` + U+0301 does not begin with a composed U+00E9, and `ABC` does not begin with `a`.
- **Empty `L`** holds for every text, which is what every lane's standard library answers. It is
  **refused at validate** (below), so a validated model never carries one, but the evaluators still
  define it.
- **`not:`** is Kleene, as for every leaf: `not Unknown` is `Unknown` (`predicate.rs:120-126`). An
  unobserved field selects neither a guarded branch nor its negation.
- **A non-text literal** answers `False` whenever `v` is observed, and `Unknown` otherwise. Only an
  unchecked caller of `ess-primitives` can reach that row; validation refuses the literal first.

The same table in every place that evaluates a predicate:

| Lane | How | Why it is byte-wise |
|---|---|---|
| Rust (`ess-primitives`, used by synthesis and the Rust runner) | `v.as_bytes().starts_with(L.as_bytes())`, `ends_with`, `v.contains(L)` | `str::contains(&str)` matches on bytes. A valid-UTF-8 needle can only match a valid-UTF-8 haystack at a character boundary, so byte and character matching agree. |
| Go runner (`src/go/predicate.go`) | `strings.HasPrefix`, `strings.HasSuffix`, `strings.Contains` | Go strings are the UTF-8 bytes `encoding/json` decoded. |
| TypeScript runner (`src/ts/`) | **not an evaluator in this unit.** It admits suites only up to `ess-conformance/11` (`SUITE_MAJORS`, `src/ts/runtime.ts:4203-4215`), so it refuses a suite carrying the operators by its version, with `unsupported suite version "ess-conformance/<N>"` (`:4231-4234`). A forged `/11` suite carrying one is refused by `admitPredicateConstraint` with `unknown predicate constraint operator` (`:4979`). B1b adds a case for each refusal. | — (see *Out of scope*: TypeScript suite /12+) |
| entity-core (`entity-runtime` at `718a702`) | `as_bytes().starts_with`, `as_bytes().ends_with` (`crates/entity-core/src/runtime.rs:2170-2206`); `contains` on two strings is `str::contains` (`:2146-2166`) | Same as Rust. |

**Null.** The Rust runner does not bind a `null` row value (`crates/verify/ess-conformance/src/runner.rs:2707`),
so it is unobserved. entity-core reads a reference that is present and null as unobserved
(`resolve_operand`, `runtime.rs:2601-2617` at `718a702`). The Go flattener does bind it
(`src/go/predicate.go:79-95`, `default: into[path] = value`), so the Go lane answers `Unknown` for
these three operators when the bound value is `nil`, not `False`. One shared vector table decides
this and the rest of the semantics table. It holds, at least: match, non-match, empty `L`, `L`
longer than `v`, a case difference, a composed against a decomposed `é`, a supplementary-plane
character (U+1F600) in both `v` and `L`, a number value, `null` and an unobserved path. The table is
read by the `ess-primitives` tests and, through the Go runtime, by an `ess-conformance` test. The
precedent is `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`. It is read by
Rust (`crates/specify/ess-primitives/tests/primitive_corpus.rs`,
`crates/verify/ess-conformance/tests/primitive_corpus.rs`), by the Go runtime and by the browser
adapter (`crates/verify/ess-conformance/tests/primitive_corpus.rs:1-8`). It has **no TypeScript
reader**, and this table does not add one, because TypeScript is not an evaluator lane for these
operators (see *Out of scope*). The browser adapter does not evaluate predicates, so it does not read this table
either.

## Validation

### Where the operators apply

The expression checker (`crates/specify/ess-domain/src/expression.rs:796-897`) admits a string
operator when the path resolves to **`String`, or a newtype of `String` at any depth**, including
through `Optional`. `resolve` already unwraps newtypes and optionals before it reports the terminal
type (`expression.rs:436-444`). A path under a quantifier binder whose element is `String` is
admitted the same way, because binders resolve through the same walk.

Neither `ScalarKind` nor `ValueType` says `String` today. `ScalarKind::Text` covers `String`,
`Timestamp`, `Duration`, `Uuid`, `Bytes` and enums (`expression.rs:28-38`, `:449`). So
`TypeEnvironment` gains `is_string(&self, reference) -> bool`, defaulting to `false` beside
`is_instant` (`expression.rs:92`). It is implemented on the resolved terminal in both environments:
`DomainEnvironment` (`expression.rs:217`) and the compiler's `Environment`
(`crates/specify/ess-compiler/src/expression.rs:24`). `ValueType` (`expression.rs:543-557`) gains a
`string` flag filled in `operand` (`:591-614`).

Because the checker is the one policy for every predicate position, the rule is the same in every
one: command guards, the input predicate beside `when_subject_state`, `when_state_changes` and
`external`, entity and type invariants, view filters, binding selections, authored-scenario
`satisfies:` (`crates/verify/ess-conformance/src/authored.rs:2135-2140`), and later #75's subject
predicate.

### What it refuses

| Written | Code | Message says |
|---|---|---|
| a path that resolves to anything but `String` or a newtype of it: an enum, `Uuid`, `Timestamp`, `Duration`, `Bytes`, a number, a `Boolean`, an aggregate | `type_mismatch`, through `mismatch` (`expression.rs:616-660`), whose `unreadable` match (`:636-652`) gains the new variant | `` `x starts_with "A"`: operator `starts_with` does not admit `shop.order.Channel` (Text) ``. This is the existing `mismatch` format: it names the declared type and prints its *scalar kind*, and an enum's scalar kind is `Text` (`expression.rs:449`, `:623-630`). So an enum field reads `shop.order.Channel` (Text). B1b keeps that format. The declared name is what tells the reader it is an enum, and `any_of` already tests an enum's finite domain. |
| a number or Boolean operand | `type_mismatch` | the operand is not text; an unquoted `+44` is the number 44 in YAML; quote it. entity-core's wording is `validation.rs:1666-1670` at `718a702`. |
| an empty text operand | `empty_declaration` (`crates/specify/ess-primitives/src/error.rs:359-361`) | `` `x starts_with ""` `` holds for every text, and its negation for none; write `defined(x)` if presence is meant |
| a path the owner does not declare | `unobservable_fact` / `undeclared_reference`, from `resolve` as today | unchanged |
| any of the three in an `ess/N` document below the version this construct takes | `unsupported_format_version` | `string predicate operators require specification format ess/N` (see *Formats*) |

The empty literal is refused rather than admitted because one of its two branches has no witness.
`x: {starts_with: ""}` is `defined(x)` spelled so that synthesis cannot see it, and `not` of it is
unsatisfiable. entity-core accepts it (the empty string is a prefix of every string). The ESS
lowering never emits one, because validation refuses it first.

### Finite coverage: an open domain

`finite::input_paths` admits only `==`/`!=` and membership against text literals, and declines
every other leaf through its wildcard arm (`crates/specify/ess-domain/src/command/finite.rs:67`).
A guard that uses a string operator is therefore outside the finite proof with no code change.
`finite_coverage` answers `None` (`crates/specify/ess-domain/src/command.rs:1908-1913`), and a
command whose outcomes are all conditional gets `non_exhaustive_branches`, "every outcome … is
conditional" (`command.rs:1873-1879`). That is the answer `amount > 0` / `amount <= 0` already gets:
the author declares a genuine default. `caller: {starts_with: "+44"}` beside
`not: {caller: {starts_with: "+44"}}` is complementary, and it is still refused without a default,
because the prover does not reason about text. B1b adds a case asserting that refusal. It does not
change `finite.rs`.

## Witness

Rules 1, 2, 4 and 5 are unchanged (`witness.rs:15-33`). Rule 3 — the values tried are the literals
the guard writes, one either side — is extended for text in the issue's shape. For a path `p` whose
base witness is its own text `b` (`"caller"`, or `"caller-1"` for a further instance;
`witness.rs:660-665`), each string-operator literal `L` read at `p` adds:

| Candidate | Built as | Decides |
|---|---|---|
| `L` | the literal itself | satisfies its own operator: `L` begins with, ends with and contains `L` |
| composed | per guard, three layouts (below) of that guard's positive literals around `b` | satisfies every positive string operator of the guard at once. Two of the three layouts keep rule 2, because the path's own text is inside them. |
| `L′` | `L` with one character replaced by `x`, or by `y` if it is `x`: the **last** character for `starts_with`, the **first** for `ends_with`, and the middle one (index ⌊n/2⌋ of its n characters) for `contains` | refutes its own operator. `L′` is no longer than `L` in bytes and differs from it, so `L` is not a prefix, suffix or substring of it. The position is chosen so that `L′` keeps the rest of `L`: a newtype invariant written on the same affix — `starts_with: "+"` under `starts_with: "+44"` — still holds for it. |
| invariant-composed | for a path whose declared type is a newtype carrying invariants (at any depth), the same three layouts built from each invariant's positive literals around `b` | a value the newtype admits that the guard's literals did not build, so a refuting candidate survives `admitted_inputs` |
| `b` | the base witness, tried first as always | refutes unless `b` itself begins with, ends with or contains `L` |

**The composed layouts.** For one guard (or one newtype invariant) and path `p`:

- `P` is the first `starts_with` literal that occurs positively, meaning under an even number of
  `not`/`none`. A literal that the same guard also negates as a `starts_with` is never taken.
- `S` is the same for `ends_with`.
- `C` is the positive `contains` literals in written order. A literal that the guard also negates
  as a `contains` is left out.

The candidates are then:

- (a) `P + C + b + S`
- (b) `P + b + C + S`
- (c) `P + C + S`, without the path text, tried only when it is not empty

`push` drops duplicates.

This is the issue's table: `L`, `L` + the path's own text and the path's own text + `L` on the
satisfying side; the path's own text and `L` with one character changed on the refuting side. The
additions:

- **`L′` for `contains`.** The issue gives `contains` only the path's own text as a refuting
  candidate, and that candidate fails exactly when `b` contains `L` (`subject` with
  `contains: "sub"`). `L′` refutes every `L`.
- **Compositions per guard, from positive occurrences only, in three layouts.** `candidates` is
  called with every outcome's guard of the command
  (`crates/verify/ess-conformance/src/synthesize.rs:2526`), so composing per guard keeps two
  branches' different prefixes apart. Worked cases:
  - The reference page's own example,
    `all: [sku: {starts_with: "SKU-"}, not: {sku: {contains: "test"}}, sku: {ends_with: "0"}]`
    (`website/docs/reference/predicates.md` on `impl/a2-predicate-reference-page`). No single `L`
    satisfies it. Layout (a) `SKU-sku0` does. A composition that took the negated `test` would not.
  - `all: [subject: {contains: "RE"}, not: {subject: {starts_with: "RE"}}]`. `P` and `S` are empty
    and `C` = `[RE]`.
    - (a) `REsubject` begins with `RE`, so it is refuted.
    - (b) `subjectRE` contains `RE` and does not begin with it, so it **satisfies** the guard.
    - (c) `RE` is refuted.

    The branch without the guard is witnessed by `b` = `subject`. With layout (a) alone, which was
    the earlier rule, this guard had no witness.
- **Candidates from the type's own invariants.** Two worked cases. Each has a default branch beside
  the guarded one, and both branches are now witnessed:

  | Path type and invariant | Guard | Candidates | Admitted by the invariant | Satisfies the guard | Refutes the guard |
  |---|---|---|---|---|---|
  | `PhoneNumber`, `value: {starts_with: "+"}` | `caller: {starts_with: "+44"}` | `caller`, `+44`, `+44caller`, `+4x`, `+caller`, `+` | `+44`, `+44caller`, `+4x`, `+caller`, `+` | `+44`, `+44caller` | `+4x` (L′), `+caller`, `+` (invariant-composed) |
  | `Email`, `value: {contains: "@"}` | `email: {ends_with: "@corp.example"}` | `email`, `@corp.example`, `email@corp.example`, `xcorp.example`, `@email`, `email@`, `@` | `@corp.example`, `email@corp.example`, `@email`, `email@`, `@` | `@corp.example`, `email@corp.example` | `@email`, `email@`, `@` (invariant-composed only) |

  In the first row the old rule, which replaced the first character (`x44`), dropped every
  refuting candidate: `caller` and `x44` both fail the invariant. That left the trial's motivating
  guard without a default witness. In the second row `L′` = `xcorp.example` loses the `@`, and only
  the invariant-composed candidates refute. For `email: {contains: "@corp"}` under the same
  invariant, `L′` = `@cxrp` (the middle character replaced) keeps the `@` and refutes.

**Where it goes.** `collect_literals` (`witness.rs:337-367`) gains an arm that pushes `L`, so the
existing `Leaf::Text` arm of `alternatives` (`witness.rs:406-412`) tries it as it tries any text
literal. A new `text_matches_at(guards, path)` returns `(op, L, positive, negated-in-guard)` per
guard. The newtype arm of `Builder::value` (`witness.rs:557-559`, `ResolvedBody::Newtype { of, .. }`)
records the newtype's invariants at the leaf path, read against the `value` pseudo-field
(`crates/specify/ess-domain/src/types.rs:644-649`), for the invariant-composed candidates.
`alternatives` appends, after the literals: the guard compositions, then the invariant
compositions, then each `L′`. `push` already deduplicates and skips the base
(`witness.rs:381-386`). `ordered_at` (`witness.rs:314-335`) is unchanged: a string operator is not
an ordering.

**Rule 4.** Each distinct literal adds at most two alternatives (`L`, `L′`). Each guard and each
newtype invariant adds at most three composed candidates per path. The product is still capped at
`MAX_CANDIDATES` = 64 (`witness.rs:286-294`). Exhausting it is the existing refusal naming the
count. The mixed-radix walk (`witness.rs:247-260`) visits combinations in index order and stops at
the cap. In a command that varies several paths, a later alternative of one path — an `L′` or a
composition — can therefore fall outside the 64 tried. This case is constructed, not observed, and
it is rule 4's bound. The outcome is the existing refusal naming the count, never a wrong witness.

**The finite fast path.** `candidates` first tries `finite::analyze` when no outcome is a default
branch (`witness.rs:191-222`). A guard using a string operator makes it decline (above), so these
guards always reach the ladder.

**Type invariants still filter.** Candidates that a declared newtype invariant refuses are dropped
by `admitted_inputs` (`witness.rs:266-281`), as today. The rows above show which candidates
survive.

**Interaction with A1** (*inferred*: A1 has not merged, and this reads its brief and stories only):

- *Byte-wise text ordering.* A path that is both ordered and string-matched reaches the same
  `Leaf::Text` ladder. If A1 generates ordering neighbours for every literal at an ordered path, the
  string literal gets them too. That is bounded by rule 4 and harmless. B1b does not change which
  literals A1 orders around. It only adds `L` to `literals_at` and appends its own candidates after
  A1's.
- *One-element input lists.* A1 builds a list from the guard's own literal for `exists`/`forall`
  over an input list. With `L` in `collect_literals`,
  `exists: {in: tags, as: t, that: {t: {starts_with: "vip"}}}` gets `[vip]` as its satisfying list
  and `[]` as its refuting one, with no code specific to this construct. B1b adds a case for it
  only if A1's list builder reads `literals_at`. If it does not, the case records the gap.
- *Optional omission.* A1 omits an `Optional` path only for presence guards. An `Optional<String>`
  under a string operator is filled by rule 1, and its omitted form is `Unknown`, which selects
  nothing. No interaction.

**Why an undecidable leaf is named.** `InputFacts::explain_leaf`
(`crates/verify/ess-conformance/src/input.rs:516-590`) is exhaustive over leaves. The new arm
reports `explain_path(path)`, because a string-operator leaf is `Unknown` only when its path is
unbound. The literal always resolves, and a resolved non-text value is `False`.

## Every site that changes

Found by `rg -n 'Predicate::(Truthy|Defined|AnyOf|NoneOf)'` over `crates/`, tests excluded (Rust
matches on predicate leaves: 10 files). Then `rg -n 'none_of'` over the Go, TypeScript and JavaScript sources
under `crates/verify/ess-conformance/`, which the compiler does not check. An arm marked **wildcard**
compiles without a change, so only a test catches it.

| Site | Today | Change |
|---|---|---|
| `crates/specify/ess-primitives/src/predicate.rs` | `Predicate` (`:327`), `from_operator` (`:1013`), `evaluate` (`:516`), `visit_free_paths` (`:732`), `visit_quantified` (`:785`), `to_node` (`:1196`, **wildcard** `:1256`), `Display` (`:1431`), schema description (`:1523-1528`) | variant, parse, evaluate, walks, render, display; the schema description names the three keys |
| `crates/specify/ess-domain/src/expression.rs` | checker `predicate` (`:796`), `mismatch` (**wildcard** `:651`), `TypeEnvironment` (`:77`) | `is_string`, the refusals above |
| `crates/specify/ess-compiler/src/expression.rs` | `Environment` (`:23-30`) | `is_string` |
| `crates/specify/ess-domain/src/command/finite.rs` | **wildcard** `:67` declines | none; a case asserts the decline and the `non_exhaustive_branches` answer |
| `crates/specify/ess-domain/src/selection.rs` | `predicate_count` refuses every other leaf (**wildcard** `:208`); `resolve_reads` admits String and enum leaves (`:572-615`) | admit `TextMatch` with a text literal; see *Selection* |
| `crates/specify/ess-domain/src/primitive_admission.rs` | per-construct format gates (`:95-150`) | the `ess/N` gate over every predicate position; see *Formats* |
| `crates/verify/ess-conformance/src/witness.rs` | `collect_literals` (`:337`), `alternatives` (`:380`), module doc rules (`:15-33`) | the candidates above; rule 3's text says so |
| `crates/verify/ess-conformance/src/input.rs` | `explain_leaf` (`:516`) | arm |
| `crates/verify/ess-conformance/src/synthesize.rs` | `map_paths` (`:4391`) | arm, re-rooting `path` |
| `crates/verify/ess-conformance/src/quoted_predicate_format.rs` | the structured-operand suite gate, the model for the new one | a sibling module for this construct's suite gate; see *Formats* |
| `crates/verify/ess-conformance/src/go/predicate.go` | `parseConstraint` (`:284-325`) returns "carries no operator this runner knows"; `evaluate` (`:530-579`); `String` (`:123-162`) | parse the three with a string operand taken verbatim; evaluate per the table, `nil` → `Unknown` |
| `crates/verify/ess-conformance/src/go/runtime.go` | `admitPredicateConstraint` refuses unknown operators (`:3987-4027`); `meaningOperator` reads **any unknown operator as `any_of`** (`:840-872`, `default:`); `admitPredicateVersion` (`:6865-6873`); suite-version lists (`:80`, `:1562-1565`, `:3417-3430`) | admit the three with a string operand; give them their own meaning tuple, not `any_of`; the version gate |
| `crates/verify/ess-conformance/src/ts/predicate.ts` | `parseConstraint` (`:446-476`) | none. TypeScript does not gain the operators in this unit (*Out of scope*). |
| `crates/verify/ess-conformance/src/ts/runtime.ts` | `SUITE_MAJORS` stops at `/11` (`:4203-4215`), and an unknown major is refused (`:4231-4234`); `admitPredicateConstraint` refuses an unknown operator (`:4979`); `meaningOperator` files an unknown operator under `any_of` (`:1516-1537`) but runs only after admission | none. Two refusal cases: a suite at the new major gets `unsupported suite version`, and a forged `/11` suite carrying an operator gets `unknown predicate constraint operator`. |
| `crates/verify/ess-conformance/assets/coverage-admission.js` (browser) | `predicate()` fails `unsupported predicate operator` (`:300-306`); replay admits only `ess-conformance/5` and `/9` (`:521`) | none; see *Browser* |
| `crates/generate/ess-synth/src/rust/selection.rs` | `predicate` (`:107-141`, **wildcard** `unreachable!`) | `read_N.as_ref().map(\|v\| v.starts_with(L))`, and likewise, `L` via Rust `{:?}` |
| `crates/generate/ess-synth/src/go/selection.rs` | `predicate` (`:108-142`, **wildcard**); helper `selectionCompare` (`:55`) | a `selectionText(value, present, L, op)` helper, `L` as a Go literal (below) |
| `crates/generate/ess-entity-runtime/src/lib.rs` | `lower_predicate` (`:3012-3072`, exhaustive) | `StartsWith { starts_with: [rewrite.path(p), fact_value(L)] }`, `EndsWith`, and `Contains` for `contains`; `fact_value` (`:3102-3106`) already escapes a `$`-leading literal, which entity-core would otherwise read as a reference (`runtime.rs:2608-2611` at `718a702`) |
| `crates/infra/infra-spec/src/raw.rs` | `WorkloadPredicate` shares the parser (`:482-508`) | refuse `TextMatch` with `SpecInvalidExpectation`: `infra-spec/1` does not gain the operators by accident; admitting them there is its own format decision |
| `Cargo.toml` (`:89`) and `Cargo.lock` | `entity-core` `tag = "0.23.0"` | `rev = "718a7026fb193222036a553eac05c6b4060f8c38"`; see *The entity-core pin* |
| `website/docs/reference/predicates.md` (on `impl/a2-predicate-reference-page` until A2 merges) | "String operators" section "Arriving with #95" (`:494`); `ess-pending="beyond10x/ess#95"` example (`:501`); the synthesis table's `starts_with` row (`:524`); the model's `format: ess/1` (`:50`) | drop the pending line and every `ess-pending="beyond10x/ess#95"` (A2's harness treats a pending example that now passes as a failure, `crates/edge/ess-cli/tests/predicate_reference_page.rs:395-405` on that branch); raise the page model's `format: ess/1` to `ess/N`, or the examples are refused as below their format; the table row says "yes" |
| `website/docs/reference/spec-versions.md` | `ess/` table to `ess/7` (`:30-57`), `ess-conformance/` to `/13` (`:79-110`) | rows for `ess/N` and the new suite pair |
| `schemas/generated/ess.schema.json` | `Predicate` described generically (`:1093`) | regenerated by `cargo xtask schema` if the description changes. Nothing enumerates operators or format numbers (`FormatVersion` is a pattern, `crates/specify/ess-domain/src/system.rs:89`). |

**A Go string literal is not a Rust `Debug` string.** The Go selection emitter writes the literal
as `{value:?}` (`go/selection.rs:123`). Rust `Debug` escapes a grapheme-extending character as
`\u{301}` (measured: `format!("{:?}", "e\u{301}")` prints `"e\u{301}"`), and that is not Go
syntax. A string-operator literal is free text, and the shared vectors include a decomposed `é`, so
B1b adds one Go-literal writer to `ess-synth/src/go/`: printable ASCII other than `"` and `\` as
itself, every other byte as `\xNN`. It is byte-exact, which is the semantics. B1b uses it at both
selection sites (`:123` and the new one). `rg -n ':\?\}' crates/generate/ess-synth/src/go/` finds
26 lines using `Debug` inside Go source. They are the same class, and they predate this page. They
are reported, not fixed here.

### Selection

`docs/design/binding-list-selection.md:101-107` admits `Always`, `Never`, `Defined`, `Compare(Eq|Ne)`,
`All`, `Any` and `Not`, and says "future additions require their own bounded contract". This is
that contract for the three operators:

- **Reads:** a declared `String` item leaf only (`resolve_reads` already restricts reads to String
  or enum leaves, `selection.rs:590-611`; the checker refuses the enum).
- **Operand:** a text literal, not empty, at most `MAX_TEXT_BYTES` = 4096 bytes (`selection.rs:26`).
  No admitted read is longer, so a longer literal could never match.
- **Cost:** one pass over a read the selection already bounds by `MAX_TEXT_BYTES` and counts toward
  `MAX_EXAMINED_BYTES` (`selection.rs:26-28`). Rust `str::contains` (two-way search) and Go
  `strings.Contains` are linear in `|v| + |L|` (*inferred* from the standard libraries'
  documentation).
- **Truth:** `Unknown` on an absent optional leaf, as for `Compare` (`read_N` is `None` /
  `presentN` is false), so an undecidable selector stays a typed refusal
  (`binding-list-selection.md:112`).

The conformance selection evaluator uses `Predicate::evaluate`
(`crates/verify/ess-conformance/src/selection.rs:487`, `:519`) and needs nothing. The Go runner
evaluates an observed selection's predicate through the same `fromNode` as `satisfies`
(`src/go/runtime.go:6545`, `:6814`), so its parser change above covers it. The TypeScript runner
would do the same (`src/ts/runtime.ts:7966`, `:8221`), but it never admits a suite that carries
the operators.

### Browser

The browser replay adapter admits only `ess-conformance/5` and `/9` (`coverage-admission.js:521`).
The new coverage major is not among them, so a suite that carries a string operator is refused with
"replay requires suite/5 or /9", as suites /11 and /13 already are. A forged /9 suite carrying one
is refused by `predicate()` with "unsupported predicate operator" (`:306`). Both are refusals, not
misreadings, and B1b adds a case for each. The adapter does not learn the operators. Replay
illustrates and does not evaluate (`:611`), and admitting the new major is a separate change with
its own scope.

## Formats

Both numbers move, for the reason `SuiteFormat` gives for moves `3` and `4`
(`crates/verify/ess-conformance/src/scenario.rs:404-422`) and `FormatVersion` gives for every
authored construct (`system.rs:35-37`). A reader older than the construct refuses it with a parse
error: the Rust reader says `unknown operator` and the Go runner says `unknown predicate constraint
operator`. That error blames the document for the age of the tool. A version number turns it into
"upgrade the tool". No older reader misreads the construct, but one reader comes close:
`meaningOperator` in both runtimes files an unknown operator under `any_of`. It sits behind
admission, which refuses first.

| Family | Moves? | Why |
|---|---|---|
| `ess/` (authored) | **yes**, to the next free `N` | New vocabulary at an existing key. Gated in `primitive_admission.rs` with `unsupported_format_version`, beside the ess/7 and ess/6 gates (`:102-114`, `:115-131`). |
| `ess-conformance/` (suite) | **yes**, a new ordinary/coverage pair, **only for a suite that carries one** | A suite carries a predicate in two places only: `ViewExpectation::Satisfies` and an observed selection's plan (`quoted_predicate_format.rs:26-72` walks exactly those). A string operator in a command guard is decided at synthesis and never reaches the suite, so such a suite keeps its format and bytes. |
| `ess-scenario/1` (authored scenarios) | no | Its claim keys have grown without a number (`halts_after`, `authored.rs:114-140`, `FORMAT` still `/1` at `:222`). The compiled suite carries the predicate and takes the suite gate. |
| compiled IR | no number exists | The IR serializes the predicate through `to_node`. A model that uses no string operator keeps its IR bytes and `spec_digest`. |
| `ess-diff/` | no | A changed guard is compared by `Predicate` equality and printed through `Display` (`diff.rs:920-935`; `crates/verify/ess-diff/src/lib.rs:36-45`). No delta vocabulary is added. |
| `infra-spec/1` | no | Refuses the construct (table above). |

**The ess/N gate answers the class, not one position.** B1b adds one function yielding every
predicate a `Specification` holds, with its site: command outcome conditions (`When`,
`SubjectState`, `StateChange`, `ExternalWhen`, `SubjectField`'s input predicate;
`crates/specify/ess-domain/src/command.rs:373-421`), entity and type invariants (`entity.rs:497-501`,
`crates/specify/ess-domain/src/types.rs:597-605`), view filters (`view.rs:322`, `:712`) and
selections (`selection.rs:75`). The gate is `Predicate::uses_text_match()` over that function. A
test per position shows each one refused at `ess/N−1`. #75's subject predicate extends the same
function rather than adding a second gate.

**The suite gate** is a sibling of `quoted_predicate_format.rs`. `used_by(suite)` walks the same
two carriers. `select_fresh_format` (`scenario.rs:155-168`) returns the new ordinary major first,
because each branch implies the ones below it. `admit_suite` and `admit_predicate` refuse a string
operator under a lower major, and so does Go's `admitPredicateVersion` (`src/go/runtime.go:6865-6873`).
Both suite readers that admit the operators, Rust admission and Go, also refuse a string-operator
operand that is not a JSON string, because suites are not type-checked. TypeScript and the browser
refuse the new major outright (above).

### Which numbers

At the time of writing the next free numbers are **`ess/8`** (`SUPPORTED_FORMATS = [1..=7]`,
`system.rs:53`) and **`ess-conformance/14` and `/15`** (`SUPPORTED_SUITE_FORMATS = [1..=13]`,
`scenario.rs:368`; coverage majors are the odd ones, `crates/verify/ess-conformance/src/coverage.rs:15-24`).
**B1b takes the next free source, suite and diff numbers at implementation time.** Today that is
`ess/8`; this construct moves no diff number (table above). Units C (#75) and D2 (#96) take the
numbers after B1b's, in the merge order A1 → B1b → C → D2. On the integration branch it forks
from, B1b takes:

1. `ess/` = `max(SUPPORTED_FORMATS) + 1`;
2. suite = the next even major after `max(SUPPORTED_SUITE_FORMATS)`, and that major + 1 for coverage;
3. the tests that use a number **above** the current maxima as "a version this build cannot read"
   move past the new numbers. This lists them (17 lines today; run from the repository root):

   ```console
   rg -n --pcre2 'ess/(?:[89]|[1-9][0-9]+)\b|ess-conformance/(?:1[4-9]|[2-9][0-9]|[1-9][0-9]{2,})\b|suite/(?:1[4-9]|[2-9][0-9])\b' crates website
   ```

   The alternations are written for the current maxima, `ess/7` and `/13`. Raise their lower
   bounds by the same step whenever the maxima move. Today it lists
   `crates/verify/ess-conformance/tests/count_reports.rs:460-461`,
   `crates/generate/ess-cli-project/tests/consumer_cli_model_ingress.rs:637,639,643`,
   `crates/specify/ess-domain/src/system.rs:1254,1263,1275,1721,1969`,
   `crates/specify/ess-domain/src/spec.rs:1746`, `crates/verify/ess-conformance/src/scenario.rs:2677`,
   `crates/verify/ess-conformance/src/evidence.rs:351,355`,
   `crates/verify/ess-conformance/tests/report_reader_adversary.rs:177`,
   `crates/edge/ess-cli/tests/go_conformance.rs:1164` and
   `crates/edge/ess-cli/tests/support/input_discovery_cases.rs:322`. Each must be read before it is
   moved: a line that means "unreadable" moves, and a line that happens to name the number does not;
4. every list that names the current maxima gains the new numbers: the admission range
   (`crates/verify/ess-conformance/src/admission.rs:182-193`), `SUPPORTED_SUITE_FORMATS`, the
   coverage list, and the Go lists (`src/go/runtime.go:80`, `:1562-1565`, `:3417-3430`). This
   finds them (15 lines today):

   ```console
   rg -n 'ess-conformance/13\b|suite/13\b|1\.\.=13\b|, 13\]|\[1, 2, 3, 4, 5, 6, 7\]' crates
   ```

   TypeScript's `SUITE_MAJORS` (`src/ts/runtime.ts:4203-4215`) is not among them, and it stays at
   `/11`.

## The entity-core pin

`ess` moves `entity-core` from `tag = "0.23.0"` (`Cargo.toml:89`; lock source
`?tag=0.23.0#77aac6e`, `Cargo.lock:334`) to `rev = "718a7026fb193222036a553eac05c6b4060f8c38"`
(beyond10x/entity-runtime#40, branch `feat/string-prefix-suffix-conditions`, open), by rev as the
organization pins. Read-only, in entity-runtime:

- `git log 0.23.0..718a702` is three commits. `6f6ed0d` and the merge `d67e901` touch only
  `.engineering/` (planning records; `git diff --stat 0.23.0 d67e901 -- . ':!.engineering'` is
  empty). `718a702` is the feature: `definition.rs`, `runtime.rs` and `validation.rs` in
  `crates/entity-core/src`, tests, docs and `CHANGELOG.md`.
- The workspace version stays `0.23.0` at `718a702`, and no manifest or `Cargo.lock` changes, so
  the pin brings no new dependency.
- `Condition` gains `StartsWith` and `EndsWith`, which breaks exhaustive matches over it. `ess` has
  none. `ess-entity-runtime` constructs conditions, and its only match on one
  (`crates/generate/ess-entity-runtime/src/lib.rs:2586-2593`) names `Truthy` and `All` and ends in
  `_ => unreachable!` (*inferred* from a search for `Condition::` patterns; `cargo check` decides it).
- Semantics that the lowering depends on, at `718a702`: byte-wise and case-sensitive under every
  semantics key; an unrecorded or null operand is `Unknown`; a resolved non-string is `False`; a
  literal non-string is refused at registration. They match the table above. That is why the
  lowering needs no guard wrapped around the condition.

The pin names the head of an **open** pull request. When entity-runtime #40 lands, it must be
merged with a merge commit, not squashed, so that `718a702` stays reachable from `main`. Otherwise
`ess` re-pins to the merge result. When entity-runtime releases (0.24.0 per the PR), `ess` may move
the pin to that tag's commit. That move is not B1b's.

## Out of scope

- **Regular expressions.** A witness would need text generated from a regex, and one dialect across
  Rust, Go and JS. The schema side already notes ECMA-262 matching as unfinished (issue #95).
- **SQL `LIKE`.** It is a pattern language with the same witness problem, restricted.
- **Case-insensitive matching.** "Case" needs a folding table, and the three lanes' tables differ
  (`binding-list-selection.md:119-125` records the same boundary for selection).
- **Fact-to-fact operands, a compact form, aliases, `not_*` spellings.** Rejected above.
- **TypeScript suite /12+.** The TypeScript runner admits suites only up to `ess-conformance/11`
  (`src/ts/runtime.ts:4203-4215`) and has no replay. It does not gain the new suite pair or the
  three operators in this unit. It refuses a suite that carries them, naming the version
  (`:4231-4234`). Taking TypeScript to `/12` and beyond, including this construct, is its own unit.
  It will need the UTF-16 argument: `startsWith`, `endsWith` and `includes` compare code units, and
  for well-formed strings those agree with byte matching, while ordering does not.
- **The agent plugin's grammar page** (`agentplugins`, ESS skill) is updated after the release, as
  the issues plan does for #92. Not in this repository.
- **The Go and TypeScript one-operator-per-mapping readers.** `parseConstraint` in both lanes
  returns the first operator it finds. Go does so over a randomly ordered map range
  (`predicate.go:303-323`), where Rust conjoins them all. Suites written by `to_node` carry one
  operator per mapping, so this is unreachable from a generated suite. It predates this page and is
  reported, not fixed.

## What B1b checks before building

These lines are inferred or rest on unmerged work. Confirm each with one read or one run:

1. A1's text ladder and list builder read `literals_at` (*Witness*, "Interaction with A1").
2. Formats are cumulative: raising the reference page model from `ess/1` to `ess/N` changes no
   other example's verdict. A2's harness shows it.
3. No other exhaustive match over `entity_core::Condition` in `ess` (`cargo check -p
   ess-entity-runtime` after the pin decides it).
4. The Go `nil` row values reach `evaluate` bound, not skipped (the shared
   vectors decide it).
