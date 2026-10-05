//! Adversary, pass 1, for unit E-U7: a byte-identity probe over models with no `.utf8_bytes`
//! operand. For each model below it writes the compiled IR, the synthesized suite and its refusals
//! to `<ADV_E_U7_PROBE_OUT>/<name>.{ir,suite,refusals}`. Compiled in the unit's tree and in an
//! export of its base `3b2565c3e`, the two directories differ exactly where a model's bytes moved.
//! Without the variable it only checks that every model compiles.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const LIFT: &str = r"format: ess/22
system: shop
version: v1
domain: shop.core
entities:
  - name: shop.core.Window
    identity: {name: window_id, type: Uuid}
    fields:
      - {name: label, type: String}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: opens_at, type: Timestamp}
      - {name: closes_at, type: Timestamp}
    invariants:
      - {INVARIANT}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: shop.core.Opened
    fields:
      - {name: window_id, type: Uuid}
commands:
  - name: shop.core.Open
    input:
      - {name: label, type: String}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: opens_at, type: Timestamp}
      - {name: closes_at, type: Timestamp}
    outcomes:
      - name: opened
        creates: shop.core.Window
        instance: window_id
        sets: {label: input.label, lower: input.lower, upper: input.upper, opens_at: input.opens_at, closes_at: input.closes_at}
        emits: [shop.core.Opened]
        payload:
          shop.core.Opened: {window_id: {generated: true}}
views:
  - name: shop.core.Windows
    source: shop.core.Window
    consistency: read_your_writes
    fields:
      - {name: window_id, type: Uuid}
      - {name: label, type: String}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: opens_at, type: Timestamp}
      - {name: closes_at, type: Timestamp}
";

const MEMBER: &str = r"format: {FORMAT}
system: shop
version: v1
domain: shop.core
types:
  - name: shop.core.Meta
    kind: struct
    fields:
      - {name: utf8_bytes, type: Integer}
entities:
  - name: shop.core.Item
    identity: {name: item_id, type: Uuid}
    fields:
      - {name: meta, type: shop.core.Meta}
    invariants:
      - meta.utf8_bytes <= 5
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: shop.core.Big, summary: Big.}
events:
  - name: shop.core.Added
    fields:
      - {name: item_id, type: Uuid}
commands:
  - name: shop.core.Add
    input:
      - {name: meta, type: shop.core.Meta}
    outcomes:
      - name: big
        when: meta.utf8_bytes > 3
        error: shop.core.Big
      - name: added
        creates: shop.core.Item
        instance: item_id
        sets: {meta: input.meta}
        emits: [shop.core.Added]
        payload:
          shop.core.Added: {item_id: {generated: true}}
views:
  - name: shop.core.Items
    source: shop.core.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: Uuid}
      - {name: meta, type: shop.core.Meta}
";

fn models() -> Vec<(&'static str, String)> {
    vec![
        (
            "lift-offset",
            LIFT.replace(
                "{INVARIANT}",
                r#"{all: ["label.count <= 5", "upper == lower + 5"]}"#,
            ),
        ),
        (
            "lift-instants",
            LIFT.replace(
                "{INVARIANT}",
                r#"{all: ["label.count <= 5", "closes_at > opens_at"]}"#,
            ),
        ),
        (
            "count-alone",
            LIFT.replace("{INVARIANT}", "label.count <= 5"),
        ),
        ("member-22", MEMBER.replace("{FORMAT}", "ess/22")),
        ("member-21", MEMBER.replace("{FORMAT}", "ess/21")),
    ]
}

#[test]
fn adv_e_u7_byte_identity_probe() {
    let out = std::env::var_os("ADV_E_U7_PROBE_OUT").map(std::path::PathBuf::from);
    if let Some(out) = &out {
        std::fs::create_dir_all(out).unwrap();
    }
    let mut compiled = 0;
    for (name, text) in models() {
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{name}: {errors}"));
        let ir =
            compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{name}: {error:?}"));
        compiled += 1;
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        // The coordinator's decision (F5, final review decision 1): a text's `.count` is asserted
        // on view rows only in an invariant that already needs suite /40 — an offset or a tagged
        // instant comparison here — which moves that suite from /34 to /40. A `.count` alone, and
        // a member named `utf8_bytes` in either format, select nothing and keep their bytes.
        let version = synthesis.suite.provenance.suite_version.to_string();
        let json = synthesis
            .suite
            .to_canonical_json()
            .unwrap_or_else(|error| format!("ERR {error}"));
        if name.starts_with("lift-") {
            assert_eq!(version, "ess-conformance/40", "{name}");
            assert!(
                json.contains("label.count"),
                "{name}: the lifted `.count` is asserted on view rows: {json}"
            );
        } else {
            assert!(
                !ess_conformance::expression_format::used_by(&synthesis.suite),
                "{name}"
            );
            assert_ne!(version, "ess-conformance/40", "{name}");
            assert!(!json.contains("label.count"), "{name}: {json}");
            assert!(
                !serde_json::to_string(&ir)
                    .unwrap()
                    .contains("{\"utf8_bytes\":\""),
                "{name}: no derived selector"
            );
        }
        if let Some(out) = &out {
            let write = |extension: &str, contents: String| {
                std::fs::write(out.join(format!("{name}.{extension}")), contents).unwrap();
            };
            write("ir", serde_json::to_string_pretty(&ir).unwrap());
            write(
                "suite",
                synthesis
                    .suite
                    .to_canonical_json()
                    .unwrap_or_else(|error| format!("ERR {error}")),
            );
            write("refusals", format!("{:#?}", synthesis.refusals));
        }
    }
    assert_eq!(compiled, 5);
}
