//! The tagged union has a page (`story:docs-a-tagged-union-has-a-page`).
//!
//! `website/docs/guides/specify/unions.md` declares a `kind: union` with a unit variant and shows
//! the wire shape of every variant. These cases hold the page to the tool: the declaration is
//! copied out of the page and validated by the built `ess`, and the wire shapes are rendered from
//! the JSON Schema `ess generate --kind schema` writes for that declaration, so a page the
//! generator has moved away from fails here. Regenerate the shapes with
//! `ESS_UNION_PAGE=write cargo test -p ess-cli --test union_page`.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const PAGE: &str = "website/docs/guides/specify/unions.md";
const FIELDS_PAGE: &str = "website/docs/guides/specify/fields-and-invariants.md";
const UNION: &str = "demo.work.Status";
const BEGIN: &str = "[ess-union-wire-begin]: #";
const END: &str = "[ess-union-wire-end]: #";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn page() -> String {
    read(&root().join(PAGE))
}

/// The body of the first `yaml` block on the page that declares a union.
fn declaration(page: &str) -> String {
    let mut lines = page.lines();
    while let Some(line) = lines.next() {
        if line.trim_end() != "```yaml" {
            continue;
        }
        let mut body = String::new();
        for line in lines.by_ref() {
            if line.trim_end() == "```" {
                break;
            }
            body.push_str(line);
            body.push('\n');
        }
        if body.contains("kind: union") {
            return body;
        }
    }
    panic!("{PAGE} has no ```yaml block declaring `kind: union`")
}

struct Run {
    code: Option<i32>,
    output: String,
}

