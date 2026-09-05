unit:                   story:review-output-containment — Compose output-set correction, pass 1
verdict:                green
cases:                  executed 225→230, red 5 observed, 0 remaining
origin:                 n/a
wrote-outside-worktree: none
needs-coordinator:      no

## 1. Unit and acceptance

An escaping or colliding artifact/page path is refused before any generated output changes inside or outside the requested root.

This correction answers the confirmed acceptance blocker preserved in review-result:review-boundaries-2-containment-adversary-pass-1: compose wrote its composition and client-plan companions before checking their relationships with generated client destinations. Both companion flags and all generated client files now form one command output set. All path admission, filesystem inspection and mutual collision checks complete before the first write.

The starting production commit is e6803c061b33dfe8d5c9fdfff10d8f1408083b31, with the adversary's two test-only additions left uncommitted. This correction leaves those tests and all their assertions intact. This is ready for the required second attack, not a claim that integration has completed.

## 2. Observed diff, class and scope

```text
$ git --no-pager diff --stat
 crates/edge/ess-cli/src/main.rs                 | 128 +++++++--
 crates/edge/ess-cli/tests/output_containment.rs | 346 ++++++++++++++++++++++++
 crates/generate/ess-gen/tests/docs.rs           |  68 +++++
 3 files changed, 520 insertions(+), 22 deletions(-)
```

This observed working-tree diff includes the preserved adversary additions (114 CLI test lines and 68 ess-gen test lines) as well as this correction. The correction itself changes only crates/edge/ess-cli/src/main.rs and adds five tests plus a snapshot helper to the existing CLI output_containment suite. It makes no ess-gen production or test edits.

| Part | Change or evidence |
| --- | --- |
| Fix | compose builds the full list of resolved absolute output destinations, including --out, --client-plan-out and every --client-rust-out artifact, then validates that set before calling the writer |
| Class | A command declaring several output files must compare the complete set and inspect every destination before any output changes; validating each writer's subset independently cannot establish this |
| Companion/companion | Exact duplicates, ASCII-case aliases, normalized aliases, file/parent conflicts in both flag orders, with and without --client-rust-out |
| Companion/generated | Both companions against generated files, parent directories, case aliases and normalized aliases; the original adversary matrix is preserved |
| Existing destination state | Named companions now share root/ancestor and leaf inspection with generated files; symlinks, hardlinks, incompatible types and missing final parent directories refuse before the other outputs change |
| Caller path compatibility | Caller-selected Unicode, spaces and punctuation remain admitted; the portable generated-name alphabet is not imposed on chosen paths |
| Parent normalization | The common root resolver checks every encountered existing directory before resolving .., and writes use the resolved destination so discarded missing directories are never created |
| Native file intent | A trailing native separator, final . or final .. remains a directory request and cannot be silently converted into a file write |
| Valid control | A compose invocation with both disjoint companions using Unicode/space/punctuation filenames and a ../out root succeeds twice with identical snapshots |
| Other scoped generated sinks | generate, synthesize, conform_web, Go conformance synthesis, build/Docker/Helm projection, and infrastructure projection each already supply one complete generated set to the common writer; their destination checks remain in place |
| Persistence and boundaries | No persisted construct, format, identity, bytes, producer package, planning artifact or public publication flow changes |

The concrete mechanism measured was moving the companions and generated client files into one preflighted sequence, with their actual absolute destinations as the comparison keys. The isolated class run changed from 1 passed/4 failed to 5 passed/0 failed. After the native-directory spelling regression was added and corrected, the complete compose lane is 6 passed/0 failed.

