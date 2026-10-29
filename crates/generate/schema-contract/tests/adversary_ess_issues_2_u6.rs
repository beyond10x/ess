//! Adversary pass on `--integer-format-widths` (beyond10x/ess#648): default bytes against files
//! the `ess 0.76.0` release binary wrote, the option's reach across every integer shape and the
//! Rust generator options, the Go decoder's range edges, and what the TypeScript report claims.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::{BundleOptions, Plan, Realization, RustOptions};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Integers with and without a width format, beside a format on a string and on a number, an
/// unknown format, nullable and optional members, array items, a typed map, an open object and a
/// referenced integer. Byte for byte the file given to `ess 0.76.0 generate schema import-bundle
/// --component Ints --dialect draft-2020-12`.
const INTS: &str = r##"{"openapi":"3.1.0","info":{"title":"demo.ints","version":"v1"},"paths":{},"components":{"schemas":{"Ints":{"type":"object","properties":{"wide":{"type":"integer","format":"int64"},"narrow":{"type":"integer","format":"int32"},"maybe":{"type":["integer","null"],"format":"int64"},"loose":{"type":["integer","null"],"format":"int32"},"bounded":{"type":"integer","format":"int32","minimum":0,"maximum":10},"text":{"type":"string","format":"int64"},"real":{"type":"number","format":"int32"},"unsigned":{"type":"integer","format":"uint32"},"list":{"type":"array","items":{"type":"integer","format":"int32"}},"ref":{"$ref":"#/components/schemas/Wide"},"map":{"type":"object","additionalProperties":{"type":"integer","format":"int64"}},"meta":{"type":"object","properties":{"n":{"type":"integer","format":"int32"}}}},"required":["wide","maybe","bounded","text","real","unsigned","list","ref","map","meta"],"additionalProperties":false},"Wide":{"type":"integer","format":"int64"}}}}
"##;

/// SHA-256 of `ints.bundle.json` as `ess 0.76.0 generate schema import-bundle` wrote it.
const BUNDLE_0_76: &str = "ecd0634d45459fa8e782716ff5ad824ef62100fbdc2e90d38270e8baf493729c";

/// SHA-256 of each file `ess 0.76.0 generate schema types-bundle --root Ints` wrote: `rust` with
/// `--package ints_types`, `rustopt` with that and `--rust-fold-null --rust-drop-unknown-members`,
/// `go` with `--package intstypes --module example.invalid/intstypes`, `typescript` bare.
const FILES_0_76: [(&str, &str); 11] = [
    (
        "rust/types.rs",
        "3d2275657aa87c944ddf45eb54e3b4574917d06e8e75375476d921e05016854d",
    ),
    (
        "rust/Cargo.toml",
        "73aba69e47cbd2d801f2ce3fbc276310578be80da60869ff2f0c0c649c15a0ed",
    ),
    (
        "rust/types-report.json",
        "ab9517fa7ccbcf3903a227932ddf53e792e12a981780fb53e90a6d8df1595026",
    ),
    (
        "rustopt/types.rs",
        "0278308f7637827892ed4345dd62a6e32d6769319fec11b3c1619c61770f2f74",
    ),
    (
        "rustopt/Cargo.toml",
        "73aba69e47cbd2d801f2ce3fbc276310578be80da60869ff2f0c0c649c15a0ed",
    ),
    (
        "rustopt/types-report.json",
        "73154a78c94d9b6c7301910a46cc5d0569f264c286a5d73a3bfdccb4270b0e85",
    ),
    (
        "go/types.go",
        "69307b5417f9013d77e2e64b70cd4d4ad0fba75a7d8006c34ccbaa3f1c8f41d0",
    ),
    (
        "go/go.mod",
        "9b6580709b481c3f591eb18fadaa1440f3a7d5502ca1a0670ddffa5efa955db1",
    ),
    (
        "go/types-report.json",
        "0e378f5f0332235d7aa65b7b654bdcff20c6216a54bcb2a8193eaf256e972a90",
    ),
    (
        "typescript/types.ts",
        "552c58163cdaa48c2398d12639b86c671261efe472045b8b29c51d595afa3149",
    ),
    (
        "typescript/types-report.json",
        "4e83aaddbb68233cbe58392a9c3e285e345933b71fcb580f185735018bd7cf1b",
    ),
];

