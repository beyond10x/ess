//! Adversary, w092-react: the host-slot registry against story
//! `the-react-project-registers-each-declared-host-slot`.
//!
//! 1. A `host` name is a node name (`is_segment`: ASCII alphanumerics and `_.-`), so the loader
//!    accepts `route-map` and `vendor.map`. The generated `src/hosts.ts` must still be a valid
//!    registry with one entry per declared name, and a project with every slot registered must
//!    type-check.
//! 2. A slot that reads a view it did not declare must fail `tsc`. `DataAdapter.read` is a method,
//!    so a host component that types its own `adapter` prop from `DataAdapter` is compared
//!    bivariantly and can be registered for a slot that declares one view, then read another.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w092-react-1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn generate(name: &str, text: &str) -> (PathBuf, ess_ui_react::GeneratedFiles) {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let out = scratch(name);
    let files = ess_ui_react::generate(&document, &root(), &out)
        .unwrap_or_else(|error| panic!("a document with host slots generates: {error}"));
    (out, files)
}

fn type_check(project: &Path, components: &str) -> (bool, String) {
    std::fs::write(project.join("src/host-components.tsx"), components)
        .expect("the host's registry is written");
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

fn document(section_host: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - {{name: summary, component: record, fields: [title]}}
      - {{name: map, host: {section_host}, reason: a map widget, stands_in_for: chart, reads: t.Routes}}
"
    )
}

#[test]
fn a_hyphenated_host_name_gives_a_registry_that_type_checks_when_registered() {
    let (project, files) = generate("hyphen", &document("route-map"));
    let hosts = &files.files["src/hosts.ts"];
    let (ok, output) = type_check(
        &project,
        r#"import type { HostComponents } from "./hosts";

const RouteMap: HostComponents["route-map"] = ({ adapter }) => {
  void adapter.read({ view: "t.Routes", params: {} });
  return null;
};
export const hosts = { "route-map": RouteMap };
"#,
    );
    assert!(
        ok,
        "`route-map` is an accepted host name; its slot registered, the project type-checks:\n\
         {output}\n--- src/hosts.ts ---\n{hosts}"
    );
}

#[test]
fn a_dotted_host_name_gives_a_registry_that_type_checks_when_registered() {
    let (project, files) = generate("dotted", &document("vendor.map"));
    let hosts = &files.files["src/hosts.ts"];
    let (ok, output) = type_check(
        &project,
        r#"import type { HostComponents } from "./hosts";

const VendorMap: HostComponents["vendor.map"] = ({ adapter }) => {
  void adapter.read({ view: "t.Routes", params: {} });
  return null;
};
export const hosts = { "vendor.map": VendorMap };
"#,
    );
    assert!(
        ok,
        "`vendor.map` is an accepted host name; its slot registered, the project type-checks:\n\
         {output}\n--- src/hosts.ts ---\n{hosts}"
    );
}

#[test]
fn a_component_typed_from_data_adapter_reading_an_undeclared_view_fails_the_type_check() {
    let (project, _) = generate("bivariant-read", &document("RouteMap"));
    let (ok, output) = type_check(
        &project,
        r#"import type { DataAdapter } from "./runtime/data";

// The host's own widget, typed against the runtime's adapter rather than its registry entry.
const RouteMap = ({ adapter }: { adapter: Pick<DataAdapter, "read">; path: string }) => {
  void adapter.read({ view: "t.Secrets", params: {} });
  return null;
};
export const hosts = { RouteMap };
"#,
    );
    assert!(
        !ok,
        "`RouteMap` declares only `t.Routes`; a component reading `t.Secrets` must not type-check \
         as its registration:\n{output}"
    );
}
