---
format: aep.planning-md/3
id: story:a-dotted-variant-name-synthesizes-as-an-identifier
kind: story
status: active
title: A dotted variant name synthesizes as an identifier
summary: 'Adopter report N8: synthesize --http refuses dotted variant names (ess#662).'
refs:
- provider: github
  reference: beyond10x/ess#662
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:56Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:56Z", actor: "human:timo", revision: 7}
---
## Problem

`ess verify conform synthesize --http` refuses an enum variant whose name contains a dot, because the Go target allocates no identifier for it (`crates/generate/ess-synth/src/go/layout.rs:458-480`), while `ess specify validate` accepts the name. Adopters add `{name, wire}` to every such variant. Adopter report N8, ess 0.68.0.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/generate/ess-synth/tests/optional_recursive_codecs.rs`:
  `dotted_labels_synthesize_as_derived_identifiers_on_every_code_target` (replaces
  `labels_used_as_identifiers_are_refused_by_every_code_target`): `call.started`,
  `demo.explanation/2` and union label `demo.shape/1` synthesize on Rust, Web and Go as
  `CallStarted`, `DemoExplanation2`, `DemoShape1`; the Go module passes `go vet` and its codec
  writes and reads `call.started` (the file's Go harness, skipped where it already skips without
  `go`).
- same file: `a_label_with_no_identifier_after_conversion_is_still_refused`: `1.0` is refused as
  `invalid-identifier` on Rust and Web, the hint naming `naming: {code: …}` (Go prefixes the type
  name, `CallEvent10`, as it already does for a label such as `1a` today).
- same file: `a_variant_code_names_its_identifier_on_every_target`: `{name: call.started, code:
  Begun}` emits `Begun` on Rust and Go, wire value `call.started`.
- same file: `labels_colliding_after_conversion_are_refused_by_rust`: `a.b` beside `a_b` is
  `symbol-collision` on Rust and Web, naming both sources.
- `crates/generate/ess-synth/tests/served_codec_names.rs`:
  `a_model_without_dotted_labels_keeps_its_bytes`: the fixtures there synthesize byte-identical
  to 0.69.2's output.

## Fit review

Checked on ess 0.69.2 (main `a3bc2df3d`), reproductions under the fit scratch directory
(`repro-n8/`, `repro-n8-idiom/`, `repro-n8-code/`, `repro-n8-collide/`).

1. **Need.** An enum whose labels are dotted wire values (`call.started`, `call.ended`) validates
   (`ess specify validate`: valid) and `ess generate types` realizes it as `CallStarted` with
   `#[serde(rename = "call.started")]`, but `ess generate synthesize` refuses it on every code
   target: Go `invalid Go identifier CallEventCall.started`, Rust and Web `invalid Rust identifier
   Call.started`. Requester's proposal (theirs): the Go target converts the name, dots as word
   breaks, and keeps the declared spelling as the wire value.
2. **Class.** Convenience at the language level (no domain fact is missing), with a generator
   inconsistency: the types generator derives an identifier from the same label
   (`crates/generate/schema-contract/src/realize.rs:1103-1150`, `declaration_name` at `:1700`)
   while ess-synth refuses it (`crates/generate/ess-synth/src/go/layout.rs:450-480`,
   `crates/generate/ess-synth/src/rust/feasibility.rs:37-44`). Second defect found: a variant's
   `naming: {code: …}` validates and every synth target ignores it (`repro-n8-code/` refuses
   exactly as without it), because `code_aliases` reads declarations only
   (`crates/generate/ess-synth/src/alias.rs:19-31`), while the Rust collision hint tells authors to
   use it (`repro-n8-collide/`: "give one of them `naming: { code: … }`").
3. **Already expressible?** Yes: `{name: CallStarted, wire: call.started}` synthesizes on Go, Rust
   and Web (`repro-n8-idiom/`: 0 refused). The generated codecs read `wire()`
   (`go/http.rs:582`, `rust/wire.rs:156`), so the wire value stays checked. Cost: the declared name
   every guard and scenario uses moves away from the wire spelling, 17 times in the reporter's case.
