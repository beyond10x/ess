---
format: aep.planning-md/3
id: story:expression-format-hints-and-the-header-refusal-name-the-system-source
kind: story
status: implemented
title: Every expression format hint, the authored-value repair and the header refusal name the source that declares the system
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:every-format-version-hint-names-the-source-that-declares-the-system
- depends_on: story:command-format-hints-name-the-source-that-declares-the-system
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_format_hint.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/spec.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/coalesce_selector.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/format_hint_emitted_expressions.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/author_generated_value.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:45:07Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-21T23:45:08Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-23T01:57:01Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

- Every format hint of an expression refusal in `ess-domain/src/expression.rs`, and the `ESS-AUTHOR-042` repair in `ess-conformance`, is built with `FormatVersion::declaration_hint()` or `declaration_hint!`.
- The 14 expression format refusals with no hint get one; the 3 that end their message with the instruction carry it as their hint instead.
- The `missing_declaration` refusal of a source that sets `format:`, `version:` or `domains:` without `system:` names the file that declares the system, where one does.
- No format version changes, no `ValidationCode` changes; the messages of 3 current-time and calendar refusals lose their trailing instruction.

## What to change

Line numbers are at `origin/main` (`f3e4477974`, release 0.97.1). Branch from the first commit of
`command-format-hints-name-the-source-that-declares-the-system`
(`feat(domain): one function builds the format hint`), which adds to
`crates/specify/ess-domain/src/system.rs`:

- `FormatVersion::declaration_hint(self) -> String`, returning
  ``write `format: ess/N` on the source that declares the system`` (`FormatVersion` is `Copy`
  and displays as `ess/N`);
- the exported macro `declaration_hint!(N)` (`ess_domain::declaration_hint!(30)` from another
  crate), the same text as a `&'static str`.

`expression.rs` names the type as `crate::system::FormatVersion` (lines 648-668); write it that
way, or add `use crate::system::FormatVersion;` once at the top.

1. **A hint setter on `ExpressionError`.** `crates/specify/ess-domain/src/expression.rs`, in
   `impl ExpressionError` after `detail_mut` (lines 487-489):

   ```rust
       /// This refusal with `hint`, the repair a newer format admits.
       pub(crate) fn with_hint(mut self, hint: String) -> Self {
           self.detail_mut().hint = Some(hint);
           self
       }
   ```

   `validation_error()` (line 467) already copies the hint onto the `ValidationError`.

2. **The literals that spell the instruction.**

   | Site (`expression.rs` unless named) | Today | After |
   |---|---|---|
   | `:2280` (`refused.detail_mut().hint = Some(format!(…))`) | ``declare `format: ess/30`, where a selector's `where:` compares `{expr}` with a literal, an absent `{parent}` reading as the default`` | ``{raise}, where a selector's `where:` compares `{expr}` with a literal, an absent `{parent}` reading as the default``, `raise = crate::system::FormatVersion::V30.declaration_hint()`: reads ``write `format: ess/30` on the source that declares the system, where …`` |
   | `:2657` (message of `stored_current_time_later`) | ``… predicate requires specification format {required} for; write `format: {required}` on the source that declares the system`` | message ``… predicate requires specification format {required} for`` (ends there), and the hint `required.declaration_hint()`; see step 3 |
   | `:2716` (the `UnsupportedFormatVersion` arm of `current_time_literal`) | ``write `format: ess/16` on the source that declares the system`` | `crate::system::FormatVersion::V16.declaration_hint()`, carried as the hint, not in the message; see step 4 |
   | `:2737` (the view-filter arm, `TypeMismatch`) | ``…, so the caller sends the window it means; or write `format: ess/30`, from which a view filter reads `now` at the instant of the read`` | ``…, so the caller sends the window it means; or {raise}, from which a view filter reads `now` at the instant of the read``, `raise = crate::system::FormatVersion::V30.declaration_hint()` added to that `format!` (its `{{name: from, …}}` braces stay doubled). Reads ``…; or write `format: ess/30` on the source that declares the system, from which a view filter reads `now` at the instant of the read``. This arm keeps its text in the message (step 4) |
   | `:3891` (message of the calendar-window refusal in `window`) | ``…, which requires specification format ess/22; write `format: ess/22` on the source that declares the system`` | message ``` `{predicate}` is a calendar window, which requires specification format ess/22``` (ends there); `.with_hint(crate::system::FormatVersion::V22.declaration_hint())` on the `error(…)` |
   | `crates/verify/ess-conformance/src/authored.rs:2397` (repair of `GeneratedValuePinned`, `ESS-AUTHOR-042`, a `&'static str` in `diagnostic_catalogue!`) | ``…, field: …}}`; from `format: ess/30` an exact value there is the scenario's own claim, which the suite checks as written`` | ``…, field: …}}`; write `format: ess/30` on the source that declares the system, from which an exact value there is the scenario's own claim, which the suite checks as written``, written as `concat!("the implementation mints … `{$observed: {event: …, field: …}}`; ", ess_domain::declaration_hint!(30), ", from which an exact value there is the scenario's own claim, which the suite checks as written")` (the head copied byte for byte from today's literal) |

