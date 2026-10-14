# Replacing serde_yaml and unsafe-libyaml

beyond10x/ess#541, `story:yaml-parser-without-unsafe-libyaml`.

## What is wrong

Every ESS crate that reads or writes YAML depends on `serde_yaml` 0.9.34+deprecated, and through it
on `unsafe-libyaml` 0.2.11. RUSTSEC-2024-0449 (`informational = "unmaintained"`, `package =
"unsafe-libyaml"`, no patched version) marks the second one: its repository was archived on
2024-03-25, and it is libyaml machine-translated into unsafe Rust. `cargo deny check advisories` in
an adopter's crate that depends on an ESS crate reports it, and the adopter carries an ignore until
ESS stops depending on it.

## What ESS uses today

Counted at `376c794368`:

- `serde_yaml` is declared in the workspace `Cargo.toml` and in 22 crate manifests: 20 as a normal
  dependency, and 2 as a dev-dependency only (`ess-compiler`, `ess-entity-runtime`). A 23rd
  manifest, `ess-diff`, names it in a comment and does not depend on it.
- 211 Rust files name it, 711 times. Some only name it in prose (the `ess-compiler` sources do). Per
  crate, files (src/tests): `ess-domain` 16/32, `ess-cli` 6/19, `ess-ui-tui` 8/15, `ess-gen` 3/16,
  `ess-conformance` 4/10, `ess-ui` 6/6, `ess-primitives` 5/6, `ess-xtask` 3/6, `ess-ui-check` 7/1,
  `ess-deployment` 6/1, `ess-ui-docs` 4/3, `ess-ui-react` 4/3, `ess-openapi` 2/4, `ess-ui-test`
  3/0, and one or two in each of `ess-compiler`, `ess-http`, `ess-realization`, `ess-transport`,
  `ess-composition`, `ess-cli-contract`, `ess-cli-project` and `infra-spec`.
- The API in use: `from_str` (352 sites), `Value` and its variants (about 200), `to_string` (105),
  `Error` (23), `from_value` (20), `from_slice` (7), `to_value` (6), `Mapping` (5),
  `Number::from` (3) and `Deserializer::from_str` (2, multi-document reads in `ess-cli`'s recovery
  chart and in `ess-ui-check`). Nothing uses `serde_yaml::with`, `Value::apply_merge` or
  `Error::location()`.
- `Value::Tagged` is read in two places that refuse a tag: `ess-conformance`'s recorded adapter
  (`first_tag`, "the YAML tag `…` is not part of …") and `ess-openapi`'s accounting. Everywhere
  else a tagged value is unwrapped.
- No generated code depends on `serde_yaml`: nothing under `generated/` or `examples/` names it.

### What the parser decides, and what it does not

**Source positions of semantic diagnostics do not come from the parser.**
`crates/specify/ess-compiler/src/source.rs` keeps each file's text, and `resolve.rs`'s `Locator`
finds a diagnostic's line and column by searching that text for a needle that occurs exactly once.
No parser position reaches a `Span`. Whatever parser ESS uses, every located diagnostic keeps its
line and column.

The parser decides:

1. **Error messages for documents that do not parse or do not fit their type.** `serde_yaml`
   appends ` at line L column C` (both 1-based) to an invalid-type or unknown-field error, and puts
   no location on `duplicate field` or `missing field`. No test under `crates/` pins one of these
   positions. Tests do pin serde's own wording: `` unknown field `related`, expected `selection` or
   `path` `` (`ess-domain/tests/binding_stored_read_idiom.rs`), `` unknown variant `resend`, expected
   one of `retry`, `escalate`, `drop` `` (`ess-domain/tests/refusal_policy.rs`), and `duplicate
   field` (`ess-conformance/tests/recorded_adapter_adversary_pass2.rs`).
   No test pins a libyaml scanner message (`did not find expected`, `mapping values are not
   allowed`, `found character that cannot start`: no match under `crates/`).
2. **Which documents ESS accepts and what each scalar means** (the tables below).
3. **The bytes ESS writes as YAML.** `cargo xtask generate --check` byte-compares
   `generated/openapi/*.yaml` and `generated/asyncapi/*.yaml`, which `ess-gen` writes through
   `serde_yaml::to_string`; `ess --format yaml` writes through `ess_primitives::json::to_yaml_string`;
   `ess-openapi` exports and the `ess-ui` crates write through it too. These are canonical bytes
   under AGENTS.md's format rules.

### Duplicate keys today

ESS depends on exactly what `serde_yaml` 0.9.34 does with a repeated key, which differs by target:

| target | `serde_yaml` 0.9.34 |
|---|---|
| a derived struct | refused: `` duplicate field `a` `` (no location) |
| a `BTreeMap` | the last value kept, silently |
| `serde_yaml::Value` | refused: `duplicate entry with key "k"` (` at line L column C` when nested) |
| a list-shaped visitor | sees both entries |

The last row is load-bearing. `MappingTable`, `PayloadTable` and the outcome `sets:` reader in
`ess-domain` read a mapping as a list so that `TryFrom` can refuse a repeated key with
`ValidationCode::DuplicateDeclaration`, naming both values and accumulating every repeat
(`binding.rs`, `command.rs`). A parser that refuses repeated keys itself replaces those typed
diagnostics with a parse error; one that collapses them silently loses the refusal.

## The behaviour every candidate was measured against

A throwaway Cargo project outside the repository (deleted after use) ran the same inputs through
each candidate. `serde_yaml` 0.9.34, plain scalars, untyped target:

| written | read as |
|---|---|
| `yes` `Yes` `on` `off` `y` `n` `no` | string |
| `true` `True` `TRUE` | bool |
| `~` `null` `Null`, empty | null |
| `0o7` `0o17` `0x1F` `0b101` `+7` `-0` | integer |
| `017` `1_000` | string |
| `1e3` `1.0e3` `1E3` `1.10` `1.` `.5` | float |
| `.inf` `-.Inf` `.nan` | float |
| `12:30:00` `2024-01-01` | string |

That is the YAML 1.2 core schema with no 1.1 booleans, sexagesimals or timestamps. Into a `String`
field every plain scalar is accepted as its text (`yes`, `1.10`, `0o7`, `~` all read as written).

Also:

- **Anchors and aliases** are expanded. A self-referencing alias and nesting deeper than 128 are
  refused (`recursion limit exceeded at line 1 column 129`), and an alias bomb is refused
  (`repetition limit exceeded`).
- **Merge keys** are not applied: `<<` is an ordinary key, and a struct with
  `deny_unknown_fields` refuses it (`` unknown field `<<` ``).
- **Tags** are kept in `Value::Tagged`. A typed string or map target ignores a custom tag, and
  `!!str 12` reads as `"12"`.
- **Enums.** A newtype or struct variant is read only from the tag form `!New hi`. The map form
  `{New: hi}` is refused (`expected a YAML tag starting with '!'`).
- **More than one document** is refused by `from_str`.
- **Writing.** libyaml writes block style with no line folding (width −1). It indents by two and
  does not indent a sequence under its key. A string that would read back as something else, or
  that is all digits with a leading zero, is single-quoted (`'1e3'`, `'017'`, `'true'`, `''`); a
  multi-line string is a literal block; `yes`, `on`, `1_000`, `12:30` and `2024-01-01` stay plain.
  Floats are written `1e20`, `1.5e-7`, `1000000000000000.0`, `1e16`, `.inf` and `-0.0`. A newtype
  variant is written `!New hi`.

## Candidates

Dependencies are the crates.io dependency lists of the named version. Advisories are from the
RustSec advisory database checked out on 2026-10-12.

| candidate | parser underneath | advisory | maintenance |
|---|---|---|---|
| `serde_yaml_ng` 0.10.0 | `unsafe-libyaml` ^0.2.11 | RUSTSEC-2024-0449 applies | last release 2024-05 |
| `serde_norway` 0.9.42 | `unsafe-libyaml-norway` ^0.2.13 | none | crate 2024-12; parser 2024-12, 3 contributors |
| `serde_yml` 0.0.13 | shim over `noyalib` | RUSTSEC-2025-0068 (unsound, unmaintained) | deprecated by its author |
| `serde_yaml_bw` 2.5.8 | `unsafe-libyaml-norway` and `granit-parser` | none | 2026-09 |
| `noyalib` 0.0.57 | own, pure Rust | RUSTSEC-2026-0333 (DoS, fixed in 0.0.53) | created 2026-05-10, 0.0.x |
| `serde-saphyr` 1.3.0 | `granit-parser` 1.3.0 | none | 2026-09 |
| `saphyr-parser` 0.2.1 | itself, pure Rust, no serde layer | none | 2026-10-12, 58 contributors |
| `yaml-rust2` 0.13.0 | itself, no serde layer | none | 2026-09 |
| `libyaml-safer` 0.3.0 | safe-Rust port of `unsafe-libyaml`, parser and emitter | none | 2025-12-22, 4 contributors |