Production diff:
```diff
diff --git a/crates/edge/ess-cli/src/main.rs b/crates/edge/ess-cli/src/main.rs
index 3f5da46..3a0f9f8 100644
--- a/crates/edge/ess-cli/src/main.rs
+++ b/crates/edge/ess-cli/src/main.rs
@@ -1858,31 +1858,35 @@ fn compose(
 
     let client_plan = composition.client_plan();
     let client_artifacts = client_plan.rust_artifacts();
+    let composition_json = composition.to_canonical_json();
+    let client_plan_json = client_plan.to_canonical_json();
+    let mut outputs = Vec::new();
+    for (path, contents) in [
+        (out, &composition_json),
+        (client_plan_out, &client_plan_json),
+    ] {
+        if let Some(path) = path {
+            outputs.push((preflight_named_output(path)?, contents.as_str()));
+        }
+    }
     if let Some(root) = client_rust_out {
-        preflight_generated_files(
+        let root = preflight_generated_files(
             root,
             &client_artifacts
                 .values()
                 .map(ess_composition::ClientArtifact::path)
                 .collect::<Vec<_>>(),
         )?;
-    }
-    let composition_json = composition.to_canonical_json();
-    if let Some(out) = out {
-        fs::write(out, &composition_json).with_context(|| format!("writing {}", out.display()))?;
-    }
-    let client_plan_json = client_plan.to_canonical_json();
-    if let Some(out) = client_plan_out {
-        fs::write(out, &client_plan_json).with_context(|| format!("writing {}", out.display()))?;
-    }
-    if let Some(root) = client_rust_out {
-        write_generated_files(
-            root,
+        outputs.extend(
             client_artifacts
                 .values()
-                .map(|artifact| (artifact.path(), artifact.contents())),
-        )?;
+                .map(|artifact| (root.join(artifact.path()), artifact.contents())),
+        );
     }
+    // All three destinations belong to this invocation. Checking only the generated subset
+    // would let either companion overwrite a client file or create a file where it needs a parent.
+    preflight_output_set(outputs.iter().map(|(path, _)| path.as_path()))?;
+    write_preflighted_files(outputs)?;
 
     match format {
         Format::Text => println!(
@@ -2037,14 +2041,14 @@ fn write_artifacts(
     Ok(())
 }
 
-/// Checks a complete generated tree before changing any output.
+/// Resolves the requested directory for output preflight.
 ///
 /// Relative artifact paths obey `ess_gen::artifact::validate_paths`. The requested root may be
 /// absolute or relative, including `..`, but cannot traverse a pre-existing symlink. Its components
 /// are inspected before resolving `..`; writes use the resolved root so discarded missing
 /// directories are never created. A Windows root must be fully qualified or have no drive/root
 /// prefix, rather than depend on a drive's separate current directory. Every existing
-/// root ancestor must be a directory; every existing destination must be a regular file with one
+/// root ancestor must be a directory. The remaining preflight checks require a regular file with one
 /// hard link on Unix. On other platforms replacing an existing file is refused because this
 /// implementation cannot verify its hard-link count. Case aliases in existing destination
 /// directories are refused even on a case-sensitive host.
@@ -2053,8 +2057,7 @@ fn write_artifacts(
 /// preflight does not defend against concurrent replacement, hostile mounts or filesystem-specific
 /// aliases beyond the portable path rules. It provides no rollback for later I/O failures and does
 /// not retire old files. Those require a separate output ownership/transaction contract.
-fn preflight_generated_files(root: &Path, paths: &[&str]) -> Result<PathBuf> {
-    ess_gen::artifact::validate_paths(paths.iter().copied()).map_err(anyhow::Error::msg)?;
+fn resolve_output_directory(root: &Path) -> Result<PathBuf> {
     let absolute = if root.is_absolute() {
         root.to_path_buf()
     } else {
@@ -2090,7 +2093,12 @@ fn preflight_generated_files(root: &Path, paths: &[&str]) -> Result<PathBuf> {
         }
         inspect_output_entry(&current, false)?;
     }
-    let absolute = current;
+    Ok(current)
+}
+
+fn preflight_generated_files(root: &Path, paths: &[&str]) -> Result<PathBuf> {
+    ess_gen::artifact::validate_paths(paths.iter().copied()).map_err(anyhow::Error::msg)?;
+    let absolute = resolve_output_directory(root)?;
     for relative in paths {
         let mut destination = absolute.clone();
         let mut components = relative.split('/').peekable();
@@ -2122,6 +2130,73 @@ fn preflight_generated_files(root: &Path, paths: &[&str]) -> Result<PathBuf> {
     Ok(absolute)
 }
 
+/// A caller-selected file is resolved against its selected parent, without imposing the
+/// generated-name alphabet on it. Parent directories must already exist, as for the original
+/// single-file writer. Existing links and incompatible file types are refused before any output.
+fn preflight_named_output(path: &Path) -> Result<PathBuf> {
+    // Path::file_name drops trailing separators. Keep a caller's directory request a directory
+    // request: `report/` must not become a successful write to a new file called `report`.
+    let last = path
+        .as_os_str()
+        .as_encoded_bytes()
+        .rsplit(|byte| std::path::is_separator(char::from(*byte)))
+        .next()
+        .unwrap_or_default();
+    if last.is_empty() || last == b"." || last == b".." {
+        bail!(
+            "output must name a file, not a directory: {}",
+            path.display()
+        );
+    }
+    let name = path
+        .file_name()
+        .with_context(|| format!("output must name a file: {}", path.display()))?;
+    let parent = resolve_output_directory(path.parent().unwrap_or(Path::new(".")))?;
+    for entry in fs::read_dir(&parent)
+        .with_context(|| format!("inspecting output parent {}", parent.display()))?
+    {
+        let existing = entry?.file_name();
+        if existing != name && existing.eq_ignore_ascii_case(name) {
+            bail!("output path aliases an existing entry: {}", path.display());
+        }
+    }
+    let destination = parent.join(name);
+    inspect_output_entry(&destination, true)?;
+    Ok(destination)
+}
+
+/// Checks the whole command's resolved absolute file set, including independently selected
+/// companion outputs. No path spellings are rewritten: folding ASCII case detects aliases,
+/// and recording all ancestors detects file/directory conflicts in either declaration order.
+/// Unicode and other caller-selected filename characters remain admitted. Filesystem-specific
+/// aliases beyond ASCII case, concurrent replacement and rollback remain outside this preflight.
+fn preflight_output_set<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Result<()> {
+    let mut entries: std::collections::BTreeMap<PathBuf, (PathBuf, bool)> =
+        std::collections::BTreeMap::new();
+    for path in paths {
+        let mut original = PathBuf::new();
+        let mut folded = PathBuf::new();
+        let mut components = path.components().peekable();
+        while let Some(component) = components.next() {
+            original.push(component.as_os_str());
+            folded.push(component.as_os_str().to_ascii_lowercase());
+            let file = components.peek().is_none();
+            if let Some((previous, previous_file)) = entries.get(&folded) {
+                if previous != &original || file || *previous_file {
+                    bail!(
+                        "colliding command output paths: {} and {}",
+                        previous.display(),
+                        path.display()
+                    );
+                }
+            } else {
+                entries.insert(folded.clone(), (original.clone(), file));
+            }
+        }
+    }
+    Ok(())
+}
+
 fn inspect_output_entry(path: &Path, file: bool) -> Result<()> {
     let metadata = match fs::symlink_metadata(path) {
         Ok(metadata) => metadata,
@@ -2169,8 +2244,17 @@ fn write_generated_files<'a>(
         root,
         &files.iter().map(|(path, _)| *path).collect::<Vec<_>>(),
     )?;
-    for (relative, contents) in files {
-        let path = root.join(relative);
+    write_preflighted_files(
+        files
+            .into_iter()
+            .map(|(relative, contents)| (root.join(relative), contents)),
+    )
+}
+
+/// Writes only paths returned by preflight. All destinations in the command must have passed
+/// preflight before this starts; this loop does not roll back a subsequent I/O failure.
+fn write_preflighted_files<'a>(files: impl IntoIterator<Item = (PathBuf, &'a str)>) -> Result<()> {
+    for (path, contents) in files {
         if let Some(parent) = path.parent() {
             fs::create_dir_all(parent)?;
         }
```

