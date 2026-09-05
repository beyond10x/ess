//! Explicit compiler-backed projection checks, enabled by `typescript-typecheck`.

#[path = "support/model.rs"]
mod model;

#[test]
fn model_wire_mapping_and_explicit_nominal_limitations_typecheck() {
    let plan = model::plan();
    let result = plan.typescript();
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["input"]["kind"], "model");
    assert!(report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["rule"] == "typescript_nominal_identity"));
    let controls = r#"
const choice: SampleDataChoice = {value: "item", content: "id"};
const omitted: SampleDataChoice = {value: "omitted"};
// @ts-expect-error: the tag's payload is required for this variant
const missing: SampleDataChoice = {value: "item"};
// @ts-expect-error: optional does not mean nullable at field position
const nullPayload: SampleDataChoice = {value: "omitted", content: null};
// @ts-expect-error: no undeclared union tag is admitted
const invalid: SampleDataChoice = {value: "unknown", content: "id"};
"#;
    let output = typecheck(&[format!("{}\n{controls}", result.declarations)]);
    assert!(output.status.success(), "{output:?}");
}

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use schema_contract::typescript::project;
use serde_json::{json, Value};

fn compiler() -> &'static Path {
    static COMPILER: OnceLock<PathBuf> = OnceLock::new();
    COMPILER.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("ESS_TYPESCRIPT_COMPILER")
                .expect("the selected typescript-typecheck lane requires ESS_TYPESCRIPT_COMPILER"),
        );
        assert!(
            path.is_file(),
            "ESS_TYPESCRIPT_COMPILER must name an installed tsc.js: {}",
            path.display()
        );
        let version = Command::new("node")
            .arg(&path)
            .arg("--version")
            .output()
            .expect("run configured TypeScript compiler");
        assert!(
            version.status.success(),
            "compiler version query failed: {version:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&version.stdout).trim(),
            "Version 6.0.3",
            "the target lane uses the repository's pinned compiler"
        );
        path
    })
}

fn typecheck(sources: &[String]) -> Output {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let compiler = compiler();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "typescript-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).expect("worktree-local compiler fixtures");
    let mut files = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let filename = format!("case-{index}.ts");
        std::fs::write(root.join(&filename), source).expect("generated declaration");
        files.push(filename);
    }
    let config = root.join("tsconfig.json");
    std::fs::write(&config, serde_json::to_vec_pretty(&json!({
        "compilerOptions": {"strict": true, "exactOptionalPropertyTypes": true, "noEmit": true, "types": [], "target": "ES2022", "module": "ESNext"},
        "files": files
    })).expect("compiler configuration")).expect("write isolated compiler configuration");
    Command::new("node")
        .arg(compiler)
        .args(["--pretty", "false", "--project"])
        .arg(config)
        .output()
        .expect("typecheck generated declarations")
}

fn accepted(source: &Value, root: &str, output: &mut Vec<String>) {
    if let Ok(generated) = project(source, root) {
        output.push(generated);
    }
}

