# A finite Binary64 in every code target

Status: binding design, written 2026-11-21 before the code, for
`story:every-code-target-carries-a-finite-binary64`. Line numbers are those of commit `3f58263009`.
It builds on [model-binary64.md](model-binary64.md) (value policy, suite contract) and
[binary64-structural-codecs.md](binary64-structural-codecs.md) (the standalone checked wrapper);
neither is restated.

**Decision in one line.** `ess generate synthesize --target rust|go|web` carries a `Binary64` in
every position the conformance suite carries (`ess_compiler::binary64::uncarried_uses` is empty
for it) as a private-field checked wrapper in the target's `primitives`, read from the raw JSON
number token and written by one fixed layout rule, compared by its shortest decimal; every
position the suite does not carry, and the whole `clap` target, keep a refusal by name.

## Where the seam is

- One refusal covers every code target: `crate::failure::binary64`
  (`crates/generate/ess-synth/src/failure.rs:156-176`) turns every
  `ess_compiler::binary64::locations` entry into a `missing-representation` cause. It is called
  from `lib.rs:488`, `rust/mod.rs:192`, `go/mod.rs:319`, `web/mod.rs:300`, `clap/mod.rs:63`.
- Behind it, 16 `Primitive::Binary64` arms are `unreachable!` or refusals: `rust/layout.rs:522`,
  `rust/wire.rs:863,879,1011,1042`, `rust/feasibility.rs:113`, `rust/invariant.rs:756`,
  `rust/store/identity.rs:42`, `go/layout.rs:1217`, `go/http.rs:998,1014,1179,1204`,
  `go/invariant.rs:1050`, `go/store/identity.rs:38`, `view_query.rs:204`. Not an arm but just as
  blocking: `determined.rs:1140-1150` classes a `Binary64` as `Kind::Opaque`, so a guard, a filter
  or an `order_by:` over one keeps its command or view query owed (`view_query.rs:353-367`,
  `380-416`).
- The Rust server reads requests with its own reader, which keeps a number as its spelling
  (`rust/json.rs:58-62`, grammar `364-411`); the Go server decodes with `UseNumber`
  (`go/http.rs:2779-2791`), so a `json.Number` keeps the token too. Both already hold the raw
  token the value policy needs (`model-binary64.md:18`).
- `Decimal` is the analogue: a wrapper in `primitives` (`rust/mod.rs:450-454`, `go/mod.rs:729-735`),
  read by `decimal_at`/`decimalAt`, compared by its decimal text through `number_parts`/
  `number_order`/`compare_numbers` (`rust/behaviour.rs:635-678`, `896-915`), classed
  `Kind::Number`. It differs from `Binary64` in its wire form only (a string, `ess-gen/src/types.rs:657`).
