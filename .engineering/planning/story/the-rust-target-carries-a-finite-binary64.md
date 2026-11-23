---
format: aep.planning-md/3
id: story:the-rust-target-carries-a-finite-binary64
kind: story
status: implemented
title: The Rust target carries a finite Binary64 in every value position, on the wire as a JSON number
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-code-target-carries-a-finite-binary64
- depends_on: story:each-code-target-admits-binary64-through-a-switch-of-its-own
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/rust/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/feasibility.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/store/identity.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binary64_rust.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/binary64-served-harness/harness.toml
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/binary64-served-harness/main.rs
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:43:28Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-11-21T23:43:29Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-11-23T01:56:57Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

- `--target rust` synthesizes a model with a `Binary64` in any value position; only an uncarried
  one (identity, view parameter, actor attribute, binding, conversion) is refused, with B1's detail.
- The types crate holds `primitives::Binary64` (finite by construction); the server reads a JSON
  number into it and writes the layout below, byte for byte.
- The generated server passes the wire table and the fixture's suite through a harness built once
  by `task binary64-harness`. No format version changes.

## What to change

Line numbers are at `origin/main` `f3e4477974`; unit B1 has added `pub(crate) mod binary64;` to
`src/rust/mod.rs` (one line after line 30, so later lines there are +1) and created
`src/rust/binary64.rs`. Paths are under `crates/generate/ess-synth/` unless written in full.

**Wire layout** of a finite value `v`, used by `token()` below. Zero is `0.0`, or `-0.0` when the
sign bit is set. Otherwise take the shortest round-trip digits `d1…dn` and the decimal exponent
`e` of `|v|` from `format!("{:e}", v.abs())` (`1.5e0`, `1e16`, `5e-324`), and let `p = e + 1`:

| case | written |
|---|---|
| `n ≤ p ≤ 16` | digits, then `p − n` zeros, then `.0` (`3.0`, `1000000000000000.0`) |
| `0 < p < n` (so `p ≤ 16`) | digits with the point after `p` (`123.456`) |
| `−5 < p ≤ 0` | `0.`, `−p` zeros, the digits (`0.73`, `0.00001`) |
| `n == 1`, otherwise | `d1` `e` `e` (`1e16`, `1e-6`, `5e-324`) |
| otherwise | `d1` `.` `d2…dn` `e` `e` (`1.5e300`) |

A `-` precedes a negative value; the exponent has `-` only, never `+` or leading zeros. This is the
layout `serde_json` writes for an `f64`.