`cargo-deny` 0.20.2, run offline against that database over a graph holding `serde_yaml`,
`serde_norway`, `noyalib` 0.0.57, `serde-saphyr`, `saphyr`, `saphyr-parser`, `granit-parser` and
`libyaml-safer`, reports one advisory: RUSTSEC-2024-0449 on `unsafe-libyaml` through `serde_yaml`.

### Forks with the serde_yaml API

- **`serde_yaml_ng`** still depends on `unsafe-libyaml`. It does not close the issue.
- **`serde_norway`** behaves byte-for-byte like `serde_yaml` in every probe. It read all 479
  committed `.yaml`/`.yml` files to the same values, and it re-emitted the four generated
  OpenAPI/AsyncAPI documents identically. RUSTSEC-2024-0449 names the package `unsafe-libyaml`
  exactly, so `cargo deny` does not report `unsafe-libyaml-norway`. But the latter is a renamed
  copy of the same c2rust translation (238 lines naming `unsafe` in its sources), last released in
  December 2024 by a three-person repository. It would clear the advisory in letter and keep the
  thing the advisory warns about. It is the fallback below, not the choice.
- **`serde_yml`** is unsound by advisory and is now a shim over `noyalib`.
- **`serde_yaml_bw`** still pulls `unsafe-libyaml-norway`.

### Pure-Rust libraries with a serde layer

**`serde-saphyr` 1.3.0** (`#![forbid(unsafe_code)]`) reports a `Spanned<T>` with line, column and
byte span for keys and values. Measured against the baseline:

- Its defaults read YAML 1.1 booleans (`yes`, `on`, `y` → bool), read `017` as 17.0 and `1_000` as
  1000, refuse `.inf`, refuse every repeated key, and apply merge keys. With `strict_booleans` and
  `MergeKeyPolicy::AsOrdinary` it read all 479 committed files to the same values as `serde_yaml`.
- No duplicate-key policy matches. `Error` refuses at parse time, which pre-empts the typed
  `DuplicateDeclaration` refusals. `LastWins` lets a list-shaped visitor see both entries but
  collapses a struct's repeated field before serde sees it, so `duplicate field` is never raised.
- It drops custom tags from untyped values, so `first_tag` cannot work. It accepts the enum map
  form.
- It rewrites serde's messages: `` unknown field `zz`, expected one of a, b `` for serde's ``expected
  `a` or `b` ``, and `invalid u32` for `invalid type: string "hello", expected u32`. Its default
  `Display` adds a multi-line source snippet.
- It has no `Value` type.
- It writes differently. It double-quotes ambiguous strings, folds long strings at 80 columns,
  writes `1.0e+20` and the enum map form, and quotes `openapi: "3.1.0"`. None of the four generated
  documents comes out byte-identical.

**`noyalib` 0.0.57** has a `Value` with `Tagged` and an API shaped like `serde_yaml`'s. But it
refuses `a: 1` into a `BTreeMap<String, String>`, and `1.10`, `0o7`, `1e3`, `true` and `~` into a
`String` field. It reads `017` as 17, collapses repeated struct fields silently and applies merge
keys. It writes with sequences indented under their key, double quotes and no trailing newline.
The generated documents differ from line 16. Its first advisory came within five months of its
first release.

### Pure-Rust parsers without a serde layer

**`saphyr-parser` 0.2.1** is the parser of the project RUSTSEC-2024-0449 names as the alternative.
Its sources have no `unsafe` (and no `forbid` attribute), and it depends only on `arraydeque` and
`thiserror`. Each event carries its scalar style, anchor id, tag and a span. The span gives a byte
index, a 1-based line and a 0-based column; the type's documentation says 1-based, the measurement
says 0. It parsed all 479 committed files. `saphyr-serde` on crates.io is an empty 0.0.0
placeholder. **`granit-parser` 1.3.0**, a fork with `#![forbid(unsafe_code)]` under `serde-saphyr`,
has the same event shape and also parsed all 479. **`yaml-rust2`** is the older crate of the same
lineage.

