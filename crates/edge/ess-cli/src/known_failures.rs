//! `--known-failing` and `--accounting-out` on `verify conform run`, `report` and `mutate`
//! (beyond10x/ess#294, beyond10x/ess#296; `docs/design/mutation-scope-and-known-failures.md`).
//!
//! Known-failure accounting never changes conformance: the ordinary report keeps its bytes, counts
//! and verdict, `run` keeps its exit status (strict included), and `report` keeps exit 0 meaning
//! that what it writes was written. A refused declaration, execution context or output destination
//! exits 2, and every input is checked before any output is written. Every output this module
//! writes is created new: it never replaces a file, and never one of the inputs.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ess_conformance::known_failures::{
    account, sha256, Accounting, Declaration, ExecutionContext, Refusal,
};
use ess_conformance::{AdmittedSuite, CountReport};

/// Every refusal here exits 2, as `report` refuses an input.
pub const REFUSED: u8 = 2;

fn refused(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(REFUSED)
}

/// The SHA-256 of the running `ess` executable's bytes: the public build of every built-in
/// target, read before any target runs.
pub fn executable_build() -> Result<String, String> {
    let path = std::env::current_exe()
        .map_err(|error| format!("known-failures.build: locating the ess executable: {error}"))?;
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "known-failures.build: reading the ess executable {}: {error}",
            path.display()
        )
    })?;
    Ok(sha256(&bytes))
}

/// `path`, resolved through its existing parent so two spellings of one file compare equal.
fn resolved(path: &Path) -> PathBuf {
    if let Ok(found) = path.canonicalize() {
        return found;
    }
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    match (parent.canonicalize(), path.file_name()) {
        (Ok(parent), Some(name)) => parent.join(name),
        _ => path.to_path_buf(),
    }
}

/// Refuses `outputs` that do not exist yet, are not distinct from one another, or name an input.
pub fn check_outputs(outputs: &[(&str, &Path)], inputs: &[&Path]) -> Result<(), String> {
    let inputs: Vec<PathBuf> = inputs.iter().map(|path| resolved(path)).collect();
    let mut seen: Vec<(PathBuf, &str)> = Vec::new();
    for (flag, path) in outputs {
        let at = resolved(path);
        if path.exists() {
            return Err(format!(
                "known-failures.output: {flag} {} already exists; it is written as a new file and \
                 never replaces one",
                path.display()
            ));
        }
        if inputs.contains(&at) {
            return Err(format!(
                "known-failures.output: {flag} {} is one of the inputs",
                path.display()
            ));
        }
        if let Some((_, other)) = seen.iter().find(|(seen, _)| *seen == at) {
            return Err(format!(
                "known-failures.output: {flag} {} is also {other}",
                path.display()
            ));
        }
        seen.push((at, flag));
    }
    Ok(())
}

/// Writes `contents` to `path`, which must not exist yet.
pub fn write_new(path: &Path, contents: &str) -> Result<(), String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(contents.as_bytes()))
        .map_err(|error| {
            format!(
                "known-failures.unwritable: writing {}: {error}",
                path.display()
            )
        })
}

fn read(path: &Path, what: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| {
        format!(
            "known-failures.unreadable: reading the {what} {}: {error}",
            path.display()
        )
    })
}

fn refusal(path: &Path, refusal: &Refusal) -> String {
    format!("{} was refused: {refusal}", path.display())
}

/// `run --known-failing FILE --accounting-out FILE`, as asked.
pub struct RunRequest {
    /// `--known-failing`.
    pub declaration: PathBuf,
    /// `--accounting-out`.
    pub accounting: PathBuf,
    /// `--report-out`, which clap requires beside them.
    pub report: Option<PathBuf>,
    /// `--suite` or `--suite-input`.
    pub inputs: Vec<PathBuf>,
}

/// A `run` the declaration binds: checked before the target is made.
pub struct RunBinding {
    request: RunRequest,
    declaration: Declaration,
    build: String,
}