1. **`src/rust/binary64.rs`** (created by B1 with `CARRIED = false`):
   - set `pub(crate) const CARRIED: bool = true;`
   - add `pub(crate) fn used(ir: &ess_compiler::EssIr) -> bool { !ess_compiler::binary64::locations(ir).is_empty() }`
     with a doc saying the wrapper and the wire helper are emitted only then, so every other
     model keeps its bytes.
   - add `pub(crate) const PRIMITIVE: &str = r##"…"##;`, the text appended to the generated
     `primitives.rs`, exactly:

     ```rust

     /// A finite IEEE 754 binary64, preserving the sign of zero.
     ///
     /// The field is private, so a NaN or an infinity cannot be held: [`Binary64::new`] refuses
     /// both. Equality is IEEE 754's, so `+0` and `-0` are equal, and `Eq` is lawful because no NaN
     /// can be held. There is no `Hash` and no `Ord`: guards and orders compare the decimal
     /// `Display` writes. The sign of zero stays readable through [`Binary64::get`],
     /// [`Binary64::token`] and `Display`.
     #[derive(Debug, Clone, Copy)]
     pub struct Binary64(f64);

     /// Why a value is not a [`Binary64`].
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub enum Binary64Error {
         /// A NaN or an infinity.
         NonFinite,
     }

     impl std::fmt::Display for Binary64Error {
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             f.write_str("Binary64 requires a finite value")
         }
     }

     impl std::error::Error for Binary64Error {}

     impl Binary64 {
         /// The value, when it is finite.
         ///
         /// # Errors
         ///
         /// [`Binary64Error::NonFinite`] for a NaN or an infinity.
         pub fn new(value: f64) -> Result<Self, Binary64Error> {
             if value.is_finite() {
                 Ok(Self(value))
             } else {
                 Err(Binary64Error::NonFinite)
             }
         }

         /// The wrapped value.
         #[must_use]
         pub fn get(self) -> f64 {
             self.0
         }

         /// A JSON number token, rounded once to the nearest binary64; `None` where it rounds to
         /// an infinity or spells no number.
         #[must_use]
         pub fn from_token(token: &str) -> Option<Self> {
             token.parse::<f64>().ok().and_then(|value| Self::new(value).ok())
         }

         /// The JSON number the wire writes: the shortest digits that read back as the value,
         /// laid out as `3.0`, `-0.0`, `0.73`, `1e16` and `1e-6` are.
         #[must_use]
         pub fn token(&self) -> String {
             let value = self.0;
             if value == 0.0 {
                 return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
             }
             let sign = if value < 0.0 { "-" } else { "" };
             let scientific = format!("{:e}", value.abs());
             let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
             let digits = mantissa.replacen('.', "", 1);
             let power: i32 = exponent.parse().unwrap_or(0);
             let point = power + 1;
             let count = i32::try_from(digits.len()).unwrap_or(i32::MAX);
             let zeros = |n: i32| "0".repeat(usize::try_from(n).unwrap_or(0));
             if count <= point && point <= 16 {
                 format!("{sign}{digits}{}.0", zeros(point - count))
             } else if 0 < point && point <= 16 {
                 let (whole, fraction) = digits.split_at(usize::try_from(point).unwrap_or(0));
                 format!("{sign}{whole}.{fraction}")
             } else if -5 < point && point <= 0 {
                 format!("{sign}0.{}{digits}", zeros(-point))
             } else if count == 1 {
                 format!("{sign}{digits}e{power}")
             } else {
                 format!("{sign}{}.{}e{power}", &digits[..1], &digits[1..])
             }
         }
     }

     impl PartialEq for Binary64 {
         fn eq(&self, other: &Self) -> bool {
             self.0 == other.0
         }
     }

     impl Eq for Binary64 {}

     impl std::fmt::Display for Binary64 {
         /// The shortest plain decimal that reads back as the value: `-0`, `0.0000001`, never an
         /// exponent. Guards, filters, invariants and orders compare this text by value.
         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
             std::fmt::Display::fmt(&self.0, f)
         }
     }
     ```

   - add `pub(crate) fn wire_helper(types: &str) -> String`, returning this text with every
     `TYPES` replaced by `types` (`WIRE_HELPER.replace("TYPES", types)` over a private const):

     ```rust

     /// The finite `Binary64` at this path: a JSON number, rounded once to the nearest binary64,
     /// and refused where it rounds to an infinity.
     #[allow(dead_code)]
     fn binary64_at(value: &json::Value, at: &str) -> Result<TYPES::primitives::Binary64, json::DecodeError> {
         match value {
             json::Value::Number(token) => TYPES::primitives::Binary64::from_token(token).ok_or_else(|| json::DecodeError {
                 at: at.to_owned(),
                 expected: "a finite Binary64 number".to_owned(),
                 found: format!("the number {token}"),
             }),
             other => Err(json::DecodeError::of(at, "a finite Binary64 number", other)),
         }
     }
     ```
     `json::Value::Number(String)` keeps the token as sent (`src/rust/json.rs:62`), the request
     reader has already held it to the JSON number grammar, and `json::DecodeError`'s fields
     `at`, `expected`, `found` are public (`src/rust/json.rs:677-683`); `DecodeError::of` is at
     line 694.

2. **`src/rust/mod.rs`**, `fn primitives_module` (line 429 before B1, 430 after): after
   `out.push_str(PRIMITIVES);` insert
   `if binary64::used(ir) { out.push_str(binary64::PRIMITIVE); }`. Leave `PRIMITIVES` unchanged.

