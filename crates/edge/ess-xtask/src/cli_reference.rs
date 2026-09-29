//! `website/docs/reference/cli.md`, generated from the `ess` command definition.
//!
//! The page's command sections were a hand-kept table that fell behind the CLI: whole commands
//! (`generate project buildkit`, `verify conform author`) and flags (`--catalog`, `--retry-of`)
//! shipped without a row. This renders every visible command and argument from the clap tree the
//! `ess` binary parses with, between two marker lines, and `--check` refuses a page that differs.
//!
//! The tree is built by `command()` in the `ess` binary, which no other crate can link. So the
//! renderer in `cli_reference/render.rs` is compiled into that binary's tests, and one of those
//! tests writes the rendered block to the file [`OUT_VARIABLE`] names; this module runs that test
//! and splices its output into the page.

// Only the fixture cases here render; the real block is rendered inside the `ess` tests.
#[cfg(test)]
mod render;

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

/// The page whose command sections are generated.
pub const PAGE: &str = "website/docs/reference/cli.md";

/// The line that opens the generated block.
const BEGIN: &str = "[ess-cli-begin]: #";

/// The line that closes the generated block.
const END: &str = "[ess-cli-end]: #";

/// The `ess` binary test that renders its own command tree.
const ESS_TEST: &str = "tests::cli_reference_block";

/// Where [`ESS_TEST`] writes the rendered block.
const OUT_VARIABLE: &str = "ESS_CLI_REFERENCE_OUT";

/// Writes, or with `check` compares, the generated block of [`PAGE`].
pub(super) fn run(root: &Path, check: bool) -> Result<String> {
    let block = ess_block(root)?;
    let path = root.join(PAGE);
    let page = fs::read_to_string(&path).with_context(|| format!("reading {PAGE}"))?;
    if check {
        compare(&page, &block)?;
        return Ok(format!(
            "{PAGE}: the command sections agree with the `ess` command definition\n"
        ));
    }
    let updated = splice(&page, &block)?;
    if updated == page {
        return Ok(format!("{PAGE}: already current\n"));
    }
    fs::write(&path, updated).with_context(|| format!("writing {PAGE}"))?;
    Ok(format!("{PAGE}: command sections regenerated\n"))
}

/// Runs [`ESS_TEST`] and returns the block it rendered from the real `ess` command tree.
fn ess_block(root: &Path) -> Result<String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("system clock precedes the Unix epoch")?
        .as_nanos();
    let out = std::env::temp_dir().join(format!(
        "ess-cli-reference-{}-{nonce}.md",
        std::process::id()
    ));
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = std::process::Command::new(&cargo)
        .args([
            "test",
            "--quiet",
            "--locked",
            "--offline",
            "--package",
            "ess-cli",
            "--bin",
            "ess",
            "--",
            ESS_TEST,
            "--exact",
        ])
        .env(OUT_VARIABLE, &out)
        .current_dir(root)
        .output()
        .with_context(|| format!("running {cargo:?} test for {ESS_TEST}"))?;
    if !output.status.success() {
        let _ = fs::remove_file(&out);
        bail!(
            "the `ess` test {ESS_TEST} failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let block = fs::read_to_string(&out).with_context(|| {
        format!("the `ess` test {ESS_TEST} wrote no block to {OUT_VARIABLE}; was it renamed?")
    })?;
    fs::remove_file(&out).with_context(|| format!("removing {}", out.display()))?;
    Ok(block)
}

/// Refuses a page whose generated block is not `block`, naming the first line that differs.
fn compare(page: &str, block: &str) -> Result<()> {
    let expected = splice(page, block)?;
    if expected == page {
        return Ok(());
    }
    let found: Vec<&str> = page.lines().collect();
    let wanted: Vec<&str> = expected.lines().collect();
    let line = (0..found.len().max(wanted.len()))
        .find(|&line| found.get(line) != wanted.get(line))
        .unwrap_or(0);
    bail!(
        "{PAGE}:{}: the generated command sections differ from the `ess` command definition\n\
         expected: {}\n   found: {}\n\
         Run `cargo xtask cli-reference` to regenerate them.",
        line + 1,
        wanted.get(line).unwrap_or(&"<end of page>"),
        found.get(line).unwrap_or(&"<end of page>"),
    )
}