impl RunRequest {
    /// The request `run` was given, or `None` without `--known-failing`; exit 2 where it was given
    /// without `--report-format 2`, which the accounting binds.
    pub fn from_flags<'a>(
        known: Option<(PathBuf, PathBuf)>,
        report: Option<&Path>,
        inputs: impl Iterator<Item = &'a PathBuf>,
        report_format: &str,
    ) -> Result<Option<Self>, ExitCode> {
        let Some((declaration, accounting)) = known else {
            return Ok(None);
        };
        if report_format != "2" {
            return Err(refused(
                "known-failures.report-format: --known-failing accounts the exact bytes of an \
                 ess-conformance-report/2, so it requires --report-format 2",
            ));
        }
        Ok(Some(Self {
            declaration,
            accounting,
            report: report.map(Path::to_path_buf),
            inputs: inputs.cloned().collect(),
        }))
    }

    /// Checks the destination, reads the declaration and binds it to the exact suite `admitted`
    /// and the running executable's build, before anything runs; exit 2 where it does not bind.
    pub fn bind(self, admitted: &AdmittedSuite) -> Result<RunBinding, ExitCode> {
        let mut inputs: Vec<&Path> = vec![self.declaration.as_path()];
        inputs.extend(self.report.as_deref());
        inputs.extend(self.inputs.iter().map(PathBuf::as_path));
        let bound = check_outputs(&[("--accounting-out", &self.accounting)], &inputs)
            .and_then(|()| executable_build())
            .and_then(|build| {
                let text = read(&self.declaration, "known-failure declaration")?;
                let declaration = Declaration::from_json(&text)
                    .and_then(|declaration| {
                        declaration.admit(admitted)?;
                        declaration.bind_build(&build)?;
                        Ok(declaration)
                    })
                    .map_err(|error| refusal(&self.declaration, &error))?;
                Ok((declaration, build))
            });
        match bound {
            Ok((declaration, build)) => Ok(RunBinding {
                request: self,
                declaration,
                build,
            }),
            Err(message) => Err(refused(message)),
        }
    }
}

impl RunBinding {
    /// Accounts the run from the exact report/2 bytes `--report-out` receives before `render`
    /// writes anything, then renders the run exactly as without a declaration and writes the
    /// accounting. The exit status is `render`'s, unless the accounting is refused or unwritable.
    pub fn finish(
        &self,
        run: &ess_conformance::ExecutedRun,
        admitted: &AdmittedSuite,
        render: impl FnOnce() -> anyhow::Result<ExitCode>,
    ) -> anyhow::Result<ExitCode> {
        let text = CountReport::from_run(run, admitted)?.to_canonical_json()?;
        let accounting = match account(&text, admitted, &self.declaration, &self.build) {
            Ok(accounting) => accounting,
            Err(error) => return Ok(refused(refusal(&self.request.declaration, &error))),
        };
        let code = render()?;
        if let Err(message) = write_accounting(&self.request.accounting, &accounting) {
            return Ok(refused(message));
        }
        Ok(code)
    }
}

/// Writes `accounting` to `out` as a new file and summarises it on standard error, so a
/// machine-readable standard output is unchanged.
pub fn write_accounting(out: &Path, accounting: &Accounting) -> Result<(), String> {
    write_new(out, &accounting.to_canonical_json())?;
    eprint!("{}", accounting.render_text());
    Ok(())
}

