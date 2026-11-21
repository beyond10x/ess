---
format: aep.planning-md/3
id: story:report-out-writers-replace-their-file-through-one-helper
kind: story
status: active
title: A conformance report replaces its file in one step
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:a-conformance-report-replaces-its-file-in-one-step
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/replace_file.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/count_reports.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:42:31Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-21T23:42:33Z", actor: "human:timo", revision: 6}
---
## Outcome

- `ess verify conform run --report-out`, `ess verify conform report --results … --report-out` (without
  `--known-failing`) and `ess verify conform mutate --report-out` write the report to a new file in
  the destination's own directory, sync it, then rename it onto the destination. At every instant
  the destination holds either its earlier bytes or the complete new report.
- If writing fails, the earlier file keeps its bytes and the temporary file is removed. Success
  leaves no temporary file either. The report bytes are what they were before this change.
- No format version changes: the same `to_canonical_json` string is written, only the way it reaches the disk changes.

## What to change

Today (at `origin/main` f3e4477974, release 0.97.1) each of these four writes calls `fs::write`.
`fs::write` truncates the destination first and then writes into it. A run that is killed, or that
loses its disk, between the truncation and the last byte leaves a truncated file, or an empty one,
where a complete report was. A reader in that window sees a document that is not a report:

| verb | file:line | today |
|---|---|---|
| `run`, `--report-format 2` | `crates/edge/ess-cli/src/main.rs:5082-5083` (`render_conformance_report`) | `fs::write(path, summary.to_canonical_json()?).with_context(\|\| format!("writing {}", path.display()))?;` |
| `run`, `--report-format 1` | `main.rs:5099-5100` (same function) | `fs::write(path, report.standalone().to_canonical_json()).with_context(…)?;` |
| `report --results` | `main.rs:3980-3986` (`conform_report`) | `if let Err(error) = fs::write(report_out, text) { eprintln!("conform-report.unwritable: writing {}: {error}", …); return ExitCode::from(REFUSED); }` |
| `mutate` | `main.rs:4995-4997` (`finish_mutation_audit`) | `fs::write(out, &json).with_context(\|\| format!("writing {}", out.display()))?;` |

The repository has no helper you can reuse. Do not use these:

- `temporary_sibling` (`main.rs:4168`) and `write_imported` (`main.rs:4180`): fixed name, no
  `create_new`, no sync.
- `recovery::journal::publish_bytes` (`src/recovery/journal.rs:266`): it never replaces a file.
- `output_ownership::filesystem::write_new`/`rename` (`src/output_ownership/filesystem.rs:687`,
  `:732`): they run on descriptors inside an ownership root.

The sync pattern to copy is `File::sync_all` on the file, then on its directory, as
`recovery/journal.rs:218-232` (`sync_directory`) does.