3. **`src/rust/layout.rs:522`**, `pub fn primitive`: `Primitive::Binary64 => unreachable!(…)` becomes
   `Primitive::Binary64 => "crate::primitives::Binary64",`.

4. **`src/rust/wire.rs`**:
   - `fn module` (lines 63-101): after the `for command in …` loop and before `out` is returned,
     insert `if super::binary64::used(surface.ir()) { out.push_str(&super::binary64::wire_helper(surface.types())); }`.
     The web target builds its bridge's wire with this same function (`web/mod.rs:320`).
   - `fn encode_primitive` line 863: `Primitive::Binary64 => format!("out.push_str(&{}.token());", place(expr)),`
   - `fn encode_key` line 879 and `fn decode_key` line 1042:
     `Primitive::Binary64 => unreachable!("ess-domain refuses a Binary64 map key"),`
     (`Map<Binary64, _>` is refused by the domain, as `Json` keys are on the next line).
   - `fn decode_primitive` line 1011: `Primitive::Binary64 => format!("binary64_at({value}, {at})?"),`

5. **`src/rust/feasibility.rs:113`**, `fn reference`: delete the `Primitive::Binary64 =>
   unreachable!(…)` line and add `| Primitive::Binary64` to the no-op group at lines 117-124
   (it is spelled `crate::primitives::Binary64`, from the crate root, like the other wrappers).

6. **`src/rust/invariant.rs:756-760`**: today `Primitive::Binary64 | Primitive::Json => { return
   Err(format!("`{path}` is a `{name}`, which the check does not read")); }`. Split it: keep that
   arm for `Primitive::Json` alone, and add before it

   ```rust
   Primitive::Binary64 => {
       let number = |at: &str| format!("iv::Fact::number(&{at}.to_string())");
       reach.and_then(number, number)
   }
   ```
   (the `Primitive::Decimal` arm at lines 740-743 is the pattern; `Fact::number(&str) -> Option`
   is at line 1150 and parses the plain decimal `Display` writes).

7. **`src/rust/store/identity.rs:42`**: the message becomes
   `unreachable!("a Binary64 identity is refused by failure::binary64")`.

8. **New `tests/fixtures/binary64-served-harness/harness.toml`**: a copy of
   `tests/fixtures/integer-spellings-harness/harness.toml` with `name = "binary64-harness"` and
   the four dependencies `demo-types`, `demo-server`, `search-service`, `demo-system`, each
   `{ path = "../demo/crates/<name>" }`.

9. **New `tests/fixtures/binary64-served-harness/main.rs`**: a copy of
   `tests/fixtures/integer-spellings-harness/main.rs` (145 lines) with these replacements and
   nothing else: `demo_server::board_service as surface` → `demo_server::search_service as surface`;
   `BoardStorage` → `DocStorage`; `demo_types::board` → `demo_types::search`; `board::BoardSnapshot`
   → `search::DocSnapshot`; `board::BoardId` → `search::DocId`; `board_id` → `doc_id`; the field
   `boards` → `docs`; `generate_demo_board_board_id` → `generate_demo_search_doc_id`;
   `board_service::BoardService::new(` → `search_service::SearchService::new(`; in comments,
   `tests/integer_spellings.rs` → `tests/binary64_rust.rs` and "a board" → "a document". These
   names are what 0.97.1 generates for the fixture (checked on its `Decimal` twin: trait
   `DocStorage` with `get`/`put`/`delete`/`list`, `Context::generate_demo_search_doc_id`,
   `demo_server::search_service::dispatch`, `search_service::SearchService::new`,
   `demo_system::System::new`, `search::DocId(pub crate::primitives::Uuid)`).

10. **New `tests/binary64_rust.rs`** (see Tests first), starting with
    `#[path = "binary64_served/support.rs"] mod support;`.

## Tests first

`tests/binary64_rust.rs`. It never runs cargo (`crates/edge/ess-xtask/tests/no_test_builds.rs`):
the harness is built once by the Taskfile task and named by `ESS_BINARY64_HARNESS`.

