---
format: aep.planning-md/3
id: story:the-go-target-carries-a-finite-binary64
kind: story
status: active
title: The Go target carries a finite Binary64 in every value position, on the wire as the Rust server writes it
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-code-target-carries-a-finite-binary64
- depends_on: story:each-code-target-admits-binary64-through-a-switch-of-its-own
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/store/identity.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binary64_go.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/binary64-served-go-harness/main.go
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:43:38Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-11-21T23:43:39Z", actor: "human:timo", revision: 12}
---
## Outcome

- `--target go` synthesizes a model with a `Binary64` in any value position; only an uncarried one
  (identity, view parameter, actor attribute, binding, conversion) is refused, with B1's detail.
- The `primitives` package holds `Binary64` (finite by construction); the server reads a JSON number
  into it and writes the same bytes the Rust server writes.
- The generated server passes the wire table and the fixture's suite, built once in one test with
  `go build`. No format version changes.

## What to change

Line numbers are at `origin/main` `f3e4477974`. Unit B1 has inserted `pub(crate) mod binary64;`
after line 40 of `src/go/mod.rs`, so every `go/mod.rs` line below 40 is shown here **after** that
shift (+1), and created `src/go/binary64.rs`. Paths are under `crates/generate/ess-synth/`.

**Wire layout** (the Rust target's, unit B2): zero is `0.0`, or `-0.0` when the sign bit is set.
Otherwise take the shortest digits `d1…dn` and exponent `e` of `|v|` from
`strconv.FormatFloat(math.Abs(v), 'e', -1, 64)` (`1.5e+00`, `1e+16`, `5e-324`), `p = e + 1`:
`n ≤ p ≤ 16` → digits, `p − n` zeros, `.0`; `0 < p ≤ 16` → digits with the point after `p`;
`−5 < p ≤ 0` → `0.`, `−p` zeros, digits; `n == 1` → `d1` `e` `e`; else `d1.d2…dn` `e` `e`. A `-`
precedes a negative value; the exponent is written with `strconv.Itoa` (no `+`, no leading zeros).

1. **`src/go/binary64.rs`** (created by B1 with `CARRIED = false`):
   - set `pub(crate) const CARRIED: bool = true;`
   - add `pub(crate) fn used(ir: &ess_compiler::EssIr) -> bool { !ess_compiler::binary64::locations(ir).is_empty() }`
     (doc: the wrapper, its imports and `binary64At` are emitted only then, so every other model
     keeps its bytes).
   - add `pub(crate) const IMPORTS: &str = "\nimport (\n\t\"errors\"\n\t\"math\"\n\t\"strconv\"\n\t\"strings\"\n)\n";`
   - add `pub(crate) const PRIMITIVE: &str = r#"…"#;` with exactly this Go text (tabs, as gofmt
     writes them):

     ```go

     // Binary64 is a finite IEEE 754 binary64, preserving the sign of zero.
     //
     // The field is unexported, so the only way to make one is [NewBinary64], which refuses a NaN
     // and an infinity — but Go's zero value needs no constructor, so `Binary64{}` is still
     // spellable, and it is +0. Two values are equal under `==` as IEEE 754 compares them, so +0
     // and -0 are equal; the sign stays readable through [Binary64.Float64] and [Binary64.Token].
     type Binary64 struct {
     	value float64
     }

     // NewBinary64 wraps a finite value as a Binary64, and refuses a NaN or an infinity.
     func NewBinary64(value float64) (Binary64, error) {
     	if math.IsNaN(value) || math.IsInf(value, 0) {
     		return Binary64{}, errors.New("Binary64 requires a finite value")
     	}
     	return Binary64{value: value}, nil
     }

     // Float64 is the wrapped value.
     func (v Binary64) Float64() float64 {
     	return v.value
     }

     // Decimal is the shortest plain decimal that reads back as the value, as guards, filters,
     // invariants and orders compare it: `-0` for negative zero, never an exponent.
     func (v Binary64) Decimal() string {
     	return strconv.FormatFloat(v.value, 'f', -1, 64)
     }

     // Token is the JSON number the wire writes: the shortest digits that read back as the value,
     // laid out as `3.0`, `-0.0`, `0.73`, `1e16` and `1e-6` are, the bytes the Rust server writes.
     func (v Binary64) Token() string {
     	if v.value == 0 {
     		if math.Signbit(v.value) {
     			return "-0.0"
     		}
     		return "0.0"
     	}
     	sign := ""
     	if v.value < 0 {
     		sign = "-"
     	}
     	mantissa, exponent, _ := strings.Cut(strconv.FormatFloat(math.Abs(v.value), 'e', -1, 64), "e")
     	digits := strings.Replace(mantissa, ".", "", 1)
     	power, _ := strconv.Atoi(exponent)
     	point := power + 1
     	count := len(digits)
     	switch {
     	case count <= point && point <= 16:
     		return sign + digits + strings.Repeat("0", point-count) + ".0"
     	case 0 < point && point <= 16:
     		return sign + digits[:point] + "." + digits[point:]
     	case -5 < point && point <= 0:
     		return sign + "0." + strings.Repeat("0", -point) + digits
     	case count == 1:
     		return sign + digits + "e" + strconv.Itoa(power)
     	default:
     		return sign + digits[:1] + "." + digits[1:] + "e" + strconv.Itoa(power)
     	}
     }
     ```
     (`strconv.Atoi` accepts `+16` and `-06`.)
   - add `pub(crate) fn wire_helper(wrapper: &str, constructor: &str) -> String` returning, with
     `{wrapper}`/`{constructor}` substituted (e.g. `primitives.Binary64`, `primitives.NewBinary64`):

     ```go

     // binary64At is the finite Binary64 at this path: a JSON number, rounded once to the nearest
     // binary64, and refused where it rounds to an infinity.
     func binary64At(value any, at string, expected string) ({wrapper}, error) {
     	number, ok := value.(json.Number)
     	if !ok {
     		return {wrapper}{}, DecodeError{At: at, Expected: expected, Found: describes(value)}
     	}
     	parsed, _ := strconv.ParseFloat(number.String(), 64)
     	held, err := {constructor}(parsed)
     	if err != nil {
     		return {wrapper}{}, DecodeError{At: at, Expected: expected, Found: "the number " + number.String()}
     	}
     	return held, nil
     }
     ```
     The token is already a JSON number (the reader uses `UseNumber`, `src/go/http.rs:2781`),
     so `ParseFloat` fails only with `ErrRange`: on overflow it answers ±Inf, which the constructor
     refuses; on underflow a zero, which is the value. `DecodeError` is at `src/go/http.rs:2241`,
     `describes` at 2256, `integerAt` at 2467 is the neighbouring pattern.

2. **`src/go/mod.rs`**:
   - `fn primitive_ctor` (line 220 after B1's shift): add `Primitive::Binary64 => "NewBinary64",`
     before `Primitive::Json => "NewJson",`.
   - `fn primitives_package` (line 686 after the shift), signature becomes
     `fn primitives_package(layout: &Layout, provenance: &Provenance, with_json: bool, with_binary64: bool) -> Artifact`.
     Today it writes `PRIMITIVES_DOC` (with `The four below have no` → `The five below have no`
     when `with_json`), the `package` line, the four wrappers, then `json::PRIMITIVE` when
     `with_json`. Required:
     - the doc substitution: `match (with_json, with_binary64)`: `(false, false)` →
       `PRIMITIVES_DOC` unchanged; `(true, true)` → `json::with_json(PRIMITIVES_DOC, &[("The four below have no", "The six below have no")])`;
       otherwise → the same call with `"The five below have no"`;
     - right after `writeln!(out, "package {}", package.name)`: `if with_binary64 { out.push_str(binary64::IMPORTS); }`;
     - after the `json::PRIMITIVE` push: `if with_binary64 { out.push_str(binary64::PRIMITIVE); }`;
     - add one sentence to the function's doc (lines 680-685 after the shift): `` `Binary64` likewise, wrapping a `float64`. ``
   - the caller (line 339 after the shift): `primitives_package(&layout, provenance, json::used(ir))`
     becomes `primitives_package(&layout, provenance, json::used(ir), binary64::used(ir))`.
   - nothing else in this file (B1 owns the `mod` line).

3. **`src/go/layout.rs:1217`**, `fn primitive`: `Primitive::Binary64 => unreachable!(…)` becomes
   `Primitive::Binary64 => "Binary64",` (qualified by the `qualify` call at line 1230).

4. **`src/go/http.rs`**:
   - `fn encode_primitive` line 998: `Primitive::Binary64 => format!("json.Number({source}.Token())"),`
     (the answer is marshalled with `json.Marshal`, `rendered` at line 2732, which writes a
     `json.Number` verbatim).
   - `fn encode_key` line 1014 and `fn decode_key` line 1204:
     `Primitive::Binary64 => unreachable!("ess-domain refuses a Binary64 map key"),`
   - `fn decode_primitive` line 1179: `Primitive::Binary64 => ("binary64At", "a finite Binary64 number"),`
     (the words are the Rust server's, as the doc at line 1175 requires). `decode_into` (lines
     1095-1110) then emits `{held}, err := binary64At(…)` and uses `held` directly: do not add
     `Binary64` to its `primitive_ctor` list.
   - `fn wire_file` (lines 478-551): after the `for command in …` loop and before `emit.file_at(…)`,
     insert
     `if super::binary64::used(ir) { body.push_str(&super::binary64::wire_helper(&emit.primitive_type(Primitive::Binary64), &emit.primitive_ctor(Primitive::Binary64))); }`
     (`primitive_type` is at `go/mod.rs:210` after the shift; both register the `primitives`
     import). `encoding/json` and `strconv` are already imported there (lines 489-493).

5. **`src/go/invariant.rs:1050-1054`**: today `Primitive::Binary64 | Primitive::Json => { return
   Err(…) }`. Keep that arm for `Primitive::Json` alone and add before it
   `Primitive::Binary64 => wrap("NumberOf", format!("{at}.Decimal()")),` (the `Decimal` arm at
   line 1045 is the pattern).

6. **`src/go/behaviour.rs`**, `fn leaf` (line 4536): before the `Kind::Number(_) => {` arm (line
   4546) add `Kind::Number(Primitive::Binary64) => ReadLeaf::Expression(format!("{at}.Decimal()")),`.
   Without it a guard, filter or order over a `Binary64` would call `strconv.FormatInt` on a struct.
   Guards then compare with `compareNumbers` and orders with `numberOrder` over that text, as for
   `Decimal`.

7. **`src/go/store/identity.rs:38`**: the message becomes
   `unreachable!("a Binary64 identity is refused by failure::binary64")`.

8. **New `tests/fixtures/binary64-served-go-harness/main.go`**: a copy of
   `tests/fixtures/integer-spellings-go-harness/main.go` (227 lines) with these replacements and
   nothing else: imports `example.invalid/demo/components/boardservice` →
   `example.invalid/demo/components/searchservice` and `example.invalid/demo/types/board` →
   `example.invalid/demo/types/search`; `board.BoardSnapshot` → `search.DocSnapshot`;
   `board.BoardId` → `search.DocId`; `.Data.BoardId` → `.Data.DocId`; the field `boards` → `docs`;
   `GenerateDemoBoardBoardId` → `GenerateDemoSearchDocId`; `board.NewBoardId(` → `search.NewDocId(`;
   `server.ServeBoardService(` → `server.ServeSearchService(`;
   `behaviour.Ports{BoardStorage: scenario` → `behaviour.Ports{DocStorage: scenario`;
   `boardservice.New(` → `searchservice.New(`; in comments `integer_spellings.rs` →
   `binary64_go.rs` and "a board" → "a document". These are the names 0.97.1 generates for the
   fixture (checked on its `Decimal` twin: `DocStorage` with `Get`/`Put`/`Delete`/`List`,
   `GenerateDemoSearchDocId`, `search.NewDocId(primitives.Uuid)`, `server.ServeSearchService`,
   `searchservice.New`, `system.NewSystem`, module `example.invalid/demo`).

9. **New `tests/binary64_go.rs`** (see Tests first).

## Tests first

`tests/binary64_go.rs`, starting with `#[path = "binary64_served/support.rs"] mod support;`, plus
copies of `shown`, `write_tree`, `has_go` and `go` from `tests/integer_spellings.rs` (lines 108-145).

- `the_go_target_synthesizes_every_carried_binary64_position`: `synthesize_for(&support::model(),
  Target::Go)` is `Ok`; `synthesis.plan.counts()` is 14 generated, 0 obligations, 0 refused;
  artifact `types/primitives/primitives.go` contains `type Binary64 struct {`,
  `func (v Binary64) Token() string {` and `The five below have no`; artifact `server/wire.go`
  contains `func binary64At(`, contains `binary64At(` at least twice, and contains
  `json.Number(` and `.Token())`. Red before: the target refuses.
- `a_go_model_without_binary64_keeps_its_bytes`: with `support::MODEL.replace("Binary64", "Decimal")`,
  `primitives.go` contains no `Binary64` and still `The four below have no`, and `wire.go` contains
  no `binary64At`. Green before and after.
- `a_binary64_identity_is_still_refused_by_the_go_target`: with `support::MODEL.replace("{name:
  demo.search.DocId, kind: newtype, of: Uuid}", "{name: demo.search.DocId, kind: newtype, of:
  Binary64}")` (assert changed), the Go target refuses with one cause, `MissingRepresentation`,
  sources `["types.demo.search.DocId.of"]`, detail starting `a Binary64 here is a key, a lookup or
  a delivery route`. Red before.
- `the_go_server_carries_a_finite_binary64`: **one** test for everything that needs Go, so the
  module is built once per run (under nextest each test is its own process and a `OnceLock`
  shares nothing). Returns early after `has_go()` prints `skipped: …` when there is no `go`.
  Otherwise: synthesize to Go, write the tree under
  `Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("binary64-go-{}", std::process::id()))/go-demo`;
  assert `gofmt -l <tree>` prints nothing; write the harness module beside it exactly as
  `integer_spellings.rs:196-225` does (`go.mod` text with module `binary64goharness`,
  `require example.invalid/demo v0.0.0`, `replace example.invalid/demo => ../go-demo`; copy
  `tests/fixtures/binary64-served-go-harness/main.go`; `go build -o <kept>/go .` with
  `GOFLAGS=-mod=mod GOPROXY=off GOWORK=off`), assert it builds, and `eprintln!` the build time.
  Then run, in order, `support::writes_every_token("go", &binary)`,
  `support::refuses_what_no_binary64_holds("go", &binary)`, `support::compares_by_value("go", &binary)`,
  `support::passes_the_synthesized_suite("go", &binary)`. Red before (synthesis refuses).

**Time.** The three synthesis tests take well under 1 s. The Go test is the build plus about 60
harness round trips: the same build shape as `integer_spellings`' Go harness, expected under 30 s
with a cold Go build cache and under 5 s with a warm one. Report the time you measured; if it is
over 60 s, say so in your report (the coordinator then moves the build behind a Taskfile harness,
as unit B2 does for Rust) and do not raise any timeout.

If `-1e-400` comes back as `0.0` instead of `-0.0` from the Go server only, do not change
`support.rs`: report the Go behaviour.

## Commands

From the worktree root:

1. `cargo test -p ess-synth --locked --test binary64_go -- --nocapture` — 4 passed, the build time
   printed, no `skipped` line on a machine with `go`.
2. `cargo test -p ess-synth --locked --test binary64_plan` — passes (B1's).
3. One command each, all pass: `cargo test -p ess-synth --locked --test go_owner`,
   `--test json_go`, `--test feasibility`, `--test adversary_served_pass2_go`,
   `--test generated_view_queries`, `--test invariant_check`.
4. Byte pins (AGENTS.md) this change could move, one command each, all pass, and `git status`
   shows no committed generated file changed: `--test go`, `--test served_codec_names`,
   `--test http_binding_rust`, `--test adversary_related_guard_behaviour_pass1`.
5. `cargo clippy -p ess-synth --all-targets --locked -- -D warnings` — clean.
6. `slot task fmt-check` — clean.

## Files this unit owns

- `crates/generate/ess-synth/src/go/binary64.rs`
- `crates/generate/ess-synth/src/go/mod.rs`
- `crates/generate/ess-synth/src/go/layout.rs`
- `crates/generate/ess-synth/src/go/http.rs`
- `crates/generate/ess-synth/src/go/invariant.rs`
- `crates/generate/ess-synth/src/go/behaviour.rs`
- `crates/generate/ess-synth/src/go/store/identity.rs`
- `crates/generate/ess-synth/tests/binary64_go.rs` (new)
- `crates/generate/ess-synth/tests/fixtures/binary64-served-go-harness/main.go` (new)

Must not touch: `src/failure.rs`, `src/determined.rs`, `src/view_query.rs`,
`tests/binary64_served/support.rs`, `tests/fixtures/binary64-served/model.yaml.in` (B1); everything
under `src/rust/` (B2) and `src/web/` (B4); `src/go/json.rs` (`with_json` is only called);
`Taskfile.yml`, `AGENTS.md`, `CHANGELOG.md`, `.github/workflows/*`, `.config/nextest.toml`,
`crates/edge/ess-xtask/**`.

## For the coordinator at integration

- No `Taskfile.yml`, workflow or `ci_lanes.rs` change: the Go harness is built in the test, skipped
  aloud where there is no `go`, as `integer_spellings` does. If the implementor reports the Go test
  over 60 s, add a `binary64-go-harness` task on the pattern of unit B2's and an
  `ESS_BINARY64_GO_HARNESS` variable read like `ESS_BINARY64_HARNESS`.
- `CHANGELOG.md`, `docs/`, `website/`: in unit B4's coordinator section.

## Out of scope

- Any Rust or web change (B2, B4).
- A Binary64 map key, identity, view parameter, literal, or `coalesce` default; the
  `unreachable!` arms and `primitive_ctor`'s panic for the other primitives stay.
- The standalone Go codec of `schema-contract` (it writes `1e+06`); not touched here.
- Go `encoding/json` struct tags or a `MarshalJSON` method on the wrapper.
- Editing `support.rs`: report a case that looks wrong instead.

## Acceptance

- The generated Go module is gofmt-clean and builds, and its server writes every token of the
  wire table as the Rust server does, refuses every listed value with `400` naming `hit.score`,
  compares by value, and passes the synthesized suite (command 1).
- A Binary64 identity is refused by the Go target with the uncarried detail (command 1).
- A model without `Binary64` keeps its `primitives.go` and `wire.go` bytes; the byte pins pass
  (commands 1, 4).
- Clippy and fmt are clean (commands 5, 6).

## Depends on

`each-code-target-admits-binary64-through-a-switch-of-its-own` (B1), which provides
`src/go/binary64.rs` with `pub(crate) const CARRIED: bool = false;` and `pub(crate) mod binary64;`
in `src/go/mod.rs`; `failure::binary64` reading it; `Kind::Number(Primitive::Binary64)`; the
fixture model; and `tests/binary64_served/support.rs` with `pub fn model() -> EssIr`,
`pub const MODEL: &str`, `pub fn writes_every_token(language: &str, binary: &Path)`,
`pub fn refuses_what_no_binary64_holds(language: &str, binary: &Path)`,
`pub fn compares_by_value(language: &str, binary: &Path)`,
`pub fn passes_the_synthesized_suite(language: &str, binary: &Path)`.
Independent of B2: the two share no file and may run in parallel.