3. **`current_time_later` returns the format.** The trait method of `pub trait TypeEnvironment`
   (line 144) `fn current_time_later(&self) -> Option<&'static str>` (default at line 245, the
   one implementation at line 705) becomes
   `fn current_time_later(&self) -> Option<crate::system::FormatVersion>`: the default stays
   `None`, the implementation returns `.then_some(crate::system::FormatVersion::V22)`. No other
   crate implements or calls it (`rg current_time_later crates`). In `stored_current_time_later`
   (line 2642), `required` is then a `FormatVersion`; the message keeps
   `requires specification format {required} for` (same bytes, `ess/22`) and drops the
   trailing `; write …`, and the pushed error is `error(…).with_hint(required.declaration_hint())`.
   This changes a public trait signature of `ess-domain`: run rustdoc (Commands).

4. **The current-time arm keeps message and hint apart.** In `current_time_literal`
   (line 2675), the match yields `(code, message, hint)` and the push at line 2788 writes
   `format!("{message}; {hint}")` for every arm. Keep that for every arm but the
   `UnsupportedFormatVersion` one:

   ```rust
           let refused = if code == ValidationCode::UnsupportedFormatVersion {
               error(self.owner, code, Some(path), None, message).with_hint(hint)
           } else {
               error(self.owner, code, Some(path), None, format!("{message}; {hint}"))
           };
           self.checked.errors.push(refused);
   ```

   The arm's message (lines 2712-2715) stays ``… which requires specification format ess/16``;
   its hint is `crate::system::FormatVersion::V16.declaration_hint()` (row `:2716`).

5. **The expression format refusals with no hint.** Add the hint on the `error(…)` each pushes or
   returns, `error(…).with_hint(crate::system::FormatVersion::VNN.declaration_hint())`. Messages
   stay byte for byte. Line = that of `ValidationCode::UnsupportedFormatVersion`.

   | Line | Message today | `VNN` |
   |---|---|---|
   | 1228 | `` `{through}`: the length of a String requires specification format ess/11`` | `V11` |
   | 1292 | `` `{through}`: the UTF-8 byte length of a String requires specification format ess/22`` | `V22` |
   | 2201 | `` `{derived}`, the UTF-8 byte length of `{parent}`, requires specification format ess/22`` | `V22` |
   | 2252 | `` `{expr}` reads `{parent}` through a function, which requires specification format ess/27; below it, quote the text to compare with it`` | `V27` |
   | 2653 | the stored current time (step 3) | `required` |
   | 2711 | the current time below `ess/16` (step 4) | `V16` |
   | 2826 | `` `{predicate}`: `distinct: {{in, as, by}}` requires specification format ess/22`` | `V22` |
   | 3029 | `` `{predicate}` is tagged to compare instants, written `as: timestamp`, which requires specification format ess/22`` | `V22` |
   | 3178 | `` `{text} {op} …` calls a function on the left of a comparison, which requires specification format ess/29; below it, write the structured form …`` (in `left_text`) | `V29`, see below |
   | 3214 | `` `{predicate}` compares with one constant offset of `{}`, written `{offset}`, which requires specification format ess/22`` | `V22` |
   | 3350 | `` `{predicate}` compares with the fact `{name}` on its right, written `{{fact: {name}}}`, which requires specification format ess/22`` | `V22` |
   | 3523 | `` `{predicate}` compares with `{written}`, which requires specification format ess/22`` | `V22` |
   | 3817 | `` `{predicate}`: `defined()` over an Optional aggregate (`{}`) requires specification format ess/16`` | `V16` |
   | 3887 | the calendar window (row `:3891` of step 2) | `V22` |

   `left_text` (line 3172): its match yields `(code, message)` for three codes and pushes at
   lines 3201-3203. Make it yield `(code, message, hint)` with
   `Some(crate::system::FormatVersion::V29.declaration_hint())` in the
   `UnsupportedFormatVersion` arm and `None` in the other two, and push
   `match hint { Some(hint) => refused.with_hint(hint), None => refused }` where
   `refused = error(self.owner, code, None, None, message)`.