/// `verify conform report --observed-report`: the accounting of a report a generated runner wrote
/// with its host's execution context, and nothing else.
///
/// The report is admitted from its original bytes against the exact suite by the same reader that
/// admits any report/2, so its producer profile, every category (Skipped included), coverage and
/// both statuses are what the runner wrote. The execution context must be of exactly that report
/// and suite, naming the report's own label; its build is the one the accounting binds. Neither
/// input is rewritten and no report or results document is created. Exit 0: the accounting was
/// written, which says nothing about conformance. Exit 2: an input or the destination was refused,
/// and nothing was written.
pub fn report_observed(
    suite: &Path,
    observed: &Path,
    context: &Path,
    declaration: &Path,
    out: &Path,
) -> ExitCode {
    let inputs = [suite, observed, context, declaration];
    if let Err(message) = check_outputs(&[("--accounting-out", out)], &inputs) {
        return refused(message);
    }
    let texts = (|| -> Result<(String, String, String, String), String> {
        Ok((
            read(suite, "suite")?,
            read(observed, "observed report")?,
            read(context, "execution context")?,
            read(declaration, "known-failure declaration")?,
        ))
    })();
    let (suite_text, report_text, context_text, declaration_text) = match texts {
        Ok(texts) => texts,
        Err(message) => return refused(message),
    };
    let admitted = match AdmittedSuite::from_json(&suite_text) {
        Ok(admitted) => admitted,
        Err(error) => return refused(format!("{} was refused: {error}", suite.display())),
    };
    if let Err(error) = CountReport::from_json(&report_text, &admitted) {
        return refused(format!(
            "{} was refused against {}: {error}",
            observed.display(),
            suite.display()
        ));
    }
    let execution = match ExecutionContext::from_json(&context_text)
        .and_then(|execution| execution.admit(&report_text, &admitted).map(|()| execution))
    {
        Ok(execution) => execution,
        Err(error) => return refused(refusal(context, &error)),
    };
    let accounting = match Declaration::from_json(&declaration_text).and_then(|parsed| {
        account(
            &report_text,
            &admitted,
            &parsed,
            execution.implementation_build(),
        )
    }) {
        Ok(accounting) => accounting,
        Err(error) => return refused(refusal(declaration, &error)),
    };
    match write_accounting(out, &accounting) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refused(message),
    }
}

/// The paths `report --results` writes with known-failure accounting.
pub struct ResultsOutputs<'a> {
    /// `--report-out`.
    pub report: &'a Path,
    /// `--execution-context-out`.
    pub context: &'a Path,
    /// `--accounting-out`.
    pub accounting: &'a Path,
}

/// `verify conform report --results` with `--known-failing`: the ordinary report/2, unchanged, and
/// beside it the execution context of the caller's explicit `build` and the accounting.
///
/// Every input is admitted and the report, context and accounting are all computed before the
/// first is written; each is created new. Exit 0: all three were written. Exit 2: refused, and
/// nothing was written.
pub fn report_results(
    suite: &Path,
    results: &Path,
    implementation: &str,
    runner: Option<&str>,
    (declaration, build): (&Path, &str),
    outputs: &ResultsOutputs<'_>,
) -> ExitCode {
    if let Err(message) = check_outputs(
        &[
            ("--report-out", outputs.report),
            ("--execution-context-out", outputs.context),
            ("--accounting-out", outputs.accounting),
        ],
        &[suite, results, declaration],
    ) {
        return refused(message);
    }
    let texts = (|| -> Result<(String, String, String), String> {
        Ok((
            read(suite, "suite")?,
            read(results, "results")?,
            read(declaration, "known-failure declaration")?,
        ))
    })();
    let (suite_text, results_text, declaration_text) = match texts {
        Ok(texts) => texts,
        Err(message) => return refused(message),
    };
    let written =
        ess_conformance::results::report(&suite_text, &results_text, implementation, runner)
            .and_then(|report| report.to_canonical_json());
    let report_text = match written {
        Ok(text) => text,
        Err(error) => {
            return refused(format!(
                "{} was refused against {}: {error}",
                results.display(),
                suite.display()
            ))
        }
    };
    let admitted = match AdmittedSuite::from_json(&suite_text) {
        Ok(admitted) => admitted,
        Err(error) => return refused(format!("{} was refused: {error}", suite.display())),
    };
    let execution = match ExecutionContext::new(&report_text, &admitted, implementation, build) {
        Ok(execution) => execution,
        Err(error) => return refused(format!("--implementation-build was refused: {error}")),
    };
    let accounting = match Declaration::from_json(&declaration_text)
        .and_then(|parsed| account(&report_text, &admitted, &parsed, build))
    {
        Ok(accounting) => accounting,
        Err(error) => return refused(refusal(declaration, &error)),
    };
    let written = write_new(outputs.report, &report_text)
        .and_then(|()| write_new(outputs.context, &execution.to_canonical_json()))
        .and_then(|()| write_accounting(outputs.accounting, &accounting));
    match written {
        Ok(()) => {
            println!(
                "{}: report written; {} and {} written beside it",
                outputs.report.display(),
                outputs.context.display(),
                outputs.accounting.display()
            );
            ExitCode::SUCCESS
        }
        Err(message) => refused(message),
    }
}