#[test]
fn qualified_bundle_types_preserve_presence_tuple_and_sibling_constraints() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::Plan;
    let source = json!({"components": {"schemas": {
        "Record": {"type": "object", "required": ["nullable", "value"], "additionalProperties": false,
            "properties": {"nullable": {"type": ["string", "null"]}, "optional": {"type": ["string", "null"]}, "value": true}},
        "Tuple": {"type": "array", "prefixItems": [{"type": "string"}, true, {"type": "boolean"}], "minItems": 3, "maxItems": 3},
        "OptionalTuple": {"type": "array", "prefixItems": [{"type": "string"}, {"type": "boolean"}], "minItems": 1, "items": {"type": "integer"}},
        "Base": {"type": ["string", "null"]},
        "Narrow": {"$ref": "#/components/schemas/Base", "type": "string", "enum": ["x", "y"], "const": "x"},
        "Open": {"type": "object", "properties": {"label": {"type": "string"}}, "additionalProperties": {"type": "integer"}},
        "Node": {"type": "object", "properties": {"children": {"type": "array", "items": {"$ref": "#/components/schemas/Node"}}}},
        "Union": {"oneOf": [{"type": "string", "enum": ["ready"]}, {"type": "object", "required": ["Other"], "properties": {"Other": {"type": "string"}}, "additionalProperties": false}]}
    }}});
    let roots = [
        "Record",
        "Tuple",
        "OptionalTuple",
        "Narrow",
        "Open",
        "Node",
        "Union",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let mut emitted = Plan::from_bundle(&bundle, &roots)
        .unwrap()
        .typescript()
        .declarations;
    emitted.push_str(
        r#"
const record: Record = {nullable: null, value: {arbitrary: [true, null]}};
const optional: Record = {nullable: "x", optional: null, value: 2};
const tuple: Tuple = ["entry", {nested: null}, true];
const shortTuple: OptionalTuple = ["entry"];
const longTuple: OptionalTuple = ["entry", true, 1, 2];
const narrow: Narrow = "x";
const open: Open = {label: "entry", count: 1};
const node: Node = {children: [{children: []}]};
const unionA: Union = "ready";
const unionB: Union = {Other: "custom"};
// @ts-expect-error required-nullable is still required
const missing: Record = {value: 2};
// @ts-expect-error absent does not mean present undefined
const undefinedField: Record = {nullable: null, value: 2, optional: undefined};
// @ts-expect-error unrestricted JSON excludes functions
const functionField: Record = {nullable: null, value: () => 1};
// @ts-expect-error exact tuple length
const tooShort: Tuple = ["entry", null];
// @ts-expect-error tuple position type
const wrongSlot: Tuple = ["entry", null, 1];
// @ts-expect-error reference siblings must not widen the constant
const tooWide: Narrow = "y";
// @ts-expect-error retain union alternatives
const wrongUnion: Union = "missing";
"#,
    );
    let checked = typecheck(&[emitted.clone()]);
    assert!(checked.status.success(), "{checked:?}");
    let control = typecheck(&[emitted.replace("// @ts-expect-error", "// negative control:")]);
    assert!(
        !control.status.success(),
        "negative controls must be rejected"
    );
}

// The pinned compiler's keyword vocabulary, plus ordinary, helper and contextual controls.
const ROOT_NAME_CORPUS: &[&str] = &[
    "Root",
    "$Root",
    "_Root",
    "Array",
    "any",
    "unknown",
    "never",
    "number",
    "bigint",
    "boolean",
    "string",
    "symbol",
    "void",
    "object",
    "undefined",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "while",
    "with",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "abstract",
    "accessor",
    "as",
    "asserts",
    "assert",
    "async",
    "await",
    "constructor",
    "declare",
    "from",
    "get",
    "global",
    "infer",
    "intrinsic",
    "is",
    "keyof",
    "module",
    "namespace",
    "of",
    "out",
    "override",
    "readonly",
    "require",
    "satisfies",
    "set",
    "type",
    "unique",
    "using",
    "defer",
    "__esModule",
    "eval",
    "arguments",
];