1. **New file `crates/edge/ess-cli/src/replace_file.rs`**, private to the `ess` binary. Write it
   exactly like this; doc comments may be reworded:

   ```rust
   //! Replacing a file in one step. The bytes go to a new file beside the destination. That file
   //! is synchronised and then renamed onto the destination, so at every instant the destination
   //! holds either its earlier bytes or the complete new ones.

   use std::ffi::{OsStr, OsString};
   use std::fs::{self, File, OpenOptions};
   use std::io::{self, Write as _};
   use std::path::{Path, PathBuf};
   use std::sync::atomic::{AtomicU64, Ordering};

   /// Writes `bytes` to `path` in one step (see the module documentation).
   pub(crate) fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
       replace_with(path, |file| file.write_all(bytes))
   }

   /// [`replace`], with the bytes written into the temporary file by `write`. This is the seam
   /// the tests use to fail a write part-way.
   ///
   /// A destination that is a symbolic link is followed: the file it names is replaced in that
   /// file's own directory, and the link stays. A file that is replaced keeps its permissions.
   /// On any failure before the rename, the temporary file is removed and the destination is left
   /// as it was.
   pub(crate) fn replace_with(
       path: &Path,
       write: impl FnOnce(&mut File) -> io::Result<()>,
   ) -> io::Result<()> {
       let destination = destination(path)?;
       let name = destination.file_name().ok_or_else(|| {
           io::Error::new(io::ErrorKind::InvalidInput, "the destination names no file")
       })?;
       let parent = match destination.parent() {
           Some(parent) if !parent.as_os_str().is_empty() => parent,
           _ => Path::new("."),
       };
       let earlier = match fs::metadata(&destination) {
           Ok(metadata) if metadata.is_file() => Some(metadata.permissions()),
           _ => None,
       };
       let (temporary, mut file) = create_temporary(parent, name)?;
       let written = write(&mut file)
           .and_then(|()| match earlier {
               Some(permissions) => file.set_permissions(permissions),
               None => Ok(()),
           })
           .and_then(|()| file.sync_all());
       drop(file);
       if let Err(error) = written.and_then(|()| fs::rename(&temporary, &destination)) {
           let _ = fs::remove_file(&temporary);
           return Err(error);
       }
       sync_directory(parent)
   }

   /// `path`, or the file it names when it is a symbolic link.
   fn destination(path: &Path) -> io::Result<PathBuf> {
       match fs::symlink_metadata(path) {
           Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path),
           _ => Ok(path.to_path_buf()),
       }
   }

   /// A new, empty `.<name>.<pid>-<n>.tmp` in `parent`. It is created with `create_new`, so it
   /// never replaces another file, and a name that is taken is retried with the next `n`.
   fn create_temporary(parent: &Path, name: &OsStr) -> io::Result<(PathBuf, File)> {
       static NEXT: AtomicU64 = AtomicU64::new(0);
       let mut taken = None;
       for _ in 0..16 {
           let mut file_name = OsString::from(".");
           file_name.push(name);
           file_name.push(format!(
               ".{}-{}.tmp",
               std::process::id(),
               NEXT.fetch_add(1, Ordering::Relaxed)
           ));
           let temporary = parent.join(file_name);
           match OpenOptions::new().write(true).create_new(true).open(&temporary) {
               Ok(file) => return Ok((temporary, file)),
               Err(error) if error.kind() == io::ErrorKind::AlreadyExists => taken = Some(error),
               Err(error) => return Err(error),
           }
       }
       Err(taken.unwrap_or_else(|| io::Error::from(io::ErrorKind::AlreadyExists)))
   }

   /// Makes the rename durable: the directory entry is synchronised like the file was.
   #[cfg(unix)]
   fn sync_directory(parent: &Path) -> io::Result<()> {
       File::open(parent)?.sync_all()
   }

   #[cfg(not(unix))]
   #[allow(clippy::unnecessary_wraps)]
   fn sync_directory(_: &Path) -> io::Result<()> {
       Ok(())
   }

   #[cfg(test)]
   mod tests {
       // the tests of § Tests first, part A
   }
   ```

   What each case does after this change. The error texts are the ones `ess` prints, as
   `error: writing <path>: <io error>` (exit 1) for `run` and `mutate`, and as
   `conform-report.unwritable: writing <path>: <io error>` (exit 2) for `report`. The head line of
   each is unchanged:
   - A destination that is an existing directory: the temporary file is written in the parent,
     the rename fails, and the temporary file is removed. On Linux the text is `Is a directory (os error 21)`,
     the same as today.
   - A destination in a directory the user cannot write to: `create_new` fails with `Permission
     denied (os error 13)` and nothing is created. This changes today's behaviour: `fs::write`
     could truncate a writable file inside a read-only directory.
   - A destination file that is read-only (mode 0444) in a writable directory: it is replaced and
     keeps mode 0444. Today this was refused with `Permission denied`.
   - A symbolic link: the file it names is replaced and the link stays. That is the same result
     as today.
   - A dangling symbolic link: `canonicalize` fails with `No such file or directory (os error 2)`.
     Today `fs::write` created the link's target.
   - A failure to sync the directory after the rename returns an error, even though the
     destination already holds the complete new bytes. The command then exits as a write failure.

2. **`crates/edge/ess-cli/src/main.rs:22-23`**: add `mod replace_file;` between `mod release_evidence;`
   (line 22) and `mod requires;` (line 23).

3. **`main.rs:5082-5083`** (`render_conformance_report`, report/2): replace the `fs::write` call
   with
   ```rust
   replace_file::replace(path, summary.to_canonical_json()?.as_bytes())
       .with_context(|| format!("writing {}", path.display()))?;
   ```

