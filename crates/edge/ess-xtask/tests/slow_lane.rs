//! The scanner behind `cargo xtask slow-list`: a test marked `#[ignore = "slow lane: <reason>"]`
//! is found, nothing else is, a marker that cannot run is refused, and the marked tests of this
//! tree are exactly the rows of [`MARKED`].

#[path = "../src/scratch.rs"]
mod scratch;
#[path = "../src/slow_lane.rs"]
mod slow_lane;

use std::fs;
use std::path::Path;

use scratch::Scratch;
use slow_lane::{run, scan, Args};

/// Every file of this tree that holds a marked test, and the names marked in it, sorted by path.
const MARKED: &[(&str, &[&str])] = &[
    (
        "crates/edge/ess-cli/tests/tutorial_page.rs",
        &["the_walkthrough_runs_the_npm_and_go_blocks_too"],
    ),
    (
        "crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs",
        &["adv_e_u2_byte_identity_probe"],
    ),
    (
        "crates/generate/ess-synth/tests/adversary_e_u6_bytes.rs",
        &["adv_e_u6_byte_identity_probe"],
    ),
    (
        "crates/generate/ess-synth/tests/generated_view_queries.rs",
        &[
            "adversary_go_typed_parameters_and_orders_answer_as_the_interpreted_target",
            "adversary_rust_typed_parameters_and_orders_answer_as_the_interpreted_target",
            "conditional_measure_all_six_functions_execute_in_the_generated_go_queries",
            "conditional_measure_all_six_functions_execute_in_the_generated_rust_queries",
            "the_generated_queries_build_with_warnings_denied_and_pass_their_own_suite",
            "the_go_queries_build_and_pass_their_own_suite",
            "the_go_servers_of_the_workbench_views_answer_their_parameters_and_orders",
            "the_rust_servers_of_the_workbench_views_answer_their_parameters_and_orders",
            "the_single_crate_layout_passes_the_same_suite_with_its_server_feature",
        ],
    ),
    (
        "crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs",
        &["adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a"],
    ),
    (
        "crates/verify/ess-conformance/tests/adversary_287_pass1.rs",
        &[
            "caller_set_wasm_matches_actual_native_outcomes",
            "caller_upsert_wasm_matches_actual_native_outcomes",
        ],
    ),
    (
        "crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs",
        &[
            "adversary_a2_count_row_set_orders_are_told_apart",
            "adversary_a2_every_step_4_order_is_told_apart_across_declaration_orders",
            "adversary_a2_forall_beside_named_rows_witnesses_every_branch",
            "adversary_a2_row_set_over_a_missing_named_row_orders_are_told_apart",
            "adversary_a2_row_set_over_the_other_named_row_orders_are_told_apart",
            "adversary_a2_separate_creator_orders_are_told_apart",
            "adversary_a2_when_beside_a_named_refusal_orders_are_told_apart",
        ],
    ),
    (
        "crates/verify/ess-conformance/tests/adversary_u22_pass1.rs",
        &[
            "a_refusal_the_step_bound_cut_short_names_the_bound",
            "a_seeded_chain_synthesizes_the_same_suite_twice",
            "every_reachable_branch_of_seeded_plain_sixteen_refusal_chains_has_its_scenario",
            "every_reachable_branch_of_seeded_sixteen_refusal_chains_over_the_adopter_shape_has_its_scenario",
        ],
    ),
    (
        "crates/verify/ess-conformance/tests/adversary_w3_266_267_dump.rs",
        &["adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison"],
    ),
    (
        "crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs",
        &["a_target_that_retries_not_eligible_fails_the_refusal"],
    ),
    (
        "crates/verify/ess-conformance/tests/enum_presence_guard.rs",
        &["no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere"],
    ),
    (
        "crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs",
        &[
            "every_repository_component_keeps_its_synthesized_bytes",
            "every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs",
            "every_repository_model_keeps_its_synthesized_bytes",
        ],
    ),
];

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn demo(scratch: &Scratch) {
    write(
        scratch.path(),
        "crates/area/demo/Cargo.toml",
        "[package]\nname = \"demo-pkg\"\nversion = \"0.0.0\"\n",
    );
}

fn args(packages: bool, package: Option<&str>, targets: bool, filter: bool) -> Args {
    Args {
        packages,
        package: package.map(str::to_owned),
        targets,
        filter,
    }
}