**`libyaml-safer` 0.3.0** is `unsafe-libyaml` ported line by line to safe Rust (`deny(unsafe_code)`,
with one `allow(unsafe_code)` block in `reader.rs`). Its emitter was driven with the events
`serde_yaml` sends libyaml (`ser.rs`'s style rules, unicode on, width −1). It reproduced
`serde_yaml::to_string` exactly for all 479 committed files. One sample diverged: after a
keep-chomped block scalar (`|+`) at the end of a document it writes a `...` line, and `serde_yaml`
does not. Its licence is declared only as a file (MIT), and it has a single active maintainer.

## Decision

**A new leaf crate, `ess-yaml`, owns YAML for ESS.** It is the only crate that names a YAML
library, it has no ESS dependency, and it is `#![forbid(unsafe_code)]`.

- **Its API** is the subset of `serde_yaml` 0.9 that ESS uses, with the same names and meanings:
  `from_str`, `from_slice`, `from_value`, `to_string`, `to_value`, `Value` (with `Tagged`),
  `Mapping`, `Number`, `Error` with its location, and the multi-document `Deserializer::from_str`.
  The migration in the other crates is then a rename of the path, not a rewrite.
- **Its serde layer** is a port of `serde_yaml` 0.9.34's own: `de.rs`, `value/`, `mapping.rs`,
  `number.rs`, `error.rs` and `ser.rs`, not `with.rs`, which nothing uses. It is MIT OR
  Apache-2.0, and the crate carries that notice. Porting it rather than adopting another library's
  is what keeps every row of the tables above: scalar resolution, lenient strings, all four
  duplicate-key behaviours, alias limits, `<<` as an ordinary key, tags, the tag form of enums, and
  serde's unmodified error wording with `serde_yaml`'s ` at line L column C` suffix.
- **Its parser** is `saphyr-parser`, pinned exactly (`=0.2.1`). Its events replace
  `serde_yaml::libyaml::parser`. A mark becomes `serde_yaml`'s location as line, column + 1, byte
  index.
- **Its emitter** is ESS's own, for the events `serde_yaml` sends and nothing more: block
  mappings and sequences, empty `[]` and `{}`, plain, single-quoted, double-quoted and literal
  scalars, and tags. It is ported from libyaml's scalar analysis, style selection and writers,
  using `libyaml-safer`'s MIT port as the reference. It must reproduce `serde_yaml`'s bytes,
  including the absence of the `...` line `libyaml-safer` writes.

**Why not a library directly.** Every pure-Rust candidate changes something ESS pins. `serde-saphyr`
cannot express today's duplicate-key behaviour. `noyalib` refuses documents ESS accepts. Neither
writes the committed generated bytes. The parser, where the security-relevant code is, comes from
the maintained project the advisory recommends. The serde layer and the emitter are the parts
where ESS's pinned behaviour lives, and ESS owns them.

## Format consequence

The intended consequence is **none**: no persisted, generated or published byte changes, so no
format version moves. That holds only if the swap reproduces `serde_yaml` exactly. Three checks
decide it, and all three are green before the swap lands:

1. `cargo xtask generate --check`, `cargo xtask site-data --check` and every existing test, passing
   unchanged.
2. A differential test in the unpublished `ess-xtask`. It keeps `serde_yaml` as a dev-dependency
   oracle and compares `ess-yaml` with it: read and write, over every committed YAML file, the
   generated documents and the scalar sample above.
3. A differential fuzz target in `fuzz/`, unpublished, comparing reads, refusals and writes.

Two differences are expected and are not formats:

- **Syntax-error wording, and possibly position.** A document that does not parse gets
  `saphyr-parser`'s message, not libyaml's. Measured: for an unterminated flow sequence libyaml says
  `did not find expected ',' or ']'`, and `granit-parser` (through `serde-saphyr`) says `illegal
  placement of ':' indicator`, at the same line and column. `saphyr-parser`'s own wording was not
  measured. No test pins one. This is diagnostic prose, not meaning, identity,
  references, canonicalization, names or an envelope. The changelog names it.
- **Edge inputs the two scanners judge differently.** None exists in the 479 committed files. Any
  that the fuzz target finds is aligned in the loader (refuse what `serde_yaml` refused, read what
  it read). If one cannot be aligned, it is recorded on this page with its consequence before the
  swap merges.

If any check shows a canonical byte moving, the swap stops. That byte needs its own decision under
AGENTS.md's format rules.

## Reversibility

Every other crate names only `ess-yaml`, so the backend is one crate's business:

- The parser can move to `granit-parser` (near-identical events) without touching another crate.
- The emitter can delegate to `libyaml-safer` if the port stalls. It must then drop the `...`
  line.
- **Fallback.** If the port cannot meet the differential bar before the release it is planned
  for, `ess-yaml` can be backed by `serde_norway` for that release. That backend is byte- and
  behaviour-identical by measurement and is not reported by RUSTSEC-2024-0449, but it keeps the
  c2rust code. It is a one-crate change, and it is recorded in the changelog as a stopgap.

## Migration units

`ess-yaml` first re-exports `serde_yaml` (`pub use serde_yaml::{Value, …}`). Every rename unit is
then behaviour-neutral, and the units are order-independent. A public ESS type that carries a
`Value` (such as `ess-domain`'s `RawPayloadSource::Captured`) is the same type on both sides of a
crate boundary while some crates are renamed and some are not. Each unit's `Cargo.lock` change
merges by regenerating.

| unit | crates | files | depends on |
|---|---|---|---|
| n1 facade | new `crates/specify/ess-yaml` (re-export only), and `ess_primitives::yaml` re-exporting it. Workspace member and dependency. AGENTS.md crate tree, plus the ui and infra boundary sentences (`ess-ui` may depend on the leaf `ess-yaml`; `infra-spec` reaches it through `ess_primitives::yaml`, so infra's only ESS dependency stays `ess-primitives`) | about 6 | — |
| n2 specify | `ess-primitives` 11, `ess-domain` 48, `ess-compiler` 2, `ess-composition` 1, `ess-realization` 2, `ess-transport` 2, `ess-http` 2, `ess-cli-contract` 1 | 69, 8 manifests | n1 |
| n3 generate, verify, infra | `ess-gen` 19, `ess-openapi` 6, `ess-deployment` 7, `ess-cli-project` 1, `ess-entity-runtime` (manifest), `ess-conformance` 14, `infra-spec` 1 | 48, 6 manifests | n1 |
| n4 ui | `ess-ui` 12, `ess-ui-check` 8, `ess-ui-docs` 7, `ess-ui-react` 7, `ess-ui-test` 3, `ess-ui-tui` 23 | 60, 6 manifests | n1 |
| n5 edge | `ess-cli` 25, `ess-xtask` 9 | 34, 2 manifests | n1 |
| n6 emitter | `ess-yaml` only: the owned emitter behind `to_string`. The differential write test in `ess-xtask` with `serde_yaml` as a dev-dependency oracle | about 6 | n1–n5 |
| n7 reader | `ess-yaml` only: the `saphyr-parser` loader and the ported serde layer. `ess-yaml` drops `serde_yaml`. The differential read test and fuzz target, `fuzz/Cargo.lock`, the changelog, and `crates/edge/ess-xtask/tests/layout.rs::no_published_crate_depends_on_serde_yaml_or_unsafe_libyaml`, written first and seen red | about 10 | n6 |

n2 to n5 can run in parallel. n6 and n7 carry the adversary: source positions, duplicate-key
refusals and emitted bytes. The `ess-domain` tests that pin `DuplicateDeclaration`, the
`duplicate field` refusal in `ess-conformance`'s recorded adapter, and the `first_tag` refusal are the
named cases. The story's estimate of about 14G for the workspace gate applies to n2 to n5 and to n7.

## The test

`no_published_crate_depends_on_serde_yaml_or_unsafe_libyaml` reads `cargo metadata --locked
--offline --format-version 1`. The `publish` field cannot say which crates are published, because
no ESS crate is on crates.io and the field is inconsistent:

- 30 members leave it unset;
- `ess-primitives` and `ess-kubernetes` inherit the workspace's `publish = false`;
- `ess-xtask` and the two example realizations set `false`.

Adopters depend on ESS by Git tag (`website/docs/start/runners/rust.md`). So **published** means
every workspace member under `crates/` except `ess-xtask`.

The test refuses `serde_yaml` or `unsafe-libyaml` in either of two places:

- among the packages reachable from a published crate over normal and build edges
  (`resolve.nodes[].deps[].dep_kinds`);
- in any published crate's own dependency list, of any kind, so a dev-dependency cannot bring it
  back.

At `376c794368` it fails: 19 published crates depend on `serde_yaml` as a normal dependency, and 2
more as a dev-dependency.

## Verified, and inferred

Verified by running:

- the usage counts;
- the duplicate-key and position facts about `ess-compiler` and `ess-domain` (read in source);
- every behaviour row for `serde_yaml`, `serde_norway`, `serde-saphyr` and `noyalib`;
- the 479-file read comparison;
- the generated-document write comparison;
- `libyaml-safer`'s write comparison;
- `saphyr-parser`'s and `granit-parser`'s parse of the 479 files, and their span shape;
- the advisory texts, and `cargo-deny`'s result over the probe graph.

Read from crates.io metadata or GitHub, not built:

- `serde_yaml_ng`, `serde_yml` and `serde_yaml_bw`'s dependencies;
- release dates, download counts and contributor counts.

Inferred, to be confirmed by n6 and n7:

- that porting `serde_yaml`'s serde layer onto `saphyr-parser` events reproduces every behaviour
  above;
- that a libyaml-derived emitter for the subset `serde_yaml` uses reproduces its bytes;
- that no edge input separates the two scanners in a way the loader cannot align.

The differential tests exist to turn these into measurements.