## 3. Red evidence

All commands ran from /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment, with this environment:
```text
TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2
RUSTC_WRAPPER=/usr/bin/sccache
CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
```

Before edits, the exact whole-package baseline was rerun: ess-cli executed 36 cases, 35 passed and the retained adversary case failed; ess-gen executed 189 cases, all passed. Complete outputs are preserved in correction-baseline-cli.log and correction-baseline-gen.log.

The initial new class test fixture accidentally pre-created both src and SRC for every generated-collision case, which caused those cases to refuse for an unrelated existing alias. That first output is retained in correction-class-red.log (2 passed, 3 failed). Before any production change, the fixture was corrected to create the selected parent only; no assertion was weakened. The following decisive run then exercised the intended alias/normalization collisions as failures:
```text
$ cargo test --locked -p ess-cli --test output_containment composition_
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.20s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 5 tests
test composition_refuses_companion_links_before_any_other_output_changes ... FAILED
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... FAILED
test composition_companions_form_one_output_set_even_without_a_generated_tree ... FAILED
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... FAILED

failures:

---- composition_refuses_companion_links_before_any_other_output_changes stdout ----

thread 'composition_refuses_companion_links_before_any_other_output_changes' (392417) panicked at crates/edge/ess-cli/tests/output_containment.rs:314:13:
--out, hard=false: Output { status: ExitStatus(unix_wait_status(0)), stdout: "devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to linked.json; client plan written to other.json; 3 Rust client artifact(s) written to out\n", stderr: "" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- composition_companion_outputs_cannot_collide_with_the_generated_client_tree stdout ----

thread 'composition_companion_outputs_cannot_collide_with_the_generated_client_tree' (392413) panicked at crates/edge/ess-cli/tests/output_containment.rs:361:5:
companion/generated destination collisions must refuse before writes:
--out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-392412-9/out/src
--client-plan-out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--client-plan-out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-392412-16/out/src

---- composition_companions_form_one_output_set_even_without_a_generated_tree stdout ----

thread 'composition_companions_form_one_output_set_even_without_a_generated_tree' (392414) panicked at crates/edge/ess-cli/tests/output_containment.rs:252:5:
["same.json", "same.json"], rust=false: exit Some(0), output changed
["same.json", "SAME.JSON"], rust=false: exit Some(0), output changed
["same.json", "working/../same.json"], rust=false: exit Some(0), output changed
["new-parent", "new-parent/child.json"], rust=false: exit Some(1), output changed
["same.json", "missing/child.json"], rust=false: exit Some(1), output changed
["same.json", "same.json"], rust=true: exit Some(0), output changed
["same.json", "SAME.JSON"], rust=true: exit Some(0), output changed
["same.json", "working/../same.json"], rust=true: exit Some(0), output changed
["new-parent", "new-parent/child.json"], rust=true: exit Some(1), output changed
["same.json", "missing/child.json"], rust=true: exit Some(1), output changed

---- composition_preflight_includes_companion_generated_aliases_and_both_companions stdout ----

thread 'composition_preflight_includes_companion_generated_aliases_and_both_companions' (392416) panicked at crates/edge/ess-cli/tests/output_containment.rs:273:5:
--out out/cargo.TOML: exit Some(1), output changed
--out out/working/../Cargo.toml: exit Some(0), output changed
--out out/src/lib.rs: exit Some(0), output changed
--client-plan-out out/cargo.TOML: exit Some(1), output changed
--client-plan-out out/working/../Cargo.toml: exit Some(0), output changed
--client-plan-out out/src/lib.rs: exit Some(0), output changed
--client-plan-out out/src/lib.rs/child: exit Some(1), output changed
--client-plan-out out: exit Some(1), output changed


failures:
    composition_companion_outputs_cannot_collide_with_the_generated_client_tree
    composition_companions_form_one_output_set_even_without_a_generated_tree
    composition_preflight_includes_companion_generated_aliases_and_both_companions
    composition_refuses_companion_links_before_any_other_output_changes

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.13s

error: test failed, to rerun pass `-p ess-cli --test output_containment`
exit: 101
```