#[test]
fn the_scanner_reads_a_marker_on_a_test_and_nothing_else() {
    let scratch = Scratch::new("slow-lane-reads");
    demo(&scratch);
    write(
        scratch.path(),
        "crates/area/demo/tests/a.rs",
        "#[test]\n#[ignore = \"slow lane: builds a crate\"]\nfn one() {}\n\
         #[test]\n#[ignore]\nfn bare() {}\n\
         #[test]\n#[ignore = \"slow probe: x\"]\nfn probe() {}\n\
         #[test]\nfn plain() {}\n",
    );
    write(
        scratch.path(),
        "crates/area/demo/tests/b/main.rs",
        "mod support;\n",
    );
    write(
        scratch.path(),
        "crates/area/demo/tests/b/support.rs",
        "mod inner {\n    #[test]\n    #[ignore = \"slow lane: nested\"]\n    fn deep() {}\n}\n",
    );
    write(
        scratch.path(),
        "crates/area/demo/tests/fixtures/f.rs",
        "#[test]\n#[ignore = \"slow lane: a fixture\"]\nfn fixture_test() {}\n",
    );
    let rows: Vec<(String, String, String)> = scan(scratch.path())
        .unwrap()
        .into_iter()
        .map(|m| (m.package, m.target, m.test))
        .collect();
    let expected: Vec<(String, String, String)> =
        [("demo-pkg", "a", "one"), ("demo-pkg", "b", "deep")]
            .into_iter()
            .map(|(p, t, n)| (p.to_owned(), t.to_owned(), n.to_owned()))
            .collect();
    assert_eq!(rows, expected);
    assert_eq!(
        run(scratch.path(), &args(false, None, false, false)).unwrap(),
        "demo-pkg\ta\tone\ndemo-pkg\tb\tdeep\n"
    );
    assert_eq!(
        run(scratch.path(), &args(true, None, false, false)).unwrap(),
        "demo-pkg\n"
    );
    assert_eq!(
        run(scratch.path(), &args(false, Some("demo-pkg"), true, false)).unwrap(),
        "a\nb\n"
    );
    assert_eq!(
        run(scratch.path(), &args(false, Some("demo-pkg"), false, true)).unwrap(),
        "test(/(^|::)(deep|one)$/)\n"
    );
}

#[test]
fn the_scanner_refuses_a_marker_it_cannot_run() {
    let cases = [
        (
            "#[ignore = \"slow lane: no test attribute\"]\nfn helper() {}\n",
            "`helper` carries a `slow lane:` marker but is not a #[test]",
        ),
        (
            "#[test]\n#[ignore = \"slow lane: \"]\nfn empty() {}\n",
            "`empty` carries a `slow lane:` marker with no reason",
        ),
    ];
    for (source, message) in cases {
        let scratch = Scratch::new("slow-lane-refuses");
        demo(&scratch);
        write(scratch.path(), "crates/area/demo/tests/a.rs", source);
        let error = scan(scratch.path()).unwrap_err();
        assert!(error.contains("tests/a.rs"), "{error}");
        assert!(error.contains(message), "{error}");
    }
    let scratch = Scratch::new("slow-lane-twice");
    demo(&scratch);
    let twice = "#[test]\n#[ignore = \"slow lane: x\"]\nfn same() {}\n";
    write(scratch.path(), "crates/area/demo/tests/a.rs", twice);
    write(scratch.path(), "crates/area/demo/tests/b.rs", twice);
    let error = scan(scratch.path()).unwrap_err();
    assert!(error.contains("two marked tests named `same`"), "{error}");
    assert!(
        error.contains("tests/a.rs") && error.contains("tests/b.rs"),
        "{error}"
    );
}

#[test]
fn every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root stands three levels above this crate");
    let mut found: Vec<(String, Vec<String>)> = Vec::new();
    for m in scan(root).unwrap() {
        match found.last_mut() {
            Some((file, names)) if *file == m.file => names.push(m.test),
            _ => found.push((m.file, vec![m.test])),
        }
    }
    for (_, names) in &mut found {
        names.sort();
    }
    found.sort();
    let table: Vec<(String, Vec<String>)> = MARKED
        .iter()
        .map(|(file, names)| {
            let mut names: Vec<String> = names.iter().map(|n| (*n).to_owned()).collect();
            names.sort();
            ((*file).to_owned(), names)
        })
        .collect();
    let mut problems = Vec::new();
    for (file, names) in &found {
        if !table.contains(&(file.clone(), names.clone())) {
            problems.push(format!("{file} {names:?}: marked, not in the table"));
        }
    }
    for row in &table {
        if !found.contains(row) {
            problems.push(format!("{} {:?}: in the table, not marked", row.0, row.1));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
    assert!(
        MARKED.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "MARKED rows are sorted by path"
    );
}

/// The workspace root, three levels above this crate.
fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the workspace root stands three levels above this crate")
}

