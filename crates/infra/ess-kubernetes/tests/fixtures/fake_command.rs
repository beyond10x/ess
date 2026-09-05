//! Synthetic process fixture: never reaches kubectl, a clock, or a cluster.

use std::io::Write;
use std::path::Path;

fn main() {
    let executable = std::env::args().next().expect("executable name");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let response = if Path::new(&executable).file_stem().expect("command name") == "date" {
        assert_eq!(args, ["-u", "+%Y-%m-%dT%H:%M:%SZ"]);
        "2026-09-05T00:00:00Z\n".to_owned()
    } else {
        let expected_context =
            std::env::var("ESS_TEST_CONTEXT").unwrap_or_else(|_| "synthetic-context".to_owned());
        let (operation, resource, all_namespaces) = match args.as_slice() {
            ["config", "get-contexts", "-o", "name"] => ("contexts", false, false),
            ["config", "current-context"] => ("current-context", false, false),
            ["--context", context, "get", kind, "-A", "-o", "json"] => {
                assert_eq!(*context, expected_context);
                (*kind, true, true)
            }
            ["--context", context, "get", kind, "-o", "json"] => {
                assert_eq!(*context, expected_context);
                (*kind, true, false)
            }
            _ => panic!("unexpected scan invocation"),
        };
        if let Some(path) = std::env::var_os("ESS_TEST_CALL_LOG") {
            let mut log = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("synthetic call log");
            writeln!(log, "{operation}:all_namespaces={all_namespaces}")
                .expect("record synthetic operation");
        }
        let failure = std::env::var("ESS_TEST_FAILURE_OPERATION").unwrap_or_default();
        let fail_this_operation = failure == operation || (resource && failure == "all-resources");
        let first_attempt_only = std::env::var_os("ESS_TEST_FAILURE_FIRST_ATTEMPT_ONLY").is_some();
        let legacy_secret_failure = (operation == "secrets")
            .then(|| std::env::var("ESS_TEST_SECRET_FAILURE").ok())
            .flatten();
        if legacy_secret_failure.is_some()
            || (fail_this_operation && (!first_attempt_only || all_namespaces))
        {
            let diagnostic = legacy_secret_failure.unwrap_or_else(|| {
                std::env::var("ESS_TEST_FAILURE_DIAGNOSTIC").expect("synthetic diagnostic")
            });
            let mut stderr = std::io::stderr();
            if std::env::var_os("ESS_TEST_INVALID_UTF8_STDERR").is_some() {
                stderr
                    .write_all(&[0xff, 0xfe])
                    .expect("invalid UTF-8 prefix");
            }
            writeln!(stderr, "malformed synthetic response: {diagnostic}")
                .expect("synthetic failure diagnostic");
            std::io::stdout()
                .write_all(diagnostic.as_bytes())
                .expect("failed response");
            std::process::exit(1);
        }
        match args.as_slice() {
            ["config", "get-contexts", "-o", "name"] => {
                "synthetic-context\nsecond-context\n".to_owned()
            }
            ["config", "current-context"] => format!("{expected_context}\n"),
            ["--context", _, "get", kind, "-A", "-o", "json"]
            | ["--context", _, "get", kind, "-o", "json"] => {
                if *kind == "secrets" {
                    std::env::var("ESS_TEST_SECRET_RESPONSE").expect("synthetic Secret response")
                } else {
                    "{\"items\":[]}".to_owned()
                }
            }
            _ => panic!("unexpected scan invocation"),
        }
    };
    std::io::stdout()
        .write_all(response.as_bytes())
        .expect("fixture response");
}
