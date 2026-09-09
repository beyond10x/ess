# Diagnostic identity carried independently of wording — binding design

Written 2026-09-09 for `story:review-typed-diagnostics` (revision 23), from finding F14 of
`docs/reviews/2026-09-05-architecture-review.md:481`. Base commit `2900f6285e22e12ba502671b23c9e6467a8e324b`.
Every `file:line` below was read in that tree.

## The defect, stated as a rule that is violated today

A machine reading a diagnostic must be able to name the rule that produced it and the construct it
is about, without reading the prose. Today it cannot, because both are derived from
`ValidationError::location` — a *human-facing dotted path* that a producer writes with `format!`:

- `resolve.rs:565` `family_of(&str)` splits `location` on `. [` and matches the first token against
  eleven literals; anything else becomes `codes::family::SPEC`. Rewriting the path prefix in a
  producer's `format!` changes the emitted `ESS-<FAMILY>-<class>` code.
- `resolve.rs:696` `needles_for(&str)` re-tokenises the same string, consults the 50-entry
  `STRUCTURAL` stop-list at `resolve.rs:637` to decide where the path stops naming a declaration,
  and hands the guesses to `Locator::span` (`resolve.rs:452`), a substring scan. The cited source
  line is therefore also a function of how the path was spelled.
- `resolve.rs:591` `class_of(ValidationCode)` is already typed, but it collapses `UndeclaredReference`,
  `UnknownWorkflow`, `UnknownProtocol` and four more into `codes::class::UNDECLARED`, so the emitted
  `Code` does not identify the rule either.

The rule identity itself is not missing: `ValidationCode` (`ess-primitives/src/error.rs:188`,
`#[non_exhaustive]`) is it, and it is already carried. What is missing is the *construct reference*
and the *source position*, both of which are currently reconstructed by parsing prose.

## The typed site

Added to `crates/specify/ess-primitives/src/error.rs`.

| Item | Shape | Why |
|---|---|---|
| `ConstructKind` | `#[non_exhaustive]` C-like enum: `Type`, `Conversion`, `Entity`, `Command`, `Event`, `Error`, `View`, `Actor`, `Binding`, `Component`, `Topology`, `Domain`, `Specification` | Exactly the eleven heads `family_of` matches, plus `Specification` for its `_ =>` arm. `as_str` is the same token the producers spell today, so a rendered path is byte-identical. |
| `Segment` | `Key(String)` \| `Name(String)` \| `Index(usize)` | The member path as segments, not a dotted string. `Key` is a schema key (`outcomes`, `payload`); `Name` is something the author wrote (`accepted`, `recipient`); `Index` is a positional element. The `Key`/`Name` split is what replaces the `STRUCTURAL` stop-list: the producer knows which it wrote, so no consumer has to guess. |
| `ConstructRef` | `{ kind, name: String, members: Vec<Segment> }`, built by `new(kind, name).key(..).named(..).index(..)` | Kind plus qualified name plus member segments. |
| `SyntaxSpan` | `{ source: String, line: usize, column: usize }` | The optional syntax span. |
| `Site` | `{ construct: ConstructRef, span: Option<SyntaxSpan> }` | The pair. |

`ValidationError` gains one **private** field `site: Option<Box<Site>>`, read through
`ValidationError::site()` and written only by `ValidationError::at(construct, code, message)`.

It is boxed, and that is not a micro-optimisation. `TypeRegistry::insert` and its siblings
(`ess-domain/src/types.rs:794`, `:858`) return `Result<(), ValidationError>`, so the error variant's
size is a lint: an unboxed `Site` takes `ValidationError` to 176 bytes and
`clippy::result_large_err` refuses the crate. The lint is the machine check for the class — any
future field on this struct is caught by the same gate step, so no list has to be maintained by
hand. Measured: `cargo clippy --locked -p ess-primitives -p ess-domain -p ess-compiler --all-targets
-- -D warnings` fails with the field inline and exits 0 with it boxed.

### The rendering rule

`ConstructRef::render` is the single definition of `location` for a sited error:

```
render = kind.as_str() , { "." Key | "." Name | "[" Index "]" }
```

`ValidationError::at` sets `location = construct.render()` and nothing else may set both. The field
is private precisely so the two cannot drift; `ValidationError::new` (the string path) leaves
`site` `None`. This makes "the rendered string stays byte-identical" a property of one function
rather than of 145 call sites, and it is asserted by `error.rs`'s own tests over a table of
`ConstructRef` values including indices and a bare construct with no members.