The coordinator independently identified a possible native-file spelling regression in the first correction. A new test was added and run before changing that code; the trailing-separator cases confirmed it. The final-component spelling is now checked before Path::file_name can discard its separator:
```text
$ cargo test --locked -p ess-cli --test output_containment composition_does_not_reinterpret_directory_spelling_as_a_named_output_file -- --exact
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.70s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... FAILED

failures:

---- composition_does_not_reinterpret_directory_spelling_as_a_named_output_file stdout ----

thread 'composition_does_not_reinterpret_directory_spelling_as_a_named_output_file' (405066) panicked at crates/edge/ess-cli/tests/output_containment.rs:372:5:
--out new-file/: exit Some(0), output changed
--client-plan-out new-file/: exit Some(0), output changed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    composition_does_not_reinterpret_directory_spelling_as_a_named_output_file

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p ess-cli --test output_containment`
exit: 101
```

## 4. Green evidence and exact runner counts

| Runner lane | Executed before → after | Final exit |
| --- | --- | --- |
| ess-cli unit | 11 → 11 | 0 |
| ess-cli command_surface | 5 → 5 | 0 |
| ess-cli command_surface_adversary | 4 → 4 | 0 |
| ess-cli go_conformance | 7 → 7 | 0 |
| ess-cli output_containment | 9 → 14 | 0 |
| ess-gen unit | 55 → 55 | 0 |
| ess-gen agreement | 4 → 4 | 0 |
| ess-gen asyncapi | 18 → 18 | 0 |
| ess-gen corpus | 3 → 3 | 0 |
| ess-gen determinism | 2 → 2 | 0 |
| ess-gen docs | 32 → 32 | 0 |
| ess-gen openapi | 35 → 35 | 0 |
| ess-gen provenance | 9 → 9 | 0 |
| ess-gen relations | 4 → 4 | 0 |
| ess-gen schema | 27 → 27 | 0 |
| ess-gen doc-tests | 0 → 0 | 0 |

Counts are taken from the runner summary lines in the baseline and final package logs. This pass adds tests only to output_containment; unchanged lanes contain no new tests. All prior implementor and adversary tests remain selected. Totals are ess-cli 36→41 and ess-gen 189→189, combined 225→230; all final cases passed, none failed or were ignored.

Baseline summary lines:
```text
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/corpus.rs (target/debug/deps/corpus-93718f8b1fc63993)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/determinism.rs (target/debug/deps/determinism-9942695ed2e87dec)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/openapi.rs (target/debug/deps/openapi-cbc5ba4392fca057)
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/relations.rs (target/debug/deps/relations-7cdc743b0b26371d)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/schema.rs (target/debug/deps/schema-e0945bde8d462715)
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
   Doc-tests ess_gen
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo test --locked -p ess-cli --test output_containment composition_
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.62s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 6 tests
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.14s

