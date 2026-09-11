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
| `ConstructKind` | An **exhaustive** C-like enum — deliberately unmarked, see *The bridge* —: `Type`, `Conversion`, `Entity`, `Command`, `Event`, `Error`, `View`, `Actor`, `Binding`, `Component`, `Topology`, `Domain`, `Specification` | Exactly the eleven heads `family_of` matches, plus `Specification` for its `_ =>` arm. `as_str` is the same token the producers spell today, and `separator` is the character they write after it, so a rendered path is byte-identical. |
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
render = kind.as_str() kind.separator() name , { "." Key | "." Name | "[" Index "]" }
```

`separator` is `' '` for `Entity` and `Component` and `'.'` for every other kind, because that is what
the producers already write: `entity billing.invoice.Invoice` (`ess-domain/src/entity.rs:830`) beside
`command.billing.invoice.CreateInvoice` (`command.rs:1052`). A renderer that emitted one shape for
both would change the adopter-facing string of every entity and component refusal — a **location**
change, not a wording change, which is exactly what the `validate_sets` row below puts outside this
story. The first version of this page instead said the deferred families were "the same shape as
`command.rs`", which was false for at least 35 of the 102 deferred sites (adversary pass 2, F3);
the separator is the fix, and it makes the entity and component migrations wording-preserving when
they come.

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

`family_of_kind` names every `ConstructKind` and has no wildcard arm. That is why `ConstructKind` is
exhaustive while `ValidationCode` is `#[non_exhaustive]`: a downstream crate matching a
non-exhaustive enum must write `_`, and a `_` here is how a new kind arrives silently in `SPEC`
without anybody deciding it should. `every_construct_kind_maps_to_a_declared_family_and_back` checks
both directions — no kind falls outside `codes::family::ALL`, and no family is unreachable.

`family_of` and `needles_for` are **not consulted** when a site is present — that is a test, not a
comment (`resolve.rs` in-crate `a_sited_refusal_takes_its_family_from_the_construct_not_the_location_head`
and `a_sited_refusal_takes_its_needles_from_the_construct_not_the_location_path`).

`needles_of_site` is **not** a second derivation. `needles_for` and `needles_of_site` call one body,
`needles_from_tokens`; the only difference is where the tokens come from — `needles_for` splits the
location string, `needles_of_site` builds them from `kind`, `name` and `members`. That is the whole
of what the typed site buys here, and it is enough: the tokens are a function of the construct, so
rewording `ValidationError::location` cannot move the cited line, and the two paths cannot disagree
about which line a path names.

The first draft of this page claimed the opposite — that the `Key`/`Name` split *replaces* the
`STRUCTURAL` stop-list and that a trailing `Name` needs no stop-list check. Adversary pass 1 refuted
it twice (F1, F2, `crates/specify/ess-compiler/tests/adversary_typed_diagnostics_pass1.rs`): a
trailing qualified event name such as `shop.tail.Missing` starts with a lowercase letter and *is* a
YAML key in every `payload:` block, and an outcome an author names `error` is one of the fifty
stop-list words. In both, the sited refusal and its own rendered string cited different lines for one
defect. Two answers to "which line does this path name" is worse than either answer alone, so there
is now one body to answer with, and `needles_of_site(c) == needles_for(&c.render())` is asserted for
every segment shape (`resolve.rs`, `typed_needles_are_the_string_needles_for_every_segment_shape`).

No parser position source exists yet — `serde_yaml` discards positions for semantic errors
(`ess-compiler/src/source.rs:1-11`) — so every migrated producer in this wave sets `span: None` and the
`Locator` still finds the line. `SyntaxSpan` exists and is honoured by `bridge` so that the first
producer able to supply a position needs no further change here; its consumption is tested directly.

## Migration order and what this wave migrated

Order is by family, largest first, because the largest family is also the one the adopter-facing guide
prints (`ESS-COMMAND-001`).

**A family is the location head a refusal writes, not the file the rule lives in.** The first version
of this page booked sites by file and therefore claimed the `command` family was migrated while
`ess-domain/src/entity.rs` was still writing four `command.…` refusals with `ValidationError::new`
(adversary pass 2, F1, F2) — rewording one of those paths moved `ESS-COMMAND-012` to `ESS-SPEC-012`.
The *Machine-checked inventory* below now carries a per-head census as well as a per-file one, so a
`command.…` site in any file counts against the command family and cannot hide in another row.