/// Adversary: a test whose name says it runs in the test step is not marked for the slow lane.
/// `CHANGELOG.md` says the seeded sweeps of `adversary_u22_pass1` run a reduced set in the test
/// step; the short twin of the seeded sweep over the adopter's input shape carries the marker, so
/// the step runs no seed of that sweep.
#[test]
fn adversary_no_test_named_for_the_step_is_marked_for_the_slow_lane() {
    let in_lane: Vec<String> = scan(workspace())
        .unwrap()
        .into_iter()
        .filter(|m| m.test.ends_with("_in_the_step"))
        .map(|m| format!("{}::{}", m.file, m.test))
        .collect();
    assert!(
        in_lane.is_empty(),
        "named for the test step and marked for the slow lane, so the step runs none of them: \
         {in_lane:#?}"
    );
}

/// Adversary: a `slow lane:` marker the scanner cannot see is a test that runs nowhere. A test
/// defined by a `macro_rules!` (this workspace has such tests, `ess-cli`'s
/// `authored_scenarios.rs`) carrying the marker is ignored by the test step and not listed for
/// the slow lane. The scanner either lists it or refuses the file.
#[test]
fn adversary_a_marker_inside_a_macro_is_listed_or_refused() {
    let scratch = Scratch::new("slow-lane-macro");
    demo(&scratch);
    write(
        scratch.path(),
        "crates/area/demo/tests/a.rs",
        "macro_rules! cases {\n    ($name:ident) => {\n        #[test]\n        \
         #[ignore = \"slow lane: generated\"]\n        fn $name() {}\n    };\n}\n\
         cases!(generated_case);\n",
    );
    let error = scan(scratch.path()).unwrap_err();
    assert!(error.contains("tests/a.rs"), "{error}");
    assert!(error.contains("inside a macro"), "{error}");
}

/// A marked test in a module directory that another target pulls in with `#[path]` (`ess-synth`'s
/// `tests/http_binding/` is one, compiled into `http_binding_rust`) has no target name `--test`
/// accepts, so the scanner refuses the marker by file and line instead of listing the directory.
#[test]
fn a_marker_in_a_module_directory_without_a_main_is_refused_by_file_and_line() {
    let scratch = Scratch::new("slow-lane-module-dir");
    demo(&scratch);
    write(
        scratch.path(),
        "crates/area/demo/tests/served.rs",
        "#[path = \"shared/body.rs\"]\nmod body;\n",
    );
    write(
        scratch.path(),
        "crates/area/demo/tests/shared/body.rs",
        "#[test]\n#[ignore = \"slow lane: drives a server\"]\nfn drives() {}\n",
    );
    let error = scan(scratch.path()).unwrap_err();
    assert!(
        error.contains("tests/shared/body.rs:2:"),
        "file and line: {error}"
    );
    assert!(
        error.contains("`tests/<name>.rs` or `tests/<name>/main.rs`"),
        "{error}"
    );
}

/// The text of one top-level job of a workflow, from its key to the next job's.
fn job<'w>(workflow: &'w str, name: &str) -> &'w str {
    let head = format!("\n  {name}:\n");
    let start = workflow
        .find(&head)
        .unwrap_or_else(|| panic!("no job `{name}`"));
    let rest = &workflow[start + head.len()..];
    let end = rest
        .match_indices("\n  ")
        .find(|(at, _)| {
            rest[at + 3..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase())
        })
        .map_or(rest.len(), |(at, _)| at);
    &rest[..end]
}

/// Adversary: `AGENTS.md` says a run that skipped the slow lane never counts for a release, and
/// `CHANGELOG.md` says a run that skipped it never stands in for a release. A patch tag publishes
/// on `Hotfix tests`, which runs `task test-packages` and so skips every marked test, the
/// repository-wide synthesis byte table among them: the job runs `task test-slow` too.
#[test]
fn adversary_a_hotfix_release_runs_the_slow_lane_it_is_said_to_need() {
    let root = workspace();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    assert!(
        agents.contains("so a run that skipped it\nnever counts for a release"),
        "the claim this case holds the workflow to"
    );
    let hotfix = job(&release, "hotfix-tests");
    assert!(
        hotfix.contains("task test-packages"),
        "the hotfix job is the one read"
    );
    assert!(
        hotfix.contains("test-slow"),
        "a patch tag is released on `Hotfix tests`, which runs no slow-lane test"
    );
}

/// The lines of the block opened by the line equal to `opening`, up to the next line indented no
/// deeper than it; empty when no line opens it.
fn block(text: &str, opening: &str) -> String {
    let indent = opening.len() - opening.trim_start().len();
    let mut lines = text.lines().skip_while(|line| *line != opening);
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut out = vec![first];
    for line in lines {
        let depth = line.len() - line.trim_start().len();
        if !line.trim().is_empty() && depth <= indent {
            break;
        }
        out.push(line);
    }
    out.join("\n")
}

