//! Test-only finite external release tools. Unknown calls refuse; ESS/Bash/jq stay real.
use std::{
    env, fs,
    io::{Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixStream},
    path::{Path, PathBuf},
};
fn digest(n: char) -> String {
    format!("sha256:{}", n.to_string().repeat(64))
}
fn main() {
    let args: Vec<_> = env::args().collect();
    let tool = Path::new(&args[0]).file_name().unwrap().to_str().unwrap();
    let root = PathBuf::from(env::var_os("ESS_RELEASE_FIXTURE").expect("fixture root"));
    let calls = root.join("calls");
    let index = fs::read_dir(&calls).unwrap().count();
    let call = calls.join(format!("{index:03}-{tool}"));
    fs::create_dir(&call).unwrap();
    fs::write(call.join("argv"), args[1..].join("\n")).unwrap();
    fs::write(
        call.join("cwd"),
        env::current_dir().unwrap().display().to_string(),
    )
    .unwrap();
    fs::write(
        call.join("cwd-mode"),
        (fs::metadata(env::current_dir().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777)
            .to_string(),
    )
    .unwrap();
    let mode = env::var("ESS_RELEASE_MODE").unwrap_or_default();
    let result = run(tool, &args[1..], &root, &call, &mode);
    let (status, stdout, stderr) = result.unwrap_or_else(|| {
        (
            97,
            Vec::new(),
            format!(
                "fixture refused unknown invocation: {tool} {:?}\n",
                &args[1..]
            )
            .into_bytes(),
        )
    });
    fs::write(call.join("status"), status.to_string()).unwrap();
    fs::write(call.join("stdout.raw"), &stdout).unwrap();
    fs::write(call.join("stderr.raw"), &stderr).unwrap();
    std::io::stdout().write_all(&stdout).unwrap();
    std::io::stderr().write_all(&stderr).unwrap();
    std::process::exit(status);
}
type ResultBytes = Option<(i32, Vec<u8>, Vec<u8>)>;
fn ok(text: impl Into<Vec<u8>>) -> ResultBytes {
    Some((0, text.into(), Vec::new()))
}
fn run(tool: &str, a: &[String], root: &Path, call: &Path, mode: &str) -> ResultBytes {
    let args: Vec<_> = a.iter().map(String::as_str).collect();
    match (tool, args.as_slice()) {
        (
            "oras",
            ["push", "--no-tty", "--artifact-type", kind, "--format", "go-template={{.digest}}", destination, payload],
        ) => {
            let expected = match *kind {
                "application/vnd.beyond10x.ess.evidence.conformance.v1" => {
                    if mode == "legacy-conformance" {
                        "text/plain"
                    } else {
                        "application/json"
                    }
                }
                "application/vnd.beyond10x.ess.evidence.provenance.v1" => "application/json",
                "application/vnd.beyond10x.ess.evidence.sbom.v1" => {
                    "application/vnd.cyclonedx+json"
                }
                "application/vnd.beyond10x.ess.evidence.signature.v1" => "application/json",
                "application/vnd.beyond10x.ess.release-bundle.v1" => {
                    "application/vnd.beyond10x.ess.release-bundle.v1+json"
                }
                _ => return None,
            };
            if !destination.starts_with("registry.example/") || !destination.contains(':') {
                return None;
            }
            let (path, media) = payload.rsplit_once(':')?;
            if media != expected {
                return None;
            }
            if let Ok(socket) = env::var("ESS_RELEASE_SYNC") {
                let mut s = UnixStream::connect(socket).unwrap();
                s.write_all(&[1]).unwrap();
                let mut ack = [0];
                s.read_exact(&mut ack).unwrap();
                assert_eq!(ack, [2]);
            }
            fs::write(call.join("payload.raw"), fs::read(path).ok()?).unwrap();
            fs::write(call.join("media-type"), media).unwrap();
            if mode == "fail-oras" || (mode == "fail-provenance" && kind.contains("provenance")) {
                return Some((23, Vec::new(), b"finite publisher failed\n".to_vec()));
            }
            if mode == "bad-digest" {
                return ok("invalid-digest\n");
            }
            if mode == "non-utf8" {
                return ok(vec![255]);
            }
            if mode == "mutate-after-conformance" && kind.contains("conformance") {
                let snapshots: Vec<_> = fs::read_dir(root.join("target/release"))
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .filter(|p| {
                        p.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with("conformance-inputs.")
                    })
                    .collect();
                assert_eq!(snapshots.len(), 1);
                fs::write(
                    snapshots[0].join("conformance-report.json"),
                    b"changed after upload",
                )
                .unwrap();
            }
            ok(format!("{}\n", digest('9')))
        }
        (
            "docker",
            ["buildx", "imagetools", "inspect", reference, "--format", "{{json .Manifest.Digest}}"],
        ) if reference.starts_with("registry.example/") => ok(format!(
            "\"{}\"\n",
            digest(if reference.ends_with(".sig") {
                '7'
            } else {
                '1'
            })
        )),
        ("docker", ["buildx", "imagetools", "inspect", reference, "--raw"])
            if reference.starts_with("registry.example/") =>
        {
            if mode == "platform-fallback" {
                ok("{\"schemaVersion\":2}\n")
            } else {
                ok(format!("{{\"manifests\":[{{\"platform\":{{\"os\":\"linux\",\"architecture\":\"amd64\"}},\"digest\":\"{}\"}}]}}\n",digest('2')))
            }
        }
        ("docker", ["buildx", "bake", "--file", bake, rest @ ..]) => {
            if !Path::new(bake).is_file() {
                return None;
            }
            let mut remaining = rest.iter();
            let mut target = None;
            while let Some(arg) = remaining.next() {
                match *arg {
                    "--set" => {
                        let v = remaining.next()?;
                        if !(v.starts_with("*.context=")
                            || v.starts_with("*.dockerfile=")
                            || v.starts_with("app.tags=registry.example/"))
                        {
                            return None;
                        }
                    }
                    "--push" => {}
                    "app" | "chart" => {
                        if target.replace(*arg).is_some() {
                            return None;
                        }
                    }
                    _ => return None,
                }
            }
            match target? {
                "app" => ok("finite image build\n"),
                "chart" => {
                    let p = root.join("out/chart");
                    fs::create_dir_all(&p).unwrap();
                    fs::write(p.join("oracle-chart.tgz"), b"finite chart artifact").unwrap();
                    ok("finite chart build\n")
                }
                _ => None,
            }
        }
        ("helm", ["show", "chart", path]) if Path::new(path).is_file() => {
            ok("name: oracle-chart\nversion: 1.2.3\n")
        }
        ("helm", ["push", path, "oci://registry.example/charts"]) if Path::new(path).is_file() => {
            if mode == "chart-no-digest" {
                ok("Pushed: registry.example/charts/oracle-chart:1.2.3\n")
            } else {
                Some((
                    0,
                    Vec::new(),
                    format!(
                        "Pushed: registry.example/charts/oracle-chart:1.2.3\nDigest: {}\n",
                        digest('3')
                    )
                    .into_bytes(),
                ))
            }
        }
        ("cosign", ["sign", "--yes", reference])
            if reference.starts_with("registry.example/") && reference.contains("@sha256:") =>
        {
            ok("opaque signing tool success\n")
        }
        ("cosign", ["triangulate", reference])
            if reference.starts_with("registry.example/") && reference.contains("@sha256:") =>
        {
            ok(format!("{}.sig\n", reference.replace('@', ":")))
        }
        ("syft", ["scan", reference, "-o", "cyclonedx-json=target/release/sbom.json"])
            if reference.starts_with("registry.example/") && reference.contains("@sha256:") =>
        {
            fs::write(
                root.join("target/release/sbom.json"),
                b"{\"opaque\":\"finite SBOM fixture\"}\n",
            )
            .unwrap();
            ok(Vec::new())
        }
        _ => None,
    }
}
