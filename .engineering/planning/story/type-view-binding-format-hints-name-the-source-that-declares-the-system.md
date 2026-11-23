---
format: aep.planning-md/3
id: story:type-view-binding-format-hints-name-the-source-that-declares-the-system
kind: story
status: implemented
title: Every format hint of a type, view, binding or admission refusal names the source that declares the system
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-format-version-hint-names-the-source-that-declares-the-system
- depends_on: story:command-format-hints-name-the-source-that-declares-the-system
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/actor.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/condition.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/context.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/periodic.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/refusal.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/retry.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/outcome_group.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/selection.rs
- confidence: cited
  path: crates/specify/ess-domain/src/shorthand.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system_issued.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-domain/src/undeclared_fields.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/absent.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/from.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/navigation.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/paging.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/format_hint_emitted_types_and_views.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/view_paging/absent_answer.rs
revision: 25
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:44:58Z", actor: "human:timo", revision: 23}
- {from: "proposed", to: "active", at: "2026-11-21T23:44:59Z", actor: "human:timo", revision: 24}
- {from: "active", to: "implemented", at: "2026-11-23T01:57:00Z", actor: "human:timo", revision: 25, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

- The 28 format hints in the type, view, binding, actor and admission files of `ess-domain` are built with `FormatVersion::declaration_hint()`.
- The 23 format refusals in those files that carry no hint today get one, among them the reported `subject-state outcome guards require specification format ess/3`.
- `tests/format_hint_emitted_types_and_views.rs` holds the emitted hints; the source scan of the first unit covers these files once the coordinator empties its `PENDING`.
- No format version changes, no `ValidationCode` changes, no message changes.

## What to change

Line numbers are at `origin/main` (`f3e4477974`, release 0.97.1). Branch from the first commit of
`command-format-hints-name-the-source-that-declares-the-system`
(`feat(domain): one function builds the format hint`), which adds to
`crates/specify/ess-domain/src/system.rs`:

- `FormatVersion::declaration_hint(self) -> String`, returning
  ``write `format: ess/N` on the source that declares the system`` (`FormatVersion` is `Copy`
  and displays as `ess/N`);
- `crate::declaration_hint!(N)`, the same text as a `&'static str` (not needed in this unit).

Every `VNN` from `V1` to `V36` exists (`system.rs` lines 437-549). Write `FormatVersion::VNN`
where the file imports it (`binding/condition.rs:51`, `view/from.rs:46`,
`primitive_admission.rs:3`, `outcome_group.rs:13`), otherwise `crate::system::FormatVersion::VNN`.

1. **Convert every literal in the table.** Each row is one string literal: the file under
   `crates/specify/ess-domain/src/` and the line it starts on, the text today, and the text after,
   with the instruction expanded. Idioms:

   - The literal is the whole instruction (``declare `format: ess/18` `` or one of the 9 that
     already read ``write `format: ess/N` on the source that declares the system``):
     `.with_hint(crate::system::FormatVersion::V18.declaration_hint())`. The 9 keep their bytes.
   - The instruction is followed by more text: a named argument `raise`, e.g.
     `.with_hint(format!("{raise}, or drop `undeclared_fields:`", raise = crate::system::FormatVersion::V24.declaration_hint()))`.
     None of the plain literals of this unit holds a `{` or `}`, so none needs doubled braces.

   Rows that need more than the idiom:

   - `binding/condition.rs:399`:
     `.with_hint(format!("{}, or test presence with `defined(...)`", BOOLEAN_FORMAT.declaration_hint()))`
     (`BOOLEAN_FORMAT` is `FormatVersion::V27`, line 65).
   - `selection.rs:306`:
     `.with_hint(crate::binding::condition::BOOLEAN_FORMAT.declaration_hint())`.
   - `primitive_admission.rs:670`, with `lifecycle_members` (line 719): change the third member of
     the returned tuple from `&'static str` to `FormatVersion`. The return type at line 721
     becomes `Vec<(String, &'static str, FormatVersion, String)>`, and the three pushes put
     `FormatVersion::V32` (line 732, today `"ess/32"`), `FormatVersion::V33` (line 740) and
     `FormatVersion::V33` (line 751). The message at line 668,
     `format!("{what} requires specification format {needs}")`, keeps its bytes. The hint becomes
     `.with_hint(format!("{}, or {instead}", needs.declaration_hint()))`. The doc comment of
     `lifecycle_members` (line 713, "the format that admits it") still holds.
   - `types.rs:1289`: the second argument of `declares_nothing(&self, part: &str, hint: &str)`
     (line 1240) becomes
     `&format!("give it at least one, delete the declaration, or {}, where a struct may declare no fields", crate::system::FormatVersion::V32.declaration_hint())`.
   - `view/from.rs:291-295`:
     `let format = if below_ess_28 { format!("{}, which reads it, and ", FormatVersion::V28.declaration_hint()) } else { String::new() };`
     The `.with_hint(format!("{format}write `field: …"))` below it stays.

   | Site | Today | After |
   |---|---|---|
   | `actor.rs:97` | `` write `format: ess/27` on the source that declares the system `` | `` write `format: ess/27` on the source that declares the system `` |
   | `actor.rs:145` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `actor.rs:258` | `` write `format: ess/22` on the source that declares the system `` | `` write `format: ess/22` on the source that declares the system `` |
   | `binding/condition.rs:399` | `` declare `format: ess/{}`, or test presence with `defined(...)` `` | `` write `format: ess/{}` on the source that declares the system, or test presence with `defined(...)` `` |
   | `binding/condition.rs:527` | `` declare `format: ess/22`, or remove `where:` `` | `` write `format: ess/22` on the source that declares the system, or remove `where:` `` |
   | `binding/context.rs:118` | `` declare `format: ess/18` `` | `` write `format: ess/18` on the source that declares the system `` |
   | `binding/context.rs:133` | `` declare `format: ess/18` `` | `` write `format: ess/18` on the source that declares the system `` |
   | `binding/refusal.rs:451` | `` declare `format: ess/22`, or write one universal policy: `retry`, `drop`, or `escalate:` with `emits:` `` | `` write `format: ess/22` on the source that declares the system, or write one universal policy: `retry`, `drop`, or `escalate:` with `emits:` `` |
   | `binding/retry.rs:231` | `` write `format: ess/16` on the source that declares the system, or write `on_failure: retry` with no block `` | `` write `format: ess/16` on the source that declares the system, or write `on_failure: retry` with no block `` |
   | `entity.rs:1108` | `` write `format: ess/33` on the source that declares the system `` | `` write `format: ess/33` on the source that declares the system `` |
   | `primitive_admission.rs:212` | `` declare `format: ess/22`, or give the variant a type `` | `` write `format: ess/22` on the source that declares the system, or give the variant a type `` |
   | `primitive_admission.rs:360` | `` declare `format: ess/18` `` | `` write `format: ess/18` on the source that declares the system `` |
   | `primitive_admission.rs:670` | `` declare `format: {needs}`, or {instead} `` | `` write `format: {needs}` on the source that declares the system, or {instead} `` |
   | `selection.rs:306` | `` declare `format: ess/{}` `` | `` write `format: ess/{}` on the source that declares the system `` |
   | `shorthand.rs:190` | `` declare `format: ess/27`, or drop `shorthand:` `` | `` write `format: ess/27` on the source that declares the system, or drop `shorthand:` `` |
   | `system_issued.rs:69` | `` write `format: ess/27` on the source that declares the system `` | `` write `format: ess/27` on the source that declares the system `` |
   | `types.rs:1271` | `` write `format: ess/36` on the source that declares the system `` | `` write `format: ess/36` on the source that declares the system `` |
   | `types.rs:1289` | `` give it at least one, delete the declaration, or declare `format: ess/32`, where a struct may declare no fields `` | `` give it at least one, delete the declaration, or write `format: ess/32` on the source that declares the system, where a struct may declare no fields `` |
   | `types.rs:1650` | `` write `format: ess/23` on the source that declares the system `` | `` write `format: ess/23` on the source that declares the system `` |
   | `undeclared_fields.rs:320` | `` declare `format: ess/24`, or drop `undeclared_fields:` `` | `` write `format: ess/24` on the source that declares the system, or drop `undeclared_fields:` `` |
   | `view.rs:1722` | `` declare `format: ess/36`, from which a measure admits every type grid O orders `` | `` write `format: ess/36` on the source that declares the system, from which a measure admits every type grid O orders `` |
   | `view.rs:1757` | `` declare `format: ess/36`, or measure one field of the source `` | `` write `format: ess/36` on the source that declares the system, or measure one field of the source `` |
   | `view.rs:1830` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `view/absent.rs:83` | `` declare `format: ess/25`, or drop `absent:` `` | `` write `format: ess/25` on the source that declares the system, or drop `absent:` `` |
   | `view/from.rs:292` | `` declare `format: ess/28`, which reads it, and  `` | `` write `format: ess/28` on the source that declares the system, which reads it, and  `` |
   | `view/navigation.rs:205` | `` declare `format: ess/28`, or copy the value onto the row when it is written `` | `` write `format: ess/28` on the source that declares the system, or copy the value onto the row when it is written `` |
   | `view/navigation.rs:220` | `` declare `format: ess/28`, or filter on a value copied onto the row `` | `` write `format: ess/28` on the source that declares the system, or filter on a value copied onto the row `` |
   | `view/paging.rs:229` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |

2. **The refusals with no hint.** Add `.with_hint(<format>.declaration_hint())` to each of these
   `ValidationCode::UnsupportedFormatVersion` refusals (the line is that of
   `ValidationCode::UnsupportedFormatVersion`). The message stays byte for byte. Where the
   refusal is pushed as `errors.push(ValidationError::new(…))`, the hint goes on the
   `ValidationError` inside the push: `errors.push(ValidationError::new(…).with_hint(…))`.

   | Site | Message today | Format of the hint |
   |---|---|---|
   | `binding/periodic.rs:188` | `periodic binding causes require ess/3` | `crate::system::FormatVersion::V3` |
   | `outcome_group.rs:283` | `outcome groups require specification format ess/12` | `FormatVersion::V12` |
   | `primitive_admission.rs:18` | `Binary64 requires specification format ess/2` | `FormatVersion::V2` |
   | `primitive_admission.rs:27` | `Json requires specification format ess/15` | `FormatVersion::V15` |
   | `primitive_admission.rs:113` | `message`: `` `presence: omitted_or_null` requires … ess/31`` or `field presence policies require … ess/15` | `required` (the `FormatVersion` bound at line 101) |
   | `primitive_admission.rs:160` | `clock reading contracts require specification format ess/3` | `FormatVersion::V3` |
   | `primitive_admission.rs:176` | `declared alphabets require specification format ess/11` | `FormatVersion::V11` |
   | `primitive_admission.rs:184` | `declared prefixes require specification format ess/15` | `FormatVersion::V15` |
   | `primitive_admission.rs:229` | `declared enum variant naming requires specification format ess/5` | `FormatVersion::V5` |
   | `primitive_admission.rs:296` | `retained results and effect-free state defaults require specification format ess/7` | `FormatVersion::V7` |
   | `primitive_admission.rs:314` | `guarded external outcomes, subject facts and preservation require specification format ess/6` | `FormatVersion::V6` |
   | `primitive_admission.rs:324` | `subject predicates require specification format ess/9` | `FormatVersion::V9` |
   | `primitive_admission.rs:331` | `subject-state outcome guards require specification format ess/3` (the reported refusal) | `FormatVersion::V3` |
   | `primitive_admission.rs:367` | `state-change outcome guards require specification format ess/4` | `FormatVersion::V4` |
   | `primitive_admission.rs:515` | `aggregate views require specification format ess/10` | `FormatVersion::V10` |
   | `primitive_admission.rs:558` | `*message` from `OPERATOR_FORMATS` (ess/8, ess/15, ess/25, …) | `admitted` (the tuple's `FormatVersion`, line 554) |
   | `primitive_admission.rs:583` | `input examples require specification format ess/11` | `FormatVersion::V11` |
   | `primitive_admission.rs:632` | `bounded list selection requires specification format ess/3` | `FormatVersion::V3` |
   | `primitive_admission.rs:642` | `bounded event accessors require specification format ess/3` | `FormatVersion::V3` |
   | `primitive_admission.rs:691` | `declared error naming requires specification format ess/4` | `FormatVersion::V4` |
   | `view.rs:2081` | `` `skip_absent` requires specification format ess/15`` | `crate::system::FormatVersion::V15` |
   | `view.rs:2093` | `` `{}` groups by `{key}`, which may be absent; a group key that may be absent requires specification format ess/15`` | `crate::system::FormatVersion::V15` |
   | `view.rs:2128` | `` `{}.{}` reads only the rows its `where:` admits; a measure's condition requires specification format ess/22`` | `crate::system::FormatVersion::V22` |

   `primitive_admission.rs:558`: `admitted` is a `&FormatVersion` there (the loop iterates the
   slice `OPERATOR_FORMATS`); `admitted.declaration_hint()` compiles as written.

3. **Re-pin the one test that compares an old hint exactly.**
   `crates/specify/ess-domain/tests/view_paging/absent_answer.rs:107-110`, in
   `absent_is_refused_below_ess_25`: the expected line becomes
   ``[unsupported_format_version] view.demo.status.FlagById.absent: a view's answer for an identity no row carries (`absent:`) requires specification format ess/25 (hint: write `format: ess/25` on the source that declares the system, or drop `absent:`)``
   (keep the `\` continuations). Put
   `// Re-pinned by story:every-format-version-hint-names-the-source-that-declares-the-system.`
   above the `assert_eq!`.

   No other test compares the text of a hint or the absence of a hint this unit changes: every
   other test that names a message of step 2 reads it with `contains` or compares `message` only
   (`aggregate_views.rs:556`, `outcome_groups.rs:664`, `string_alphabet_and_length.rs:192`,
   `:376`, `stored_field_guards.rs:140`, `state_change.rs:359`, `subject_state.rs:105`,
   `adversary_guards_positions.rs:116`, `ess-compiler/tests/error_wire.rs:40`).

## Tests first

New file `crates/specify/ess-domain/tests/format_hint_emitted_types_and_views.rs`. Each test
validates one small model in-process and runs in well under a second.

```rust
//! The format hints of type, view and admission refusals, as emitted, name the source that
//! declares the system (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError};

/// An order shop whose `CancelOrder` guards on held states, one a list and one on a refusal that
/// names no subject: an `ess/3` construct holding two `ess/18` shapes. Valid at `ess/18`.
const SHOP: &str = "…";

/// A struct that ignores undeclared fields, an `ess/24` key. Valid at `ess/24`.
const NOTE: &str = "format: ess/1
system: demo
version: v1
domain: demo.shop
types:
  - name: demo.shop.Note
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: text, type: String}
";

fn refusals(text: &str) -> Vec<ValidationError> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("parses: {error}"));
    Specification::assemble([(Source::new("model.yaml"), raw)])
        .expect_err("refused")
        .as_slice()
        .to_vec()
}

fn refusal_at<'a>(errors: &'a [ValidationError], location: &str) -> &'a ValidationError {
    errors
        .iter()
        .find(|error| error.location == location)
        .unwrap_or_else(|| panic!("no refusal at {location}: {errors:#?}"))
}
```

`SHOP` is, byte for byte, the model below (YAML lines start in column 0 inside the Rust string;
checked with `ess specify validate --path` on the released 0.97.1: valid at `ess/18`, and at
`ess/2` refused with exactly the four format refusals named in the first test):

```yaml
format: ess/18
system: demo
version: v1
domain: demo.ship
types:
  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}
entities:
  - name: demo.ship.Order
    identity: {name: order_id, type: demo.ship.OrderId}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed, Shipped, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: ship, from: [Placed], to: Shipped}
        - {name: cancel, from: [Placed, Shipped], to: Cancelled}
errors:
  - {name: demo.ship.Gone, fields: []}
events:
  - name: demo.ship.OrderPlaced
    fields: [{name: order_id, type: demo.ship.OrderId}]
  - name: demo.ship.OrderShipped
    fields: [{name: order_id, type: demo.ship.OrderId}]
  - name: demo.ship.OrderCancelled
    fields: [{name: order_id, type: demo.ship.OrderId}]
commands:
  - name: demo.ship.PlaceOrder
    input: []
    outcomes:
      - name: placed
        creates: demo.ship.Order
        instance: order_id
        emits: [demo.ship.OrderPlaced]
        payload: {demo.ship.OrderPlaced: {order_id: {generated: true}}}
  - name: demo.ship.ShipOrder
    input: [{name: order_id, type: demo.ship.OrderId}]
    outcomes:
      - name: shipped
        moves: demo.ship.Order.ship
        instance: order_id
        emits: [demo.ship.OrderShipped]
        payload: {demo.ship.OrderShipped: {order_id: input.order_id}}
  - name: demo.ship.CancelOrder
    input: [{name: order_id, type: demo.ship.OrderId}]
    outcomes:
      - name: cancelled
        when_subject_state: [Placed, Shipped]
        moves: demo.ship.Order.cancel
        instance: order_id
        emits: [demo.ship.OrderCancelled]
        payload: {demo.ship.OrderCancelled: {order_id: input.order_id}}
      - name: gone
        when_subject_state: Cancelled
        error: demo.ship.Gone
```

Tests:

1. `held_state_guards_below_their_formats_name_the_source`: `refusals(&SHOP.replacen("format: ess/18", "format: ess/2", 1))`.
   - At `command.demo.ship.CancelOrder.outcomes`: code `UnsupportedFormatVersion`, message
     `subject-state outcome guards require specification format ess/3`, hint
     `Some("write `format: ess/3` on the source that declares the system")`.
   - At `command.demo.ship.CancelOrder.outcomes.cancelled.when_subject_state` and at
     `command.demo.ship.CancelOrder.outcomes.gone.when_subject_state`: hint
     `Some("write `format: ess/18` on the source that declares the system")`.
   - Do not assert on the fourth refusal,
     `command.demo.ship.PlaceOrder.outcomes.placed.payload.demo.ship.OrderPlaced.order_id`
     (`ess/4`): its hint comes from `command.rs`, which the first unit owns.

   Red before the change: the first hint is `None`, the other two read ``declare `format: ess/18` ``.
2. `undeclared_fields_below_ess_24_name_the_source`: `refusals(NOTE)`; at
   `type.demo.shop.Note.undeclared_fields`: code `UnsupportedFormatVersion`, hint
   `Some("write `format: ess/24` on the source that declares the system, or drop `undeclared_fields:`")`.
   Red before: ``declare `format: ess/24`, or drop `undeclared_fields:` ``.
3. `a_binary64_field_at_ess_1_names_the_source`: a copy of `NOTE` whose field is
   `{name: text, type: Binary64}` and with no `undeclared_fields:` line; the one refusal with
   code `UnsupportedFormatVersion` has message `Binary64 requires specification format ess/2` and
   hint `Some("write `format: ess/2` on the source that declares the system")`. Red before: `None`.
   (Its location is the field's path; find the refusal by code, not by location.)

## Commands

From the worktree root. Prefix every `cargo` command with
`RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`
(`AGENTS.md`, "Local compile cache"); `cargo` goes through the `slot` shim on its own.

1. `cargo test -p ess-domain --locked --test format_hint_emitted_types_and_views`: 3 failed
   before the change, 3 passed after.
2. `cargo test -p ess-domain --locked --test view_paging`: passes after the re-pin.
3. One command each, all pass: `--test aggregate_views`, `--test outcome_groups`,
   `--test string_alphabet_and_length`, `--test subject_state`, `--test state_change`,
   `--test stored_field_guards`, `--test union_unit_variants`, `--test adversary_guards_positions`
   (each `cargo test -p ess-domain --locked --test <name>`).
4. `cargo test -p ess-domain --locked`: all pass (the package gate).
5. `cargo test -p ess-compiler --locked --test typed_diagnostics`, `--test error_wire` and
   `--test string_alphabet_and_length` (the last a byte-pin target of `AGENTS.md`): pass.
6. `cargo clippy -p ess-domain --all-targets --locked -- -D warnings`: no warnings.
7. `cargo fmt -p ess-domain`, then `slot task fmt-check`: clean. Never `cargo fmt --all`.

Do not run `--test format_hint_source_scan` as a gate in this unit: its `PENDING` list (owned by
the first unit) already names every file of this unit, so it passes whether or not they are
converted. To check this unit's files yourself, read
`rg -n '`format: ess/|`format: \{' crates/specify/ess-domain/src/<file>` over the files below: the
only lines left are comments.

## Files this unit owns

All under `crates/specify/ess-domain/`:

- `src/actor.rs`, `src/entity.rs`, `src/outcome_group.rs`, `src/primitive_admission.rs`,
  `src/selection.rs`, `src/shorthand.rs`, `src/system_issued.rs`, `src/types.rs`,
  `src/undeclared_fields.rs`, `src/view.rs`
- `src/binding/condition.rs`, `src/binding/context.rs`, `src/binding/periodic.rs`,
  `src/binding/refusal.rs`, `src/binding/retry.rs`
- `src/view/absent.rs`, `src/view/from.rs`, `src/view/navigation.rs`, `src/view/paging.rs`
- `tests/format_hint_emitted_types_and_views.rs` (new)
- `tests/view_paging/absent_answer.rs`

Must not touch:

- `src/system.rs`, `src/command.rs`, `src/command/**`, `tests/format_hint_source_scan.rs`,
  `tests/format_hint_emitted_commands.rs` (first unit);
- `src/expression.rs`, `src/spec.rs`, `tests/coalesce_selector.rs`,
  `crates/verify/ess-conformance/**`, `crates/edge/ess-cli/**` (third unit);
- `crates/generate/ess-synth/**`, `crates/edge/ess-xtask/**`, `.config/nextest.toml`,
  `Taskfile.yml`, `.github/workflows/*`, `rust-toolchain.toml` (other units of this wave);
- `CHANGELOG.md`, `docs/`, `website/`, `AGENTS.md` (coordinator).

## For the coordinator at integration

- Nothing of its own. `PENDING` in `crates/specify/ess-domain/tests/format_hint_source_scan.rs`
  names this unit's files; the first unit's coordinator lines empty it once all three units are
  merged.
- The `CHANGELOG.md` and `docs/design/` lines for the whole change are given in the unit
  `expression-format-hints-and-the-header-refusal-name-the-system-source`.

## Out of scope

- Hints that name a version in prose only and are emitted at every format as a note on an
  alternative: `binding.rs:1388`, `binding/condition.rs:323`, `view.rs:1707`, `view.rs:1886`,
  `view.rs:2035`, `view/from.rs:316` ("(ess/28)", "(ess/27)"). They stay as they are: they are
  not refusals of a format below the one they name.
- `binding.rs` is not in this unit: its only format text is the prose hint at line 1388.
- Any refusal message: none changes.
- No new `ValidationCode`, no new refusal, no format version.

## Acceptance

- The reported model at `ess/2` draws the `ess/3` refusal with the hint
  ``write `format: ess/3` on the source that declares the system``, and its two `ess/18`
  refusals name the source (Tests first 1, command 1).
- `undeclared_fields:` below `ess/24` and `Binary64` at `ess/1` name the source (Tests first 2
  and 3).
- The 23 refusals of step 2 carry a hint and the 28 literals of step 1 are gone from the source;
  after integration the first unit's scan and corpus tests hold both.
- The `ess-domain` package gate, the `ess-compiler` targets, clippy and fmt pass (commands 2-7).

## Depends on

`command-format-hints-name-the-source-that-declares-the-system`, its first commit only
(`FormatVersion::declaration_hint`). Branch from that commit; no other file of that unit is read
or changed here.
