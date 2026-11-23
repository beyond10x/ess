---
format: aep.planning-md/3
id: story:the-web-target-carries-a-finite-binary64
kind: story
status: implemented
title: The web target carries a finite Binary64 through its bridge and gives it a number control on the page
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-code-target-carries-a-finite-binary64
- depends_on: story:each-code-target-admits-binary64-through-a-switch-of-its-own
- depends_on: story:the-rust-target-carries-a-finite-binary64
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/web/binary64.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/page.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binary64_web.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:43:45Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-21T23:43:46Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-23T01:56:58Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

- `--target web` synthesizes a model with a `Binary64` in any carried position, its bridge using
  B2's codec; only an uncarried position is refused, with B1's detail.
- The page gives a `binary64` field `<input type="number" step="any">` (finite number or absent);
  `TARGET.md` states what the page loses. A model without `Binary64` keeps every byte.
- No format version changes (`ess-browser-catalog/1` already spells the primitive `binary64`).

## What to change

Line numbers are at `origin/main` `f3e4477974`; unit B1 has inserted `pub(crate) mod binary64;`
before line 31 of `src/web/mod.rs` (so later lines there are +1) and created `src/web/binary64.rs`;
unit B2 has created `crate::rust::binary64::used(ir: &EssIr) -> bool` and the Rust codec. Paths
are under `crates/generate/ess-synth/`.

1. **`src/web/binary64.rs`**: set `pub(crate) const CARRIED: bool = true;`. Nothing else.
   (`failure::carries_binary64` admits Web only when `crate::rust::binary64::CARRIED` is also
   `true`, which B2 set.)

2. **`src/web/page.rs`**:
   - after `const JSON_PRIMITIVE` (it ends at line 149 with `if (name === "boolean") {"#;`), add:

     ```rust
     /// Where the fixed script builds the `Integer` control, the anchor of the `Binary64` one.
     const INTEGER_PRIMITIVE: &str = "  if (name === \"integer\") {";

     /// The same, with a `Binary64` control ahead of it.
     ///
     /// A number input that accepts any step; it reads as the finite number it holds, and
     /// otherwise marks itself invalid and reads as absent, so the bridge names the missing value
     /// rather than receiving `Infinity` or text. The page holds the value as a JavaScript number
     /// (see the `Binary64` weakening in `TARGET.md`).
     const BINARY64_PRIMITIVE: &str = r#"  if (name === "binary64") {
         const node = /** @type {HTMLInputElement} */ (element("input", { type: "number", step: "any", value: "0" }));
         return {
           node,
           read: () => {
             const value = node.valueAsNumber;
             if (Number.isFinite(value)) {
               node.setCustomValidity("");
               return value;
             }
             node.setCustomValidity("a finite number");
             node.reportValidity();
             return undefined;
           },
         };
       }
       if (name === "integer") {"#;
     ```
     The JavaScript inside the raw string is indented two spaces per level from column 0 (the
     first line starts with two spaces, `const node` with four), exactly like the `integer` branch
     at line 262; the block above shows it shifted by the Markdown indentation only.
   - `fn script` (line 176): after the `Json` substitution (lines 187-189) add
     `if crate::rust::binary64::used(bridge.ir) { script = script.replacen(INTEGER_PRIMITIVE, BINARY64_PRIMITIVE, 1); }`.
     Both substitutions compose: `JSON_PRIMITIVE` ends at the `boolean` branch, before the
     `integer` one.
   - in the unit tests (`the_json_control_replaces_exactly_the_primitive_switch_the_script_writes`
     is at line 729), add
     `fn the_binary64_control_replaces_exactly_the_integer_branch_the_script_writes()`:
     `assert_eq!(SCRIPT.matches(INTEGER_PRIMITIVE).count(), 1); assert!(BINARY64_PRIMITIVE.ends_with(INTEGER_PRIMITIVE)); assert!(!SCRIPT.contains("binary64"));`