4. **`main.rs:5099-5100`** (report/1): replace it with
   ```rust
   replace_file::replace(path, report.standalone().to_canonical_json().as_bytes())
       .with_context(|| format!("writing {}", path.display()))?;
   ```

5. **`main.rs:3980`** (`conform_report`): change only the call, to
   `if let Err(error) = replace_file::replace(report_out, text.as_bytes()) {`. Keep the
   `eprintln!` text and `ExitCode::from(REFUSED)` exactly as they are.

6. **`main.rs:4996`** (`finish_mutation_audit`): change it to
   `replace_file::replace(out, json.as_bytes()).with_context(|| format!("writing {}", out.display()))?;`.

Nothing else in `main.rs` changes. The write still happens where it happens today: before
anything is printed to standard output, and inside the `render` closure that
`known_failures::RunBinding::finish` calls (`src/known_failures.rs:194-210`).

## Tests first

Write these tests first. Part A is red before step 1 because `replace_file` does not exist, which
is a compile error. Part B is green before the change and after it: it guards the report bytes and
the error path. No test compiles generated code. Each part-A test is a few file operations in a
private directory and takes milliseconds. Each part-B test synthesizes the billing suite once and
runs it once against the built-in `billing` target, in a few seconds.

**A. `crates/edge/ess-cli/src/replace_file.rs`, `#[cfg(test)] mod tests`.** Use
`use super::*; use ess_cli::TemporaryDirectory;`. Make each directory with
`TemporaryDirectory::create("ess-replace-file").unwrap()`, which uses the process TMPDIR and
removes itself. Add one helper, `fn entries(directory: &Path) -> Vec<String>`, which returns every
entry name of `fs::read_dir`, sorted, hidden ones included. Every "nothing left beside it" check
compares `entries` with an exact list, so a leftover `.report.json.<pid>-<n>.tmp` fails the test.
Use `const EARLIER: &[u8] = b"earlier report\n";`.

1. `a_write_that_fails_after_the_first_byte_leaves_the_earlier_file`: write `EARLIER` to
   `report.json`, then call `replace_with(&path, |file| { file.write_all(b"{")?; Err(io::Error::other("injected failure")) })`.
   It returns `Err`, and `error.to_string() == "injected failure"`. Afterwards
   `fs::read(&path) == EARLIER` and `entries == ["report.json"]`.
2. `the_destination_keeps_its_earlier_bytes_until_the_new_ones_are_complete`: start from `EARLIER` in
   `report.json`. Inside the closure, `file.write_all(b"{\"new\": true}\n")?`, then
   `assert_eq!(fs::read(&path).unwrap(), EARLIER)`, then `Ok(())`. The closure runs after the new
   bytes are in the temporary file and before the rename, so this check runs inside the window,
   with no race and no sleep. Afterwards `fs::read(&path) == b"{\"new\": true}\n"` and
   `entries == ["report.json"]`.
3. `an_absent_destination_appears_only_complete`: no `report.json` exists. Inside the closure,
   write `b"complete\n"`, then `assert!(!path.exists())`, then `Ok(())`. Afterwards the file holds
   `b"complete\n"` and `entries == ["report.json"]`.
4. `a_failed_write_without_an_earlier_file_leaves_nothing`: no `report.json` exists. The closure
   writes `b"{"` and returns `Err(io::Error::other("injected failure"))`. Afterwards
   `entries(&directory).is_empty()`.
5. `#[cfg(unix)] a_symbolic_link_destination_replaces_the_file_it_names`: write `real.json` =
   `EARLIER` and `std::os::unix::fs::symlink("real.json", directory.join("report.json"))`, then
   `replace(&link, b"new\n").unwrap()`. Afterwards `fs::symlink_metadata(&link)` is a symlink,
   `fs::read_link(&link) == Path::new("real.json")`, `fs::read(directory.join("real.json")) ==
   b"new\n"`, and `entries == ["real.json", "report.json"]`.
6. `a_directory_destination_is_refused_and_leaves_no_temporary`: create the directory
   `report.json` holding `keep.txt` = `b"keep\n"`, then call `replace(&path, b"x")`. It is `Err`;
   do not assert the kind, which differs between platforms. Afterwards `path.is_dir()`,
   `fs::read(path.join("keep.txt")) == b"keep\n"` and `entries(&directory) == ["report.json"]`.
