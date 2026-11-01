//! Adversary, pass 1: the generated `integer_at` and `integerAt` on whole numbers however spelt.
//!
//! The committed generated trees are the emitted text (`cargo xtask generate --check` holds them
//! to it), so the Rust reader is compiled here straight from the committed `json.rs`, and the Go
//! reader is cut from the committed `wire.go` and run beside it on the same documents. Every
//! document gets one answer from each, and the two must agree: `ok <n>`, `refused <message>`, or
//! `notjson` where the body is not one JSON value.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[allow(dead_code, missing_docs, clippy::all, clippy::pedantic)]
#[rustfmt::skip]
#[path = "../../../../generated/rust/gatepass/crates/gatepass-server/src/json.rs"]
mod json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// What the generated Rust server answers for `document` sent as an `Integer`.
fn rust(document: &str) -> String {
    let Ok(value) = json::parse(&format!("{{\"limit\":{document}}}")) else {
        return "notjson".to_owned();
    };
    match json::integer_at(
        json::member_at(&value, "", "limit").expect("the member"),
        "limit",
        "an integer",
    ) {
        Ok(held) => format!("ok {held}"),
        Err(error) => format!("refused {error}"),
    }
}

/// Whole numbers in `i64` range, each with the integer it denotes.
fn accepted() -> Vec<(String, i64)> {
    let mut cases: Vec<(String, i64)> = [
        ("3", 3),
        ("3.0", 3),
        ("3.00", 3),
        ("3e0", 3),
        ("30e-1", 3),
        ("0.3e1", 3),
        ("300E-2", 3),
        ("3E+0", 3),
        ("1e-0", 1),
        ("1e+0", 1),
        ("-0", 0),
        ("-0.0", 0),
        ("0.000", 0),
        ("0e999999999", 0),
        ("-0e-999999999", 0),
        ("0.0e99999999999999999999999999", 0),
        ("1e18", 1_000_000_000_000_000_000),
        ("-1e18", -1_000_000_000_000_000_000),
        ("9e18", 9_000_000_000_000_000_000),
        ("1e00000000000000000000000000000002", 100),
        ("9223372036854775807", i64::MAX),
        ("9223372036854775807.0", i64::MAX),
        ("9223372036854775807e0", i64::MAX),
        ("92233720368547758070e-1", i64::MAX),
        ("0.9223372036854775807e19", i64::MAX),
        ("922337203685477580.7e1", i64::MAX),
        ("-9223372036854775808", i64::MIN),
        ("-9223372036854775808.000", i64::MIN),
        ("-92233720368547758080e-1", i64::MIN),
        ("-0.9223372036854775808E19", i64::MIN),
    ]
    .into_iter()
    .map(|(spelling, held)| (spelling.to_owned(), held))
    .collect();
    let zeros = "0".repeat(200_000);
    cases.push((format!("1{zeros}e-200000"), 1));
    cases.push((format!("0.{zeros}1e200001"), 1));
    cases.push((format!("0.{zeros}"), 0));
    cases.push((format!("-7.{zeros}"), -7));
    cases
}