3. **`src/web/mod.rs`**:
   - `fn weakenings` (line 434 before B1's shift): after the `Json` push add
     `if crate::rust::binary64::used(ir) { out.push(binary64_weakening()); }`, and extend its doc
     (lines 432-433): `A model that uses \`Binary64\` gets one more the same way: [\`binary64_weakening\`].`
   - after `fn json_weakening` (ends line 470), add:

     ```rust
     /// What the page does to a `Binary64` value, which the bridge does not.
     ///
     /// The bridge carries it as the Rust types crate's `primitives::Binary64` and writes it as the
     /// Rust server does. The page's glue parses every answer with `JSON.parse` and sends with
     /// `JSON.stringify`, so on the page the value is a JavaScript number.
     fn binary64_weakening() -> TargetWeakening {
         TargetWeakening {
             guarantee: "a `Binary64` value crosses with its sign of zero and in its shortest \
                         digits, as the Rust server writes it"
                 .to_owned(),
             instead: "the bridge carries it unchanged, as the Rust types crate's \
                       `primitives::Binary64`; the page holds it as a JavaScript number, so a \
                       negative zero shows and is sent as `0`, and a number shows in ECMAScript's \
                       spelling (`1e+21`, `0.000001`). A number control that holds no finite \
                       number reads as absent, and the bridge names the missing value"
                 .to_owned(),
             affects: vec![
                 CapabilityKind::DomainType,
                 CapabilityKind::CommandContract,
                 CapabilityKind::EventType,
                 CapabilityKind::ErrorType,
                 CapabilityKind::ViewType,
             ],
         }
     }
     ```
   - nothing else in this file (the `failure::binary64` call at line 300 stays; B1 owns the `mod`
     line).

## Tests first

New `tests/binary64_web.rs`, starting with `#[path = "binary64_served/support.rs"] mod support;`.
No test compiles generated code.

- `the_web_target_carries_a_finite_binary64`: `synthesize_for(&support::model(), Target::Web)` is
  `Ok`; artifact `index.html` contains `if (name === "binary64") {` and `step: "any"`; artifact
  `TARGET.md` contains `a \`Binary64\` value crosses with its sign of zero`; artifact
  `crates/demo-web/src/wire.rs` contains `fn binary64_at(` and `.token());`. Red before: the target
  refuses (`web/binary64.rs` `CARRIED` is `false`).
- `a_web_model_without_binary64_keeps_its_bytes`: with `support::MODEL.replace("Binary64", "Decimal")`,
  `index.html` does not contain `"binary64"` and `TARGET.md` does not contain `Binary64`. Green
  before and after.
- `a_binary64_identity_is_still_refused_by_the_web_target`: with
  `support::MODEL.replace("{name: demo.search.DocId, kind: newtype, of: Uuid}", "{name: demo.search.DocId, kind: newtype, of: Binary64}")`
  (assert changed), `synthesize_for(.., Target::Web)` refuses with one cause, code
  `MissingRepresentation`, sources `["types.demo.search.DocId.of"]`, detail starting
  `a Binary64 here is a key, a lookup or a delivery route`. Red before (old detail, one cause per location).
- `the_page_with_a_binary64_control_type_checks_with_tsc`: write `// @ts-check\n` + the page's
  module script (copy `page_script`, `tests/json_web.rs:336-344`) to `page.js` and `bridge.js`
  beside it under `CARGO_TARGET_TMPDIR/binary64-web-tsc`, and run `tsc` with the arguments of
  `tests/json_web.rs:442-457` (`--noEmit --allowJs --strict false --target es2022 --module es2022
  --moduleResolution bundler --lib es2022,dom page.js`); assert success, printing stdout. Like
  `json_web`'s `issue_224_the_page_type_checks_with_tsc`, it needs `tsc` on `PATH` (CI installs
  `typescript@6.0.3`; locally `npm install --prefix <dir> typescript@6.0.3` and put
  `<dir>/node_modules/.bin` first on `PATH`). Red before (the target refuses). Under 10 s.