fn ess(arguments: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .output()
        .expect("run the built ess binary");
    Run {
        code: output.status.code(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

/// The page's declaration, written to a file of its own in `directory`.
fn written(directory: &Path) -> PathBuf {
    let path = directory.join("status.yaml");
    fs::write(&path, declaration(&page())).expect("write the declaration");
    path
}

/// The JSON Schema definition `ess generate --kind schema` writes for the page's union, parsed.
fn generated_union() -> serde_json::Value {
    generated().0
}

/// The definition parsed, and its text as the generator wrote it: the lines of its `$defs` entry,
/// from `"<union>": {` to the closing brace at the same indentation, moved to the left margin.
fn generated() -> (serde_json::Value, String) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    let model = written(directory.path());
    let out = directory.path().join("generated");
    let run = ess(&[
        "generate",
        "--path",
        model.to_str().expect("a UTF-8 path"),
        "--kind",
        "schema",
        "--out",
        out.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(run.code, Some(0), "generates:\n{}", run.output);
    let text = read(&out.join(format!("schema/types/{UNION}.schema.json")));
    let schema: serde_json::Value = serde_json::from_str(&text).expect("the schema is JSON");
    let union = schema["$defs"][UNION].clone();

    let opening = format!("\"{UNION}\": {{");
    let mut lines = text.lines().skip_while(|line| line.trim_start() != opening);
    let first = lines
        .next()
        .unwrap_or_else(|| panic!("the schema has a line `{opening}`:\n{text}"));
    let indent = first.len() - first.trim_start().len();
    let closing = format!("{}}}", " ".repeat(indent));
    let mut written = String::from("{\n");
    for line in lines {
        let closes = line.trim_end_matches(',') == closing;
        written.push_str(line.get(indent..).unwrap_or(line.trim_start()));
        written.push('\n');
        if closes {
            break;
        }
    }
    let written = written.trim_end().trim_end_matches(',').to_owned();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&written).expect("the lines are one JSON value"),
        union,
        "the lines cut out of the schema are the union's definition"
    );
    (union, written)
}

/// What one branch's payload is, in the table: the type a `$ref` names, or the JSON type.
fn payload(schema: &serde_json::Value) -> String {
    if let Some(reference) = schema["$ref"].as_str() {
        let name = reference.rsplit('/').next().unwrap_or(reference);
        return format!("<a {name} object>");
    }
    if let Some(kind) = schema["type"].as_str() {
        return format!("<a {kind}>");
    }
    format!("<{schema}>")
}

/// The generated section: one row per `oneOf` branch, then the definition itself.
fn rendered((union, written): &(serde_json::Value, String)) -> String {
    let tag = union["x-ess-union-tag"]
        .as_str()
        .expect("the definition names its tag");
    let branches = union["oneOf"].as_array().expect("a union is `oneOf`");
    let mut text = format!(
        "{BEGIN} (generated from the declaration above by ess generate --kind schema; \
         regenerate with ESS_UNION_PAGE=write cargo test -p ess-cli --test union_page)\n\n\
         | variant | required members | wire value |\n|---|---|---|\n"
    );
    for branch in branches {
        let label = branch["properties"][tag]["const"]
            .as_str()
            .expect("each branch pins its tag with `const`");
        let required: Vec<String> = branch["required"]
            .as_array()
            .expect("each branch lists its required members")
            .iter()
            .map(|member| format!("`{}`", member.as_str().expect("a member name")))
            .collect();
        let mut value = format!("{{\"{tag}\": \"{label}\"");
        for (member, schema) in branch["properties"]
            .as_object()
            .expect("each branch lists its members")
        {
            if member != tag {
                write!(value, ", \"{member}\": {}", payload(schema)).expect("a string");
            }
        }
        value.push('}');
        writeln!(text, "| `{label}` | {} | `{value}` |", required.join(", ")).expect("a string");
    }
    text.push_str("\nThe definition `ess generate --kind schema` writes for `");
    text.push_str(UNION);
    text.push_str("`:\n\n```json\n");
    text.push_str(written);
    text.push_str("\n```\n\n");
    text.push_str(END);
    text
}

#[test]
fn the_union_declaration_on_the_page_validates() {
    let declaration = declaration(&page());
    for phrase in ["kind: union", "tag: kind", "variants:"] {
        assert!(declaration.contains(phrase), "the declaration has {phrase}");
    }
    let directory = tempfile::tempdir().expect("a temporary directory");
    let model = written(directory.path());
    let run = ess(&[
        "specify",
        "validate",
        "--path",
        model.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(run.code, Some(0), "validates:\n{}", run.output);

    let union = generated_union();
    let tag = union["x-ess-union-tag"].as_str().expect("a tag");
    let branches = union["oneOf"].as_array().expect("a union is `oneOf`");
    assert!(
        branches
            .iter()
            .any(|branch| branch["required"] == serde_json::json!([tag])),
        "the declaration has a unit variant, a branch requiring the tag alone: {union}"
    );
    assert!(
        branches
            .iter()
            .any(|branch| branch["required"].as_array().is_some_and(|r| r.len() == 2)),
        "the declaration has a payload variant: {union}"
    );
}

#[test]
fn the_wire_shapes_on_the_page_are_the_generated_schema() {
    let path = root().join(PAGE);
    let section = rendered(&generated());
    let page = page();
    let start = page
        .find(BEGIN)
        .unwrap_or_else(|| panic!("{PAGE} has no `{BEGIN}` line"));
    let end = start
        + page[start..]
            .find(END)
            .unwrap_or_else(|| panic!("{PAGE} has no `{END}` line after `{BEGIN}`"))
        + END.len();
    let regenerated = format!("{}{section}{}", &page[..start], &page[end..]);
    if std::env::var_os("ESS_UNION_PAGE").is_some_and(|value| value == "write") {
        fs::write(&path, &regenerated).expect("the page writes");
    }
    assert!(
        read(&path) == regenerated,
        "the wire shapes on {PAGE} are not what `ess generate --kind schema` writes for its \
         declaration; regenerate them with \
         `ESS_UNION_PAGE=write cargo test -p ess-cli --test union_page`\n--- generated ---\n{section}"
    );
}

#[test]
fn the_page_states_the_tagging_rule_and_when_to_choose_a_union() {
    let page = page();
    for phrase in [
        "always tagged",
        "does not round-trip",
        "two variants of the same shape cannot be told apart",
        "unit variant",
        "the tag alone",
        "a struct of optional fields",
    ] {
        assert!(page.contains(phrase), "{PAGE} says {phrase:?}");
    }
    let choose = page
        .find("\n## When to choose a union\n")
        .expect("the page has `## When to choose a union`");
    assert!(
        page[choose..].contains("optional fields"),
        "`## When to choose a union` sets it against a struct of optional fields"
    );
}

#[test]
fn the_fields_page_links_to_the_union_page() {
    let fields = read(&root().join(FIELDS_PAGE));
    assert!(
        fields.contains("](unions.md)"),
        "{FIELDS_PAGE} links to the union page"
    );
}