**Migrated (this wave):** the `command` family, wherever it is written:

| Function | `file:line` (working tree) | Sites |
|---|---|---|
| `validate_wrong_state_answer` | `command.rs:252` | 2 |
| `CommandSpec::validate_shape` | `command.rs:1052` | 2 |
| `CommandSpec::declared_input` | `command.rs:1102` | 1 |
| `CommandSpec::validate_outcome` | `command.rs:1119` | 4 |
| `CommandSpec::validate_payload_shape` | `command.rs:1223` | 2 |
| `CommandSpec::validate_sets_shape` | `command.rs:1280` | 2 |
| `CommandSpec::validate_guard` | `command.rs:1335` | 2 |
| `CommandSpec::validate_typed_guard` | `command.rs:1395` | 1 |
| `CommandSpec::validate_branch_coverage` | `command.rs:1457` | 4 |
| `CommandSpec::validate` | `command.rs:1566` | 2 |
| `check_payload_entry` | `command.rs:1662` | 2 |
| `check_payload_literal` | `command.rs:1819` | 4 |
| `CommandSpec::validate`, through `TypeRegistry::resolve_at` | `command.rs:1566`, `types.rs:900` | 1 |
| `validate_lifecycle_causes` (**`entity.rs`**) | `entity.rs:929` | 2 |
| `validate_wrong_state_is_reachable` (**`entity.rs`**) | `entity.rs:1073` | 1 |
| `validate_instance` (**`entity.rs`**) | `entity.rs:1129` | 1 |
| **total** | | **33** |

Those functions take a `&ConstructRef` where they took an `&str` location, so the path is typed the
whole way down rather than re-parsed at the end.

28 is `grep -c 'ValidationError::at' crates/specify/ess-domain/src/command.rs`, and the inventory
below is the same grep for `ValidationError::new`. The first version of this page counted by hand and
was wrong twice in one table (adversary pass 1, F5), so the counts are no longer prose: the block
under *Machine-checked inventory* is read by
`crates/specify/ess-compiler/tests/typed_diagnostics.rs::the_inventory_on_the_design_page_is_the_count_in_the_tree`,
which fails when the page and the tree disagree in either direction.

## Inventory: what is still on the string heuristic

Every entry closes with a typed replacement or an explicit unsupported result, per the story's
compatibility clause. Nothing here is a silent omission.