- In `src/web/page.rs`, the unit test of step 2 (red before: the constants do not exist).

## Commands

From the worktree root:

1. `cargo test -p ess-synth --locked --lib web::page` — passes, including the new unit test.
2. `cargo test -p ess-synth --locked --test binary64_web` — 4 passed.
3. The bridge compiles with the Rust tree beside it (one line each):
   - `cargo build -p ess-cli --locked --bin ess`
   - `rm -rf target/binary64-web && mkdir -p target/binary64-web && cp crates/generate/ess-synth/tests/fixtures/binary64-served/model.yaml.in target/binary64-web/model.yaml`
   - `target/debug/ess generate synthesize --path target/binary64-web/model.yaml --target rust --out target/binary64-web/rust/demo`
   - `target/debug/ess generate synthesize --path target/binary64-web/model.yaml --target web --out target/binary64-web/web/demo`
   - `env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS -u RUSTC_WRAPPER RUSTFLAGS='-D warnings' cargo check --offline --manifest-path target/binary64-web/web/demo/Cargo.toml --target-dir target/binary64-web/target` — clean (the web tree names the Rust crates by `../../rust/demo/`, `web/mod.rs:534`). Where `rustup target list --installed` shows `wasm32-unknown-unknown`, the same with `--target wasm32-unknown-unknown` is clean too.
   - `rm -rf target/binary64-web`
4. One command each, all pass: `cargo test -p ess-synth --locked --test web`,
   `--test json_web`, `--test feasibility`, `--test binary64_plan`.