6. **The header refusal names the system's source.** `crates/specify/ess-domain/src/spec.rs`:

   - `struct Collected` (line 1145, `#[derive(Default)]`): add the field
     ```rust
         /// The one source that names the system, when exactly one does: the file a header key
         /// written elsewhere belongs on.
         system_source: Option<Source>,
     ```
   - `assemble_traced` (line 345): after `headers` is computed (lines 364-368) and before the
     loop, set it from `files`:
     ```rust
             let mut systems = files.iter().filter(|(_, file)| file.system.is_some());
             collected.system_source = match (systems.next(), systems.next()) {
                 (Some((source, _)), None) => Some(source.clone()),
                 _ => None,
             };
     ```
     (`Source` is `Clone`, `system.rs:656`.)
   - `absorb_header` (line 1246): the hint at lines 1265-1268 becomes, when
     `self.system_source` is `Some(system)`,
     ``format!("the source that carries the format, the version and the domain list is the one that names the system: move these keys to `{system}`")``,
     and otherwise today's text byte for byte. `Source` displays as its label
     (`domains/jobs.yaml`, as `format!("{source}.system")` at line 1261 shows).

   The other `Collected::default()` (line 2351) is in a test module and leaves the field `None`:
   its refusals keep today's text. The sibling refusal in `system.rs:1650-1661`
   (`RawSystemSpec::split`, one document, no other file to name) is not changed.

