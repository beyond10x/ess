//! Adversary pass 1 on unit W3-2 (beyond10x/ess#450): the types-only accessors of typed enum
//! attributes, held to the values the specification declares and to the compilers that read them.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;

const SOURCE: &str = r"format: ess/23
system: probe
version: v1
domains: [probe.rules]
domain: probe.rules
types:
  - name: probe.rules.Family
    kind: enum
    variants:
      - {name: Numeric, wire: numeric}
      - {name: Textual, wire: textual}
  - name: probe.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
      - {name: family, type: probe.rules.Family}
      - {name: weight, type: Optional<Decimal>}
    variants:
      - {name: Contains, attributes: {takes_number: false, family: Textual}}
      - {name: GreaterThan, wire: gt, attributes: {takes_number: true, family: Numeric, weight: 0.5}}
  - name: probe.rules.Solo
    kind: enum
    attributes:
      - {name: rank, type: Integer}
    variants:
      - {name: Only, attributes: {rank: 7}}
";

fn plan(names: &[&str]) -> Plan {
    Plan::from_model(&model::selection(SOURCE, names)).expect("plan")
}

/// An enum-typed attribute's value is a value of that enum, and an enum's value on the wire — the
/// spelling its own generated type (de)serializes and its JSON Schema `enum` lists — is its `wire:`
/// spelling. The accessor answers `numeric` for `gt`, not the authored variant name `Numeric`,
/// which no generated `Family` accepts.
#[test]
fn adv_w3_2_enum_typed_attribute_accessor_answers_the_wire_spelling() {
    let names = ["probe.rules.Operator", "probe.rules.Family"];
    let rust = plan(&names).rust("probe-rules").expect("rust").declarations;
    assert!(
        rust.contains("#[serde(rename = \"numeric\")]"),
        "Family's own type reads `numeric`:\n{rust}"
    );
    assert!(
        rust.contains("Self::V1 => \"numeric\","),
        "Operator::family() for `gt` answers Family's wire spelling:\n{rust}"
    );
    let typescript = plan(&names).typescript().declarations;
    assert!(
        typescript.contains("case \"gt\":\n      return \"numeric\";"),
        "probeRulesOperatorFamily(\"gt\") answers Family's wire spelling:\n{typescript}"
    );
}

/// A one-variant enum is still an enum: every output gives it its accessor.
#[test]
fn adv_w3_2_single_variant_enum_gets_its_accessor() {
    let names = ["probe.rules.Solo"];
    let rust = plan(&names).rust("probe-rules").expect("rust").declarations;
    assert!(
        rust.contains("pub fn rank(&self) -> i64"),
        "Rust accessor for a one-variant enum:\n{rust}"
    );
    let go = plan(&names)
        .go("rules", "example.invalid/rules")
        .expect("go")
        .declarations;
    assert!(
        go.contains("Rank() int64"),
        "Go accessor for a one-variant enum:\n{go}"
    );
    let typescript = plan(&names).typescript().declarations;
    assert!(
        typescript.contains("export function probeRulesSoloRank(value: ProbeRulesSolo): number"),
        "TypeScript accessor for a one-variant enum:\n{typescript}"
    );
}

/// The TypeScript accessors type-check under `--strict` and answer each declared value.
#[test]
fn adv_w3_2_typescript_accessors_compile_and_answer() {
    let tsc = std::env::var_os("ESS_TYPES_NODE")
        .map(|modules| std::path::PathBuf::from(modules).join(".bin/tsc"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("tsc"));
    if std::process::Command::new(&tsc)
        .arg("--version")
        .output()
        .is_err()
    {
        println!("skipped: no tsc");
        return;
    }
    let declarations = plan(&["probe.rules.Operator", "probe.rules.Solo"])
        .typescript()
        .declarations;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adv-w3-2-ts-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("scratch");
    std::fs::write(
        root.join("probe.ts"),
        format!(
            "{declarations}\n
const gt: ProbeRulesOperator = \"gt\";
const contains: ProbeRulesOperator = \"Contains\";
const checks: boolean[] = [
  probeRulesOperatorTakesNumber(gt) === true,
  probeRulesOperatorTakesNumber(contains) === false,
  probeRulesOperatorWeight(gt) === \"0.5\",
  probeRulesOperatorWeight(contains) === undefined,
  probeRulesSoloRank(\"Only\") === 7,
];
if (checks.some((ok) => !ok)) {{ throw new Error(JSON.stringify(checks)); }}
console.log(\"accessors ok\");
"
        ),
    )
    .expect("source");
    let compiled = std::process::Command::new(&tsc)
        .args([
            "--strict", "--target", "es2022", "--module", "commonjs", "--outDir",
        ])
        .arg(root.join("out"))
        .arg(root.join("probe.ts"))
        .output()
        .expect("tsc runs");
    assert!(
        compiled.status.success(),
        "tsc refused the accessors:\n{}\n{}\n{declarations}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let ran = std::process::Command::new("node")
        .arg(root.join("out/probe.js"))
        .output()
        .expect("node runs");
    assert!(
        ran.status.success() && String::from_utf8_lossy(&ran.stdout).contains("accessors ok"),
        "{}\n{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
}