4. **Fit.** Grid: an enum/union label → generated identifier, a generator-only cell; no authored
   key. Deriving with the rule `type_fragment` already applies to arbitrary spellings
   (`rust/name.rs:53-58`, `go/name.rs:145-150`: split on every non-alphanumeric, pascal each word)
   matches the types generator's rule. Sibling: union labels (`demo.shape/1`) take the same path as
   enum variants (`rust/feasibility.rs:364-370`, `go/layout.rs:740-748`). Targets: Rust, Web (the
   Rust emitter), Go; ess-gen types already does it; interpreter, diff, Entity Runtime read no
   identifiers (inferred).
5. **Second adopter.** An enum of media types (`application/json`, `text/plain`) or of
   reverse-DNS event types (`order.placed`, `order.shipped`) that a wire contract fixes.
6. **Cost.** No format version, keyword or diagnostic code. Only allocations refused today change
   (a name holding a character other than letters, digits, `-`, `_` fails `valid_ident` on every
   target now), so every model that synthesizes today keeps its bytes, except a model declaring
   `code:` on a variant, whose generated variant identifier moves to the authored one (generated-API
   change for that model; how many exist: I don't know). The existing test asserting the refusal
   (`crates/generate/ess-synth/tests/optional_recursive_codecs.rs:637-704`) is rewritten.
7. **Alternatives.** (a) Change nothing, answer with the `{name, wire}` idiom: rejected, the two
   generators keep disagreeing on one label and the hint the Rust target gives does not work.
   (b) Go-only conversion, as proposed: rejected, Rust and Web refuse the same label (shown).
   (c) Derive on all synth targets with the existing `type_fragment` rule, honour a variant's
   `code:` as the override (chosen). (d) Positional `V<n>` as the types generator's fallback:
   rejected for synth, served code names variants in behaviour and selection code.

## Decisions

- Accept, redesigned. Enum and union variant identifiers on the Rust, Web and Go synth targets
  are derived with `type_fragment` (every non-alphanumeric character a word break), the rule the
  types generator applies; a name with no identifier after it (on Rust and Web a leading digit,
  `1.0`) keeps `invalid-identifier`.
- A variant's `naming: {code: …}` names its identifier on every synth target, winning over the
  derived one, as a declaration's `code:` does (`alias.rs`). This is the remedy both refusal hints
  name; the Go hint's `{name: <identifier>, wire: …}` text is replaced by it.
- Collisions: Rust and Web keep `symbol-collision` by name (shown for `a-b`/`a_b`). Go keeps its
  deterministic `_` suffix (`go/layout.rs:1114-1116`), which it already applies to `a-b`/`a_b`
  today; aligning Go's collision policy is out of scope and not filed here.
- No format gate: no authored surface changes; `code:` on a variant is admitted from `ess/5`.
- Not already fixed on main: 0.69.2 refuses `repro-n8/` on all three targets.

## Scope

| unit | files | confidence |
|---|---|---|
| Go allocation | `crates/generate/ess-synth/src/go/layout.rs` (`allocate` enum/union arms `:725-748`, `invalid_identifiers` hint `:450-480`), `go/name.rs` (`type_fragment` `:145`) | cited |
| Rust/Web identifiers | `crates/generate/ess-synth/src/rust/name.rs` (a variant-identifier helper beside `type_fragment` `:53`); call sites `rust/feasibility.rs:358,368`, `rust/items.rs:142`, `rust/wire.rs:155,263`, `rust/selection.rs:92,328`, `rust/behaviour.rs:3373,3393`, `rust/invariant.rs:681` | cited (sites), inferred (helper) |
| variant `code:` | `crates/generate/ess-synth/src/alias.rs` (`code_aliases` `:19-31`), read by both layouts | cited |
| tests | `crates/generate/ess-synth/tests/optional_recursive_codecs.rs` (`:637-704` rewritten), `tests/served_codec_names.rs` | cited |
| docs | `docs/design/optional-recursive-rust.md:116-121`, `docs/design/enum-variant-wire-names.md` (reader table row for ess-synth is stale: synth reads `wire()`) | cited |

Crate: ess-synth only. No new spec format version (`ess/31` stays newest). Test binaries:
ess-synth `optional_recursive_codecs`, `served_codec_names`. Sizing: M (about 12 call sites, one
rewritten test, two design notes). Relation: `story:generated-enum-variants-keep-their-declared-names`
(N3) fixed the same label in schema-contract (`5ea632bdd`); no shared file.

confidence: high