7. `#[cfg(unix)] the_replaced_file_keeps_its_permissions`: write `EARLIER` and
   `fs::set_permissions(&path, fs::Permissions::from_mode(0o640))`, using
   `std::os::unix::fs::PermissionsExt`, then call `replace(&path, b"new\n").unwrap()`. Afterwards
   `fs::metadata(&path).permissions().mode() & 0o777 == 0o640`.

Mutation check, once after everything is green and never committed: temporarily replace the body
of `replace_with` with `let mut file = File::create(path)?; write(&mut file)`. Tests 1, 2 and 3
must go red: 1 finds `{`, 2 reads the new bytes inside the closure, 3 finds the path present.
Then revert.

**B. `crates/edge/ess-cli/tests/count_reports.rs`.** Reuse the file's own `suite_files(label)`
(line 38) and `run_suite(directory, args)` (line 51). Add one helper, the same `entries` as in
part A, built on `std::fs::read_dir`.

8. `a_report_replaces_an_earlier_report_with_its_canonical_bytes_and_leaves_nothing_beside_it`:
   `let directory = suite_files("replace");`, then write `report.json` = `"earlier\n"` and call
   `run_suite(&directory, &["--report-format", "2", "--report-out", <report.json>])`. Assert
   success, with stderr in the message. The text read back is not `"earlier\n"`. With
   `admitted = ess_conformance::AdmittedSuite::from_json(&read(suite.json))`, it holds that
   `ess_conformance::CountReport::from_json(&text, &admitted).unwrap().to_canonical_json().unwrap() == text`,
   so the file is exactly the canonical report/2 bytes. And
   `entries(&directory) == ["report.json", "suite.json"]`.
9. `a_report_onto_a_directory_fails_and_leaves_nothing_beside_it`:
   `let directory = suite_files("directory");`, then create the directory `report.json` holding
   `keep.txt` = `"keep\n"` and run as in 8. Assert `status.code() == Some(1)`, `stdout` is empty,
   and stderr contains `format!("writing {}", <report.json>.display())`. The released 0.97.1
   prints `error: writing <dir>/report.json: Is a directory (os error 21)` here. `keep.txt` still
   reads `"keep\n"` and `entries(&directory) == ["report.json", "suite.json"]`.

The existing byte expectations, which must stay green unchanged:
`count_cli_preserves_default_bytes_and_standalone_detailed_pairing` (count_reports.rs:61,
report-out equals the detailed summary) and
`count_cli_configuration_and_original_suite_refusals_preserve_destinations` (:129).
`known_failures_cli.rs` `run_accounts_known_failures_and_keeps_its_report_verdict_and_exit`
(:182): the accounting is validated against the exact bytes of the report file.
`conform_report.rs` `supplied_results_match_the_report_ess_writes_for_its_own_run` (:100). And
`mutate_cli.rs` `the_json_output_is_the_report_bytes` (:220).

These tests do not use `CARGO_TARGET_TMPDIR`. If you add one that does, create it with
`std::fs::create_dir_all` first. A TMPDIR inside a Git checkout breaks
`observed_bindings::tests::an_output_created_after_preflight_is_preserved` (repository
`AGENTS.md`). The `--bin ess` command below filters on `replace_file::`, so that test is not run.

## Commands

Run these in order from the worktree root. `cargo` goes through the `slot` shim on its own.

1. `cargo test -p ess-cli --locked --bin ess replace_file::`: before step 1 it fails to compile.
   Afterwards it reports `7 passed` (5 on a non-unix host).
2. `cargo test -p ess-cli --locked --test count_reports`: `5 passed` (3 existing, 2 new).
3. `cargo test -p ess-cli --locked --test known_failures_cli run_`: the three `run_` tests pass.
4. `cargo test -p ess-cli --locked --test conform_report`: all pass.
5. `cargo test -p ess-cli --locked --test mutate_cli`: all pass.
6. `cargo clippy -p ess-cli --all-targets --locked -- -D warnings`: no warnings.
7. `slot task fmt-check`: clean.