5. `ESS_BINARY64_HARNESS=$PWD/target/nextest-archives/binary64-harness cargo test -p ess-synth --locked --test binary64_rust`
   when that harness exists (B2's command 1 builds it) — passes; skip otherwise.
6. `cargo clippy -p ess-synth --all-targets --locked -- -D warnings` — clean.
7. `slot task fmt-check` — clean.

## Files this unit owns

- `crates/generate/ess-synth/src/web/binary64.rs`
- `crates/generate/ess-synth/src/web/mod.rs`
- `crates/generate/ess-synth/src/web/page.rs`
- `crates/generate/ess-synth/tests/binary64_web.rs` (new)

Must not touch: everything under `src/rust/` (B2; `rust::binary64::used` and `rust::wire` are only
called), `src/go/` (B3), `src/failure.rs`, `src/determined.rs`, `src/view_query.rs`,
`tests/binary64_served/support.rs`, the fixture (B1); `src/web/catalog.rs` (the catalog format is
unchanged); `tests/json_web.rs`, `tests/web.rs`; `Taskfile.yml`, `AGENTS.md`, `CHANGELOG.md`,
`docs/`, `website/`, `.github/workflows/*`.

## For the coordinator at integration

This section carries the release note and the documentation for the whole feature (units B1-B4).
Run `task test-xtask` after the `CHANGELOG.md` and `docs/` edits, and `task site-build` after the
`website/` edit.

**`CHANGELOG.md`**, under `## [Unreleased]` (line 3), a `### Added` heading with:

```markdown
- `ess generate synthesize --target rust`, `go` and `web` carry a finite `Binary64` in entity
  fields, command input and response, event and error fields, view fields and rows, struct fields,
  `Optional`, `List`, map values and newtypes, where they refused every model that declared one.
  The value is a checked wrapper, `primitives::Binary64` in Rust and Go, that holds no NaN or
  infinity. A server reads a JSON number into it, rounding once to the nearest binary64, answers
  `400` for a number that rounds to an infinity, and writes the shortest round-trip digits in
  `serde_json`'s layout (`3.0`, `-0.0`, `0.73`, `1e16`, `1e-6`), the same bytes in Rust and Go.
  Guards, view filters and `order_by:` compare it by value; membership (`any_of`/`none_of`) over
  one stays an obligation. A `Binary64` identity, view parameter, actor attribute, binding position
  or conversion is still refused, now with a detail saying the target does not carry one there;
  `--target clap` still refuses every `Binary64`. On the web page a `Binary64` is a JavaScript
  number, and `TARGET.md` says what that loses. A model without `Binary64` generates the same
  bytes, and no format version changes.
```

**`docs/design/model-binary64.md`**:
- line 125 (item 4), append: `Whole-system Rust, Go and web synthesis now carry it
  ([code-target-binary64.md](code-target-binary64.md)); Clap still refuses.`
- line 126 (item 5), append: `Done for Rust, Go and web in [code-target-binary64.md](code-target-binary64.md).`
- line 226: `Whole-system Rust/Go/Web/Clap workspaces refuse before constructing artifacts.` becomes
  `Whole-system Clap workspaces refuse before constructing artifacts; Rust, Go and Web refuse only a
  position they do not carry ([code-target-binary64.md](code-target-binary64.md) §5).`
- line 283: `mutation run, and whole-system \`ess synthesize --target rust|go|web|clap\`.` becomes
  `mutation run, and whole-system \`ess synthesize --target clap\` (Rust, Go and web carry one, see
  [code-target-binary64.md](code-target-binary64.md)).`

**`docs/design/binary64-structural-codecs.md`**, lines 41-42: `Whole-system synthesis and
conformance keep their existing refusals;` becomes `Whole-system Rust, Go and web synthesis carry a
finite Binary64 ([code-target-binary64.md](code-target-binary64.md)); Clap synthesis and the
conformance routes keep their refusals;` (line 5's "standalone data-library support" stays true).

**`website/docs/guides/synthesize.md`**, lines 79-80: `All four full synthesis targets refuse
unsupported modeled Binary64, as covered by the [feasibility tests](…).` becomes `The \`rust\`,
\`go\` and \`web\` targets carry a finite \`Binary64\` in every value position and write it as a
JSON number in its shortest digits (\`3.0\`, \`-0.0\`, \`1e16\`); they refuse one used as an
identity, a view parameter, an actor attribute, a binding position or a conversion. The \`clap\`
target refuses every \`Binary64\`.`

No `Taskfile.yml`, workflow, `ci_lanes.rs` or `README.md` change for this unit.

## Out of scope

- Keeping the sign of zero or a number's spelling on the page (`JSON.rawJSON` wraps a primitive
  only; the glue is the same bytes for every model). Stated as a weakening instead.
- Any change to `bridge.js` (the glue), `catalog.rs`, or the `Integer`/`Json` controls.
- Any Rust or Go codec change; any change to `failure.rs`.
- A wasm round-trip test of `Binary64` through the bridge (the codec is B2's, exercised by its
  harness); a browser test.

## Acceptance

- The web target synthesizes the fixture, its page carries the `binary64` control, `TARGET.md` the
  weakening, the bridge's wire `binary64_at` (command 2); the page type-checks (command 2).
- The bridge crate compiles with warnings denied (command 3).
- A Binary64 identity is still refused by Web with the uncarried detail (command 2).
- A model without `Binary64` keeps its page and `TARGET.md` bytes; `web`, `json_web`, `feasibility`
  pass (commands 2, 4).
- Clippy and fmt are clean (commands 6, 7).

## Depends on

- `each-code-target-admits-binary64-through-a-switch-of-its-own` (B1): `src/web/binary64.rs` with
  `pub(crate) const CARRIED: bool = false;`, `pub(crate) mod binary64;` in `src/web/mod.rs`,
  `failure::carries_binary64` (`Target::Web => rust::binary64::CARRIED && web::binary64::CARRIED`),
  the fixture and `tests/binary64_served/support.rs` (`pub fn model() -> EssIr`, `pub const MODEL: &str`).
- `the-rust-target-carries-a-finite-binary64` (B2): `crate::rust::binary64::CARRIED = true`,
  `pub(crate) fn crate::rust::binary64::used(ir: &ess_compiler::EssIr) -> bool`, and
  `crate::rust::wire::module` emitting `binary64_at` (the bridge's wire is that module,
  `web/mod.rs:320`). Independent of B3.