| Path | Sites | Result | Reason |
|---|---|---|---|
| `command.rs` `validate_sets` (`:1722`) | 2 | **unsupported this wave** | It writes `commands.<name>.…` — plural. `family_of` trims a trailing `s` so the code is right by accident. Rendering it from a `ConstructRef` produces `command.…`, which is a **location change, not a wording change**, and is therefore outside this story's acceptance. Filed for a follow-up that may change the string. |
| `command.rs` `field_shape` (`:1987`), used by `ErrorSpec::validate` (`:1974`) | 1 | deferred | Shared with the `error` family; migrating it belongs with that family, not with `command`. |
| `TypeRegistry::resolve(&type_ref, &location)` call sites (`command.rs:1573`, `:1978`; the signature is `types.rs:900`) | 0 of `command.rs`'s own | deferred | These pass a location *into* `types.rs`, which constructs the refusal; they are counted against `types.rs`, not `command.rs`. `validate_typed_guard` is the same shape into `expression.rs` and is handled the same way — it passes `owner.render()`, so the two spellings cannot drift. |
| `Outcome::try_from` (`command.rs:2163`), `keyed_sets` (`:2259`), `keyed_payload` (`:2286`), `subject_of` (`:2336`) — relative locations rebased by string concatenation at `command.rs:2447` | 1 + 1 + 2 + 4 = **8** | deferred | These build a location *relative* to a construct they do not know, and the prefix is prepended later by mutating `error.location`. The typed replacement is a `ConstructRef` passed into admission; it is a signature change on the raw→admitted conversion and is the next unit of this migration. |
| `ess-domain/src/entity.rs` (16), `binding.rs` (17), `component.rs` (15), `view.rs` (11), `topology.rs` (10), `system.rs` (10), `spec.rs` (7), `types.rs` (6), `domain.rs` (5), `wire.rs` (1) | 98 | deferred, by family | **Not** "the same shape as `command.rs`", which is what this row used to say. `entity.rs` and `component.rs` write `entity <name>` and `component <name>` with a space — 16 of the location literals in the head census below open that way (`entity entity.rs 4`, `component component.rs 12`), and every refusal those two files raise is rooted at one of them; `system.rs` and `types.rs` write the plural heads `types.` and `domain.`, for which no `ConstructKind` exists. The space form is now renderable (`ConstructKind::separator`), so those two families migrate without moving a string. The plural heads are not, and migrating them would move the string — the same result as the `validate_sets` row. |
| `ess-domain/src/wire.rs:30`, one `command.…` location | 1 | **deferred with a stated reason** | `wire.rs`'s `Namespace` helper is shared by five heads, one of which is `types.` — a head no `ConstructKind` renders. Migrating only its command caller needs the helper to accept "a site or a string", which is the shape this story is removing. It moves with the `type` family. Pinned at 1 by the head census. |
| `ess-domain/src/primitive_admission.rs:96`, one `command.…` location | 1 | **not this unit** | `story:review-primitive-semantics` owns the file. A ready patch is at `target/review-boundaries-21/scratch/primitive-admission-command-site.patch`; it is not applied. Pinned at 1 by the head census, so it cannot grow unnoticed. |
| `command.rs` `validate_sets` (`:1727`) + `field_shape` (`:1987`) + the eight admission sites above | 11 | see rows above | `grep -c 'ValidationError::new' crates/specify/ess-domain/src/command.rs` = 11, which is 2 + 1 + 8. |
| `ess-domain/src/actor.rs` | 1 | deferred | An `actor`-family site the story's own scope does not list; recorded here so the inventory is complete rather than equal to the scope. |
| `ess-domain/src/expression.rs` (1), `primitive_admission.rs` (2) | 3 | **not this unit** | `story:review-primitive-semantics`, same wave. `CommandSpec::validate_typed_guard` hands `check_predicate` the *rendered* form of its own site, so the two spellings cannot drift while that file waits. |
| `ess-primitives/src/error.rs:78` `ParseError::Shape { location: String }` | — | deferred | The same pattern on the *parse* side. A parse error has a real `serde_yaml` position, so its typed form is a `SyntaxSpan`, not a `ConstructRef`; different work. |
| A positional segment choosing a source line | — | **explicitly unsupported** | `Segment::Index` contributes a token and no needle: an index always sits behind a `STRUCTURAL` key in the paths the migrated producers write, so it never reaches the trailing-key test. `command.…input[1]` is cited at the command's own declaration, not at the second input field. Making the index choose the line means counting occurrences of a key inside a block, which `Locator`'s whole-file substring search cannot do; it is parser-position work, beside `SyntaxSpan`. The arm is kept and pinned by `typed_tokens_are_the_string_tokens_for_every_segment_shape`, which fails if it stops contributing (adversary pass 2, F7). |
| `crates/infra/infra-domain/src/code.rs:190` | — | **out of this story** | The second `ValidationError { location: String }` envelope F14 names. Recorded here, deliberately not fixed: `crates/infra/**` is another unit's assignment in this wave. |

`ess-domain/src/locate.rs` is **not** in this table: the story's scope lists it as a file whose
constructors would need migrating, and it constructs no `ValidationError` at all (grep: 0 in 259
lines). That scope line is wrong and nothing was built on it.

`bridge`'s string fallback stays until this table is empty.

## Two representations of one fact

`ValidationError::location` is `pub`, and a sited refusal renders it from its site. Nothing in the
type system stops a caller writing to it, and `ess-domain/src/command.rs` used to do exactly that,
prefixing admission's relative locations by assignment (adversary pass 2, F6).

`location` is **not** made private, and the reason is a constraint rather than a preference: the
suite's own rewording transformation — pass 1's
`a_sited_refusal_takes_its_family_from_the_construct_not_the_location_head` and pass 2's
`rewording_the_path_of_a_command_family_refusal_does_not_move_its_code` — *assigns* to `location` to
prove the machine facts do not follow it. An accessor-only field would make the story's own
acceptance statement unverifiable.