- `const HARNESS: &str = "ESS_BINARY64_HARNESS";` and `fn rust_harness() -> PathBuf`, a copy of
  `tests/integer_spellings.rs:149-180` (doc and function) whose panics name `task binary64-harness` instead of
  `task integer-spellings-harness`.
- `the_rust_target_synthesizes_every_carried_binary64_position`: `synthesize_for(&support::model(),
  Target::Rust)` is `Ok`; `synthesis.plan.counts()` is 14 generated, 0 obligations, 0 refused (the
  fixture's `Decimal` twin under 0.97.1); `synthesis.target.is_none()` (the twin has no target
  report); artifact `crates/demo-types/src/primitives.rs` contains `pub struct Binary64(f64);` and
  `impl Eq for Binary64 {}`; artifact `crates/demo-server/src/wire.rs` contains
  `fn binary64_at(`, contains `binary64_at(` at least twice, and contains `.token());`.
  Red before: the target refuses.
- `a_model_without_binary64_keeps_its_bytes`: with `support::MODEL.replace("Binary64", "Decimal")`,
  `primitives.rs` contains no `Binary64` and `wire.rs` no `binary64_at`. Green before and after.
- `a_binary64_identity_is_still_refused_with_the_uncarried_detail`: with
  `support::MODEL.replace("{name: demo.search.DocId, kind: newtype, of: Uuid}", "{name: demo.search.DocId, kind: newtype, of: Binary64}")`
  (assert changed), the Rust target refuses with exactly one cause, code `MissingRepresentation`,
  sources `["types.demo.search.DocId.of"]`, detail `a Binary64 here is a key, a lookup or a
  delivery route, which this target does not carry; it carries one in entity fields, command input
  and response, event and error fields, and view fields and row`. Red before (old detail, one cause per
  location).
- `the_rust_server_writes_every_binary64_token_in_the_wire_layout`:
  `support::writes_every_token("rust", &rust_harness())`.
- `the_rust_server_refuses_what_no_finite_binary64_holds`:
  `support::refuses_what_no_binary64_holds("rust", &rust_harness())`.
- `the_rust_server_compares_binary64_by_value`: `support::compares_by_value("rust", &rust_harness())`.
- `the_synthesized_suite_passes_against_the_rust_server`:
  `support::passes_the_synthesized_suite("rust", &rust_harness())`.

The four server tests are red before (no harness can be built: synthesis refuses). Each starts
one harness process and sends under 40 lines: well under 5 s. The literal tables are in
`support.rs` (`WRITTEN`, `REFUSED`), e.g. `0.73`→`0.73`, `-0`→`-0.0`, `3`→`3.0`, `1e16`→`1e16`,
`1e15`→`1000000000000000.0`, `0.000001`→`1e-6`, `9007199254740993`→`9007199254740992.0`,
`1e-400`→`0.0`, `-1e-400`→`-0.0`; `1e400` → `400` with
`hit.score: expected a finite Binary64 number, found the number 1e400`.

## Commands

From the worktree root:

1. Build the harness, as the Taskfile task will (one line each; build `ess` first):
   - `cargo build -p ess-cli --locked --bin ess`
   - `rm -rf target/binary64-harness && mkdir -p target/binary64-harness/harness/src target/nextest-archives`
   - `cp crates/generate/ess-synth/tests/fixtures/binary64-served/model.yaml.in target/binary64-harness/model.yaml`
   - `target/debug/ess generate synthesize --path target/binary64-harness/model.yaml --target rust --out target/binary64-harness/demo` — prints `14 capabilities: 14 generated, 0 obligation(s), 0 refused`
   - `cp crates/generate/ess-synth/tests/fixtures/binary64-served-harness/harness.toml target/binary64-harness/harness/Cargo.toml`
   - `cp crates/generate/ess-synth/tests/fixtures/binary64-served-harness/main.rs target/binary64-harness/harness/src/main.rs`
   - `env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS -u RUSTC_WRAPPER CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings' cargo build --offline --quiet --manifest-path target/binary64-harness/harness/Cargo.toml --target-dir target/binary64-harness/target` — no warnings, no errors (this compiles the generated workspace)
   - `cp target/binary64-harness/target/debug/harness target/nextest-archives/binary64-harness`