- The published contract already says `type: number` (`ess-gen/src/types.rs:31,653-656`; the
  reproduction's OpenAPI: `score: {type: number}`), with finiteness a realizer's obligation.

## 1. Wire form

Both servers, and the web bridge (which compiles the Rust target's `wire` module and types crate,
`web/layout.rs:1-24`), read and write the same way.

**Decode.** The position must hold a JSON number token, which the request reader has already held
to the JSON number grammar. The token is rounded once to the nearest binary64, ties to even
(`str::parse::<f64>` in Rust, `strconv.ParseFloat(token, 64)` in Go; both are correctly rounded).

| token at the position | answer |
|---|---|
| `0.73`, `1e-7`, `5e-324`, `1.7976931348623157e308` | admitted |
| `-0`, `-0.0` | admitted as negative zero |
| `1e-400` / `-1e-400` | admitted as `+0` / `-0` (underflow is a value, `model-binary64.md:357`) |
| `1e400`, `1.7976931348623159e308`, `-1e400` | refused: overflows to an infinity |
| `"0.73"`, `null`, `true`, an array or object | refused: not a number |
| `NaN`, `Infinity`, `+1`, `.5` | never reach the decoder: the body is not JSON (`400 the body is not JSON: …`) |

A refusal is the existing decode refusal: `json::DecodeError` in Rust (`rust/json.rs:676-701`),
`DecodeError` in Go (`go/http.rs:2236-2255`), so the route answers what it answers for any
malformed input: `400` with `{"refused": "<path>: expected <expected>, found <found>"}`
(`rust/http.rs:1399-1405`, `1878-1885`; Go `refusal(400, err.Error())`, `go/http.rs:1940-1944`),
or the binding's own malformed refusal on a bound route (`rust/http/bound.rs:940`). The words, the
same in both servers as `go/http.rs:1175` requires: expected `a finite Binary64 number`; found
`a string` (and the other kinds as `describes` spells them) or `the number 1e400`. Example:
`body.hit.score: expected a finite Binary64 number, found the number 1e400`.

**Encode.** A held value is finite by construction (private field, checked constructor), so
writing never fails and never writes `null`. The token is laid out from the shortest round-trip
digits `d1…dn` and decimal exponent `e` (Rust `format!("{:e}", v)`, Go
`strconv.FormatFloat(v, 'e', -1, 64)`), with `p = e + 1`:

| case | written | examples |
|---|---|---|
| zero | `0.0`, or `-0.0` for negative zero | `-0.0` |
| `n ≤ p ≤ 16` | the digits, `p − n` zeros, `.0` | `3.0`, `9007199254740992.0`, `1000000000000000.0` |
| `0 < p < n`, `p ≤ 16` | digits with the point after `p` | `123.456`, `1.5` |
| `−5 < p ≤ 0` | `0.`, `−p` zeros, the digits | `0.73`, `0.00001` |
| otherwise | `d1`, then `.d2…dn` if `n > 1`, `e`, the exponent (`-` only, no leading zeros) | `1e-6`, `1e16`, `1.5e300`, `5e-324` |

This is the layout `serde_json` writes for an `f64` (its `ryu` formatter), and therefore the bytes
of the suite's `Node` serializer (`ess-primitives/src/facts.rs:413`, `model-binary64.md:300-302`)
and of the standalone Rust codec (`schema-contract/src/realize/rust_binary64.rs.txt:26`,
`serialize_f64`). It keeps the integral marker and `-0.0` (`model-binary64.md:19`). The generated
code implements the table itself: the generated trees take no dependency (`rust/json.rs:6-10`),
and Rust's `Debug` float layout is not a stability promise. The standalone Go codec writes `'g'`
(`1e+06`, `go_binary64.go.txt:16`), which differs; see open question 1.

## 2. The value type

| | Rust (`crate::primitives`, types crate) | Go (`primitives` package) | web page (JavaScript) |
|---|---|---|---|
| type | `pub struct Binary64(f64)`, field private | `type Binary64 struct { value float64 }` | a JS `number` |
| construct | `Binary64::new(f64) -> Result<Self, Binary64Error>`; `Binary64Error::NonFinite` displays `Binary64 requires a finite value` | `NewBinary64(float64) (Binary64, error)`, same words | the number control, below |
| read | `get(self) -> f64` | `Float64() float64` | — |
| wire | `from_token(&str) -> Option<Self>`, `token(self) -> String` (§1) | `binary64FromToken`, `Token() string` | — |
| decimal | `Display`: `f64`'s `Display`, shortest plain decimal (`-0`, `0.0000001`) | `Decimal() string`: `FormatFloat(v, 'f', -1, 64)`, the same text | — |
| equality | `PartialEq` by IEEE `==`, and `impl Eq` | Go `==` on `float64` | — |

The API is the standalone wrapper's (`binary64-structural-codecs.md` § Checked native API) without
its serde impls, which need `serde_json::RawValue`, a dependency the generated workspace cannot
take. The name drops `Ess` because it sits beside `Decimal`, `Timestamp` and `Uuid` in
`primitives`, where no model type can collide with it.

**Rust `Eq`.** Every generated record derives `PartialEq, Eq` (`rust/items.rs:76,115`,
`rust/entity.rs:77`). IEEE `==` on values that exclude NaN is reflexive, symmetric and
transitive, so `impl Eq for Binary64 {}` is lawful and no derive changes. `+0 == -0`, as
`model-binary64.md:20` requires; the sign stays observable through `get`, `token` and `Display`.

**No `Hash`, no `Ord`.** No generated item derives either (`rg 'Hash|PartialOrd' src/rust` finds
only `Actor`, the four text wrappers and private key enums), and the one place that needs a key
order, the in-memory identity key (`rust/store/identity.rs:28-43`), is not reached because a
`Binary64` identity stays refused (§5). Order and comparison go through `Display`/`Decimal()`
(§4). This keeps the "no hashing or total-order promise" of `model-binary64.md:20`.

**Go zero value.** `primitives.Binary64{}` is `+0` and spellable, like every Go wrapper's zero
value (`go/mod.rs:698-713`); a missing required member is still refused by the decoder.

**Emitted only where used.** The Rust wrapper is appended to `primitives.rs` after `PRIMITIVES`
(`rust/mod.rs:429-439`, where `reading::helpers` and `invariant::runtime` are already appended
conditionally), and the wire helper `binary64_at` to `wire.rs`, only when
`locations_of(ir, Binary64)` is non-empty, the test `rust::json::used` uses for `Json`
(`rust/json.rs:1218-1220`). Go appends its wrapper after `PRIMITIVES` and replaces
`The four below have no` with the right count, as `json::with_json` does (`go/mod.rs:685-716`).
The fixed `JSON` reader (`rust/json.rs:34`) and Go's fixed wire helpers do not change.

**Web page.** The bridge needs nothing beyond the Rust codec. The page script is fixed text; a
model with a `Binary64` gets one substitution, as `Json` does (`web/page.rs:100-101`, `187-189`),
anchored on `if (name === "integer") {` in `primitive()` (`web/page.rs:262`): an
`<input type="number" step="any">` whose `read()` answers `Number(value)` when it is finite and
otherwise marks itself invalid and answers `undefined`, so the bridge names the missing value
instead of receiving `Infinity` or a string. `TARGET.md` gains a weakening for such models only,
as `json_weakening` is added (`web/mod.rs:434-470`): the page sends `-0` as `0`
(`JSON.stringify`) and shows numbers in ECMAScript's spelling (`1e+21`); the bridge keeps both.

## 3. Positions

"Model" is what `ess specify validate` 0.97.1 admits (probed with the reproduction, varied).

| position | model | generated code after this story |
|---|---|---|
| struct field, `Optional<…>`, `List<…>` | admitted | `Binary64`, `Option<Binary64>` / `*Binary64`, `Vec<…>` / `[]…` |
| `Map<String, Binary64>` value | admitted | `BTreeMap<String, Binary64>` / `map[string]…` |
| `Map<Binary64, V>` key | refused (`ess-domain/src/types.rs:211-212`, `primitive_admission.rs:33-37`) | — (key arms stay `unreachable!`, reason: the domain) |
| newtype `of: Binary64`, with invariants | admitted | wrapper of `Binary64`; an invariant reads `iv::Fact::number(&v.to_string())` (Rust, `rust/invariant.rs:740-743` pattern) / `NumberOf(v.Decimal())` (Go, `go/invariant.rs:1045`); the decoder refuses a breaking value as for `Decimal` |
| entity field | admitted, carried | stored in the snapshot like any field |
| entity identity, or a type it reaches | admitted (probe), uncarried | **refused** (§5) |
| command input, response | admitted, carried | decoded / encoded (§1) |
| event field, error field | admitted, carried | encoded; event decoders (`rust/wire.rs:389-403`) decode |
| view field, row shape | admitted, carried | encoded |
| view parameter | admitted (probe), uncarried | **refused** (§5); on a served view `refuse_unqueryable` refuses it too (`view_query.rs:220-260`) |
| guard (`when:`), view `filter:` | admitted: `Binary64` is `ScalarKind::Number` (`ess-domain/src/expression.rs:73`), numeric literals included; `when: "input.added >= 0.5"` validates | `Kind::Number(Binary64)` (new, `determined.rs:1140`); compared exactly by decimal text (§4) |
| `order_by:` | admitted (`ess-domain/src/command/ordered_head.rs:676-686`; view order checks projection only, `view.rs:2228-2275`) | `Values::Numeric` (new, `view_query.rs:353-358`): `number_order` over `Display`/`Decimal()` |
| `sets:` / `payload:` / `example:` literal | refused (`command.rs:5785-5791`, `command/outcome_shapes.rs:916`, `command.rs:6362-6364`) | — |
| `{expr: …}` operand, offset comparison, `distinct` key | refused (`expression/value.rs:71,84-86`; `expression.rs:3265`; `expression.rs:1268`) | — |
| aggregate `sum`/`avg`/`min`/`max`/`count_distinct`, `group_by` | refused (`view.rs:1062-1097`, `2196-2215`; probe: `ESS-VIEW-002`) | — |
| `coalesce` default over a `Binary64` | admitted | stays owed (`determined.rs:1186-1196` admits no `Binary64` default) |
| `form`/`query`/path field of an `ess-http/1` binding | refused at binding compile (`ess-gen/src/http.rs:812-842`, `ESS-HTTP-021`) | — |
| actor attribute, binding delivery/mapping/selection, conversion | uncarried | **refused** (§5) |

## 4. Storage and comparison

The generated servers store nothing themselves: storage is a port, and the network executable's
store is the ephemeral in-memory `Vec` of snapshots kept in identity order (`rust/store.rs:34`,
`go/store.rs:14-32`). A `Binary64` is a field of the snapshot, stored as the wrapper; no
persistence is generated, so nothing is spelled to disk.

- **Equality** in generated Rust/Go types: IEEE (`+0 == -0`), §2.
- **Guards, filters, invariants, `order_by:`** compare the shortest plain decimal (`Display` /
  `Decimal()`) exactly, through the helpers `Decimal` already uses (`number_parts` treats `-0` as
  zero, `rust/behaviour.rs:652-653`; Go `numberParts`, `go/behaviour.rs:1106-1130`). Distinct
  binary64 values have distinct, identically ordered shortest decimals, so this is IEEE order
  with `-0 == 0`. It is also the runner's reading: `Number` carries a binary64 as the shortest
  decimal `f64`'s `Display` prints (`facts.rs:127-138`) and compares by exact value
  (`model-binary64.md:316-321`). A literal is written as the IR's `Number` displays it
  (`rust/behaviour.rs:3467`, `go/behaviour.rs:4238`), which is the runner's reading of it too.
- **Ranking.** Two equal values (including `-0` and `0`) tie and the next key decides, as the
  runner's `ranked` does.
- **Stated limit.** Overwriting `0.0` with `-0.0` is not a change to anything that compares the two
  values; the stored sign is the new one. The suite never claims a sign of zero.

## 5. The refusal that remains

`failure::binary64` takes the target's capability:

- `rust`, `go`, `web`: one cause per `ess_compiler::binary64::uncarried_uses(ir)` entry
  (`ess-compiler/src/binary64.rs:61`), code `missing-representation`, detail:
  `a Binary64 here is a key, a lookup or a delivery route, which this target does not carry; it
  carries one in entity fields, command input and response, event and error fields, and view
  fields and row`. A position in a type nothing reaches is carried (emitted as a value), not
  refused: the codec emits every declared type and the wrapper serves it.
- `clap`: unchanged, every `locations` entry, `this target has no qualified finite Binary64 codec`
  (open question 2).

Failure formats do not move: the envelope, the code and the format chosen by
`TargetFailure::new` are unchanged; only the detail text of the Rust/Go/Web cause is new.

## 6. Formats and bytes

- No source format, IR, suite format, normalization recipe, target-failure format or OpenAPI
  change: every type, version and envelope is as it was, and the new behaviour applies only to
  models a released `ess` refused outright.
- `ess-browser-catalog/1`: a type reference is the IR's `ResolvedTypeRef` serialization
  (`web/catalog.rs:104-108`), whose primitive spellings have included `binary64` since `ess/2`;
  the only reader is the page emitted beside it. No version.
- Committed trees: no model under `examples/` or `generated/` declares a `Binary64`
  (`rg Binary64 examples generated`: nothing), and every addition is conditional on
  `locations_of(ir, Binary64)`, so no committed tree changes a byte. Pins that would show it:
  `ess-synth` `feasibility`, `served_codec_names`, `go`, `http_binding_rust`,
  `adversary_related_guard_behaviour_pass1`, `single_crate_layout`, `web`, `json_primitive`.
- Tests that use `Binary64` as a convenient refusal trigger and must be restated with an uncarried
  position (a `Binary64` identity) or with Clap: `ess-synth/tests/feasibility.rs:51-68`,
  `go_owner.rs:124-143`, `bounded_accessor.rs:100-118`, `clock_reading.rs:94-107`, and
  `ess-cli/tests/binary64_publication.rs:51-108` (its Rust/Go/Web rows; Clap and
  `verify conform web` keep refusing). `ess-cli/tests/target_failure.rs:36` uses Clap and stays.

## 7. Conformance

The released 0.97.1 already writes the suite such a server is held to:

- The reproduction (`Hit.score: Binary64`) synthesizes `ess-conformance/62`, one scenario, score
  witnesses `1.5`, `2.5`, `3.5`. `Optional<Binary64>` writes `101.5`…, `List<Binary64>` `[1.5]`,
  `Map<String, Binary64>` synthesizes, and `Binary64` on the entity field, input and view field
  writes `1.5`/`2.5` and a `ranked` expectation over `added_ms desc`.
- A guard `input.added >= 0.5` on a `Binary64` gets the boundary witnesses `0.5` (accepted) and
  `-0.5` (rejected).
- A newtype `of: Binary64` with invariants synthesizes its command scenarios; its wrapped
  value-invariant observer is refused on its own (`ESS-SYNTH-001`), as `model-binary64.md:287-290`
  says.
- An identity or a view parameter of `Binary64` refuses the whole suite: `UnsupportedPrimitive`,
  "a Binary64 this route does not carry" (`ess-conformance/src/admission.rs:813-822`).

**Test.** One model holds every carried position: struct field (bare, `Optional`, `List`, map
value), a newtype with invariants `value >= 0`, `value <= 1`, entity field, command input with a
guard `input.added >= 0.5` and a rejected branch whose error carries the value, an event field,
and a `read_your_writes` view with `filter: "added > 0.75"` and `order_by: [added desc, doc_id
asc]` (0.97.1: valid, `/62`, two scenarios, one `ESS-SYNTH-001` refusal). It is a fixture,
`tests/fixtures/binary64-served/model.yaml.in` (`.in`, so the model walkers skip it). Each server
is compiled once per run, never per test:

- Rust: `task binary64-harness` (the pattern of `integer-spellings-harness`, `Taskfile.yml:161-182`)
  synthesizes the tree with `ess generate synthesize`, builds a harness that holds only storage and
  identities over the generated in-process dispatcher, and the test reads it from
  `ESS_BINARY64_HARNESS`; `task test`, `test-archive*` and the shards set it beside the others.
- Go: built in the test with `go build`, whose content-keyed build cache is the disk cache
  (`integer_spellings.rs:183-231` pattern), skipped aloud where there is no `go`. All Go cases sit
  in one `#[test]`, so nextest builds it once.

Each server test sends literal tokens and compares literal answer bytes (never values computed by
the parser under test, `binary64-structural-codecs.md:218`): `0.73`→`0.73`, `-0.0`→`-0.0`,
`-0`→`-0.0`, `3`→`3.0`, `1e16`→`1e16`, `1e15`→`1000000000000000.0`, `0.00001`→`0.00001`,
`0.000001`→`1e-6`, `9007199254740993`→`9007199254740992.0`, `5e-324`→`5e-324`,
`1e-400`→`0.0`, `-1e-400`→`-0.0`; `1e400`, `1.7976931348623159e308` and `"0.73"` are `400` naming
`body.hit.score`; ratio `1.0000000000000002` is `400` (invariant); added `0.49999999999999994` is
`rejected`. Then the synthesized suite runs against the server through the Rust `Runner`, every
scenario passing. Web: a synthesis test only (`tests/binary64_web.rs`), asserting the page
control, the weakening and that the bridge's `wire.rs` calls `binary64_at`; the codec is the Rust
one the harness exercised.

## 8. Units

Each lands green alone; the failing test is written first.

| unit | files | failing test first | pins to run | depends on |
|---|---|---|---|---|
| **U1 admission and shared semantics** | `failure.rs` (per-target capability read from a `CARRIES_BINARY64` const in `rust/mod.rs`, `go/mod.rs`, `web/mod.rs`, all `false`; uncarried cause and wording); `determined.rs` (`Kind::Number(Binary64)`); `view_query.rs` (`values()`); the six restated tests of §6; fixture `binary64-served/model.yaml.in` | an identity of `Binary64` refuses Rust/Go/Web with the new detail; Clap keeps the old one; restated triggers | `ess-synth` `feasibility`, `go_owner`, `bounded_accessor`, `clock_reading`, `generated_view_queries`, `adversary_related_guard_behaviour_pass1`, `http_binding_rust`, `served_codec_names`, `go`; `ess-cli` `binary64_publication`, `target_failure` | — |
| **U2 Rust codec** | `rust/mod.rs` (flag `true`, wrapper append), new `rust/binary64.rs` (wrapper and wire helper text), `rust/layout.rs`, `rust/wire.rs`, `rust/invariant.rs`, `rust/feasibility.rs`, `rust/store/identity.rs` (reason text); `Taskfile.yml` (`binary64-harness`, env in `test`, `test-archive*`, `test-shard`); `.github/workflows/ci.yml` if the harness is uploaded with the archives; harness fixture; `tests/binary64_rust.rs` | the reproduction synthesizes `--target rust` (workspace and `--layout crate`, with `--http`); then the wire table and suite of §7 against the harness | U1's set plus `single_crate_layout`, `json_primitive`, `integer_spellings`; `task ci-lint` (Taskfile, `ci_lanes.rs`) | U1 |
| **U3 Go codec** | `go/mod.rs` (flag, primitives package), new `go/binary64.rs`, `go/layout.rs`, `go/http.rs` (arms, conditional `binary64At`), `go/invariant.rs`, `go/behaviour.rs` (`leaf` arm, `4546`), `go/store/identity.rs` (reason); Go harness fixture; `tests/binary64_go.rs` | the reproduction synthesizes `--target go --http`; then §7 against the Go harness | `go`, `go_owner`, `json_go`, `served_codec_names`, `feasibility`, `adversary_served_pass2_go` | U1 |
| **U4 web** | `web/mod.rs` (flag, weakening), `web/page.rs` (substitution); `tests/binary64_web.rs` | the reproduction synthesizes `--target web`; page and `TARGET.md` assertions | `web`, `json_web`, `feasibility` | U2 |
| **U5 docs and release notes** | `docs/design/model-binary64.md` (section "Whole-system codecs" pointing here, the superseded sentences at `:125-126`, `:226`, `:283`), `binary64-structural-codecs.md:5,41-42`, `website/docs/guides/synthesize.md:79-80`, `CHANGELOG.md` | — | `task test-xtask`, `task site-build` | U2, U3, U4 merged |

U2 ∥ U3 touch no common file (each flips its own flag; the harness of U3 is built in-test, so
only U2 edits `Taskfile.yml`). U4 may run beside U3. U5's text can be drafted in parallel and
lands last. No unit adds a `ValidationCode` or a diagnostic.

## 9. Open questions

1. **The standalone Go codec's spelling.** It writes `'g'` (`1e+06`, `1e-07`), the servers write
   §1's layout. (a) Leave it: byte identity across languages is not claimed
   (`model-binary64.md:19`) and its output is pinned per generator version. (b) Move it to §1's
   layout: one spelling everywhere, but `ess generate types --target go` output moves for every
   `Binary64` model. Recommendation: (a) now; (b) when an adopter compares a Go library's bytes
   with a server's.
2. **Clap.** (a) Keep the refusal. (b) A `Binary64` flag with a finite `value_parser`, printed by
   §1's rule. Recommendation: (a) until a command-line adopter asks; no reported need names Clap.
3. **Identity and view parameter.** (a) Keep them refused with the suite (§5). (b) Carry them:
   an identity key with `-0` folded to `+0` and a `QueryScalar::Number`; both also need the
   suite's `uncarried_uses` line moved, a conformance format change. Recommendation: (a); open
   them in one story with conformance when a model needs a float key.
4. **Story scope.** The story's inferred `Scope` omits `determined.rs`, `go/behaviour.rs`,
   `web/page.rs`, `web/mod.rs`, `Taskfile.yml` and the restated tests of §6. Recommendation:
   correct it with `aep plan artifact` when the units are filed; the design does not depend on it.