const WIDTHS: BundleOptions = BundleOptions {
    integer_format_widths: true,
};

const PROPERTIES: &str = "/components/schemas/Ints/properties";

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        })
}

fn bundle() -> (Bundle, BTreeSet<String>) {
    let roots = BTreeSet::from(["Ints".to_owned()]);
    let bundle = import(INTS, &roots, Dialect::Draft202012).unwrap();
    (bundle, roots)
}

fn rust_options() -> RustOptions {
    RustOptions {
        fold_null: true,
        drop_unknown_members: true,
        ..RustOptions::default()
    }
}

/// The files `types-bundle` writes for each target, keyed as in `FILES_0_76`.
fn files(plan: &Plan) -> Vec<(String, String)> {
    let report = |realization: &Realization| {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&realization.report).unwrap()
        )
    };
    let mut files = Vec::new();
    for (target, realization, declarations, manifest) in [
        (
            "rust",
            plan.rust("ints_types").unwrap(),
            "types.rs",
            Some("Cargo.toml"),
        ),
        (
            "rustopt",
            plan.rust_with("ints_types", &rust_options()).unwrap(),
            "types.rs",
            Some("Cargo.toml"),
        ),
        (
            "go",
            plan.go("intstypes", "example.invalid/intstypes").unwrap(),
            "types.go",
            Some("go.mod"),
        ),
        ("typescript", plan.typescript(), "types.ts", None),
    ] {
        files.push((
            format!("{target}/{declarations}"),
            realization.declarations.clone(),
        ));
        if let Some(manifest) = manifest {
            files.push((
                format!("{target}/{manifest}"),
                realization.supporting[manifest].clone(),
            ));
        }
        files.push((format!("{target}/types-report.json"), report(&realization)));
    }
    files
}

fn field<'a>(declarations: &'a str, name: &str) -> &'a str {
    declarations
        .lines()
        .find(|line| line.trim_start().starts_with(name))
        .unwrap_or_else(|| panic!("no `{name}` in:\n{declarations}"))
        .trim()
}

