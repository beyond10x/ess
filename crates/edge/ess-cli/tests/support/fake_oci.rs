//! Standalone Rust process fixture: strict operation/identity matching, no registry.
use std::io::Write;
use std::path::{Path, PathBuf};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(std::env::var_os("ESS_CACHE_FIXTURE").unwrap());
    let executable = Path::new(&args[0]).file_stem().unwrap().to_str().unwrap();
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(root.join("calls")).unwrap();
    log.write_all(format!("{executable}\t{}\n", args[1..].join("\t")).as_bytes()).unwrap();
    if executable == "helm" {
        assert_eq!(&args[1..4], ["upgrade", "--install", "first"]);
        if root.join("replace-entry").exists() {
            let path = std::fs::read_to_string(root.join("replace-entry")).unwrap();
            let replacement = root.join("replacement");
            std::fs::write(&replacement, b"untrusted replacement after admission").unwrap();
            std::fs::rename(&replacement, path.trim()).unwrap();
        }
        std::fs::copy(&args[4], root.join("helm-chart")).unwrap();
        std::fs::write(root.join("helm-path"), &args[4]).unwrap();
        return;
    }
    if root.join("trap").exists() { std::process::exit(91); }
    assert_eq!(args.len(), 6, "explicit separate argv");
    assert_eq!(&args[2..4], ["fetch", "--output"]);
    assert!(args[1] == "manifest" || args[1] == "blob");
    if root.join("stall-after-manifest").exists() && args[1] == "manifest" {
        std::thread::sleep(std::time::Duration::from_secs(15));
    }
    if root.join("stall").exists() || (root.join("stall-after-manifest").exists() && args[1] == "blob") {
        std::fs::write(root.join("child-pid"), std::process::id().to_string()).unwrap();
        loop { std::thread::sleep(std::time::Duration::from_millis(20)); }
    }
    if root.join("barrier").exists() && args[1] == "manifest" {
        std::fs::write(root.join(format!("writer-{}", std::process::id())), b"ready").unwrap();
        let start = std::time::Instant::now();
        while std::fs::read_dir(&root).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with("writer-")).count() < 2 {
            assert!(start.elapsed().as_secs() < 10, "second real writer did not arrive");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    if root.join("diagnostic-flood").exists() {
        for _ in 0..32 { std::io::stdout().write_all(&[b'o'; 8192]).unwrap(); std::io::stderr().write_all(&[b'e'; 8192]).unwrap(); }
        std::process::exit(92);
    }
    let requests = std::fs::read_to_string(root.join("requests")).unwrap();
    let mut found = None;
    for line in requests.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields[0] == args[1] && fields[1] == args[5] { found = Some(root.join(fields[2])); break; }
    }
    let source = found.expect("unrequested operation or repository/digest");
    if root.join("missing-output").exists() { return; }
    if root.join("nonzero").exists() { std::process::exit(93); }
    std::fs::copy(source, &args[4]).unwrap();
}
