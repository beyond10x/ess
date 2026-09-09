//! Deterministic Rust/Clap projection of a resolved CLI presentation binding.

pub use ess_cli_contract::wire;
use ess_cli_contract::CompiledBinding;
use std::collections::BTreeMap;
use std::fmt::Write;
pub mod runtime;

/// Project without filesystem mutation.
pub fn project(binding: &CompiledBinding) -> BTreeMap<String, String> {
    let plan = binding.plan();
    let binary = &plan.binary;
    let package = format!("{binary}-cli-contract");
    let crate_name = "cli_contract";
    let mut artifacts = BTreeMap::from([
        ("binding.json".to_owned(), binding.to_canonical_json()),
        ("Cargo.toml".to_owned(), format!("[package]\nname = {package:?}\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[lib]\nname = \"cli_contract\"\n\n[[bin]]\nname = {binary:?}\npath = \"src/main.rs\"\n\n[workspace]\n\n[dependencies]\nclap = {{ version = \"4\", features = [\"string\"] }}\nclap_complete = \"4\"\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\nrpassword = \"7\"\nrustix = {{ version = \"1\", features = [\"fs\", \"process\"] }}\n")),
        ("src/wire.rs".to_owned(), include_str!("../../../specify/ess-cli-contract/src/wire.rs").to_owned()),
        ("src/runtime.rs".to_owned(), include_str!("runtime.rs").to_owned()),
        ("src/lib.rs".to_owned(), "//! Generated typed CLI adapter.\npub mod wire;\npub mod runtime;\npub use runtime::*;\npub fn plan() -> wire::Plan { serde_json::from_str(include_str!(\"../binding.json\")).expect(\"compiled CLI plan\") }\npub fn command() -> clap::Command { runtime::command(&plan()) }\npub fn run(args: Vec<std::ffi::OsString>, sources: &mut dyn Sources, handler: &mut dyn Handler, dynamic: Option<&mut dyn DynamicValidator>) -> ProcessOutput { runtime::run(&plan(), args, sources, handler, dynamic) }\n".to_owned()),
        ("src/main.rs".to_owned(), format!("fn main() {{\n    let output = {crate_name}::run(std::env::args_os().collect(), &mut {crate_name}::OsSources, &mut {crate_name}::UnavailableHandler, None);\n    print!(\"{{}}\", output.stdout);\n    eprint!(\"{{}}\", output.stderr);\n    std::process::exit(output.exit_code);\n}}\n")),
    ]);
    let mut completions = Vec::new();
    clap_complete::generate(
        clap_complete::Shell::Bash,
        &mut runtime::command(plan),
        binary,
        &mut completions,
    );
    artifacts.insert(
        format!("completions/{binary}.bash"),
        String::from_utf8(completions).expect("bash completion is UTF-8"),
    );
    artifacts.insert(
        "help.txt".to_owned(),
        runtime::command(plan).render_long_help().to_string(),
    );
    artifacts.insert("README.md".to_owned(), reference(plan));
    let manifest = serde_json::json!({"format":"ess-cli-artifacts/1", "binary":binary, "package":package, "files": artifacts.keys().collect::<Vec<_>>(), "obligations":plan.obligations});
    artifacts.insert(
        "manifest.json".to_owned(),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("manifest serializes")
        ),
    );
    artifacts
}

fn reference(plan: &wire::Plan) -> String {
    let mut text = format!(
        "# `{}` CLI contract\n\n{}\n\nThis generated package installs an unavailable handler. Application behavior enters through the `Handler` seam; schema-selected calls also require `DynamicValidator`. See `help.txt` for process options and `binding.json` for resolved types and targets.\n\n## Commands\n\n",
        plan.binary, plan.about
    );
    for command in &plan.commands {
        let callable = &plan.callables[&command.callable];
        writeln!(
            text,
            "### `{}`\n\n{}\n",
            command.path.join(" "),
            command.about
        )
        .expect("String write");
        writeln!(
            text,
            "Callable: `{}`. Input: `{}`. Result: `{}`.\n",
            command.callable,
            callable
                .input
                .as_ref()
                .map_or("null (inputless)", |input| input.type_ref.as_str()),
            callable.result.type_ref
        )
        .expect("String write");
        for alias in &command.aliases {
            writeln!(text, "Alias: `{}`.\n", alias.join(" ")).expect("String write");
        }
        for (code, contract) in &callable.errors {
            writeln!(text, "Error `{code}`: `{}`.\n", contract.type_ref).expect("String write");
        }
    }
    text.push_str("## Runtime obligations\n\n");
    for obligation in &plan.obligations {
        writeln!(text, "- {obligation}").expect("String write");
    }
    text
}