exit: 0
```

```text
$ cargo test --locked -p ess-cli
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.23s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 14 tests
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

exit: 0
```

```text
$ cargo test --locked -p ess-gen
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)

running 55 tests
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)

running 4 tests
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)

running 18 tests
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok
test every_component_gets_one_document_named_after_it ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/corpus.rs (target/debug/deps/corpus-93718f8b1fc63993)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-9942695ed2e87dec)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test every_page_says_which_specification_produced_it ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/openapi.rs (target/debug/deps/openapi-cbc5ba4392fca057)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_command_is_only_ever_a_post ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 9 tests
test a_text_without_both_digests_reads_as_nothing ... ok
test a_damaged_digest_reads_as_nothing ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/relations.rs (target/debug/deps/relations-7cdc743b0b26371d)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/schema.rs (target/debug/deps/schema-e0945bde8d462715)

running 27 tests
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test every_schema_says_which_specification_it_came_from ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

```text
$ cargo fmt -p ess-cli --check
exit: 0
```

```text
$ cargo fmt -p ess-gen --check
exit: 0
```

```text
$ cargo clippy --locked -p ess-cli --all-targets -- -D warnings
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.91s
exit: 0
```

```text
$ cargo clippy --locked -p ess-gen --all-targets -- -D warnings
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
    Finished `dev` profile [unoptimized] target(s) in 0.15s
exit: 0
```

## 5. Deliberate boundaries

- The first implementor report's statement that compose companions remain unchanged is superseded by this correction. They are part of the command's complete declared output set and now share the bounded preflight guarantees.
- Caller-selected filenames keep native spelling rules and accept Unicode, spaces and punctuation. Only directory-like final spellings are refused to preserve file intent; generated artifacts retain their separate portable-name contract.
- Named output parent directories must already exist, preserving the original single-file writer's requirement. Missing or incompatible parents are now refused before any companion or generated output is changed.
- Existing symlinks and Unix hardlinks are refused for companion destinations because they can otherwise alias generated destinations or change another file. Existing non-Unix replacement remains conservatively refused as documented in the original implementation; this correction was tested on Linux.
- ASCII case is the declared alias comparison. Filesystem-specific aliases beyond that rule, hostile mounts, concurrent replacement and rollback after later I/O failures remain outside stable-filesystem preflight.
- The correction validates existing paths and output relationships; it introduces no persisted noun or binding-design construct.
- No tests were removed, weakened, ignored or skipped. No planning mutation, staging, commit, branch/worktree lifecycle, cache cleanup, full-workspace formatter or live external operation was performed by this correction.
- Coordinator owns the second adversarial attack, integrated gate, publication, commits and all managed cleanup.

## 6. Writes and retained resource evidence

Authored paths outside the worktree: none. All new logs and this separate report are under /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2. Temporary fixtures used the assigned TMPDIR and were removed only by their fixture Drop implementation. Existing raw reports and logs were retained, including the first fixture-error run. Cargo used this worktree's target directory and the prescribed shared sccache service; no cache or build directory was removed.

Final observed check and resource output:
```text
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 729953665024 133865947136  85% /
1107928	target
```

