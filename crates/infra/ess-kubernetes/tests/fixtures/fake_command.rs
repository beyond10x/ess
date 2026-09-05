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
        match args.as_slice() {
            ["--context", "synthetic-context", "get", kind, "-A", "-o", "json"] => {
                if *kind == "secrets" {
                    std::env::var("ESS_TEST_SECRET_RESPONSE").expect("synthetic Secret response")
                } else {
                    "{\"items\":[]}".to_owned()
                }
            }
            _ => panic!("unexpected scan invocation"),
        }
    };
    std::io::stdout().write_all(response.as_bytes()).expect("fixture response");
}