7. **Re-pin the tests that compare a changed text.** Put
   `// Re-pinned by story:every-format-version-hint-names-the-source-that-declares-the-system.`
   above each.

   - `crates/specify/ess-domain/tests/coalesce_selector.rs:182`:
     `assert_eq!(error.hint, None, "{error:?}");` becomes
     `assert_eq!(error.hint.as_deref(), Some("write `format: ess/29` on the source that declares the system"), "{error:?}");`
     and the doc comment at lines 166-167 ("…naming `ess/29` and the structured form, with no
     hint.") ends "…and the structured form, its hint naming the source that declares the
     system." The other `hint, None` assertions of that file (lines 217, 290) read
     `UnsupportedConstruct` refusals and stay.
   - `crates/verify/ess-conformance/tests/author_generated_value.rs:236-246` (outside the
     assigned files; see Files): in the expected text, ``{event: …, field: …}}`; from `format:
     ess/30` an exact value there is the scenario's own claim, which the suite checks as written``
     becomes ``{event: …, field: …}}`; write `format: ess/30` on the source that declares the
     system, from which an exact value there is the scenario's own claim, which the suite checks
     as written`` (keep the `\` continuations and the two-space indent after `\n`). Its line 832,
     `help.contains("`format: ess/30`")`, stays true.

   No other test pins these texts: no test compares the messages of steps 3, 4 or the
   calendar window in full (`rg 'ess/16; write|for; write|is a calendar window, which requires'`
   over every `tests/` finds nothing), and no test names the `missing_declaration` hint.

## Tests first

1. New file `crates/specify/ess-domain/tests/format_hint_emitted_expressions.rs`. Each test
   validates one or two small models in-process, well under a second.

   ```rust
   //! The format hints of expression refusals, and the header refusal, name the source that
   //! declares the system
   //! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

   use ess_domain::spec::{RawSpecFile, Specification};
   use ess_domain::system::Source;
   use ess_primitives::error::{ValidationCode, ValidationError};

   const STORED_NOW: &str =
       include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");
   const VIEW_NOW: &str =
       include_str!("../../../verify/ess-conformance/tests/fixtures/now-view-filter.yaml");

   /// A job scheduler guarding on the current time (`ess/16`) and a text length (`ess/11`).
   const JOBS: &str = "format: ess/16
   system: demo
   version: v1
   domain: demo.jobs
   entities:
     - name: demo.jobs.Job
       identity: {name: job_id, type: Uuid}
       fields: []
       lifecycle:
         initial: Scheduled
         states: [Scheduled]
         terminal: [Scheduled]
   events:
     - name: demo.jobs.JobScheduled
       fields: [{name: job_id, type: Uuid}]
   errors:
     - {name: demo.jobs.InPast, fields: []}
   commands:
     - name: demo.jobs.Schedule
       input:
         - {name: starts_at, type: Timestamp}
         - {name: label, type: String}
       outcomes:
         - name: start-in-past
           when: starts_at < now
           error: demo.jobs.InPast
         - name: label-too-long
           when: label.count > 40
           error: demo.jobs.InPast
         - name: scheduled
           creates: demo.jobs.Job
           instance: job_id
           emits: [demo.jobs.JobScheduled]
           payload: {demo.jobs.JobScheduled: {job_id: {generated: true}}}
   ";

   fn refusals(files: &[(&str, &str)]) -> Vec<ValidationError> {
       let parsed = files.iter().map(|(name, text)| {
           let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name}: {error}"));
           (Source::new(*name), raw)
       });
       Specification::assemble(parsed.collect::<Vec<_>>())
           .expect_err("refused")
           .as_slice()
           .to_vec()
   }

   fn at<'a>(errors: &'a [ValidationError], location: &str) -> &'a ValidationError {
       errors
           .iter()
           .find(|error| error.location == location)
           .unwrap_or_else(|| panic!("no refusal at {location}: {errors:#?}"))
   }
   ```

   (YAML lines start in column 0 inside the Rust string; the indentation above is this
   document's. `JOBS` is valid at `ess/16`, checked with the released 0.97.1.)

   Tests:

   - `the_current_time_and_a_text_length_below_their_formats_name_the_source`:
     `refusals(&[("model.yaml", &JOBS.replacen("format: ess/16", "format: ess/10", 1))])`.
     At `command.demo.jobs.Schedule.outcomes.start-in-past.when`: code
     `UnsupportedFormatVersion`, message exactly
     ``` `starts_at < now` orders the Timestamp `starts_at` against the current time, `now`, which requires specification format ess/16```,
     hint `Some("write `format: ess/16` on the source that declares the system")`. At
     `command.demo.jobs.Schedule.outcomes.label-too-long.when`: message
     ``` `label.count`: the length of a String requires specification format ess/11```, hint
     `Some("write `format: ess/11` on the source that declares the system")`. Red before: the
     first message ends ``; write `format: ess/16` on …`` with no hint, the second has no hint.
   - `the_stored_current_time_below_ess_22_names_the_source`: `STORED_NOW` with
     `format: ess/22` replaced by `format: ess/21` (first occurrence). At
     `command.demo.leases.RenewLease.outcomes.renewed.when_subject`: message exactly
     ``` `expires_at >= now - 1h` orders the stored Timestamp `expires_at` against the current time, `now - 1h`, which a `when_subject:` or `when_related:` predicate requires specification format ess/22 for```,
     hint `Some("write `format: ess/22` on the source that declares the system")`.
   - `a_view_filter_over_now_below_ess_30_names_the_source`: `VIEW_NOW` with `format: ess/30`
     replaced by `format: ess/29`. At `view.demo.ledger.RecentIncidents.filter`: code
     `TypeMismatch`, and the message ends with
     ``; or write `format: ess/30` on the source that declares the system, from which a view filter reads `now` at the instant of the read``.
   - `a_header_key_on_another_source_names_the_source_of_the_system`:
     `refusals(&[("system.yaml", SYSTEM), ("domains/jobs.yaml", DOMAIN)])`, where `SYSTEM` is
     `"format: ess/16\nsystem: demo\nversion: v1\ndomains: [demo.jobs]\n"` and `DOMAIN` is
     `format!("format: ess/16\n{}", &JOBS[JOBS.find("domain: demo.jobs").expect("a domain")..])`.
     At `domains/jobs.yaml.system`: code
     `MissingDeclaration`, hint
     ``Some("the source that carries the format, the version and the domain list is the one that names the system: move these keys to `system.yaml`")``.
   - `with_no_source_naming_the_system_the_header_hint_keeps_its_text`: only
     `("domains/jobs.yaml", DOMAIN)`. At `domains/jobs.yaml.system`: hint
     `Some("the source that carries the format, the version and the domain list is the one that names the system")`
     (beside it stands the refusal at `system`, `no source declares the system`, unchanged).
     Green before and after.

2. New file `crates/edge/ess-cli/tests/validate_format_hint.rs`, the command a writer runs.
   Copy the helpers of `crates/edge/ess-cli/tests/validate_names_the_file_read_from.rs`
   (lines 68-99: `root()`, a fresh directory under `env!("CARGO_TARGET_TMPDIR")` named with the
   process id and an `AtomicUsize`, and `validate(directory, json)` running
   `env!("CARGO_BIN_EXE_ess")` with `specify validate [--format json] --path <dir>`). Each test
   writes `system.yaml` and `domains/jobs.yaml` and runs the binary once or twice, in seconds.

   - `SYSTEM_AT_10` = `"format: ess/10\nsystem: demo\nversion: v1\ndomains: [demo.jobs]\n"`;
     `JOBS_DOMAIN` = the `JOBS` model above from its `domain: demo.jobs` line on (no header).
   - `the_text_report_names_the_source_in_every_format_help`: exit code 1; standard error (the
     text report is written there) contains
     ``  help: write `format: ess/16` on the source that declares the system`` and
     ``  help: write `format: ess/11` on the source that declares the system``, contains
     `--> domains/jobs.yaml:`, and does not contain `ess/16; write`. Red before: no `help:` line
     for either refusal.
   - `the_json_report_carries_the_hint`: with `--format json`, standard output parses as JSON;
     among `diagnostics` the one whose `message` starts with `` `label.count` `` has `code`
     `ESS-COMMAND-009`, `hint` ``write `format: ess/11` on the source that declares the system``
     and `span.source` `domains/jobs.yaml`.
   - `a_header_key_on_the_domain_file_is_sent_to_the_system_file`: `system.yaml` at `ess/16`,
     `domains/jobs.yaml` starting `format: ess/16\n` then `JOBS_DOMAIN`: exit 1; standard error
     contains `[missing_declaration] domains/jobs.yaml.system:` and
     ``move these keys to `system.yaml` ``.
   - `the_same_files_at_ess_16_are_valid`: `system.yaml` at `ess/16`, `domains/jobs.yaml` as
     `JOBS_DOMAIN`: exit 0 and standard output contains `demo v1 — 2 file(s), valid`.

   Checked against the released 0.97.1 binary with the same two files: at `ess/10` it refuses
   both guards, cites `--> domains/jobs.yaml:16:5`, prints no `help:` line for them, and at
   `ess/16` prints `demo v1 — 2 file(s), valid` with exit 0.

## Commands

From the worktree root. Prefix every `cargo` command with
`RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`
(`AGENTS.md`, "Local compile cache"); `cargo` goes through the `slot` shim on its own.

1. `cargo test -p ess-domain --locked --test format_hint_emitted_expressions`: 4 failed, 1
   passed before; 5 passed after.
2. `cargo test -p ess-domain --locked --test coalesce_selector`, then `--test calendar_window`,
   `--test expression`, `--test expression_a1`, `--test adversary_now_guard_sites`: pass.
3. `cargo test -p ess-domain --locked`: all pass (the package gate).
4. `cargo test -p ess-conformance --locked --test author_generated_value`: passes after the
   re-pin. Also the `AGENTS.md` byte-pin targets of `ess-conformance`, one `--test` at a time:
   `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
   `adversary_related_values_all_bytes`, `external_beside_held_guard` (no suite byte should move).