/// Whether `text` names `word` as a whole name: `test-slow`, never the `test-slowest` it begins.
fn names_word(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        let part = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        !before.is_some_and(part) && !after.is_some_and(part)
    })
}

/// A patch release is gated by `release.yml`'s targeted `hotfix-tests` job instead of `ci.yml`.
/// That job runs `task test-packages`, a `cargo nextest run --profile ci` without
/// `--run-ignored`, so a test moved to the slow lane runs in no gate of a hotfix release: the
/// release of a change to synthesis compares the sources digest and never the bytes. A release
/// that skipped the slow lane must not pass; the targeted gate runs the lane of the crates it
/// tests, or `hotfix-scope.sh` sends a crate holding a slow test to the full gate.
#[test]
fn a_hotfix_release_runs_the_slow_tests_of_the_crates_it_tests() {
    let root = workspace();
    let release = fs::read_to_string(root.join(".github/workflows/release.yml")).unwrap();
    let job = block(&release, "  hotfix-tests:");
    assert!(
        job.contains("task test-packages"),
        "the hotfix-tests job runs `task test-packages`: {job}"
    );
    let taskfile = fs::read_to_string(root.join("Taskfile.yml")).unwrap();
    let packages = block(&taskfile, "  test-packages:");
    assert!(!packages.is_empty(), "Taskfile.yml holds `test-packages`");
    let scope = fs::read_to_string(root.join(".github/scripts/hotfix-scope.sh")).unwrap();
    let runs_the_lane = names_word(&job, "test-slow")
        || names_word(&packages, "test-slow")
        || packages.contains("--run-ignored");
    let sends_slow_crates_to_the_full_gate = scope.contains("slow-list");
    assert!(
        runs_the_lane || sends_slow_crates_to_the_full_gate,
        "a hotfix release runs no slow-lane test: `hotfix-tests` runs `task test-packages`, \
         which runs nextest without `--run-ignored`, and `hotfix-scope.sh` never consults \
         `cargo xtask slow-list`; the crates holding slow tests: {:?}",
        MARKED.iter().map(|(file, _)| *file).collect::<Vec<_>>()
    );
}

/// Adversary: `AGENTS.md` § Gate says the byte comparison of the two synthesis tables runs in the
/// slow lane, and the module comment of `synthesis_suite_bytes_table.rs` says the test step
/// synthesizes nothing there. The component table's test carries no marker.
#[test]
fn adversary_both_synthesis_tables_compare_in_the_slow_lane_as_the_gate_section_says() {
    let root = workspace();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(
        agents.contains("byte comparison itself (the two tables"),
        "the claim this case holds the tree to"
    );
    let marked: Vec<String> = scan(root)
        .unwrap()
        .into_iter()
        .filter(|m| m.target == "synthesis_suite_bytes_table")
        .map(|m| m.test)
        .collect();
    assert!(
        marked
            .iter()
            .any(|test| test == "every_repository_component_keeps_its_synthesized_bytes"),
        "the component table is compared in the test step, not the slow lane: {marked:?}"
    );
}

/// The lines of `ci.yml`'s slow-lane decision that define `skippable`, `needed`, `decide` and
/// `reaches_slow`, unindented.
fn slow_decision() -> String {
    let ci = fs::read_to_string(workspace().join(".github/workflows/ci.yml")).unwrap();
    let lines: Vec<&str> = ci.lines().collect();
    let from = lines
        .iter()
        .position(|line| line.trim_start().starts_with("skippable=("))
        .expect("the slow step's `skippable` list");
    let to = lines[from..]
        .iter()
        .position(|line| line.trim_start().starts_with("parents=$(git rev-list"))
        .expect("the slow step's history read")
        + from;
    lines[from..to]
        .iter()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Adversary: `cargo xtask slow-list`, which `task test-slow` runs to pick every slow test, is
/// dispatched by `crates/edge/ess-xtask/src/main.rs` and built from `crates/edge/ess-xtask/Cargo.toml`;
/// the deciding step lists `crates/edge/ess-xtask/*` as skippable and only `slow_lane.rs` as
/// needed, so a pull request changing what the lane selects skips the lane.
#[test]
fn adversary_a_change_to_what_slow_list_runs_on_reaches_the_slow_lane() {
    let script = slow_decision();
    for path in [
        "crates/edge/ess-xtask/src/main.rs",
        "crates/edge/ess-xtask/Cargo.toml",
    ] {
        let output = std::process::Command::new("bash")
            .arg("-c")
            .arg(format!("{script}\nreaches_slow \"$1\""))
            .arg("decide")
            .arg(path)
            .output()
            .expect("bash runs");
        assert!(output.status.success(), "{output:?}");
        assert!(
            !output.stdout.is_empty(),
            "a pull request changing only {path} skips the slow lane"
        );
    }
}