Byte pins (repository `AGENTS.md`): this change writes the same bytes, so none of the listed pin
targets moves. Do not run `ess-conformance`, `ess-synth`, `ess-openapi` or `ess-compiler`
targets, and no `task test-xtask`: no `ValidationCode`, diagnostic or format changes.

## Files this unit owns

- `crates/edge/ess-cli/src/replace_file.rs` (new)
- `crates/edge/ess-cli/src/main.rs` (only the six places of steps 2-6)
- `crates/edge/ess-cli/tests/count_reports.rs`

Must not touch:
- `crates/edge/ess-cli/src/known_failures.rs`
- `crates/edge/ess-cli/src/lib.rs`: the library is the recovery driver's surface, and this helper stays in the binary.
- `crates/edge/ess-cli/src/output_ownership/**`
- `crates/edge/ess-cli/src/recovery/**`
- `crates/edge/ess-cli/tests/validate_format_hint.rs` (another unit)
- `crates/specify/ess-domain/src/**`, `crates/generate/ess-synth/**`, `crates/edge/ess-xtask/**`,
  `crates/verify/ess-conformance/src/authored.rs`
- `.config/nextest.toml`, `Taskfile.yml`, `.github/workflows/*`, `CHANGELOG.md`, `AGENTS.md`

## For the coordinator at integration

`CHANGELOG.md`: under `## [Unreleased]`, add a `### Fixed` heading if there is none, then this
bullet:

```markdown
- `ess verify conform run --report-out`, `ess verify conform report --results … --report-out` and
  `ess verify conform mutate --report-out` write the report to a new file beside the destination,
  synchronise it and rename it onto the destination, so the destination holds either the earlier
  report or the complete new one at every instant: a run killed while writing no longer leaves a
  truncated file where a complete report was. The report bytes are unchanged and a replaced file
  keeps its permissions. A destination that is a symbolic link replaces the file it names and
  keeps the link. A destination in a directory that cannot be written is now refused even where
  the file itself was writable, a read-only destination file in a writable directory is now
  replaced, and a dangling symbolic link is refused instead of creating its target.
```

The plan story's second acceptance line is answered as follows, for the review:
- `run --accounting-out` is already written only as a new file: `known_failures::write_new`,
  `src/known_failures.rs:88-100`, uses `create_new`, and `check_outputs` refuses an existing path
  (`:63-68`).
- `run` writes no execution context. Only `report --results --known-failing` writes one, also
  through `write_new`.

No other shared file changes.

## Out of scope

- The new-file writers in `known_failures.rs`: `write_new` for `--accounting-out` and
  `--execution-context-out`, and the report of `report --results` with `--known-failing`
  (`:360-362`). They never replace an earlier file. An interrupted run can still leave a partial
  new file under the final name. That is a possible follow-up story; it is not this unit.
- `write_imported`/`temporary_sibling` (`main.rs:4168-4240`, `import-history`): it already
  renames, without a sync. Leave it.
- `write_named_output`/`write_preflighted_files` (`main.rs:3374`, `:3487`; `synthesize --out` and
  other single-file outputs): these also write in place, but they are different verbs. They are a
  follow-up.
- The generated Go and TypeScript runners' `ESS_REPORT_OUT` write, in `ess-synth`, which another
  unit owns.
- Do not add a crate dependency (`tempfile` is a dev-dependency only, and `NamedTempFile::persist`
  is not to be used in production code). Do not move the helper into `lib.rs`. Do not refuse
  symbolic links. Do not change any error head line or exit status.

## Acceptance

- `replace_with` that fails after the first byte leaves the earlier bytes and no temporary file:
  tests 1 and 4.
- The destination is never partial: it holds the earlier bytes, or nothing, while the new bytes
  are being written. Tests 2 and 3, and the mutation check turns 1-3 red.
- No temporary file remains after success or failure, including a directory destination: tests
  1-6 and 8-9.
- A symbolic link and the file's permissions are kept: tests 5 and 7.
- The report/2 bytes of `run` are canonical and unchanged: test 8, and the existing tests named in
  § Tests first stay green (commands 2-5).
- Clippy and fmt are clean: commands 6 and 7.
- No format version changes.

## Depends on

nothing
