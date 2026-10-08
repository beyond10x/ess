//! `ess specify validate` refuses a wire name a generated path segment reads when it holds `/`
//! or is `.` or `..` (beyond10x/ess#493).
//!
//! The fit review's reproducer: the gatepass example with its `ExpectedVisits` view renamed on the
//! wire to `../.well-known/demo-configuration`, which `ess generate` used to publish as the path
//! `/visits/views/../.well-known/demo-configuration`. The refusal names the declaration and the
//! wire name, and `ess generate` writes no `OpenAPI` document for it.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const CLIMBING: &str = "../.well-known/demo-configuration";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// The gatepass example, copied under this test target's scratch space with `edit` applied to its
/// one domain file.
fn gatepass(name: &str, edit: impl Fn(&str) -> String) -> PathBuf {
    let from = repo().join("examples/gatepass");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "path-segment-wire-names/{name}-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("domains")).unwrap();
    for file in ["system.yaml", "components.yaml"] {
        fs::copy(from.join(file), root.join(file)).unwrap();
    }
    let domain = fs::read_to_string(from.join("domains/visit.yaml")).unwrap();
    let edited = edit(&domain);
    assert_ne!(edited, domain, "the edit must change the fixture");
    fs::write(root.join("domains/visit.yaml"), edited).unwrap();
    root
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn printed(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn a_view_wire_name_that_climbs_out_of_its_segment_is_refused_by_name() {
    let root = gatepass("view", |text| {
        text.replace(
            "      wire: expected\n",
            &format!("      wire: {CLIMBING}\n"),
        )
    });
    let output = ess(&["specify", "validate", "--path", root.to_str().unwrap()]);
    let text = printed(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("ESS-VIEW-012"), "{text}");
    assert!(text.contains("path_segment_wire_name"), "{text}");
    assert!(text.contains("gatepass.visit.ExpectedVisits"), "{text}");
    assert!(text.contains(&format!("{CLIMBING:?}")), "{text}");
}

#[test]
fn a_domain_and_a_command_wire_name_are_refused_in_the_same_run() {
    let root = gatepass("domain-and-command", |text| {
        text.replace("  wire: visits\n", "  wire: '..'\n")
            .replace("      wire: admit-visitor\n", "      wire: admit/visitor\n")
    });
    let output = ess(&["specify", "validate", "--path", root.to_str().unwrap()]);
    let text = printed(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("ESS-DOMAIN-012"), "{text}");
    assert!(text.contains("ESS-COMMAND-012"), "{text}");
    assert!(text.contains("\"..\""), "{text}");
    assert!(text.contains("gatepass.visit.AdmitVisitor"), "{text}");
    assert!(text.contains("\"admit/visitor\""), "{text}");
}

#[test]
fn generate_writes_no_openapi_document_for_a_refused_wire_name() {
    let root = gatepass("generate", |text| {
        text.replace(
            "      wire: expected\n",
            &format!("      wire: {CLIMBING}\n"),
        )
    });
    let out = root.join("generated");
    let output = ess(&[
        "generate",
        "--path",
        root.to_str().unwrap(),
        "--kind",
        "openapi",
        "--out",
        out.to_str().unwrap(),
    ]);
    let text = printed(&output);
    assert_ne!(output.status.code(), Some(0), "{text}");
    assert!(!out.join("openapi").exists(), "{text}");
}

#[test]
fn the_unedited_example_still_validates() {
    let root = repo().join("examples/gatepass");
    let output = ess(&["specify", "validate", "--path", root.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "{}", printed(&output));
}
