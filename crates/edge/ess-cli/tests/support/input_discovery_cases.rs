//! The accepted A1–A24 directory configuration matrix, through actual CLI adapters.
use ess_conformance::AdmittedSuite;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}
const BILLING: [&str; 5] = [
    "system.yaml",
    "components.yaml",
    "domains/invoice.yaml",
    "domains/email.yaml",
    "topology.yaml",
];
const TINY: &str = "format: ess/1\nsystem: tiny\nversion: v1\ndomains: [tiny.core]\ndomain: tiny.core\ntypes:\n  - {name: tiny.core.Id, kind: newtype, of: String}\n";
struct Tree {
    root: PathBuf,
}
impl Tree {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = repo().join(format!(
            "target/review-boundaries-17/authored-discovery/fixtures/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> PathBuf {
        let p = self.root.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
        p
    }
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.root.join(name);
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn model(&self, name: &str, reverse: bool) -> PathBuf {
        let mut paths = BILLING.to_vec();
        if reverse {
            paths.reverse();
        }
        for p in paths {
            self.write(
                &format!("{name}/model/{p}"),
                fs::read(repo().join("examples/billing").join(p)).unwrap(),
            );
        }
        self.root.join(name)
    }
    fn scenario(&self, name: &str, id: &str) -> PathBuf {
        self.write(
            name,
            fs::read_to_string(
                repo()
                    .join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
            )
            .unwrap()
            .replace(
                "scenario: outstanding-invoices-rank-latest-first",
                &format!("scenario: {id}"),
            ),
        )
    }
    fn manifest(&self, name: &str, spec: &[&str], scenarios: &[&str]) {
        self.write(
            &if name.is_empty() {
                "ess-inputs.yaml".to_owned()
            } else {
                format!("{name}/ess-inputs.yaml")
            },
            serde_json::to_vec_pretty(
                &json!({"format":"ess-inputs/1","specification":spec,"scenarios":scenarios}),
            )
            .unwrap(),
        );
    }
    fn mixed(&self, name: &str, reverse: bool) -> PathBuf {
        let root = self.model(name, reverse);
        for (file, id) in if reverse {
            [
                ("authored/z.scenario", "last"),
                ("authored/a.yaml", "first"),
            ]
        } else {
            [
                ("authored/a.yaml", "first"),
                ("authored/z.scenario", "last"),
            ]
        } {
            self.scenario(&format!("{name}/{file}"), id);
        }
        let mut spec = BILLING.map(|p| format!("model/{p}"));
        if reverse {
            spec.reverse();
        }
        let mut scenarios = ["authored/a.yaml", "authored/z.scenario"];
        if reverse {
            scenarios.reverse();
        }
        self.manifest(
            name,
            &spec.iter().map(String::as_str).collect::<Vec<_>>(),
            &scenarios,
        );
        self.write(
            &format!("{name}/generated/openapi.yaml"),
            "openapi: 3.1.0\n",
        );
        self.write(
            &format!("{name}/generated/asyncapi.yaml"),
            "asyncapi: 3.0.0\n",
        );
        self.write(&format!("{name}/generated/bad.yaml"), "[:\n");
        self.write(
            &format!("{name}/generated/copy.yaml"),
            fs::read(repo().join("examples/billing/domains/invoice.yaml")).unwrap(),
        );
        root
    }
    fn run(&self, command: &mut Command) -> Output {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = self.dir(&format!(
            "commands/{}",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(dir.join("command.txt"), format!("{command:?}\n")).unwrap();
        let start = Instant::now();
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = command.spawn().unwrap();
        let pid = child.id();
        let output = child.wait_with_output().unwrap();
        let seconds = start.elapsed().as_secs_f64();
        fs::write(dir.join("stdout"), &output.stdout).unwrap();
        fs::write(dir.join("stderr"), &output.stderr).unwrap();
        fs::write(
            dir.join("direct.json"),
            serde_json::to_vec_pretty(
                &json!({"pid":pid,"exit":output.status.code(),"seconds":seconds}),
            )
            .unwrap(),
        )
        .unwrap();
        output
    }
    fn good(&self, command: &mut Command) -> Output {
        let out = self.run(command);
        assert!(out.status.success(), "{command:?}: {out:?}");
        out
    }
}
fn cli(args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
    c.current_dir(repo()).args(args);
    c
}
fn model(path: &Path) -> Command {
    let mut c = cli(&["specify", "compile", "--format", "json", "--path"]);
    c.arg(path);
    c
}
fn author(path: &Path, scenarios: Option<&Path>, version: &str) -> Command {
    let mut c = cli(&[
        "conform",
        "author",
        "--format",
        "json",
        "--suite-format",
        version,
        "--path",
    ]);
    c.arg(path);
    if let Some(s) = scenarios {
        c.arg("--scenarios").arg(s);
    }
    c
}
fn same(a: &Output, b: &Output) {
    assert_eq!(a.status, b.status);
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.stderr, b.stderr);
}
fn refused(out: &Output, words: &[&str]) {
    assert!(!out.status.success(), "{out:?}");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for word in words {
        assert!(text.contains(word), "expected {word:?}: {text}");
    }
}
fn names(out: &Output) -> Vec<String> {
    AdmittedSuite::from_json(std::str::from_utf8(&out.stdout).unwrap())
        .unwrap()
        .coverage()
        .unwrap()
        .authored_sources
        .keys()
        .map(|v| v.as_str().to_owned())
        .collect()
}

#[test]
fn a01_legacy_billing_headerless_fragments_and_explicit_combined_file() {
    let t = Tree::new();
    let old = t.good(&mut model(&repo().join("examples/billing")));
    let p = t.mixed("mixed", false);
    same(&old, &t.good(&mut model(&p)));
    let direct = t.write("tiny.other", TINY);
    let expected = t.good(&mut model(&direct));
    t.manifest("", &["tiny.other"], &[]);
    same(&expected, &t.good(&mut model(&t.root)));
    // Explicit file selection ignores a poisoned parent manifest.
    t.write("ess-inputs.yaml", "not a manifest");
    same(&expected, &t.good(&mut model(&direct)));
}
#[test]
fn a02_legacy_shallow_extensions_and_explicit_other_names() {
    let t = Tree::new();
    let p = t.dir("selected");
    let file = t.scenario("selected/a.yml", "one");
    t.write("selected/nested/bad.yaml", "[:");
    for version in ["4", "5"] {
        let expected = t.good(&mut author(
            &repo().join("examples/billing"),
            Some(&p),
            version,
        ));
        same(
            &expected,
            &t.good(&mut author(
                &repo().join("examples/billing"),
                Some(&file),
                version,
            )),
        );
    }
    for name in ["one.json", "extensionless", "one.scenario"] {
        let file = t.scenario(name, "one");
        for version in ["4", "5"] {
            assert_eq!(
                t.good(&mut author(
                    &repo().join("examples/billing"),
                    Some(&file),
                    version
                ))
                .status
                .code(),
                Some(0)
            );
        }
    }
}
#[test]
fn a03_omitted_role_never_reads_a_manifest_or_working_directory_default() {
    let t = Tree::new();
    let p = t.mixed("mixed", false);
    t.write("scenarios/bad.yaml", "[:");
    for version in ["4", "5"] {
        let expected = t.good(&mut author(&repo().join("examples/billing"), None, version));
        same(
            &expected,
            &t.good(author(&p, None, version).current_dir(&t.root)),
        );
    }
}
#[test]
fn a05_a06_a07_exact_mixed_inventory_ignores_unlisted_changes_and_relocation() {
    let t = Tree::new();
    let a = t.mixed("first", false);
    let b = t.mixed("relocated", true);
    let expected = t.good(&mut author(&a, Some(&a), "5"));
    assert_eq!(names(&expected), ["authored/a.yaml", "authored/z.scenario"]);
    same(&expected, &t.good(&mut author(&b, Some(&b), "5")));
    same(&t.good(&mut model(&a)), &t.good(&mut model(&b)));
    for (name, contents) in [
        ("generated/openapi.yaml", "[unterminated"),
        ("generated/new.yml", TINY),
        ("output/model.yaml", "type: ess-scenario/987"),
    ] {
        t.write(&format!("first/{name}"), contents);
        same(&expected, &t.good(&mut author(&a, Some(&a), "5")));
    }
    fs::remove_file(a.join("generated/bad.yaml")).unwrap();
    same(&expected, &t.good(&mut author(&a, Some(&a), "5")));
    // An ancestor and a nested manifest never activate for a different explicit root.
    let legacy = t.model("legacy", false).join("model");
    t.write("legacy/ess-inputs.yaml", "poisoned");
    t.write("legacy/model/sub/ess-inputs.yaml", "poisoned");
    refused(&t.run(&mut model(&legacy)), &["ess-inputs.yaml"]);
}
#[test]
fn a08_selected_headers_have_no_filename_or_extension_requirement() {
    let t = Tree::new();
    let direct = t.write("renamed/header.input", TINY);
    let expected = t.good(&mut model(&direct));
    t.manifest("", &["renamed/header.input"], &[]);
    same(&expected, &t.good(&mut model(&t.root)));
}
#[test]
fn a09_a12_selected_document_errors_are_not_filtered() {
    let t = Tree::new();
    for (i, text) in [
        "types: []\n",
        "format: ess/999\nsystem: tiny\nversion: v1\ndomains: []\n",
        "openapi: 3.1.0\n",
        "asyncapi: 3.0.0\n",
        "[unterminated",
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("invalid-{i}");
        t.write(&format!("{name}/selected"), text);
        t.manifest(&name, &["selected"], &[]);
        let o = t.run(&mut model(&t.root.join(name)));
        refused(&o, &[]);
        assert!(!String::from_utf8_lossy(&o.stderr).contains("must contain `system.yaml`"));
    }
    t.write("conflict/a", TINY);
    t.write("conflict/b", TINY.replace("system: tiny", "system: other"));
    t.manifest("conflict", &["a", "b"], &[]);
    refused(&t.run(&mut model(&t.root.join("conflict"))), &[]);
    for (i, text) in [
        "type: ess-scenario/999\nscenario: nope\n",
        "openapi: 3.1.0\n",
        "[unterminated",
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("scenario-{i}");
        t.write(&format!("{name}/selected"), text);
        t.manifest(&name, &[], &["selected"]);
        for version in ["4", "5"] {
            refused(
                &t.run(&mut author(
                    &repo().join("examples/billing"),
                    Some(&t.root.join(&name)),
                    version,
                )),
                &[],
            );
        }
    }
}
fn invalid_manifests() -> Vec<Value> {
    let base = json!({"format":"ess-inputs/1","specification":["model/system.yaml"],"scenarios":["authored/a.yaml"]});
    let mut cases = Vec::new();
    for field in ["format", "specification", "scenarios"] {
        let mut v = base.clone();
        v.as_object_mut().unwrap().remove(field);
        cases.push(v);
        for wrong in [
            Value::Null,
            json!(42),
            json!(true),
            json!({}),
            json!("wrong"),
            json!([42]),
            json!([true]),
        ] {
            let mut v = base.clone();
            v[field] = wrong;
            cases.push(v);
        }
    }
    let mut unknown = base;
    unknown["extra"] = json!([]);
    cases.push(unknown);
    cases
}
fn no_effects(t: &Tree, path: &Path, word: &str) {
    for version in ["4", "5"] {
        for route in ["author", "synthesize", "go", "web", "run"] {
            for output in ["absent", "existing", "omitted"] {
                let out = t
                    .root
                    .join(format!("destination-{version}-{route}-{output}"));
                let tree = route == "web" || route == "go";
                if output == "existing" {
                    if tree {
                        fs::create_dir_all(&out).unwrap();
                        fs::write(out.join("sentinel"), b"owned").unwrap();
                    } else {
                        fs::write(&out, b"owned").unwrap();
                    }
                }
                let mut c = cli(&["conform", if route == "go" { "synthesize" } else { route }]);
                if route == "go" {
                    c.args(["--target", "go"]);
                }
                c.arg("--path")
                    .arg(repo().join("examples/billing"))
                    .arg("--scenarios")
                    .arg(path)
                    .args(["--suite-format", version, "--format", "json"]);
                if route == "run" {
                    c.args([
                        "--target",
                        "billing",
                        "--report-format",
                        if version == "5" { "2" } else { "1" },
                    ]);
                }
                if output != "omitted" {
                    c.arg(if route == "run" {
                        "--report-out"
                    } else {
                        "--out"
                    })
                    .arg(&out);
                }
                let o = t.run(&mut c);
                refused(&o, &[word]);
                assert!(o.stdout.is_empty(), "{o:?}");
                if output == "existing" {
                    if tree {
                        assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
                        assert_eq!(fs::read(out.join("sentinel")).unwrap(), b"owned");
                    } else {
                        assert_eq!(fs::read(&out).unwrap(), b"owned");
                    }
                } else {
                    assert!(!out.exists());
                }
            }
        }
    }
}
#[test]
fn a10_closed_shape_and_duplicate_yaml_keys_refuse_without_legacy_fallback() {
    let t = Tree::new();
    let p = t.mixed("mixed", false);
    for v in invalid_manifests() {
        t.write("mixed/ess-inputs.yaml", serde_json::to_vec(&v).unwrap());
        for mut c in [
            model(&p),
            author(&repo().join("examples/billing"), Some(&p), "4"),
            author(&repo().join("examples/billing"), Some(&p), "5"),
        ] {
            refused(&t.run(&mut c), &["ess-inputs.yaml"]);
        }
    }
    for text in [
        "format: ess-inputs/1\nformat: ess-inputs/1\nspecification: []\nscenarios: []\n",
        "format: ess-inputs/1\nspecification: []\nscenarios: []\n---\n{}\n",
        "format: ess-inputs/2\nspecification: []\nscenarios: []\n",
    ] {
        t.write("mixed/ess-inputs.yaml", text);
        no_effects(&t, &p, "ess-inputs.yaml");
    }
}
#[test]
fn a11_inactive_paths_are_only_structurally_checked() {
    let t = Tree::new();
    t.write("tiny", TINY);
    t.manifest("", &["tiny"], &["missing/scenario"]);
    t.good(&mut model(&t.root));
    t.manifest("", &["tiny"], &[]);
    t.good(&mut model(&t.root));
    t.scenario("one", "one");
    t.manifest("", &["missing/model"], &["one"]);
    for v in ["4", "5"] {
        t.good(&mut author(
            &repo().join("examples/billing"),
            Some(&t.root),
            v,
        ));
    }
    t.manifest("", &[], &["one"]);
    for v in ["4", "5"] {
        t.good(&mut author(
            &repo().join("examples/billing"),
            Some(&t.root),
            v,
        ));
    }
    t.manifest("", &[], &[]);
    refused(&t.run(&mut model(&t.root)), &["specification", "no"]);
    no_effects(&t, &t.root, "scenarios");
    t.manifest("", &["tiny"], &["../inactive"]);
    refused(&t.run(&mut model(&t.root)), &["scenarios", "../inactive"]);
}
#[test]
fn a13_duplicates_and_cross_role_spelling_refuse_before_reading() {
    let t = Tree::new();
    for (spec, scenarios) in [
        (vec!["missing", "missing"], vec![]),
        (vec![], vec!["missing", "missing"]),
        (vec!["missing"], vec!["missing"]),
    ] {
        t.manifest("", &spec, &scenarios);
        no_effects(&t, &t.root, "missing");
        refused(&t.run(&mut model(&t.root)), &["missing"]);
    }
}
#[test]
fn a14_copies_and_hardlinks_retain_distinct_semantic_duplicate_inputs() {
    let t = Tree::new();
    for hard in [false, true] {
        let name = if hard { "hard" } else { "copy" };
        let a = t.scenario(&format!("{name}/a"), "duplicate");
        let b = t.root.join(name).join("b");
        if hard {
            fs::hard_link(a, b).unwrap();
        } else {
            fs::copy(a, b).unwrap();
        }
        t.manifest(name, &[], &["b", "a"]);
        for v in ["4", "5"] {
            let o = t.run(&mut author(
                &repo().join("examples/billing"),
                Some(&t.root.join(name)),
                v,
            ));
            refused(&o, &["duplicate"]);
            if v == "5" {
                assert_eq!(names(&o), ["a", "b"]);
            }
        }
    }
}
#[test]
fn a15_path_grammar_is_exact_and_preserves_case_unicode_and_literal_globs() {
    let t = Tree::new();
    for bad in [
        "",
        "/absolute",
        ".",
        "..",
        "a/../b",
        "a/./b",
        "a//b",
        "a/",
        "a\\b",
        "a:b",
        "a\nb",
        "a\tb",
        "a\0b",
    ] {
        t.manifest("", &[bad], &[]);
        refused(
            &t.run(&mut model(&t.root)),
            &["specification", "ess-inputs.yaml"],
        );
        t.manifest("", &[], &[bad]);
        no_effects(&t, &t.root, "ess-inputs.yaml");
    }
    let good = [
        "Case/É.yaml",
        "Case/e\u{301}.yaml",
        "case/é.yaml",
        "literal*.scenario",
        " space ",
    ];
    for (i, name) in good.iter().enumerate() {
        t.scenario(name, &format!("case-{i}"));
    }
    t.manifest("", &[], &good);
    let o = t.good(&mut author(
        &repo().join("examples/billing"),
        Some(&t.root),
        "5",
    ));
    let mut expected = good.map(str::to_owned).to_vec();
    expected.sort();
    assert_eq!(names(&o), expected);
}
#[test]
#[cfg(unix)]
fn a16_selected_links_kinds_and_manifest_links_refuse_unlisted_ones_are_ignored() {
    use std::os::unix::fs::symlink;
    let t = Tree::new();
    let p = t.mixed("mixed", false);
    let linked = t.root.join("linked-root");
    symlink(&p, &linked).unwrap();
    for mut c in [
        model(&linked),
        author(&repo().join("examples/billing"), Some(&linked), "4"),
        author(&repo().join("examples/billing"), Some(&linked), "5"),
    ] {
        refused(&t.run(&mut c), &["symlink"]);
    }
    let root = t.dir("bad");
    symlink(p.join("authored/a.yaml"), root.join("file")).unwrap();
    symlink(p.join("authored"), root.join("directory")).unwrap();
    symlink("absent", root.join("broken")).unwrap();
    socket(&root.join("socket"));
    t.dir("bad/folder");
    for entry in [
        "file",
        "directory/a.yaml",
        "broken",
        "socket",
        "folder",
        "absent",
    ] {
        t.manifest("bad", &[], &[entry]);
        no_effects(&t, &root, entry);
    }
    t.manifest("bad", &[], &["file", "directory/a.yaml"]);
    no_effects(&t, &root, "symlink");
    // Do not inspect unlisted links, including an escaping link and a nonregular socket.
    symlink(&root, p.join("unlisted")).unwrap();
    t.good(&mut model(&p));
    t.good(&mut author(&p, Some(&p), "5"));
    for name in [
        "linked-config",
        "broken-config",
        "directory-config",
        "socket-config",
    ] {
        let r = t.dir(name);
        let m = r.join("ess-inputs.yaml");
        match name {
            "linked-config" => symlink(p.join("ess-inputs.yaml"), m).unwrap(),
            "broken-config" => symlink("absent", m).unwrap(),
            "directory-config" => fs::create_dir(m).unwrap(),
            _ => {
                socket(&m);
            }
        }
        refused(&t.run(&mut model(&r)), &["ess-inputs.yaml"]);
    }
}
#[test]
#[cfg(unix)]
fn a17_legacy_model_alias_visit_order_is_lexical_and_link_policy_is_preserved() {
    use std::os::unix::fs::symlink;
    let t = Tree::new();
    let mut outputs = Vec::new();
    for (name, reverse) in [("first", false), ("second", true)] {
        let p = t.dir(name);
        t.write(
            &format!("{name}/system.yaml"),
            "format: ess/1\nsystem: tiny\nversion: v1\ndomains: [tiny.core]\n",
        );
        t.write(
            &format!("{name}/z-real/domain.yaml"),
            "domain: tiny.core\ntypes:\n - {name: tiny.core.Id, kind: newtype, of: Missing}\n",
        );
        for alias in if reverse {
            ["b-alias", "a-alias"]
        } else {
            ["a-alias", "b-alias"]
        } {
            symlink("z-real", p.join(alias)).unwrap();
        }
        let o = t.run(&mut model(&p));
        refused(&o, &["a-alias/domain.yaml"]);
        outputs.push(o.stdout);
    }
    assert_eq!(outputs[0], outputs[1]);
    let source = t.write("combined", TINY);
    let linked = t.root.join("linked-file");
    symlink(&source, &linked).unwrap();
    same(&t.good(&mut model(&source)), &t.good(&mut model(&linked)));
    let legacy = t.model("legacy", false).join("model");
    let alias = t.root.join("legacy-root");
    symlink(&legacy, &alias).unwrap();
    same(&t.good(&mut model(&legacy)), &t.good(&mut model(&alias)));
}
#[test]
fn a18_lf_crlf_preserve_semantics_but_change_suite5_source_evidence() {
    let t = Tree::new();
    t.scenario("source", "one");
    t.manifest("", &[], &["source"]);
    let a = t.good(&mut author(
        &repo().join("examples/billing"),
        Some(&t.root),
        "5",
    ));
    let source = fs::read_to_string(t.root.join("source")).unwrap();
    t.write("source", source.replace('\n', "\r\n"));
    let b = t.good(&mut author(
        &repo().join("examples/billing"),
        Some(&t.root),
        "5",
    ));
    let a = AdmittedSuite::from_json(std::str::from_utf8(&a.stdout).unwrap()).unwrap();
    let b = AdmittedSuite::from_json(std::str::from_utf8(&b.stdout).unwrap()).unwrap();
    assert_eq!(a.suite().scenarios, b.suite().scenarios);
    assert_ne!(a.digest(), b.digest());
}
#[test]
fn a19_missing_and_unreadable_selected_bytes_precede_outputs_and_runner() {
    let t = Tree::new();
    for entry in ["missing", "invalid-utf8"] {
        t.write("invalid-utf8", [0xff, 0xfe]);
        t.manifest("", &[], &[entry]);
        no_effects(&t, &t.root, entry);
    }
}
#[test]
fn a20_committed_suite_and_input_bypass_discovery_but_keep_conflicts() {
    use ess_conformance::coverage::AdmittedInput;
    let t = Tree::new();
    let p = t.mixed("mixed", false);
    let generated = t.good(&mut author(&p, Some(&p), "5"));
    let suite = t.write("suite.json", &generated.stdout);
    let admitted =
        AdmittedSuite::from_json(std::str::from_utf8(&generated.stdout).unwrap()).unwrap();
    let input = t.write(
        "input.json",
        AdmittedInput::from_suite(admitted)
            .unwrap()
            .document()
            .to_canonical_json()
            .unwrap(),
    );
    t.write("mixed/ess-inputs.yaml", "poisoned");
    for (flag, file) in [("--suite", &suite), ("--suite-input", &input)] {
        let mut c = cli(&[
            "conform",
            "run",
            "--target",
            "billing",
            "--report-format",
            "2",
            "--format",
            "json",
            flag,
        ]);
        c.arg(file).arg("--path").arg(t.root.join("missing-model"));
        if flag == "--suite" {
            c.arg("--scenarios").arg(&p);
        }
        t.good(&mut c);
        if flag == "--suite-input" {
            let o = t.run(c.arg("--scenarios").arg(&p));
            assert_eq!(o.status.code(), Some(2));
        }
    }
}
#[test]
fn a21_authored_routes_aliases_and_presentations_use_the_same_manifest() {
    let t = Tree::new();
    let p = t.mixed("mixed", false);
    for version in ["4", "5"] {
        for operation in ["ir", "go", "author", "web", "run"] {
            for format in ["text", "json", "yaml"] {
                let mut outputs = Vec::new();
                for flat in [false, true] {
                    let mut c = cli(&[]);
                    if !flat {
                        c.arg("verify");
                    }
                    c.arg("conform");
                    match operation {
                        "ir" | "go" => {
                            c.args(["synthesize", "--target", operation]);
                        }
                        "run" => {
                            c.args([
                                "run",
                                "--target",
                                "billing",
                                "--report-format",
                                if version == "5" { "2" } else { "1" },
                            ]);
                        }
                        _ => {
                            c.arg(operation);
                        }
                    }
                    c.arg("--path").arg(&p).arg("--scenarios").arg(&p).args([
                        "--suite-format",
                        version,
                        "--format",
                        format,
                    ]);
                    outputs.push(t.good(&mut c));
                }
                same(&outputs[0], &outputs[1]);
            }
        }
    }
}
#[test]
fn a22_reserved_collision_is_actionable_and_frozen_old_reader_does_not_accept_manifest() {
    let t = Tree::new();
    let p = t.model("mixed", false);
    t.write(
        "mixed/system.yaml",
        fs::read(repo().join("examples/billing/system.yaml")).unwrap(),
    );
    t.write("mixed/ess-inputs.yaml", TINY);
    let o = t.run(&mut model(&p));
    refused(&o, &["ess-inputs.yaml", "rename"]);
    let p = t.mixed("valid", false);
    let source = fs::read_to_string(p.join("ess-inputs.yaml")).unwrap();
    assert!(ess_domain::spec::RawSpecFile::parse(&source).is_err());
    assert!(serde_yaml::from_str::<ess_conformance::authored::Document>(&source).is_err());
    // The maintained old document readers are unconditional. The frozen historical executable
    // is an explicit qualification input, never downloaded or substituted by the current binary.
    let Some(old) = std::env::var_os("ESS_DISCOVERY_OLD_CLI") else {
        eprintln!(
            "historical CLI qualification not requested; maintained document readers executed"
        );
        return;
    };
    for args in [
        vec!["specify", "compile"],
        vec!["conform", "author", "--suite-format", "4"],
        vec!["conform", "author", "--suite-format", "5"],
    ] {
        let mut c = Command::new(&old);
        c.args(&args).arg("--path").arg(&p);
        if args[0] == "conform" {
            c.arg("--scenarios").arg(&p);
        }
        refused(&t.run(&mut c), &[]);
    }
}
#[test]
fn a23_model_types_preserves_output_containment() {
    let t = Tree::new();
    t.write(
        "model/source",
        fs::read(repo().join("crates/generate/ess-gen/tests/fixtures/model-types.yaml")).unwrap(),
    );
    t.manifest("model", &["source"], &[]);
    let p = t.root.join("model");
    let out = p.join("generated");
    let mut c = cli(&["generate", "types", "--path"]);
    c.arg(&p)
        .args([
            "--root",
            "sample.data.Record",
            "--target",
            "typescript",
            "--out",
        ])
        .arg(&out);
    refused(&t.run(&mut c), &["must not replace or reside within"]);
    assert!(!out.exists());
}
#[test]
fn a24_named_model_projection_and_reader_agree_without_claiming_expression_gaps() {
    let t = Tree::new();
    let declaration = repo().join("docs/design/models/authored-discovery");
    t.good(cli(&["specify", "validate", "--path"]).arg(&declaration));
    t.good(&mut model(&declaration));
    let out = t.root.join("types");
    t.good(
        cli(&["generate", "types", "--path"])
            .arg(&declaration)
            .args([
                "--root",
                "discovery.inputs.InputManifest",
                "--target",
                "typescript",
                "--out",
            ])
            .arg(&out),
    );
    let schema: Value =
        serde_json::from_slice(&fs::read(out.join("source.schema.json")).unwrap()).unwrap();
    let defs = &schema["$defs"];
    let manifest = &defs["discovery.inputs.InputManifest"];
    assert_eq!(manifest["additionalProperties"], false);
    assert_eq!(
        manifest["required"],
        json!(["format", "specification", "scenarios"])
    );
    assert_eq!(
        defs["discovery.inputs.ManifestFormat"]["enum"],
        json!(["ess-inputs/1"])
    );
    for field in ["specification", "scenarios"] {
        assert_eq!(manifest["properties"][field]["type"], "array");
    }
    t.write("fixture/source", TINY);
    t.manifest("fixture", &["source"], &[]);
    t.good(&mut model(&t.root.join("fixture")));
    validate_manifest_instances(&t, &schema);
    // Empty inactive lists are permitted by both; the runtime additionally enforces activation,
    // complete path grammar, cross-list uniqueness and filesystem facts absent from this model.
    let binding =
        fs::read_to_string(repo().join("docs/design/review-authored-discovery.md")).unwrap();
    for gap in [
        "filesystem",
        "uniqueness",
        "role activation",
        "exact source bytes",
    ] {
        assert!(binding.to_lowercase().contains(gap));
    }
}

#[test]
fn a21_model_routes_preserve_selected_semantics_across_aliases_and_presentations() {
    let t = Tree::new();
    let selected = t.mixed("selected", false);
    let legacy = repo().join("examples/billing");
    for (area, verb, extra) in [
        ("specify", "validate", vec![]),
        ("specify", "compile", vec![]),
        ("specify", "inspect", vec!["billing.invoice.Invoice"]),
        ("specify", "graph", vec![]),
        ("generate", "generate", vec!["--kind", "schema"]),
        ("generate", "project", vec!["openapi"]),
        ("generate", "synthesize", vec!["--target", "go"]),
    ] {
        let formats: &[&str] = if verb == "graph" {
            &["mermaid", "dot", "json", "yaml"]
        } else {
            &["text", "json", "yaml"]
        };
        for &format in formats {
            for flat in [false, true] {
                let mut outputs = Vec::new();
                for path in [&legacy, &selected] {
                    let mut c = cli(&[]);
                    if !flat {
                        c.arg(area);
                    }
                    c.arg(verb)
                        .args(&extra)
                        .arg("--path")
                        .arg(path)
                        .args(["--format", format]);
                    outputs.push(t.good(&mut c));
                }
                same(&outputs[0], &outputs[1]);
            }
        }
    }
    for verb in ["diff", "impact"] {
        for flat in [false, true] {
            let mut outputs = Vec::new();
            for path in [&legacy, &selected] {
                let mut c = cli(&[]);
                if !flat {
                    c.arg("verify");
                }
                c.arg(verb)
                    .arg("--from")
                    .arg(path)
                    .arg("--to")
                    .arg(path)
                    .args(["--format", "json"]);
                outputs.push(t.good(&mut c));
            }
            same(&outputs[0], &outputs[1]);
        }
    }
    for verb in ["validate", "compile", "generate"] {
        for flat in [false, true] {
            let mut outputs = Vec::new();
            for path in [&legacy, &selected] {
                let mut c = cli(&[]);
                if !flat {
                    c.arg("specify");
                }
                c.args(["realization", verb, "--path"])
                    .arg(repo().join("examples/realizations/billing-local.yaml"))
                    .arg("--spec")
                    .arg(path)
                    .args(["--format", "json"]);
                if verb == "generate" {
                    c.arg("--out").arg(t.root.join("realization.md"));
                }
                outputs.push(t.good(&mut c));
            }
            same(&outputs[0], &outputs[1]);
        }
    }
    composition_routes(&t);
}

fn composition_routes(t: &Tree) {
    let sources = repo().join("crates/specify/ess-composition/tests/fixtures");
    let source = sources.join("two-components");
    let p = t.dir("composition");
    // Copy the existing four-file composition model, preserving headerless fragments.
    for name in [
        "system.yaml",
        "components.yaml",
        "domains/todo.yaml",
        "domains/usage.yaml",
    ] {
        t.write(
            &format!("composition/{name}"),
            fs::read(source.join(name)).unwrap(),
        );
    }
    t.manifest(
        "composition",
        &[
            "system.yaml",
            "components.yaml",
            "domains/todo.yaml",
            "domains/usage.yaml",
        ],
        &[],
    );
    for flat in [false, true] {
        let mut outputs = Vec::new();
        for path in [&source, &p] {
            let mut c = cli(&[]);
            if !flat {
                c.arg("specify");
            }
            c.args(["compose", "--path"])
                .arg(sources.join("compositions/workbench.yaml"))
                .args(["--format", "json"]);
            for key in ["todo", "usage"] {
                c.arg("--service").arg(format!("{key}={}", path.display()));
            }
            outputs.push(t.good(&mut c));
        }
        same(&outputs[0], &outputs[1]);
    }
}

#[cfg(unix)]
fn socket(path: &Path) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let short = std::env::temp_dir().join(format!(
        "discovery-socket-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::os::unix::net::UnixListener::bind(&short).unwrap());
    fs::rename(short, path).unwrap();
}

fn validate_manifest_instances(t: &Tree, schema: &Value) {
    // The schema registry uses an external `schema` selector. Wrap the instance instead of
    // adding a selector field to InputManifest or dropping any generated x-ess annotation.
    let mut wrapper = schema.clone();
    wrapper["$id"] = json!("urn:ess:discovery-manifest-test");
    wrapper["type"] = json!("object");
    wrapper["additionalProperties"] = json!(false);
    wrapper["required"] = json!(["schema", "manifest"]);
    wrapper["properties"] = json!({"schema":{"const":"urn:ess:discovery-manifest-test"},"manifest":{"$ref":"#/$defs/discovery.inputs.InputManifest"}});
    assert_eq!(wrapper["$defs"], schema["$defs"]);
    let registry = t.dir("validation-registry");
    t.write(
        "validation-registry/manifest.schema.json",
        serde_json::to_vec(&wrapper).unwrap(),
    );
    let validate = |value: Value| {
        let instance = t.write(
            "manifest-instance.json",
            serde_json::to_vec(
                &json!({"schema":"urn:ess:discovery-manifest-test","manifest":value}),
            )
            .unwrap(),
        );
        t.run(
            cli(&["schema", "validate", "--schemas"])
                .arg(&registry)
                .arg(instance),
        )
    };
    let value: Value =
        serde_json::from_slice(&fs::read(t.root.join("fixture/ess-inputs.yaml")).unwrap()).unwrap();
    let output = validate(value);
    assert!(output.status.success(), "{output:?}");
    // These shape violations are refused by both the actual projected model and reader.
    for bad in invalid_manifests() {
        refused(&validate(bad), &[]);
    }
    // The projected model deliberately cannot express complete relative-path admission.
    let gap = json!({"format":"ess-inputs/1","specification":["../source"],"scenarios":[]});
    let output = validate(gap.clone());
    assert!(output.status.success(), "{output:?}");
    t.write("fixture/ess-inputs.yaml", serde_json::to_vec(&gap).unwrap());
    refused(&t.run(&mut model(&t.root.join("fixture"))), &["../source"]);
}