`location` stays `pub` and stays a `String`, so the 48 in-crate `.location` assertions, the
`ValidationErrors` display used by `ess specify validate`, and the guide sample at
`website/docs/guides/write-a-specification.md:134-137` are unchanged by construction.

### Persistence

`site` is `#[serde(skip)]`. `ValidationError` is `serde::Serialize + Deserialize + schemars::JsonSchema`
with `deny_unknown_fields` (`error.rs:387-398`), and it appears in no file under `schemas/`
(grep: 0 hits). Serialising the site would make it wire-visible in a `deny_unknown_fields` struct,
which under `AGENTS.md` ("Determinism and formats") needs an old-reader compatibility test and a
format decision. The site is consumed in-process by `bridge`, so it does not need to travel; skipping
it keeps the serialized bytes and the generated schema identical and costs nothing this story needs.
`cargo xtask schema --check` is the gate that holds this claim.

## The bridge

`crates/specify/ess-compiler/src/resolve.rs::bridge` reads the site first:

| Input | family | span |
|---|---|---|
| `site = Some(Site { span: Some(s), .. })` | `family_of_kind(site.construct.kind())` | `s` verbatim, `path = construct.render()` |
| `site = Some(Site { span: None, .. })` | `family_of_kind(site.construct.kind())` | `locator.span(construct.render(), needles_of_site(construct))` |
| `site = None` | `family_of(&error.location)` | `locator.span(error.location, needles_for(&error.location))` |

`family_of` and `needles_for` are **not consulted** when a site is present — that is a test, not a
comment (`resolve.rs` in-crate `a_sited_refusal_ignores_the_location_strings_head` and
`a_sited_refusal_takes_its_needles_from_the_construct_not_the_path`).

`needles_of_site` derives the same needles the heuristic derives, from typed data instead of tokens:
the trailing `Segment::Name` as `"<name>:"` when it starts with an ASCII lowercase letter, then
`"name: <qualified>"`, `"id: <qualified>"`, `"component: <qualified>"`. It never consults `STRUCTURAL`,
because a `Key` segment is already known not to be a declaration name. The lowercase test is kept
rather than dropped: an event name written `InvoiceCreated` is a `Name` segment but is not a YAML key
in the documents this repository has, and pushing `InvoiceCreated:` as a needle would move located
lines that are pinned today.

No parser position source exists yet — `serde_yaml` discards positions for semantic errors
(`ess-compiler/src/source.rs:1-11`) — so every migrated producer in this wave sets `span: None` and the
`Locator` still finds the line. `SyntaxSpan` exists and is honoured by `bridge` so that the first
producer able to supply a position needs no further change here; its consumption is tested directly.

## Migration order and what this wave migrated

Order is by family, largest first, because the largest family is also the one the adopter-facing guide
prints (`ESS-COMMAND-001`).

**Migrated (this wave):** the `command` family in `crates/specify/ess-domain/src/command.rs` — every
site whose location is built locally from `CommandSpec::name`:

| Function | `file:line` (base) | Sites |
|---|---|---|
| `validate_wrong_state_answer` | `command.rs:250` | 2 |
| `CommandSpec::validate_shape` | `command.rs:1040` | 18 |
| `CommandSpec::validate` | `command.rs:1546` | 2 |
| `validate_payloads` / `check_payload_entry` / `check_payload_literal` | `command.rs:1610`, `:1679`, `:1793` | 7 |

Those functions take a `&ConstructRef` where they took an `&str` location, so the path is typed the
whole way down rather than re-parsed at the end.

## Inventory: what is still on the string heuristic

Every entry closes with a typed replacement or an explicit unsupported result, per the story's
compatibility clause. Nothing here is a silent omission.