What is fixed instead is the writer. `ValidationError::rebase(&ConstructRef)` is the only supported
re-rooting; it prefixes a string-only refusal and **returns a sited one untouched**, because on a
sited refusal the site is the authority and a string prefix cannot update it. `command.rs` calls it,
no production file assigns to `.location` any more, and
`no_production_code_assigns_a_refusals_location` is the check that keeps it that way — the class,
not the instance. What remains, and is stated rather than hidden: a *test* may still assign to
`location`, which is the point, and `bridge` ignores `location` entirely for a sited refusal
(`resolve.rs`), so an assignment cannot move a code or a line.

`with_span` is gone. It took a span, and on a refusal with no site it discarded it and said nothing
(adversary pass 2, F8). A span is only meaningful beside a construct, so it is now an argument of
`ValidationError::at_span(construct, span, code, message)` — the discarding call is unspellable
rather than documented.

## Machine-checked inventory

Every `ValidationError` construction site in `crates/specify/ess-domain/src/`, as
`<file> <ValidationError::new sites> <ValidationError::at sites>`. Read and checked against the tree
by `crates/specify/ess-compiler/tests/typed_diagnostics.rs`; a file with sites that is missing from
this block, a count that does not match, and a listed file that has none, are each a failing test.

<!-- inventory:begin -->
```text
accessor.rs 1 0
actor.rs 1 0
binding.rs 28 0
command.rs 11 35
component.rs 15 0
domain.rs 5 0
entity.rs 16 4
expression.rs 1 0
primitive_admission.rs 6 1
selection.rs 2 0
spec.rs 7 0
system.rs 10 0
topology.rs 10 0
types.rs 8 1
view.rs 11 0
wire.rs 1 0
```
<!-- inventory:end -->

### By location head

The same census grouped by the head each site writes, `<head> <file> <count>`, counting every
production string literal that opens a document path — the rule adversary pass 2 F1 asked for, so a
`command.…` site booked in the `entity.rs` row cannot pass for entity work. Literals inside
`#[cfg(test)]` modules are excluded: an assertion pinning a location is a reader, not a producer.

**The `command` head is the one this wave migrated, and every remaining writer of it is named with a
reason in the table above.** There are two, both pinned at 1.

<!-- heads:begin -->
```text
actor actor.rs 1
actors actor.rs 1
binding binding.rs 18
binding primitive_admission.rs 2
command primitive_admission.rs 1
command wire.rs 1
commands command.rs 1
component component.rs 12
components component.rs 1
conversions types.rs 1
domain component.rs 1
domain domain.rs 5
domain spec.rs 1
domain system.rs 4
domains domain.rs 1
entity entity.rs 4
entity primitive_admission.rs 2
entity wire.rs 1
error command.rs 3
error primitive_admission.rs 2
error wire.rs 1
event accessor.rs 2
event binding.rs 1
event command.rs 3
event component.rs 1
event primitive_admission.rs 1
event wire.rs 1
topology topology.rs 6
types primitive_admission.rs 1
types system.rs 3
types types.rs 4
types wire.rs 1
view primitive_admission.rs 2
view view.rs 5
view wire.rs 2
```
<!-- heads:end -->

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

1. `repeated_names.yaml` — every refusal it produces is **unlocated**, and that is what it is for.
   The outcome is written `- name: filed`, so the needle `filed:` occurs zero times; the fallback
   needle `name: shop.repeat.File` occurs three times as a substring — the command itself, the
   sibling command `shop.repeat.FileTwo`, and the event `shop.repeat.Filed`. A substring search
   matching three lines knows nothing, and `Locator` reports `located: None` instead of picking the
   first. The first version of this page claimed the fixture asserted `None` while the test pinned
   all three refusals to line 12 (adversary pass 1, F3, F4); the fixture now does what the page says,
   and `adversary_typed_diagnostics_pass1.rs` holds it to that.

   It also carries the **located** half, added because answering pass 1's F3 by making every refusal
   unlocated left the suite checking no location at all for the hazard the story's Validation clause
   names (adversary pass 2, F5). `shop.repeat.Solo` repeats an outcome name too, and no other
   declaration's name contains its own, so its fallback needle is unique and both of its refusals are
   cited at `repeated_names.yaml:35:5` — pinned exactly, beside the three unlocated ones. Neither
   half was removed to make the other pass; a fixture that answers a finding by deleting the evidence
   is how F5 happened.
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