5. `cargo test -p ess-cli --locked --test validate_format_hint`: 3 failed (text, JSON, header)
   and 1 passed before; 4 passed after. Then `--test specify_formats` and
   `--test validate_names_the_file_read_from`: pass.
6. `cargo test -p ess-compiler --locked --test typed_diagnostics`, `--test diagnostic_catalogue`
   and `--test string_alphabet_and_length`: pass.
7. `RUSTDOCFLAGS="-D warnings" cargo doc -p ess-domain --no-deps --locked`: clean (the trait
   signature and the new doc comments).
8. `cargo clippy -p ess-domain --all-targets --locked -- -D warnings`, and the same for
   `-p ess-conformance` and `-p ess-cli`: no warnings.
9. `cargo fmt -p ess-domain -p ess-conformance -p ess-cli`, then `slot task fmt-check`: clean.
   Never `cargo fmt --all`.

`cargo xtask diagnostics --check` fails after step 2 until
`website/docs/reference/diagnostics.md` is regenerated; that file is the coordinator's (below).
Do not run `--test format_hint_source_scan` as a gate here: its `PENDING` list names
`expression.rs` and `authored.rs`, so it passes either way.

## Files this unit owns

- `crates/specify/ess-domain/src/expression.rs`
- `crates/specify/ess-domain/src/spec.rs`
- `crates/verify/ess-conformance/src/authored.rs`
- `crates/specify/ess-domain/tests/format_hint_emitted_expressions.rs` (new)
- `crates/specify/ess-domain/tests/coalesce_selector.rs`
- `crates/edge/ess-cli/tests/validate_format_hint.rs` (new)
- `crates/verify/ess-conformance/tests/author_generated_value.rs` (outside the assigned files:
  its full-text pin of the `ESS-AUTHOR-042` help has to move with `authored.rs`; no other unit
  of this wave names it)

