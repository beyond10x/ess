//! Local ORAS/Helm stand-in used only by delivery boundary tests.
use std::io::Write;

fn main() {
    let arguments: Vec<_> = std::env::args().collect();
    let executable = std::path::Path::new(&arguments[0])
        .file_stem()
        .unwrap()
        .to_string_lossy();
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(std::env::var_os("ESS_TEST_DELIVERY_LOG").unwrap())
        .unwrap();
    writeln!(log, "{executable}").unwrap();
    if executable == "oras" {
        let output = arguments
            .windows(2)
            .find(|pair| pair[0] == "--output")
            .unwrap();
        std::fs::write(
            std::path::Path::new(&output[1]).join("chart.tgz"),
            b"local fake chart",
        )
        .unwrap();
    }
}
