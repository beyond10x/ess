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
        let root = std::path::PathBuf::from(std::env::var_os("ESS_TEST_DELIVERY_LOG").unwrap()).parent().unwrap().to_path_buf();
        let identities = std::fs::read_to_string(root.join("identities")).unwrap();
        let identities: Vec<_> = identities.lines().collect();
        assert_eq!(arguments.len(), 6);
        assert_eq!(&arguments[2..4], ["fetch", "--output"]);
        let name = match arguments[1].as_str() {
            "manifest" => { assert_eq!(arguments[5], format!("example.invalid/chart@{}",identities[0])); "manifest" },
            "blob" if arguments[5] == format!("example.invalid/chart@{}",identities[1]) => "config",
            "blob" if arguments[5] == format!("example.invalid/chart@{}",identities[2]) => "chart",
            _ => panic!("unexpected OCI operation or identity"),
        };
        std::fs::copy(root.join(name), &arguments[4]).unwrap();
    } else {
        assert_eq!(std::fs::read(&arguments[4]).unwrap(), b"local fake chart");
    }
}