#[test]
fn accepted_binding_collision_and_keyword_corpus_typechecks() {
    let mut sources = Vec::new();
    let scalar = json!({"$id": "urn:example:bindings:1", "type": "string"});
    for root in ROOT_NAME_CORPUS {
        accepted(&scalar, root, &mut sources);
    }
    for key in ["item", "Item", "item-name", "item_name", "array", "Array"] {
        let source = json!({"$id": "urn:example:bindings:1", "$ref": format!("#/$defs/{key}"), "$defs": {key: {"type": "string"}}});
        for root in ["Root", "Item", "ItemName", "Array"] {
            accepted(&source, root, &mut sources);
        }
    }
    for root in ["Array", "Root"] {
        for definition in ["array", "item"] {
            let source = json!({"$id": "urn:example:bindings:1", "type": "array", "items": {"$ref": format!("#/$defs/{definition}")}, "$defs": {definition: {"type": "string"}}});
            accepted(&source, root, &mut sources);
        }
    }
    assert!(
        sources.len() >= 10,
        "the corpus must contain accepted controls"
    );
    let output = typecheck(&sources);
    assert!(
        output.status.success(),
        "accepted bindings failed TypeScript:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting() {
    let source = json!({"$id": "urn:example:properties:1", "type": "object", "additionalProperties": false,
        "properties": {"class": {"type": "string"}, "string": {"type": "string"}, "Array": {"type": "string"}, "wire-key": {"type": "string"}},
        "examples": [{"type": "array"}]
    });
    let mut sources = Vec::new();
    for root in [
        "Array",
        "type",
        "namespace",
        "readonly",
        "abstract",
        "$Type",
        "_Type",
    ] {
        let generated = project(&source, root).expect("valid alias and property control");
        assert!(
            generated.contains("  class?: string;\n  string?: string;\n  \"wire-key\"?: string;")
        );
        sources.push(generated);
    }
    sources.push(include_str!("fixtures/registry.ts").to_owned());
    let output = typecheck(&sources);
    assert!(
        output.status.success(),
        "valid controls failed TypeScript:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn compiler_rejects_a_known_duplicate_binding() {
    let output = typecheck(&[
        "export type Duplicate = string;\nexport type Duplicate = number;\n".to_owned(),
    ]);
    assert!(
        !output.status.success(),
        "the real compiler must reject duplicate aliases"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("TS2300"),
        "expected duplicate-identifier diagnostics: {output:?}"
    );
}

#[test]
fn nested_nullable_arrays_and_global_looking_aliases_typecheck() {
    let source = json!({
        "$id": "urn:example:adversary:nested",
        "$defs": {
            "leaf~node": {"type": "string"},
            "branch/node": {
                "type": "object", "additionalProperties": false,
                "properties": {
                    "next": {"type": ["null", "array"], "items": {"$ref": "#/$defs/leaf~0node"}}
                }
            }
        },
        "type": ["null", "array"],
        "items": {"type": "array", "items": {"$ref": "#/$defs/branch~1node"}}
    });
    let mut declarations = Vec::new();
    for root in [
        "array",
        "$Array",
        "_Array",
        "ReadonlyArray",
        "Object",
        "Promise",
        "globalThis",
    ] {
        let generated = project(&source, root).expect("distinct bindings remain feasible");
        assert!(generated.contains("export type BranchNode"));
        assert!(generated.contains("export type LeafNode"));
        assert!(generated.contains("Array<Array<BranchNode>>"));
        declarations.push(generated);
    }
    let output = typecheck(&declarations);
    assert!(
        output.status.success(),
        "accepted nested helper cases: {output:?}"
    );
}

#[test]
fn contextual_roots_and_keyword_properties_keep_compilable_spelling() {
    let source = json!({
        "$id": "urn:example:adversary:context",
        "$defs": {"array": {"type": "string", "items": {
            "type": "array", "items": {"type": "string"}
        }}},
        "type": "object", "additionalProperties": false,
        "properties": {
            "Array": {"$ref": "#/$defs/array"},
            "await": {"type": "string"},
            "constructor": {"type": "string"},
            "__proto__": {"type": "string"},
            "readonly": {"type": "string"},
            "foo/bar": {"type": "string"}
        },
        "required": ["Array", "__proto__"]
    });
    let mut declarations = Vec::new();
    for root in [
        "namespace",
        "type",
        "satisfies",
        "infer",
        "using",
        "constructor",
    ] {
        let generated = project(&source, root).expect("contextual root stays valid");
        assert!(generated.contains("  Array: Array;\n"));
        assert!(generated.contains("  __proto__: string;\n"));
        assert!(generated.contains("  readonly?: string;\n"));
        assert!(generated.contains("  \"foo/bar\"?: string;\n"));
        declarations.push(generated);
    }
    let output = typecheck(&declarations);
    assert!(
        output.status.success(),
        "property/binding separation: {output:?}"
    );
}

#[test]
fn multiple_normalized_definitions_keep_references_and_binding_order() {
    let source = json!({
        "$id": "urn:example:adversary:order",
        "$defs": {
            "z-last": {"type": "boolean"},
            "foo/bar": {"type": "string"},
            "a~b": {"type": "object", "additionalProperties": false,
                "properties": {"ref": {"$ref": "#/$defs/foo~1bar"}}}
        },
        "type": "object", "additionalProperties": false,
        "properties": {
            "a~b": {"$ref": "#/$defs/a~0b"},
            "foo/bar": {"$ref": "#/$defs/foo~1bar"},
            "z-last": {"$ref": "#/$defs/z-last"}
        }
    });
    let generated = project(&source, "$Root").expect("noncolliding normalized definitions");
    let first = generated.find("export type AB").unwrap();
    let middle = generated.find("export type FooBar").unwrap();
    let last = generated.find("export type ZLast").unwrap();
    let root = generated.find("export type $Root").unwrap();
    assert!(first < middle && middle < last && last < root);
    assert!(generated.contains("  \"a~b\"?: AB;\n"));
    assert!(generated.contains("  \"foo/bar\"?: FooBar;\n"));
    assert_eq!(project(&source, "$Root").unwrap(), generated);
    let output = typecheck(&[generated]);
    assert!(
        output.status.success(),
        "normalized declaration references: {output:?}"
    );
}
