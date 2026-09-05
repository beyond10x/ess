//! Explicit compiler-backed projection checks, enabled by `typescript-typecheck`.

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
        "compilerOptions": {"strict": true, "noEmit": true, "types": [], "target": "ES2022", "module": "ESNext"},
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