fn annotation_detail(realization: &Realization, pointer: &str) -> String {
    let report = serde_json::to_value(&realization.report).unwrap();
    report["annotations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["pointer"] == pointer)
        .unwrap_or_else(|| panic!("no annotation at {pointer}: {report}"))["detail"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn obligation_rules(realization: &Realization, pointer: &str) -> BTreeSet<String> {
    let report = serde_json::to_value(&realization.report).unwrap();
    report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["pointer"] == pointer)
        .map(|finding| finding["rule"].as_str().unwrap().to_owned())
        .collect()
}

/// Without the option every file is the one the 0.76.0 binary wrote, for each target and for
/// the Rust generator options beside it, across every integer shape in `INTS`.
#[test]
fn without_the_option_every_target_writes_the_0_76_bytes_for_every_integer_shape() {
    let (bundle, roots) = bundle();
    assert_eq!(sha256(&bundle.to_json().unwrap()), BUNDLE_0_76);
    let read = Bundle::read(&bundle.to_json().unwrap()).unwrap();
    for (label, plan) in [
        ("from_bundle", Plan::from_bundle(&bundle, &roots).unwrap()),
        (
            "from_bundle_with(default)",
            Plan::from_bundle_with(&bundle, &roots, &BundleOptions::default()).unwrap(),
        ),
        ("read bundle", Plan::from_bundle(&read, &roots).unwrap()),
    ] {
        let actual = files(&plan)
            .into_iter()
            .map(|(path, text)| (path, sha256(&text)))
            .collect::<Vec<_>>();
        let frozen = FILES_0_76
            .iter()
            .map(|(path, digest)| ((*path).to_owned(), (*digest).to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(actual, frozen, "{label}");
    }
}

/// With the option exactly the `int32`/`int64` integers change width, wherever they sit; a
/// format on a string or a number, and an unknown format, keep their carrier and their
/// annotation. The Rust generator options do not undo the width.
#[test]
fn the_option_reaches_every_integer_position_and_nothing_else() {
    let (bundle, roots) = bundle();
    let plan = Plan::from_bundle_with(&bundle, &roots, &WIDTHS).unwrap();
    for realization in [
        plan.rust("ints_types").unwrap(),
        plan.rust_with("ints_types", &rust_options()).unwrap(),
    ] {
        let rust = &realization.declarations;
        for (name, expected) in [
            ("pub wide:", "pub wide: i64,"),
            ("pub narrow:", "pub narrow: EssPresence<i32>,"),
            ("pub maybe:", "pub maybe: ::std::option::Option<i64>,"),
            ("pub bounded:", "pub bounded: i32,"),
            ("pub text:", "pub text: ::std::string::String,"),
            ("pub real:", "pub real: ::serde_json::Number,"),
            ("pub unsigned:", "pub unsigned: ::serde_json::Number,"),
            ("pub list:", "pub list: ::std::vec::Vec<i32>,"),
            ("pub n:", "pub n: EssPresence<i32>,"),
        ] {
            assert_eq!(field(rust, name), expected, "{rust}");
        }
        assert!(
            field(rust, "pub loose:").contains("i32"),
            "{}",
            field(rust, "pub loose:")
        );
        assert!(
            !rust.contains("pub struct Wide(pub ::serde_json::Number)"),
            "{rust}"
        );
    }
    let rust = plan.rust("ints_types").unwrap();
    for keyword_site in ["text", "real", "unsigned"] {
        assert_eq!(
            annotation_detail(&rust, &format!("{PROPERTIES}/{keyword_site}/format")),
            "retained in the source bundle; no decoder behavior inferred",
            "{keyword_site}"
        );
    }
    let go = plan
        .go("intstypes", "example.invalid/intstypes")
        .unwrap()
        .declarations;
    for (name, expected) in [
        ("Wide ", "int64"),
        ("Narrow ", "EssPresence[int32]"),
        ("Maybe ", "EssNullable[int64]"),
        ("Loose ", "EssPresence[EssNullable[int32]]"),
        ("Bounded ", "int32"),
        ("Real ", "EssNumber"),
        ("Unsigned ", "EssNumber"),
    ] {
        assert!(field(&go, name).ends_with(expected), "{name}: {go}");
    }
}

/// The TypeScript target emits declarations and no decoder: `narrow` is a plain `number`, and a
/// value past 2^31 type-checks and is kept. Its report must not say that a decoder asserts the
/// `int32` range there, or must keep an obligation that names the range as the consumer's.
#[test]
fn the_typescript_report_does_not_claim_a_decoder_asserts_the_format_range() {
    let (bundle, roots) = bundle();
    let typescript = Plan::from_bundle_with(&bundle, &roots, &WIDTHS)
        .unwrap()
        .typescript();
    assert!(
        field(&typescript.declarations, "\"narrow\"").contains("number"),
        "{}",
        typescript.declarations
    );
    assert!(
        !typescript.declarations.contains("function"),
        "{}",
        typescript.declarations
    );
    let pointer = format!("{PROPERTIES}/narrow");
    let detail = annotation_detail(&typescript, &format!("{pointer}/format"));
    let rules = obligation_rules(&typescript, &pointer);
    assert!(
        !detail.contains("decoder asserts") || rules.iter().any(|rule| rule.contains("range")),
        "the TypeScript report says {detail:?} at {pointer}/format, and its obligations there \
         are {rules:?}: no TypeScript decoder exists, and no obligation leaves the int32 range \
         to the consumer"
    );
}

/// The Go decoder accepts each width's two edges and refuses one past each, wherever the
/// integer sits: a required, optional, nullable, array, map and referenced member.
#[test]
fn the_go_decoder_holds_both_edges_of_each_width_at_every_position() {
    use std::process::Command;
    let (bundle, roots) = bundle();
    let realization = Plan::from_bundle_with(&bundle, &roots, &WIDTHS)
        .unwrap()
        .go("intstypes", "example.invalid/intstypes")
        .unwrap();
    if Command::new("go").arg("version").output().is_err() {
        println!("skipped compiling: no `go` on PATH");
        return;
    }
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-ess-issues-2-u6-go-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("go.mod"), &realization.supporting["go.mod"]).unwrap();
    std::fs::write(root.join("types.go"), &realization.declarations).unwrap();
    std::fs::write(root.join("edges_test.go"), GO_EDGES).unwrap();
    let output = Command::new("go")
        .args(["test", "."])
        .current_dir(&root)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const GO_EDGES: &str = r#"package intstypes

import (
	"encoding/json"
	"testing"
)

const base = `{"wide":1,"maybe":null,"bounded":1,"text":"x","real":1,"unsigned":1,"list":[],"ref":1,"map":{},"meta":{}}`


func set(member, value string) string {
	var object map[string]json.RawMessage
	if err := json.Unmarshal([]byte(base), &object); err != nil {
		panic(err)
	}
	object[member] = json.RawMessage(value)
	out, err := json.Marshal(object)
	if err != nil {
		panic(err)
	}
	return string(out)
}

func TestEdges(t *testing.T) {
	accepted := []string{
		set("wide", "-9223372036854775808"),
		set("wide", "9223372036854775807"),
		set("narrow", "-2147483648"),
		set("narrow", "2147483647"),
		set("maybe", "-9223372036854775808"),
		set("loose", "null"),
		set("loose", "-2147483648"),
		set("list", "[-2147483648,2147483647]"),
		set("map", `{"k":-9223372036854775808}`),
		set("meta", `{"n":2147483647}`),
		set("ref", "9223372036854775807"),
	}
	for _, text := range accepted {
		var value Ints
		if err := json.Unmarshal([]byte(text), &value); err != nil {
			t.Fatalf("refused %s: %v", text, err)
		}
	}
	refused := []string{
		set("wide", "-9223372036854775809"),
		set("narrow", "-2147483649"),
		set("narrow", "2147483648"),
		set("maybe", "9223372036854775808"),
		set("loose", "2147483648"),
		set("bounded", "2147483648"),
		set("list", "[2147483648]"),
		set("map", `{"k":9223372036854775808}`),
		set("meta", `{"n":-2147483649}`),
		set("ref", "-9223372036854775809"),
	}
	for _, text := range refused {
		var value Ints
		if err := json.Unmarshal([]byte(text), &value); err == nil {
			t.Fatalf("accepted %s as %+v", text, value)
		}
	}
}
"#;

/// The TypeScript `int64` field past 2^53 keeps its precision obligation with the option.
#[test]
fn a_typescript_int64_keeps_its_precision_obligation_everywhere() {
    let (bundle, roots) = bundle();
    let typescript = Plan::from_bundle_with(&bundle, &roots, &WIDTHS)
        .unwrap()
        .typescript();
    let report: Value = serde_json::to_value(&typescript.report).unwrap();
    for pointer in [
        format!("{PROPERTIES}/wide"),
        format!("{PROPERTIES}/maybe"),
        "/components/schemas/Wide".to_owned(),
    ] {
        assert!(
            obligation_rules(&typescript, &pointer).contains("json_number_precision"),
            "{pointer}: {report}"
        );
    }
}
