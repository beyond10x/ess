//! Independent local process peer for cache-origin source attacks.
use std::io::Write;
use std::path::{Path, PathBuf};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(std::env::var_os("ESS_CACHE_ATTACK").unwrap());
    let helm = Path::new(&args[0]).file_stem().unwrap() == "helm";
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("calls"))
        .unwrap();
    log.write_all(
        format!(
            "{}\t{}\n",
            if helm { "helm" } else { "oras" },
            args[1..].join("\t")
        )
        .as_bytes(),
    )
    .unwrap();
    if helm {
        assert_eq!(&args[1..3], ["upgrade", "--install"]);
        assert_eq!(args.len(), 16);
        assert_eq!(&args[5..8], ["--namespace", "test", "--create-namespace"]);
        assert_eq!(&args[8..11], ["--kube-context", "test-cluster", "--values"]);
        assert_eq!(&args[12..16], ["--atomic", "--wait", "--timeout", "5m"]);
        if root.join("replace-target").exists() {
            let target = std::fs::read_to_string(root.join("replace-target")).unwrap();
            std::fs::write(root.join("replacement"), b"changed after verification").unwrap();
            std::fs::rename(root.join("replacement"), target).unwrap();
        }
        std::fs::copy(&args[4], root.join(format!("consumed-{}", args[3]))).unwrap();
        std::fs::write(root.join(format!("snapshot-{}", args[3])), &args[4]).unwrap();
        return;
    }
    assert!(!root.join("trap").exists(), "offline trap was invoked");
    assert_eq!(args.len(), 6);
    assert_eq!(&args[2..4], ["fetch", "--output"]);
    let table = std::fs::read_to_string(root.join("requests")).unwrap();
    let row = table
        .lines()
        .find_map(|line| {
            let f: Vec<_> = line.split('\t').collect();
            (f[0] == args[1] && f[1] == args[5]).then(|| f[2].to_owned())
        })
        .expect("unexpected operation, repository or digest");
    if root.join("deadline").exists() {
        let delay = match row.as_str() {
            "manifest" => 20,
            "blob-0" => 10,
            "blob-1" => 5,
            _ => 0,
        };
        if row == "blob-2" {
            std::fs::write(root.join("stalled-pid"), std::process::id().to_string()).unwrap();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(delay));
    }
    if let Ok(control) = std::fs::read_to_string(root.join("fail")) {
        let (selected, mode) = control.trim().split_once('\t').unwrap();
        if row == selected {
            match mode {
                "exit" => std::process::exit(79),
                "missing" => return,
                "directory" => {
                    std::fs::create_dir(&args[4]).unwrap();
                    return;
                }
                #[cfg(unix)]
                "symlink" => {
                    std::os::unix::fs::symlink(root.join(&row), &args[4]).unwrap();
                    return;
                }
                "truncated" => {
                    std::fs::write(&args[4], []).unwrap();
                    return;
                }
                "flood" => {
                    for _ in 0..128 {
                        std::io::stdout().write_all(&[b'o'; 4096]).unwrap();
                        std::io::stderr().write_all(&[b'e'; 4096]).unwrap();
                    }
                }
                _ => panic!("unknown fixture mode"),
            }
        }
    }
    if root.join("winner-target").exists() && row == "blob-1" {
        let target = std::fs::read_to_string(root.join("winner-target")).unwrap();
        let mut winner = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .unwrap();
        winner
            .write_all(&std::fs::read(root.join("winner-bytes")).unwrap())
            .unwrap();
    }
    std::fs::copy(root.join(row), &args[4]).unwrap();
}
