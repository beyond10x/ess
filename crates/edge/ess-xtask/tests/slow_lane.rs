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
        "crates/verify/ess-conformance/tests/adversary_w3_266_267_dump.rs",
        &["adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison"],
    ),
    (
        "crates/verify/ess-conformance/tests/enum_presence_guard.rs",
        &["no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere"],
    ),
    (
        "crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs",
        &[
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
