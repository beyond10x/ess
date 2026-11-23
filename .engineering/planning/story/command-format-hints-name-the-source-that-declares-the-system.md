---
format: aep.planning-md/3
id: story:command-format-hints-name-the-source-that-declares-the-system
kind: story
status: implemented
title: Every format hint of a command refusal names the source that declares the system
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-format-version-hint-names-the-source-that-declares-the-system
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/absent_input.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/append_list.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/caller_value.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/identity_write.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/input_path.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/minted_value.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/one_time_response.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/ordered_head.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/published_operand.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/reference_integrity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_list.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_rows.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/row_set.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/adversary_m2_pass1.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass1.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/format_hint_emitted_commands.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/format_hint_source_scan.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/published_value_operand.rs
revision: 30
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:44:36Z", actor: "human:timo", revision: 28}
- {from: "proposed", to: "active", at: "2026-11-21T23:44:37Z", actor: "human:timo", revision: 29}
- {from: "active", to: "implemented", at: "2026-11-23T01:56:59Z", actor: "human:timo", revision: 30, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

- `FormatVersion::declaration_hint()` and `declaration_hint!(N)` build ``write `format: ess/N` on the source that declares the system``.
- The 71 format hints in `command.rs` and `command/**` are built with them; the 6 format refusals there with no hint get one.
- `tests/format_hint_source_scan.rs` refuses any other literal spelling the instruction (follow-up units' files on `PENDING`).
- Its ignored corpus test fails on any format refusal without the hint; the coordinator turns it on at integration.
- No format version changes, no `ValidationCode` changes.

## What to change

Line numbers are at `origin/main` (`f3e4477974`, release 0.97.1). Commit step 1 on its own
first (`feat(domain): one function builds the format hint`): the units
`type-view-binding-format-hints-name-the-source-that-declares-the-system` and
`expression-format-hints-and-the-header-refusal-name-the-system-source` are written against it and
branch from that commit.

1. **The function.** `crates/specify/ess-domain/src/system.rs`, inside `impl FormatVersion`,
   directly after `pub const fn major(self) -> u32 { self.0 }` (lines 569-572), add:

   ```rust
       /// The repair of a refusal that needs this format, ``write `format: ess/N` on the source
       /// that declares the system``.
       ///
       /// The `format:` key belongs to the one source that carries `system:`. A hint that names
       /// only the version sends a writer to the file the diagnostic points at, which is the
       /// construct's file and not the system's
       /// (`story:every-format-version-hint-names-the-source-that-declares-the-system`). Every
       /// hint that asks for a specification format is built here, or with
       /// [`crate::declaration_hint!`] where a `&'static str` is needed;
       /// `tests/format_hint_source_scan.rs` refuses any other literal that spells one.
       #[must_use]
       pub fn declaration_hint(self) -> String {
           format!("write `format: {self}` on the source that declares the system")
       }
   ```

   `FormatVersion` is `Copy` and displays as `ess/N` (`impl fmt::Display`, line 606), so
   `FormatVersion::V18.declaration_hint()` is
   ``write `format: ess/18` on the source that declares the system``. It is exported as
   `ess_domain::FormatVersion` already (`lib.rs:91`, `pub use system::{FormatVersion, SystemSpec}`).

2. **The macro.** Same file, directly after the `impl fmt::Display for FormatVersion` block
   (lines 606-610), add, with the `concat!` on ONE line exactly as written (the scan test exempts
   that line by its text):

   ```rust
   /// [`crate::system::FormatVersion::declaration_hint`] for a format written as a literal, as a
   /// `&'static str`, for a hint that must be a constant: `declaration_hint!(31)` is
   /// ``"write `format: ess/31` on the source that declares the system"``.
   #[macro_export]
   macro_rules! declaration_hint {
       ($major:literal) => {
           concat!("write `format: ess/", $major, "` on the source that declares the system")
       };
   }
   ```

   Inside `ess-domain` call it as `crate::declaration_hint!(31)`; from another crate as
   `ess_domain::declaration_hint!(30)`. It nests inside `concat!`:
   `concat!(crate::declaration_hint!(27), ", where …")` is a `&'static str`.

3. **Convert every site in the table below.** Each row is one string literal: the file and the
   line the literal starts on, the text today, and the text after. The "after" column is the
   string as it reads in the source after the change, with the instruction already expanded;
   `format!` placeholders that are there today (`{field}`, `{}`) stay as they are. Code idioms:

   - The literal is the whole instruction (`"declare `format: ess/18`"`, or one of the 10 that
     already read ``write `format: ess/N` on the source that declares the system``):
     `.with_hint(FormatVersion::V18.declaration_hint())`.
   - The instruction is followed or surrounded by more text: build it with a named argument
     `raise`, for example
     `.with_hint(format!("{raise}, or drop `before_row:`", raise = FormatVersion::V25.declaration_hint()))`.
     Where the literal is already a `format!` string, add `raise = …` as its last argument and put
     `{raise}` where the instruction stood. Where it was a plain literal and becomes a `format!`
     string, double every `{` and `}` it already holds. That applies to 4 rows:
     `command.rs:4800` and `command.rs:7784` (`{predicate: …}`), `command/row_set.rs:590`
     (`{via: input.<field>, …}`) and `command/value_expression.rs:1481` (`{generated: true}`).
   - Write `FormatVersion::VNN` where the file has `FormatVersion` in scope
     (`command/value_expression.rs` does: it uses `FormatVersion::V14` at line 403), otherwise
     `crate::system::FormatVersion::VNN`. Every `VNN` from `V1` to `V36` exists (system.rs
     lines 437-549); there is no `V37`.

   Rows that need more than that idiom:

   - `command.rs:6231` and `command.rs:6307`: `const BELOW: &str`, used as the
     `&'static str` third member of `ExampleRefusal` (line 6216). Keep them `&'static str`:
     `const BELOW: &str = concat!("an example sits on a scalar input below ess/31; ", crate::declaration_hint!(31), " to give a list input one");`
     and the same shape for `ess/33` with ``"an example sits on a scalar or list input below
     ess/33; "`` and ``" to give a struct, map or Json input one"``.
   - `command/subject_fact.rs:24`: `pub(crate) const LIFECYCLE_MIX_HINT: &str`, used at
     `command.rs:4166` and `command/subject_fact.rs:251`. Keep it a constant:
     `concat!(crate::declaration_hint!(27), ", where the one precedence rule reads both at its held-state step in declaration order, or read the held state in the predicate instead: replace `when_subject_state:` with `when_subject: {predicate: state == Active}`, or refuse every other state with `when_subject: {predicate: state != Active}` declared before the default")`
     (the tail copied byte for byte from today's literal, without the leading
     ``declare `format: ess/27` ``).
   - `command.rs:7784`: the third argument of the local `conflict` closure, which takes `&str`
     (line 7540: `|key: &str, message: String, hint: &str|`). Pass `&format!(…)`.
   - `command/row_set.rs:389`: the `if identity { … } else { … }` inside `.with_hint(…)` has two
     `&str` arms. The first becomes `format!(…)`, the second gets `.to_owned()`.
   - `command/set_effects.rs:884` and `:911`: the hint is the second member of a
     `(message, hint)` tuple whose other branch is a `&str` literal (lines 877-898 and 905-921).
     The first branch becomes `format!(…)`, the other branch gets `.to_owned()`.
   - `command/set_effects.rs:2167`: `refused.with_hint(format!(…))`.
   - `command/reference_integrity.rs:59`: already `format!` with the captured `{hint}`; add
     `raise = …`.
   - `command/related_guard.rs:782` is the hint of ESS-COMMAND-004 that the active story
     `story:the-ess-27-hint-states-the-order-of-an-unknown-identity-and-a-missing-related-row`
     rewrites later. Change only its first words, ``declare `format: ess/27` `` to `{raise}`.
     Everything after them stays byte for byte (``, where the unknown addressed identity answers
     after an `exists: false` over the input's row and before the related row's predicates, or
     split the other guard into a command of its own``).

   | Site | Today | After |
   |---|---|---|
   | `command.rs:3762` | `` declare `format: ess/27`, where the one precedence rule answers `before_row:` at step 1 and the related row at step 4, or drop `before_row:`, or the `when_related:` guard `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule answers `before_row:` at step 1 and the related row at step 4, or drop `before_row:`, or the `when_related:` guard `` |
   | `command.rs:4608` | `` from specification format ess/27 this branch reads `input.{}` present, since its `when_related:` reads the row through it: declare `format: ess/27` `` | `` from specification format ess/27 this branch reads `input.{}` present, since its `when_related:` reads the row through it: write `format: ess/27` on the source that declares the system `` |
   | `command.rs:4709` | `` declare `format: ess/19` `` | `` write `format: ess/19` on the source that declares the system `` |
   | `command.rs:4751` | `` declare `format: ess/22`, or drop `compensates:` and the change it declares `` | `` write `format: ess/22` on the source that declares the system, or drop `compensates:` and the change it declares `` |
   | `command.rs:4769` | `` declare `format: ess/25`, or drop `before_row:` `` | `` write `format: ess/25` on the source that declares the system, or drop `before_row:` `` |
   | `command.rs:4800` | `` declare `format: ess/29`, where `when_subject: {predicate: …}` narrows an `external:` branch to the records it selects, or drop one of the two `` | `` write `format: ess/29` on the source that declares the system, where `when_subject: {predicate: …}` narrows an `external:` branch to the records it selects, or drop one of the two `` |
   | `command.rs:4858` | `` declare `format: ess/34`, where a compensating refusal may delete its addressed row, or declare the change with `moves:` or `updates:` and `sets:` `` | `` write `format: ess/34` on the source that declares the system, where a compensating refusal may delete its addressed row, or declare the change with `moves:` or `updates:` and `sets:` `` |
   | `command.rs:4880` | `` declare `format: ess/30`, where a race narrowed by `when_subject:` may compensate, or drop `compensates:` and the change it declares `` | `` write `format: ess/30` on the source that declares the system, where a race narrowed by `when_subject:` may compensate, or drop `compensates:` and the change it declares `` |
   | `command.rs:5702` | `` declare `format: ess/31` `` | `` write `format: ess/31` on the source that declares the system `` |
   | `command.rs:6231` | `` an example sits on a scalar input below ess/31; declare `format: ess/31` to give a list input one `` | `` an example sits on a scalar input below ess/31; write `format: ess/31` on the source that declares the system to give a list input one `` |
   | `command.rs:6307` | `` an example sits on a scalar or list input below ess/33; declare `format: ess/33` to give a struct, map or Json input one `` | `` an example sits on a scalar or list input below ess/33; write `format: ess/33` on the source that declares the system to give a struct, map or Json input one `` |
   | `command.rs:7784` | `` from `format: ess/29` an `external:` branch may be narrowed to the records `when_subject: {predicate: <field> == <value>}` selects; write the field test as that predicate `` | `` write `format: ess/29` on the source that declares the system, from which an `external:` branch may be narrowed to the records `when_subject: {predicate: <field> == <value>}` selects; write the field test as that predicate `` |
   | `command/absent_input.rs:266` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/append_list.rs:164` | `` declare `format: ess/31` `` | `` write `format: ess/31` on the source that declares the system `` |
   | `command/caller_value.rs:300` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/caller_value.rs:433` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/identity_write.rs:99` | `` declare `format: ess/23` to rename the record, or drop `{identity}` from `sets:` `` | `` write `format: ess/23` on the source that declares the system to rename the record, or drop `{identity}` from `sets:` `` |
   | `command/input_path.rs:228` | `` write `format: ess/22` on the source that declares the system `` | `` write `format: ess/22` on the source that declares the system `` |
   | `command/minted_value.rs:234` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/ordered_head.rs:362` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/outcome_shapes.rs:369` | `` declare `format: ess/27`, where a creation pairs with a branch that preserves `{}`, or declare the `updates:` or `moves:` branch on it with `instance: {identity}` `` | `` write `format: ess/27` on the source that declares the system, where a creation pairs with a branch that preserves `{}`, or declare the `updates:` or `moves:` branch on it with `instance: {identity}` `` |
   | `command/outcome_shapes.rs:546` | `` write `format: ess/15` on the source that declares the system `` | `` write `format: ess/15` on the source that declares the system `` |
   | `command/outcome_shapes.rs:555` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/outcome_shapes.rs:622` | `` write `format: ess/15` on the source that declares the system `` | `` write `format: ess/15` on the source that declares the system `` |
   | `command/published_operand.rs:294` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/reference_integrity.rs:59` | `` {hint}, or declare `format: ess/30`, which needs none here: every write of the reference checks the row, and no command deletes one `` | `` {hint}, or write `format: ess/30` on the source that declares the system, which needs none here: every write of the reference checks the row, and no command deletes one `` |
   | `command/related_guard.rs:782` | `` declare `format: ess/27`, where the unknown addressed identity answers after an `exists: false` over the input's row and before the related row's predicates, or split the other guard into a command of its own `` | `` write `format: ess/27` on the source that declares the system, where the unknown addressed identity answers after an `exists: false` over the input's row and before the related row's predicates, or split the other guard into a command of its own `` |
   | `command/related_guard.rs:818` | `` declare `format: ess/27`, where the one precedence rule orders the two, guard the command on the related row alone, or split the other guard into a command of its own `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule orders the two, guard the command on the related row alone, or split the other guard into a command of its own `` |
   | `command/related_guard.rs:1092` | `` declare `format: ess/22`, or read the related row through `{first}` `` | `` write `format: ess/22` on the source that declares the system, or read the related row through `{first}` `` |
   | `command/related_guard.rs:1167` | `` declare `format: ess/27`, where the one precedence rule orders the two, guard the command on the related row alone, or split the other guard into a command of its own `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule orders the two, guard the command on the related row alone, or split the other guard into a command of its own `` |
   | `command/related_guard.rs:1297` | `` declare `format: ess/18` `` | `` write `format: ess/18` on the source that declares the system `` |
   | `command/related_guard.rs:1322` | `` declare `format: ess/28`, or split the rows into one refusal each `` | `` write `format: ess/28` on the source that declares the system, or split the rows into one refusal each `` |
   | `command/related_guard.rs:1428` | `` declare `format: ess/22`, or make the input required `` | `` write `format: ess/22` on the source that declares the system, or make the input required `` |
   | `command/related_guard.rs:1472` | `` declare `format: ess/22`, or read the identity from the input: `via: input.<field>` `` | `` write `format: ess/22` on the source that declares the system, or read the identity from the input: `via: input.<field>` `` |
   | `command/related_guard.rs:1777` | `` declare `format: ess/20` `` | `` write `format: ess/20` on the source that declares the system `` |
   | `command/related_guard.rs:1948` | `` declare `format: ess/27`, where the one precedence rule answers the branch it reads first, within one step the first declared, or make the guards disjoint `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule answers the branch it reads first, within one step the first declared, or make the guards disjoint `` |
   | `command/related_list.rs:98` | `` declare `format: ess/28` to refuse a list naming a missing row with `exists: false`, or read an input typed as the other entity's identity `` | `` write `format: ess/28` on the source that declares the system to refuse a list naming a missing row with `exists: false`, or read an input typed as the other entity's identity `` |
   | `command/related_rows.rs:104` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/row_set.rs:389` | `` declare `format: ess/32`, where the one precedence rule answers the named rows' `exists: false` first and then every refusal over a present row or a row set in declaration order, guard the command on the row set alone, or move the other guard to a command of its own `` | `` write `format: ess/32` on the source that declares the system, where the one precedence rule answers the named rows' `exists: false` first and then every refusal over a present row or a row set in declaration order, guard the command on the row set alone, or move the other guard to a command of its own `` |
   | `command/row_set.rs:590` | `` declare `format: ess/22`, or guard on one row named by an input identity with `when_related: {via: input.<field>, …}` `` | `` write `format: ess/22` on the source that declares the system, or guard on one row named by an input identity with `when_related: {via: input.<field>, …}` `` |
   | `command/set_effects.rs:447` | `` declare `format: ess/25`, or list `{event}` by name to publish it once `` | `` write `format: ess/25` on the source that declares the system, or list `{event}` by name to publish it once `` |
   | `command/set_effects.rs:884` | `` drop `instance:`, or declare `format: ess/23` and write one row per element with `each:` and `instance:`; to change one existing row by its identity, write `where: <identity> == subject.<field>` (ess/27) `` | `` drop `instance:`, or write `format: ess/23` on the source that declares the system and write one row per element with `each:` and `instance:`; to change one existing row by its identity, write `where: <identity> == subject.<field>` (ess/27) `` |
   | `command/set_effects.rs:911` | `` select the rows with `where: <predicate>`, or declare `format: ess/23` and write one row per element with `each:` and `instance:` `` | `` select the rows with `where: <predicate>`, or write `format: ess/23` on the source that declares the system and write one row per element with `each:` and `instance:` `` |
   | `command/set_effects.rs:1332` | `` declare `format: ess/16` `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/set_effects.rs:1415` | `` declare `format: ess/35`, or hold the member distinct with a declared `distinct:` `` | `` write `format: ess/35` on the source that declares the system, or hold the member distinct with a declared `distinct:` `` |
   | `command/set_effects.rs:1974` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/set_effects.rs:2167` | `` a set effect's filter reads the row's held state as `state` from ess/30; declare `format: ess/30` `` | `` a set effect's filter reads the row's held state as `state` from ess/30; write `format: ess/30` on the source that declares the system `` |
   | `command/set_effects.rs:2435` | `` declare `format: ess/16` `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/set_effects.rs:2480` | `` declare `format: ess/22`, or declare `sets:` on the affected rows and move them with a command of their own `` | `` write `format: ess/22` on the source that declares the system, or declare `sets:` on the affected rows and move them with a command of their own `` |
   | `command/set_effects.rs:2565` | `` declare `format: ess/23`, or name one row with `instance:` `` | `` write `format: ess/23` on the source that declares the system, or name one row with `instance:` `` |
   | `command/set_effects.rs:2581` | `` declare `format: ess/23`, or move `affects:` to the branch that moves or updates an existing row `` | `` write `format: ess/23` on the source that declares the system, or move `affects:` to the branch that moves or updates an existing row `` |
   | `command/set_effects.rs:2594` | `` declare `format: ess/23` `` | `` write `format: ess/23` on the source that declares the system `` |
   | `command/set_effects.rs:3046` | `` declare `format: ess/23` `` | `` write `format: ess/23` on the source that declares the system `` |
   | `command/set_effects.rs:3063` | `` keep `each:` with `instance:` and `sets:`, or write the other effect as an entry of its own; from `format: ess/27` the entry may move or delete the row each element names `` | `` keep `each:` with `instance:` and `sets:`, or write the other effect as an entry of its own; write `format: ess/27` on the source that declares the system, from which the entry may move or delete the row each element names `` |
   | `command/set_effects.rs:3076` | `` declare `format: ess/27` `` | `` write `format: ess/27` on the source that declares the system `` |
   | `command/subject_fact.rs:24` | `` declare `format: ess/27`, where the one precedence rule reads both at its held-state step in declaration order, or read the held state in the predicate instead: replace `when_subject_state:` with `when_subject: {predicate: state == Active}`, or refuse every other state with `when_subject: {predicate: state != Active}` declared before the default `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule reads both at its held-state step in declaration order, or read the held state in the predicate instead: replace `when_subject_state:` with `when_subject: {predicate: state == Active}`, or refuse every other state with `when_subject: {predicate: state != Active}` declared before the default `` |
   | `command/subject_fact.rs:299` | `` declare `format: ess/25`, where an `unknown_instance:` branch may create the row beside `when_subject:` branches, or select the update branches with `when_subject_state:` `` | `` write `format: ess/25` on the source that declares the system, where an `unknown_instance:` branch may create the row beside `when_subject:` branches, or select the update branches with `when_subject_state:` `` |
   | `command/subject_fact.rs:516` | `` declare `format: ess/15`, or compare the stored field with a literal `` | `` write `format: ess/15` on the source that declares the system, or compare the stored field with a literal `` |
   | `command/subject_fact.rs:528` | `` declare `format: ess/18`, or guard the state with `when_subject_state:` `` | `` write `format: ess/18` on the source that declares the system, or guard the state with `when_subject_state:` `` |
   | `command/subject_state.rs:180` | `` declare `format: ess/27`, where the one precedence rule answers `wrong_state:` after the explicit subject-state guards `` | `` write `format: ess/27` on the source that declares the system, where the one precedence rule answers `wrong_state:` after the explicit subject-state guards `` |
   | `command/value_expression.rs:779` | `` declare `format: ess/22` to read through an Optional reference — the value is then absent where the reference is — or read a required field typed as the other entity's identity `` | `` write `format: ess/22` on the source that declares the system to read through an Optional reference — the value is then absent where the reference is — or read a required field typed as the other entity's identity `` |
   | `command/value_expression.rs:1003` | `` from specification format ess/27 this branch reads `{field}` present, since its `when_related:` predicate requires `defined({field})`: declare `format: ess/27` `` | `` from specification format ess/27 this branch reads `{field}` present, since its `when_related:` predicate requires `defined({field})`: write `format: ess/27` on the source that declares the system `` |
   | `command/value_expression.rs:1039` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/value_expression.rs:1051` | `` declare `format: ess/22`, or copy the value onto the row the first reference names and read it in one hop `` | `` write `format: ess/22` on the source that declares the system, or copy the value onto the row the first reference names and read it in one hop `` |
   | `command/value_expression.rs:1090` | `` declare `format: ess/28`, or read a declared field of the referenced row `` | `` write `format: ess/28` on the source that declares the system, or read a declared field of the referenced row `` |
   | `command/value_expression.rs:1189` | `` from specification format ess/27 this branch reads `{field}` present, since its `when_subject:` requires `defined({field})`: declare `format: ess/27` `` | `` from specification format ess/27 this branch reads `{field}` present, since its `when_subject:` requires `defined({field})`: write `format: ess/27` on the source that declares the system `` |
   | `command/value_expression.rs:1217` | `` declare `format: ess/23`, or fill the field from the input or a literal state `` | `` write `format: ess/23` on the source that declares the system, or fill the field from the input or a literal state `` |
   | `command/value_expression.rs:1263` | `` from specification format ess/27 a refusal guarded by a related row the subject's stored reference names reads that subject; declare `format: ess/27` `` | `` from specification format ess/27 a refusal guarded by a related row the subject's stored reference names reads that subject; write `format: ess/27` on the source that declares the system `` |
   | `command/value_expression.rs:1481` | `` write `format: ess/16` on the source that declares the system, or `else: {generated: true}` `` | `` write `format: ess/16` on the source that declares the system, or `else: {generated: true}` `` |
   | `command/value_expression.rs:1752` | `` write `format: ess/16` on the source that declares the system `` | `` write `format: ess/16` on the source that declares the system `` |
   | `command/value_expression.rs:2026` | `` declare `format: ess/22` `` | `` write `format: ess/22` on the source that declares the system `` |

4. **The refusals with no hint.** Add `.with_hint(<format>.declaration_hint())` to each of these
   `ValidationCode::UnsupportedFormatVersion` refusals (the line is that of the code). The message
   stays byte for byte.

   | Site | Message today | Hint to add |
   |---|---|---|
   | `command.rs:5021` | `fixture input declarations require specification format ess/13` | `crate::system::FormatVersion::V13.declaration_hint()` |
   | `command.rs:5028` | `typed response declarations require specification format ess/4` | `…V4.declaration_hint()` |
   | `command.rs:5041` | `direct return outcomes require specification format ess/17` | `…V17.declaration_hint()` |
   | `command.rs:5075` | `explicit response/generated payload sources require specification format ess/4` (one line of about 230 characters that rustfmt leaves alone; write the new push laid out like the one at line 5039) | `…V4.declaration_hint()` |
   | `command/one_time_response.rs:30` | `one_time_response requires specification format ess/21` | `…V21.declaration_hint()` |
   | `command/value_expression.rs:406` | `a {} source requires specification format {}`, the second `{}` being `"ess/16"` or `"ess/14"` | see below |

   `command/value_expression.rs:401-419`: bind the format first,
   `let needs = if matches!(source, PayloadSource::RelatedField { .. } | PayloadSource::CallerAttribute { .. }) { FormatVersion::V16 } else { FormatVersion::V14 };`,
   write the message as `format!("a {} source requires specification format {needs}", kind(source))`
   (the same bytes: `FormatVersion` displays as `ess/16`), and add
   `.with_hint(needs.declaration_hint())`.

5. **Re-pin the three tests that compare an old hint exactly.** Put the comment
   `// Re-pinned by story:every-format-version-hint-names-the-source-that-declares-the-system.`
   on the line above each changed assertion or constant.

   - `crates/specify/ess-domain/tests/published_value_operand.rs:355`:
     `Some("declare `format: ess/27`")` becomes
     `Some("write `format: ess/27` on the source that declares the system")`.
   - `crates/specify/ess-domain/tests/adversary_m2_pass1.rs:44-47`: the expected line becomes
     ``[unsupported_format_version] command.demo.desk.SignIn.outcomes.bad-credential.before_row: an external refusal answered before the row — `before_row: true` beside `external:` — requires specification format ess/25 (hint: write `format: ess/25` on the source that declares the system, or drop `before_row:`)``
     (keep the `\` line continuations; only the hint part changes).
   - `crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass1.rs:218-222`,
     `R2_REFUSAL`: the hint part becomes
     ``(hint: write `format: ess/27` on the source that declares the system, where the one precedence rule orders the two, guard the command on the related row alone, or split the other guard into a command of its own)``.

   No other test compares the text of a hint this unit changes. Checked with
   `rg -n 'declare .format' crates --glob '**/tests/**'` (the three above plus
   `view_paging/absent_answer.rs`, which belongs to the unit that converts `view/absent.rs`) and
   with `rg -n 'hint, None|hint\.is_none\(\)'` over the `ess-domain` tests: none of those asserts
   on a refusal of this unit.

## Tests first

Write these three before any conversion. Each test is a text read or a few small validations and
runs in under a second, except the ignored corpus test, which validates 197 fixtures in a few
seconds.

1. **The function**, `crates/specify/ess-domain/src/system.rs`, in `mod tests` (line 1718), after
   `a_format_version_round_trips_and_nothing_else_is_read_as_one` (line 1774):

   ```rust
   #[test]
   fn a_format_hint_names_the_source_that_declares_the_system() {
       assert_eq!(
           FormatVersion::V18.declaration_hint(),
           "write `format: ess/18` on the source that declares the system"
       );
       assert_eq!(FormatVersion::V18.declaration_hint(), crate::declaration_hint!(18));
       assert_eq!(FormatVersion::V31.declaration_hint(), crate::declaration_hint!(31));
   }
   ```

   Red before step 1 (does not compile).

2. **The source scan**, new file `crates/specify/ess-domain/tests/format_hint_source_scan.rs`,
   complete content:

   ```rust
   //! Every hint that asks for a specification format names where the `format:` key goes: on the
   //! source that declares the system
   //! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).
   //!
   //! `FormatVersion::declaration_hint` and the `declaration_hint!` macro in `src/system.rs` are
   //! the one place that spells the instruction. This reads the production source of `ess-domain`
   //! and `ess-conformance` and fails, naming the file and the line, at any other string literal
   //! that spells `` `format: ess/`` or `` `format: {``.

   use std::path::{Path, PathBuf};

   /// Files a later unit of the same change converts. It only shrinks, and is empty when the change
   /// is complete.
   const PENDING: &[&str] = &[
       "crates/specify/ess-domain/src/actor.rs",
       "crates/specify/ess-domain/src/binding/condition.rs",
       "crates/specify/ess-domain/src/binding/context.rs",
       "crates/specify/ess-domain/src/binding/refusal.rs",
       "crates/specify/ess-domain/src/binding/retry.rs",
       "crates/specify/ess-domain/src/entity.rs",
       "crates/specify/ess-domain/src/expression.rs",
       "crates/specify/ess-domain/src/primitive_admission.rs",
       "crates/specify/ess-domain/src/selection.rs",
       "crates/specify/ess-domain/src/shorthand.rs",
       "crates/specify/ess-domain/src/system_issued.rs",
       "crates/specify/ess-domain/src/types.rs",
       "crates/specify/ess-domain/src/undeclared_fields.rs",
       "crates/specify/ess-domain/src/view.rs",
       "crates/specify/ess-domain/src/view/absent.rs",
       "crates/specify/ess-domain/src/view/from.rs",
       "crates/specify/ess-domain/src/view/navigation.rs",
       "crates/specify/ess-domain/src/view/paging.rs",
       "crates/verify/ess-conformance/src/authored.rs",
   ];

   /// The two literals that are the instruction itself, each with the file that owns it.
   const EXEMPT: &[(&str, &str)] = &[
       (
           "crates/specify/ess-domain/src/system.rs",
           "\"write `format: {self}` on the source that declares the system\"",
       ),
       (
           "crates/specify/ess-domain/src/system.rs",
           "concat!(\"write `format: ess/\", $major,",
       ),
   ];

   fn repository() -> PathBuf {
       Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
   }

   fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
       let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
           .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
           .map(|entry| entry.expect("a directory entry").path())
           .collect();
       entries.sort();
       for path in entries {
           if path.is_dir() {
               rust_files(&path, found);
           } else if path.extension().is_some_and(|extension| extension == "rs") {
               found.push(path);
           }
       }
   }

   /// Whether the first line after an attribute that is neither blank nor another attribute opens a
   /// module.
   fn opens_module(rest: &[&str]) -> bool {
       rest.iter()
           .map(|line| line.trim())
           .find(|line| !line.is_empty() && !line.starts_with("#["))
           .is_some_and(|line| {
               line.starts_with("mod ")
                   || line.starts_with("pub mod ")
                   || line.starts_with("pub(crate) mod ")
           })
   }

   /// The production lines of one file, numbered by the line each starts on. A line ending in `\`
   /// is joined to the next with that line's leading whitespace dropped, as Rust reads a continued
   /// string literal. Reading stops at the first `#[cfg(test)]` that opens a module: every test
   /// module of these crates sits at the end of its file.
   fn production_lines(text: &str) -> Vec<(usize, String)> {
       let lines: Vec<&str> = text.lines().collect();
       let mut joined = Vec::new();
       let mut open: Option<(usize, String)> = None;
       for (index, line) in lines.iter().enumerate() {
           if open.is_none() && line.trim() == "#[cfg(test)]" && opens_module(&lines[index + 1..]) {
               break;
           }
           let piece = if open.is_some() { line.trim_start() } else { line };
           let (number, mut current) = open.take().unwrap_or((index + 1, String::new()));
           current.push_str(piece);
           match current.strip_suffix('\\') {
               Some(continued) => open = Some((number, continued.to_owned())),
               None => joined.push((number, current)),
           }
       }
       joined.extend(open);
       joined
   }

   #[test]
   fn no_literal_outside_the_function_spells_a_format_to_declare() {
       let root = repository();
       let mut files = Vec::new();
       rust_files(&root.join("crates/specify/ess-domain/src"), &mut files);
       rust_files(&root.join("crates/verify/ess-conformance/src"), &mut files);
       let mut found = Vec::new();
       for path in &files {
           let relative = path
               .strip_prefix(&root)
               .expect("under the repository")
               .to_string_lossy()
               .replace('\\', "/");
           if PENDING.contains(&relative.as_str()) {
               continue;
           }
           let text = std::fs::read_to_string(path)
               .unwrap_or_else(|error| panic!("{relative}: {error}"));
           for (number, line) in production_lines(&text) {
               if line.trim_start().starts_with("//") {
                   continue;
               }
               if !(line.contains("`format: ess/") || line.contains("`format: {")) {
                   continue;
               }
               if EXEMPT
                   .iter()
                   .any(|(file, needle)| relative == *file && line.contains(needle))
               {
                   continue;
               }
               found.push(format!("{relative}:{number}: {}", line.trim()));
           }
       }
       assert!(
           found.is_empty(),
           "build these hints with `FormatVersion::declaration_hint`, or `declaration_hint!` where \
            a `&'static str` is needed:\n{}",
           found.join("\n")
       );
   }

   #[test]
   fn a_continued_literal_is_read_whole_and_a_test_module_is_not_read() {
       let text = r#"    .with_hint("declare \
                   `format: ess/18`")
   #[cfg(test)]
   mod tests {
       const T: &str = "`format: ess/1`";
   }
   "#;
       let lines = production_lines(text);
       assert_eq!(lines.len(), 1, "{lines:?}");
       assert_eq!(lines[0].0, 1);
       assert!(lines[0].1.contains("declare `format: ess/18`"), "{lines:?}");
   }

   #[test]
   fn every_pending_file_exists() {
       let root = repository();
       for file in PENDING {
           assert!(root.join(file).is_file(), "{file} is not a file: fix PENDING");
       }
   }

   /// Every single-file fixture of the conformance suite, read at `ess/1`, where every format
   /// gate it uses fires: each `unsupported_format_version` refusal carries a hint naming the
   /// source that declares the system, and none spells the instruction in its message. On
   /// 0.97.1 this corpus draws 802 such refusals from 197 files.
   #[test]
   #[ignore = "live once PENDING is empty: the coordinator removes this line at integration"]
   fn every_format_refusal_of_the_fixture_corpus_carries_the_hint() {
       use ess_domain::spec::{RawSpecFile, Specification};
       use ess_domain::system::Source;
       use ess_primitives::error::ValidationCode;

       let directory = repository().join("crates/verify/ess-conformance/tests/fixtures");
       let mut names: Vec<PathBuf> = std::fs::read_dir(&directory)
           .expect("the fixture directory")
           .map(|entry| entry.expect("a directory entry").path())
           .filter(|path| path.extension().is_some_and(|extension| extension == "yaml"))
           .collect();
       names.sort();
       let mut seen = 0;
       let mut missing = Vec::new();
       for path in &names {
           let text = std::fs::read_to_string(path).expect("a fixture");
           let Some(start) = text.find("\nformat: ess/").map(|at| at + 1).or_else(|| {
               text.starts_with("format: ess/").then_some(0)
           }) else {
               continue;
           };
           if !text.lines().any(|line| line.starts_with("system:")) {
               continue;
           }
           let end = text[start..].find('\n').map_or(text.len(), |at| start + at);
           let lowered = format!("{}format: ess/1{}", &text[..start], &text[end..]);
           let Ok(raw) = RawSpecFile::parse(&lowered) else {
               continue;
           };
           let name = path.file_name().expect("a name").to_string_lossy().into_owned();
           let Err(errors) = Specification::assemble([(Source::new(name.clone()), raw)]) else {
               continue;
           };
           for error in errors.as_slice() {
               if error.code != ValidationCode::UnsupportedFormatVersion {
                   continue;
               }
               seen += 1;
               let named = error
                   .hint
                   .as_deref()
                   .is_some_and(|hint| hint.contains("on the source that declares the system"));
               if !named || error.message.contains("`format:") {
                   missing.push(format!("{name}: {} — {:?}", error.message, error.hint));
               }
           }
       }
       assert!(seen >= 500, "the corpus draws {seen} format refusals; expected about 800");
       assert!(missing.is_empty(), "{}", missing.join("\n"));
   }
   ```

   `no_literal_outside_the_function_spells_a_format_to_declare` is red before step 3: it lists
   the 71 literals of the table. `a_continued_literal_is_read_whole_and_a_test_module_is_not_read`
   and `every_pending_file_exists` are green from the start; they hold the scanner itself.
   `every_format_refusal_of_the_fixture_corpus_carries_the_hint` is the check that a refusal
   emitted without the hint fails a test. It stays `#[ignore]` in this unit because the refusals
   of the two follow-up units' files fail it until they merge; run it with `-- --ignored` once
   after step 4 and read that every line it lists comes from a refusal of the follow-up units
   (none of the six messages of step 4, none naming an `each:` entry or a payload source).

3. **The emitted hints**, new file
   `crates/specify/ess-domain/tests/format_hint_emitted_commands.rs`:

   ```rust
   //! The format hints of command refusals, as emitted, name the source that declares the system
   //! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

   use ess_domain::spec::{RawSpecFile, Specification};
   use ess_domain::system::Source;
   use ess_primitives::error::{ValidationCode, ValidationError};

   const CLAUSE: &str = "on the source that declares the system";

   const EACH_MOVES: &str =
       include_str!("../../../verify/ess-conformance/tests/fixtures/each-moves.yaml");

   /// An order shop whose `PlaceOrder` fills a payload field with `{generated: true}`, an `ess/4`
   /// source, and whose `CancelOrder` guards on a list of held states, an `ess/18` shape.
   const SHOP: &str = "format: ess/18
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
   ";
   ```

   (The `SHOP` text must start each YAML line in column 0 inside the Rust string; the indentation
   above is only this document's. It validates at `ess/18`: checked with `ess specify validate`
   on 0.96.0, `demo v1 — 1 file(s), valid`.)

   Helpers: `fn refusals(text: &str) -> Vec<ValidationError>` parses with
   `RawSpecFile::parse(text).expect("parses")`, assembles with
   `Specification::assemble([(Source::new("model.yaml"), raw)])`, `.expect_err("refused")`, and
   returns `errors.as_slice().to_vec()`. `fn names_a_format(hint: &str) -> bool` is true when some
   `ess/` in `hint` is followed by an ASCII digit.

   Tests:

   - `each_moves_below_ess_27_names_the_source_in_every_hint`: `refusals` of `EACH_MOVES` with
     `format: ess/27` replaced by `format: ess/26` (`replacen(…, 1)`). Assert that at least one
     error has a hint for which `names_a_format` holds, and that every such hint contains
     `CLAUSE`. Then the error at location
     `command.perrow.batch.Apply.outcomes.applied.affects[0].each.where` has code
     `UnsupportedFormatVersion` and hint
     ``write `format: ess/27` on the source that declares the system``. The error at
     `command.perrow.batch.Apply.outcomes.applied.affects[0].each` has code
     `ConflictingDeclaration` and hint
     ``keep `each:` with `instance:` and `sets:`, or write the other effect as an entry of its own; write `format: ess/27` on the source that declares the system, from which the entry may move or delete the row each element names``.
     Red before step 3: the hints read ``declare `format: ess/27` `` and
     ``…; from `format: ess/27` the entry may move…``.
   - `a_generated_payload_source_below_ess_4_names_the_source`: `refusals` of `SHOP` with
     `format: ess/18` replaced by `format: ess/3`. The error at
     `command.demo.ship.PlaceOrder.outcomes.placed.payload.demo.ship.OrderPlaced.order_id` has
     code `UnsupportedFormatVersion`, message
     `explicit response/generated payload sources require specification format ess/4`, and hint
     ``write `format: ess/4` on the source that declares the system``. Red before step 4: the
     hint is `None`. (At `ess/3` the model also draws two `ess/18` refusals from
     `primitive_admission.rs`. This test does not look at them; the follow-up unit does.)

## Commands

From the worktree root. Prefix every `cargo` command with the repository's compile cache,
`RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`
(`AGENTS.md`, "Local compile cache"); `cargo` goes through the `slot` shim on its own.

1. `cargo test -p ess-domain --locked --lib system::tests::a_format_hint_names_the_source_that_declares_the_system`:
   fails to compile before step 1, then 1 passed.
2. `cargo test -p ess-domain --locked --test format_hint_source_scan`: before step 3, 1 failed
   (71 lines listed), 2 passed, 1 ignored. After: 3 passed, 1 ignored. Then once
   `cargo test -p ess-domain --locked --test format_hint_source_scan -- --ignored`: it fails,
   listing only refusals of the follow-up units (see Tests first 2); this one red run is
   expected and is not a gate.
3. `cargo test -p ess-domain --locked --test format_hint_emitted_commands`: 2 failed before steps 3
   and 4, then 2 passed.
4. `cargo test -p ess-domain --locked --test published_value_operand`, then
   `--test adversary_m2_pass1`, `--test adversary_validation_selection_plan_pass1`,
   `--test set_effects`, `--test held_state_order`, `--test row_sets` and
   `--test subject_fact_create_or_update`, one command each: all pass.
5. `cargo test -p ess-domain --locked`: all pass. This is the package gate; it also proves no
   other `ess-domain` test pinned a changed hint.
6. `cargo test -p ess-compiler --locked --test typed_diagnostics`: passes unchanged (no
   `ValidationError::new`/`::at` site was added or removed).
7. `cargo test -p ess-compiler --locked --test string_alphabet_and_length`: passes (a byte-pin
   target of `AGENTS.md`; it reads codes only).
8. `cargo clippy -p ess-domain --all-targets --locked -- -D warnings`: no warnings.
9. `cargo fmt -p ess-domain`, then `slot task fmt-check`: clean. Never `cargo fmt --all`.

No `ess-conformance`, `ess-synth` or `ess-openapi` pin target can move: no hint reaches a suite, a
generated file or an IR byte. `task test-xtask` is not needed: no `ValidationCode`, diagnostic
class, format version or `CHANGELOG.md` change is in this unit.

## Files this unit owns

- `crates/specify/ess-domain/src/system.rs`
- `crates/specify/ess-domain/src/command.rs`
- `crates/specify/ess-domain/src/command/` (these files of it change: `absent_input.rs`,
  `append_list.rs`, `caller_value.rs`, `identity_write.rs`, `input_path.rs`, `minted_value.rs`,
  `one_time_response.rs`, `ordered_head.rs`, `outcome_shapes.rs`, `published_operand.rs`,
  `reference_integrity.rs`, `related_guard.rs`, `related_list.rs`, `related_rows.rs`,
  `row_set.rs`, `set_effects.rs`, `subject_fact.rs`, `subject_state.rs`, `value_expression.rs`)
- `crates/specify/ess-domain/tests/format_hint_source_scan.rs` (new)
- `crates/specify/ess-domain/tests/format_hint_emitted_commands.rs` (new)
- `crates/specify/ess-domain/tests/published_value_operand.rs`
- `crates/specify/ess-domain/tests/adversary_m2_pass1.rs`
- `crates/specify/ess-domain/tests/adversary_validation_selection_plan_pass1.rs`

Must not touch:

- every other file under `crates/specify/ess-domain/src/`, in particular `expression.rs`,
  `primitive_admission.rs`, `spec.rs`, `view*.rs`, `binding/**`, `types.rs`: the two follow-up
  units own them;
- `crates/verify/ess-conformance/**`, `crates/edge/ess-cli/**`;
- `crates/specify/ess-domain/tests/view_paging/absent_answer.rs` and
  `crates/specify/ess-domain/tests/coalesce_selector.rs` (follow-up units);
- `crates/generate/ess-synth/**`, `crates/edge/ess-xtask/**`, `.config/nextest.toml`,
  `Taskfile.yml`, `.github/workflows/*`, `rust-toolchain.toml` (other units of this wave);
- `CHANGELOG.md`, `docs/`, `website/`, `AGENTS.md` (coordinator).

## For the coordinator at integration

- After `type-view-binding-format-hints-name-the-source-that-declares-the-system` and
  `expression-format-hints-and-the-header-refusal-name-the-system-source` are merged, empty
  `PENDING` in `crates/specify/ess-domain/tests/format_hint_source_scan.rs`
  (`const PENDING: &[&str] = &[];`), delete the line
  `#[ignore = "live once PENDING is empty: the coordinator removes this line at integration"]`
  above `every_format_refusal_of_the_fixture_corpus_carries_the_hint`, and run
  `cargo test -p ess-domain --locked --test format_hint_source_scan`: 4 passed. Doing it earlier
  fails both tests on the files not yet converted. A line the corpus test lists names a refusal
  one of the three units missed.
- `CHANGELOG.md` and `docs/design/` lines for the whole change are given in the unit
  `expression-format-hints-and-the-header-refusal-name-the-system-source`; this unit adds none.

## Out of scope

- Any file outside the list above, even where a format hint sits in it: the follow-up units
  convert them.
- Hints that name a version in prose only, emitted at every format as a note on an alternative
  (`command.rs:3387` "(ess/19)", `command/absent_input.rs:178` "(ess/16)",
  `command/ordered_head.rs:349` "(format ess/27)", `command/related_guard.rs:1696` "(ess/22)",
  `command/set_effects.rs:3311` "from ess/35", `command/value_expression.rs:438`
  "(format ess/14)"): they stay as they are. They are not refusals of a format below the one they
  name, and the clause would be wrong for a writer already at that format.
- The rest of the ESS-COMMAND-004 hint at `command/related_guard.rs:782`: owned by
  `story:the-ess-27-hint-states-the-order-of-an-unknown-identity-and-a-missing-related-row`.
- No new `ValidationCode`, no message change, no new refusal, no format version.

## Acceptance

- `FormatVersion::V18.declaration_hint()` is
  ``write `format: ess/18` on the source that declares the system`` and equals
  `declaration_hint!(18)` (Tests first 1, command 1).
- No literal under `command.rs` or `command/**` spells `` `format: ess/`` or `` `format: {``
  (Tests first 2, command 2); restoring `"declare `format: ess/18`"` anywhere in those files turns
  `no_literal_outside_the_function_spells_a_format_to_declare` red naming that file and line.
- The emitted hints of the `each:` refusals and of the generated payload source below `ess/4`
  name the source that declares the system (Tests first 3, command 3).
- The six refusals of step 4 carry a hint (command 3 for `command.rs:5075`; the `--ignored` run
  of command 2 lists none of them).
- The `ess-domain` package gate, `typed_diagnostics`, clippy and fmt pass (commands 4-9).

## Depends on

Nothing. It provides, for the two units that branch from its first commit:

- `ess_domain::system::FormatVersion::declaration_hint(self) -> String`, returning
  ``write `format: ess/N` on the source that declares the system``;
- `ess_domain::declaration_hint!($major:literal)`, a `&'static str` of the same text
  (`crate::declaration_hint!(N)` inside `ess-domain`);
- `crates/specify/ess-domain/tests/format_hint_source_scan.rs` with the `PENDING` list above.