| Path | Sites | Result | Reason |
|---|---|---|---|
| `command.rs` `validate_sets` (`:1722`) | 2 | **unsupported this wave** | It writes `commands.<name>.…` — plural. `family_of` trims a trailing `s` so the code is right by accident. Rendering it from a `ConstructRef` produces `command.…`, which is a **location change, not a wording change**, and is therefore outside this story's acceptance. Filed for a follow-up that may change the string. |
| `command.rs` `field_shape` (`:1968`), used by `ErrorSpec::validate` (`:1960`) | 1 | deferred | Shared with the `error` family; migrating it belongs with that family, not with `command`. |
| `TypeRegistry::resolve(&type_ref, &location)` call sites (`command.rs:1558`, `:1963`, and `types.rs`) | 3 in `command.rs` | deferred | The location crosses a crate module boundary into `types.rs`, whose signature is the `type` family's migration. |
| `Outcome::try_from` / `subject_of` relative locations (`command.rs:2151`–`:2402`), rebased by string concatenation at `command.rs:2425` | 7 | deferred | These build a location *relative* to a construct they do not know, and the prefix is prepended later by mutating `error.location`. The typed replacement is a `ConstructRef` passed into admission; it is a signature change on the raw→admitted conversion and is the next unit of this migration. |
| `ess-domain/src/entity.rs` (20), `binding.rs` (17), `component.rs` (15), `view.rs` (11), `topology.rs` (10), `system.rs` (10), `spec.rs` (7), `types.rs` (6), `domain.rs` (5), `wire.rs` (1) | 102 | deferred, by family | Same shape as `command.rs`; each is one family's worth of work with its own fixtures. |
| `ess-domain/src/actor.rs` | 1 | deferred | An `actor`-family site the story's own scope does not list; recorded here so the inventory is complete rather than equal to the scope. |
| `ess-domain/src/expression.rs` (1), `primitive_admission.rs` (2) | 3 | **not this unit** | `story:review-primitive-semantics`, same wave. `CommandSpec::validate_typed_guard` hands `check_predicate` the *rendered* form of its own site, so the two spellings cannot drift while that file waits. |
| `ess-primitives/src/error.rs:78` `ParseError::Shape { location: String }` | — | deferred | The same pattern on the *parse* side. A parse error has a real `serde_yaml` position, so its typed form is a `SyntaxSpan`, not a `ConstructRef`; different work. |
| `crates/infra/infra-domain/src/code.rs:190` | — | **out of this story** | The second `ValidationError { location: String }` envelope F14 names. Recorded here, deliberately not fixed: `crates/infra/**` is another unit's assignment in this wave. |

`ess-domain/src/locate.rs` is **not** in this table: the story's scope lists it as a file whose
constructors would need migrating, and it constructs no `ValidationError` at all (grep: 0 in 259
lines). That scope line is wrong and nothing was built on it.

`bridge`'s string fallback stays until this table is empty.

## Consumer accounting

The 39 new concrete entries — the types, their variants, their methods, `CommandSpec::site`, and
`ess-compiler`'s `family_of_kind`, `needles_of_site`, `span_of_site` — are classified in
`crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json`, each `OwnedHelper` with
the reason its nearest sibling carries: the data/API wording for a `struct::`/`enum::`/`variant`
entry (as `error::struct::ValidationError` has), the implementation wording for an `fn::` or
`impl<…>::method` entry (as `resolve::fn::family_of` has). `task consumer-check` is the gate.

## What the fixtures assert

`crates/specify/ess-compiler/tests/typed_diagnostics.rs` against
`crates/specify/ess-compiler/tests/fixtures/typed_diagnostics/`:

1. `repeated_names.yaml` — one outcome name used in two commands, and one field name repeated inside
   one command, so a needle that is *not* unique is exercised and the honest `located: None` is
   asserted rather than a confidently wrong line.
2. `nested.yaml` — a `payload:` entry three member levels below the command, so the member path is
   more than one segment deep.
3. `cross_file_a.yaml` + `cross_file_b.yaml` — a command in one file emitting an event declared in
   another, so the cited `Span::source` must be the file the *command* is in, not the file the
   reference resolves to.

For each fixture the test asserts, for every diagnostic:

- the emitted `Code` and the `Span` (`source`, `path`, `located`) — the pin;
- that the same specification, with **every `message` and `hint` replaced by different prose**,
  produces pairwise-identical `Code` and `Span` — the acceptance statement, checked over every
  diagnostic in the fixture rather than a chosen one;
- that each migrated rule's `ValidationError::site()` is `Some` and carries the expected
  `ConstructKind`, qualified name and member segments;
- that `site().construct.render()` equals `location` for every error in the run — the invariant that
  keeps the rendered string byte-identical.

Unchanged pins that must keep passing: `ess-compiler/tests/billing.rs:395-443` (`ESS-TYPE-008` at
`types.yaml:6`, a family this wave did not migrate, so it exercises the fallback) and the guide sample
string `[undeclared_reference] command.…` at `website/docs/guides/write-a-specification.md:134-137`,
asserted directly by the fixture test rather than only by reading the page.