Must not touch:

- `crates/specify/ess-domain/src/system.rs`, `src/command.rs`, `src/command/**`,
  `tests/format_hint_source_scan.rs`, `tests/format_hint_emitted_commands.rs` (first unit);
- `src/primitive_admission.rs`, `src/types.rs`, `src/view*.rs`, `src/binding/**`, every other
  file under `ess-domain/src`, `tests/view_paging/**` (second unit);
- `crates/generate/ess-synth/**`, `crates/edge/ess-xtask/**`, the conformance report writer in
  `ess-cli`, `.config/nextest.toml`, `Taskfile.yml`, `.github/workflows/*`,
  `rust-toolchain.toml` (other units of this wave);
- `website/`, `CHANGELOG.md`, `docs/`, `AGENTS.md` (coordinator).

## For the coordinator at integration

- `website/docs/reference/diagnostics.md:175`: run `cargo xtask diagnostics` after merging this
  unit (it renders the page from the catalogue), or replace in that row
  ``{event: …, field: …}}`; from `format: ess/30` an exact value there is the scenario's own claim``
  with
  ``{event: …, field: …}}`; write `format: ess/30` on the source that declares the system, from which an exact value there is the scenario's own claim``.
  `cargo xtask diagnostics --check` must pass.
- `CHANGELOG.md`, under `## [Unreleased]`, a `### Changed` heading and this bullet (for all three
  units):

  ```markdown
  - Every refusal that asks for a newer specification format says where the key goes: its hint
    reads ``write `format: ess/N` on the source that declares the system``, where 79 hints said
    ``declare `format: ess/N` `` and the file the diagnostic points at is often a domain file
    that may not carry `format:`. The 43 such refusals that had no hint now carry one, among
    them `subject-state outcome guards require specification format ess/3`; three current-time
    and calendar-window refusals carry the instruction as their hint instead of at the end of
    their message. A domain file that sets `format:`, `version:` or `domains:` without `system:`
    is told the file that names the system. No format version changes.
  ```
- `docs/design/` (engineering record; the old wording is quoted as the planned hint):
  - `docs/design/set-effect-identity-filters.md:163`: `(declare `format: ess/27`)` becomes
    `(write `format: ess/27` on the source that declares the system)`.
  - `docs/design/related-rows-quantifier.md:215`, `docs/design/ordered-related-head.md:219` and
    `docs/design/published-value-operand.md:148`: ``hint `declare \`format: ess/27\`` `` becomes
    ``hint `write \`format: ess/27\` on the source that declares the system` ``.
- After all three units are merged, the first unit's coordinator lines: empty `PENDING` and
  remove the `#[ignore]` of the corpus test in
  `crates/specify/ess-domain/tests/format_hint_source_scan.rs`.

## Out of scope

- Hints and messages that name a version in prose, emitted at every format as a note on an
  alternative, in this unit's files: they stay.
- The `missing_declaration` refusal of the single-document shape (`system.rs:1650-1661`): one
  document has no other file to name, and `system.rs` is the first unit's.
- Any change to the `ESS-AUTHOR-042` meaning or to which acts it refuses.
- No new `ValidationCode`, no new refusal, no format version.

## Acceptance

- Below `ess/16` a current-time guard is refused with a message that ends at
  `requires specification format ess/16` and the hint
  ``write `format: ess/16` on the source that declares the system``; a text length below `ess/11`
  and a stored current time below `ess/22` carry theirs (Tests first 1, command 1).
- `ess specify validate` on a two-file specification prints a `help:` line naming the source
  that declares the system for each format refusal, in text and JSON, and sends a header key
  written on a domain file to `system.yaml` (Tests first 2, command 5).
- The `ESS-AUTHOR-042` help names the source; `author_generated_value` holds it (command 4).
- The `ess-domain` package gate, the named `ess-conformance`, `ess-cli` and `ess-compiler`
  targets, rustdoc, clippy and fmt pass (commands 2-9).

## Depends on

`command-format-hints-name-the-source-that-declares-the-system`, its first commit only
(`FormatVersion::declaration_hint` and `declaration_hint!`). Branch from that commit.