/// Numbers no `i64` holds: fractions, magnitudes past 64 bits, hostile exponents.
fn refused() -> Vec<String> {
    let mut cases: Vec<String> = [
        "1.5",
        "-1.5",
        "0.5",
        "5e-1",
        "30e-2",
        "1.0000000000000000000001",
        "9223372036854775808",
        "9223372036854775808.0",
        "9223372036854775808e0",
        "-9223372036854775809",
        "-9223372036854775809.0",
        "9.3e18",
        "1e19",
        "10e18",
        "0.1e20",
        "1e999999999",
        "-1e999999999",
        "1e-999999999",
        "1e99999999999999999999999999999999999999",
        "1E+00019",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();
    let zeros = "0".repeat(200_000);
    cases.push(format!("1{zeros}"));
    cases.push(format!("1{zeros}.0"));
    cases.push(format!("1{zeros}e-200001"));
    cases.push(format!("0.{zeros}1"));
    cases
}

/// Spellings JSON does not have, which neither server may read as a number.
const NOT_JSON: [&str; 9] = ["+1", "+1.0", "1.", ".5", "-", "1e", "1e+", "--1", "1.e1"];

/// Bounded so a reader that builds the digits of `1e999999999` fails here rather than hanging.
const QUICK: Duration = Duration::from_secs(2);

#[test]
fn the_rust_reader_takes_every_whole_spelling_as_its_integer() {
    for (spelling, held) in accepted() {
        let started = Instant::now();
        let answer = rust(&spelling);
        assert!(started.elapsed() < QUICK, "slow on {spelling:.40}");
        assert_eq!(answer, format!("ok {held}"), "{spelling:.60}");
    }
}

#[test]
fn the_rust_reader_refuses_fractions_overflow_and_hostile_exponents_quickly() {
    for spelling in refused() {
        let started = Instant::now();
        let answer = rust(&spelling);
        assert!(started.elapsed() < QUICK, "slow on {spelling:.40}");
        assert_eq!(
            answer,
            format!("refused limit: expected an integer, found the number {spelling}"),
            "{spelling:.60}"
        );
    }
}

#[test]
fn the_rust_reader_admits_no_spelling_json_does_not_have() {
    for spelling in NOT_JSON {
        assert_eq!(rust(spelling), "notjson", "{spelling}");
    }
}

/// The Go harness: the committed `DecodeError`, `describes`, `integerAt` and `wholeNumber`, and a
/// `main` that reads each line as the generated `readJSON` reads a body.
fn go_program() -> String {
    let wire = std::fs::read_to_string(root().join("generated/go/gatepass/server/wire.go"))
        .expect("the committed Go wire");
    let cut = |from: &str, to: &str| {
        let start = wire
            .find(from)
            .unwrap_or_else(|| panic!("`{from}` in wire.go"));
        let end = start
            + wire[start..]
                .find(to)
                .unwrap_or_else(|| panic!("`{to}` in wire.go"));
        wire[start..end].to_owned()
    };
    let mut program = String::from(
        "package main\n\nimport (\n\t\"bufio\"\n\t\"bytes\"\n\t\"encoding/json\"\n\t\"fmt\"\n\t\
         \"os\"\n\t\"strconv\"\n\t\"strings\"\n)\n\n",
    );
    program.push_str(&cut("type DecodeError struct", "// nested is one step"));
    program.push_str(&cut("func integerAt(", "// decimalAt is"));
    program.push_str(
        "func main() {\n\
         \tscanner := bufio.NewScanner(os.Stdin)\n\
         \tscanner.Buffer(make([]byte, 1<<20), 1<<26)\n\
         \tfor scanner.Scan() {\n\
         \t\tdecoder := json.NewDecoder(bytes.NewReader(scanner.Bytes()))\n\
         \t\tdecoder.UseNumber()\n\
         \t\tvar value any\n\
         \t\tif err := decoder.Decode(&value); err != nil {\n\
         \t\t\tfmt.Println(\"notjson\")\n\
         \t\t\tcontinue\n\
         \t\t}\n\
         \t\tobject, _ := value.(map[string]any)\n\
         \t\theld, err := integerAt(object[\"limit\"], \"limit\", \"an integer\")\n\
         \t\tif err != nil {\n\
         \t\t\tfmt.Println(\"refused \" + err.Error())\n\
         \t\t} else {\n\
         \t\t\tfmt.Println(\"ok \" + strconv.FormatInt(held, 10))\n\
         \t\t}\n\
         \t}\n\
         \t_ = strings.TrimSpace\n\
         }\n",
    );
    program
}

/// The Go harness's answer to each document, in order, or `None` where this machine has no `go`.
fn go(documents: &[String]) -> Option<Vec<String>> {
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `go` on PATH, so the generated Go reader was not run");
        return None;
    }
    let run = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-u15-go-{}-{run}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("mkdir");
    std::fs::write(directory.join("main.go"), go_program()).expect("write main.go");
    let built = Command::new("go")
        .args(["build", "-o", "harness", "main.go"])
        .current_dir(&directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go builds");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut input = String::new();
    for document in documents {
        writeln!(input, "{{\"limit\":{document}}}").expect("write");
    }
    let started = Instant::now();
    let mut child = Command::new(directory.join("harness"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("harness runs");
    // Fed from a thread: the answers echo long spellings, and a pipe that fills while this side
    // is still writing would hold both sides forever.
    let mut stdin = child.stdin.take().expect("stdin");
    let feeder = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let output = child.wait_with_output().expect("harness ends");
    feeder.join().expect("feeder").expect("feed");
    assert!(
        started.elapsed() < QUICK * 5,
        "the Go reader took {:?}",
        started.elapsed()
    );
    let _ = std::fs::remove_dir_all(&directory);
    Some(
        String::from_utf8(output.stdout)
            .expect("utf-8")
            .lines()
            .map(ToOwned::to_owned)
            .collect(),
    )
}

#[test]
fn the_go_reader_gives_the_rust_reader_s_answer_on_every_document() {
    let mut documents: Vec<String> = accepted().into_iter().map(|(s, _)| s).collect();
    documents.extend(refused());
    documents.extend(NOT_JSON.iter().map(|s| (*s).to_owned()));
    let Some(answers) = go(&documents) else {
        return;
    };
    assert_eq!(answers.len(), documents.len());
    for (document, answer) in documents.iter().zip(&answers) {
        assert_eq!(*answer, rust(document), "{document:.60}");
    }
}