2. The single-crate layout compiles too:
   `target/debug/ess generate synthesize --path target/binary64-harness/model.yaml --target rust --layout crate --out target/binary64-single`, then
   `env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS -u RUSTC_WRAPPER RUSTFLAGS='-D warnings' cargo check --offline --manifest-path target/binary64-single/Cargo.toml --features server --target-dir target/binary64-single/target` — clean.
3. `ESS_BINARY64_HARNESS=$PWD/target/nextest-archives/binary64-harness cargo test -p ess-synth --locked --test binary64_rust` — 7 passed.
4. `cargo test -p ess-synth --locked --test binary64_plan` — passes (B1's).
5. One command each, all pass: `cargo test -p ess-synth --locked --test feasibility`,
   `--test single_crate_layout`, `--test json_primitive`, `--test generated_view_queries`,
   `--test invariant_check`.
6. Byte pins (AGENTS.md) this change could move: `--test served_codec_names`, `--test go`,
   `--test http_binding_rust`, `--test adversary_related_guard_behaviour_pass1`, one command each,
   all pass, and `git status` shows no committed generated file changed (no example uses Binary64).
7. `cargo clippy -p ess-synth --all-targets --locked -- -D warnings` — clean.
8. `slot task fmt-check` — clean.
9. `rm -rf target/binary64-harness target/binary64-single` (keep `target/nextest-archives/binary64-harness`).

## Files this unit owns

- `crates/generate/ess-synth/src/rust/binary64.rs`
- `crates/generate/ess-synth/src/rust/mod.rs`
- `crates/generate/ess-synth/src/rust/layout.rs`
- `crates/generate/ess-synth/src/rust/wire.rs`
- `crates/generate/ess-synth/src/rust/feasibility.rs`
- `crates/generate/ess-synth/src/rust/invariant.rs`
- `crates/generate/ess-synth/src/rust/store/identity.rs`
- `crates/generate/ess-synth/tests/binary64_rust.rs` (new)
- `crates/generate/ess-synth/tests/fixtures/binary64-served-harness/harness.toml` (new)
- `crates/generate/ess-synth/tests/fixtures/binary64-served-harness/main.rs` (new)

Must not touch: `src/failure.rs`, `src/determined.rs`, `src/view_query.rs`,
`tests/binary64_served/support.rs`, `tests/fixtures/binary64-served/model.yaml.in`,
`tests/binary64_plan.rs` (B1); everything under `src/go/` (B3); everything under `src/web/` (B4);
`src/rust/json.rs` (the fixed reader must keep its bytes); `Taskfile.yml`, `AGENTS.md`,
`CHANGELOG.md`, `.github/workflows/*`, `.config/nextest.toml`, `crates/edge/ess-xtask/**`.

## For the coordinator at integration

**`Taskfile.yml`** (the test fails, naming the task, until this lands):

- top-level `vars:`, after `CLAP_SHARED_BINARY: …` (line 17):
  `  BINARY64_HARNESS: '{{.ROOT_DIR}}/{{.NEXTEST_ARCHIVES}}/binary64-harness'`
- `env:` of `test` (after line 84), of `test-packages` (after line 112) and of `test-shard`
  (after line 321), beside `ESS_CLAP_SHARED_BINARY`:
  `      ESS_BINARY64_HARNESS: '{{.BINARY64_HARNESS}}'`
- `cmds:` of `test` (line 90), `test-packages` (118) and `test-archive-workspace` (154), after `- task: clap-shared-binary`:
  `      - task: binary64-harness`
- a new task after `integer-spellings-harness` (after line 182):

  ```yaml
    # The harness ess-synth's `binary64_rust` test runs against the generated Rust server, built
    # here once for the reason `integer-spellings-harness` gives, and handed to the test as
    # ESS_BINARY64_HARNESS.
    binary64-harness:
      desc: Build the Rust harness ess-synth's binary64_rust test runs, at BINARY64_HARNESS.
      vars:
        WORK: target/binary64-harness
        FIXTURES: crates/generate/ess-synth/tests/fixtures
      cmds:
        - cargo build --workspace --exclude ess-xtask --locked --bin ess
        - rm -rf {{.WORK}}
        - mkdir -p {{.WORK}}/harness/src {{.NEXTEST_ARCHIVES}}
        - cp {{.FIXTURES}}/binary64-served/model.yaml.in {{.WORK}}/model.yaml
        - target/debug/ess generate synthesize --path {{.WORK}}/model.yaml --target rust --out {{.WORK}}/demo
        - cp {{.FIXTURES}}/binary64-served-harness/harness.toml {{.WORK}}/harness/Cargo.toml
        - cp {{.FIXTURES}}/binary64-served-harness/main.rs {{.WORK}}/harness/src/main.rs
        - >-
          env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS -u RUSTC_WRAPPER
          CARGO_INCREMENTAL=0 RUSTFLAGS='-D warnings'
          cargo build --offline --quiet --manifest-path {{.WORK}}/harness/Cargo.toml --target-dir {{.WORK}}/target
        - cp {{.WORK}}/target/debug/harness {{.BINARY64_HARNESS}}
        - rm -rf {{.WORK}}
  ```

  No workflow change: the harness lands in `target/nextest-archives/`, which the shards already
  receive with the archives. `ci_lanes.rs` checks no harness list (checked: no `HARNESS` there).
  Run `task ci-lint` after the Taskfile edit.

**`AGENTS.md`**, the harness paragraph (lines 227-228 at `origin/main`): after
"The same holds for `task append-set-harness` (`append_set`, through `ESS_APPEND_SET_HARNESS`),"
insert "`task binary64-harness` (`binary64_rust`, through `ESS_BINARY64_HARNESS`),".

`CHANGELOG.md` and `docs/`/`website/`: the bullet and the doc edits for the whole feature are in
unit B4's coordinator section.

## Out of scope

- Any Go or web change (B3, B4). Flipping `web/binary64.rs`'s `CARRIED`.
- A Binary64 map key, identity, view parameter, literal in `sets:`/`payload:`, or `coalesce`
  default; the `unreachable!` arms that state those stay.
- serde impls on the wrapper (the generated tree takes no dependency), `Hash`, `Ord`, arithmetic.
- Changing `src/rust/json.rs`, the `Decimal` wrapper, or `PRIMITIVES`' text.
- Editing `support.rs`: if a shared case looks wrong, report it rather than change it.

## Acceptance

- The generated Rust workspace and the single-crate layout compile with warnings denied
  (commands 1, 2).
- `binary64_rust`: every token of the wire table is written as listed, every refused value answers
  `400` naming `hit.score`, the filter and order compare by value, and the synthesized suite passes
  (command 3).
- A Binary64 identity is refused with the uncarried detail (command 3).
- The byte pins and neighbouring targets pass with no committed file changed (commands 5, 6).
- Clippy and fmt are clean (commands 7, 8).

## Depends on

`each-code-target-admits-binary64-through-a-switch-of-its-own` (B1), which provides:
`src/rust/binary64.rs` with `pub(crate) const CARRIED: bool = false;` and
`pub(crate) mod binary64;` in `src/rust/mod.rs`; `failure::binary64` reading it;
`Kind::Number(Primitive::Binary64)` in `determined.rs`; the fixture
`tests/fixtures/binary64-served/model.yaml.in`; and `tests/binary64_served/support.rs` with
`pub fn model() -> EssIr`, `pub const MODEL: &str`, `pub fn writes_every_token(language: &str, binary: &Path)`,
`pub fn refuses_what_no_binary64_holds(language: &str, binary: &Path)`,
`pub fn compares_by_value(language: &str, binary: &Path)`,
`pub fn passes_the_synthesized_suite(language: &str, binary: &Path)`.