/// `page` with the text between its markers replaced by `block`; everything else byte for byte.
fn splice(page: &str, block: &str) -> Result<String> {
    let mut begins = Vec::new();
    let mut ends = Vec::new();
    let mut offset = 0;
    for line in page.split_inclusive('\n') {
        match line.strip_suffix('\n').unwrap_or(line) {
            BEGIN => begins.push(offset + line.len()),
            END => ends.push(offset),
            _ => {}
        }
        offset += line.len();
    }
    let ([begin], [end]) = (begins.as_slice(), ends.as_slice()) else {
        bail!(
            "{PAGE}: needs exactly one `{BEGIN}` line and one `{END}` line; found {} and {}",
            begins.len(),
            ends.len()
        );
    };
    if begin > end || !page[..*begin].ends_with('\n') {
        bail!("{PAGE}: `{END}` must follow `{BEGIN}` on a line of its own");
    }
    Ok(format!("{}\n{block}{}", &page[..*begin], &page[*end..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    /// Refuses a page whose generated block is not the one `command` renders.
    fn check(page: &str, command: &Command) -> Result<()> {
        compare(page, &render::render(command))
    }

    fn fixture() -> Command {
        Command::new("demo")
            .about("A demonstration command.")
            .arg(
                Arg::new("strict")
                    .long("strict")
                    .global(true)
                    .action(ArgAction::SetTrue)
                    .help("Refuse instead of warning."),
            )
            .subcommand(
                Command::new("project")
                    .about("Project artifacts.")
                    .subcommand(
                        Command::new("helm")
                            .about("Emit a chart.")
                            .long_about("Emit a chart.\n\nThe chart is written under `--out`.")
                            .arg(
                                Arg::new("chart")
                                    .long("chart")
                                    .value_name("DIR")
                                    .required(true)
                                    .help("Where the chart goes."),
                            )
                            .arg(
                                Arg::new("timeout")
                                    .long("timeout")
                                    .value_name("SECONDS")
                                    .default_value("30")
                                    .help("How long to wait."),
                            )
                            .arg(
                                Arg::new("format")
                                    .long("format")
                                    .value_parser(["text", "json"])
                                    .default_value("text")
                                    .help("How to print a {record} | <row>."),
                            ),
                    )
                    .subcommand(
                        Command::new("buildkit")
                            .about("Emit a bake file.")
                            .visible_alias("bake")
                            .alias("secret-alias")
                            .arg(Arg::new("path").required(true).help("The input.")),
                    ),
            )
            .subcommand(
                Command::new("old")
                    .hide(true)
                    .about("A hidden flat spelling."),
            )
    }

    fn page(block: &str) -> String {
        format!(
            "---\ntitle: Demo\n---\n\nProse before, kept  as written.\n\n{BEGIN}\n{block}{END}\n\nProse after.\n"
        )
    }

    fn generated(command: &Command) -> String {
        splice(&page("stale\n"), &render::render(command)).expect("the page has markers")
    }

    #[test]
    fn every_leaf_is_listed_with_its_invocation_and_arguments() {
        let block = render::render(&fixture());
        for expected in [
            "`demo project helm`",
            "`demo project buildkit`",
            "demo project helm [OPTIONS] --chart <DIR>",
            "| `--chart` | `<DIR>` | yes |",
            "| `--timeout` | `<SECONDS>` | no | `30` |",
            "`text`, `json`",
            "| `<path>` |",
            "The chart is written under `--out`.",
            "`--strict`",
            "`bake`",
            "generated",
        ] {
            assert!(block.contains(expected), "missing {expected:?} in\n{block}");
        }
    }

    #[test]
    fn hidden_commands_and_hidden_aliases_are_not_listed() {
        let block = render::render(&fixture());
        assert!(!block.contains("demo old"), "{block}");
        assert!(!block.contains("hidden flat spelling"), "{block}");
        assert!(!block.contains("secret-alias"), "{block}");
    }

    #[test]
    fn help_text_is_escaped_for_the_site_and_the_table() {
        let block = render::render(&fixture());
        assert!(
            block.contains(r"How to print a \{record\} \| &lt;row&gt;."),
            "{block}"
        );
    }

    #[test]
    fn prose_outside_the_markers_is_kept_byte_for_byte() {
        let page = generated(&fixture());
        let (before, rest) = page.split_once(BEGIN).expect("begin marker");
        let (_, after) = rest.split_once(END).expect("end marker");
        assert_eq!(
            before,
            "---\ntitle: Demo\n---\n\nProse before, kept  as written.\n\n"
        );
        assert_eq!(after, "\n\nProse after.\n");
        assert_eq!(splice(&page, &render::render(&fixture())).unwrap(), page);
    }

    #[test]
    fn rendering_is_deterministic() {
        assert_eq!(render::render(&fixture()), render::render(&fixture()));
    }

    #[test]
    fn check_accepts_a_current_page_and_refuses_one_missing_a_new_flag() {
        let page = generated(&fixture());
        check(&page, &fixture()).expect("a freshly generated page is current");

        let grown = fixture().mut_subcommand("project", |project| {
            project.mut_subcommand("helm", |helm| {
                helm.arg(
                    Arg::new("retry-of")
                        .long("retry-of")
                        .value_name("RUN")
                        .help("The run this one retries."),
                )
            })
        });
        let error = check(&page, &grown).expect_err("a new flag makes the page stale");
        let message = format!("{error:#}");
        assert!(message.contains("cargo xtask cli-reference"), "{message}");
        assert!(message.contains(PAGE), "{message}");
    }

    #[test]
    fn a_page_without_exactly_one_marker_pair_is_refused() {
        let block = render::render(&fixture());
        assert!(splice("no markers\n", &block).is_err());
        assert!(splice(&format!("{END}\n{BEGIN}\n"), &block).is_err());
        assert!(splice(&format!("{BEGIN}\n{END}\n{BEGIN}\n{END}\n"), &block).is_err());
    }

    /// The committed page names what the shipped command offers; `--check` keeps it that way.
    #[test]
    fn the_committed_page_names_the_commands_and_flags_adopters_asked_for() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("ess-xtask is inside the workspace");
        let page = std::fs::read_to_string(root.join(PAGE)).expect("the reference page");
        let (_, rest) = page.split_once(BEGIN).expect("begin marker");
        let (block, _) = rest.split_once(END).expect("end marker");
        for name in [
            "`ess generate project buildkit`",
            "`ess generate project helm`",
            "`ess verify conform author`",
            "`ess verify conform select`",
            "`--catalog`",
            "`--chart`",
            "`--stack-lock`",
            "`--timeout`",
            "`--retry-of`",
        ] {
            assert!(block.contains(name), "{PAGE} does not name {name}");
        }
    }
}
