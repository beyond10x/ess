//! Source attack against the accepted original-byte OCI cache binding.
use ess_deployment::Digest;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
#[path = "support/bundle_fixture.rs"]
mod bundle_fixture;
const OCI: &str = "application/vnd.oci.image.manifest.v1+json";
const HELM: &str = "application/vnd.cncf.helm.config.v1+json";
const CHART: &str = "application/vnd.cncf.helm.chart.content.v1.tar+gzip";
const PROV: &str = "application/vnd.cncf.helm.chart.provenance.v1.prov";
const BUNDLE: &str = "application/vnd.beyond10x.ess.release-bundle.v1";
const BUNDLE_LAYER: &str = "application/vnd.beyond10x.ess.release-bundle.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
const REPO: &str = "registry.invalid/independent";
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "oci-attack-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
        c.env("PATH", clients()).env("ESS_CACHE_ATTACK", &self.0);
        c
    }
    fn save(&self, label: &str, output: &Output) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        for (suffix, bytes) in [
            ("stdout", output.stdout.as_slice()),
            ("stderr", output.stderr.as_slice()),
        ] {
            std::fs::write(self.0.join(format!("{label}-{n}.{suffix}")), bytes).unwrap();
        }
        std::fs::write(
            self.0.join(format!("{label}-{n}.exit")),
            output.status.to_string(),
        )
        .unwrap();
    }
    fn calls(&self) -> Vec<Vec<String>> {
        std::fs::read_to_string(self.0.join("calls"))
            .unwrap_or_default()
            .lines()
            .map(|l| l.split('\t').map(str::to_owned).collect())
            .collect()
    }
}
fn clients() -> &'static Path {
    static CLIENTS: OnceLock<PathBuf> = OnceLock::new();
    CLIENTS
        .get_or_init(|| {
            let f = Fixture::new();
            let out = Command::new("rustc")
                .args(["--edition=2021", "-C", "debuginfo=0"])
                .arg(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("tests/support/cache_origin_attack_client.rs"),
                )
                .arg("-o")
                .arg(f.0.join("oras"))
                .output()
                .unwrap();
            f.save("client-build", &out);
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            std::fs::copy(f.0.join("oras"), f.0.join("helm")).unwrap();
            f.0
        })
        .as_path()
}
fn descriptor(kind: &str, bytes: &[u8]) -> String {
    format!(
        r#"{{"mediaType":"{kind}","size":{},"digest":"{}"}}"#,
        bytes.len(),
        Digest::of_bytes(bytes)
    )
}
#[derive(Clone, Copy)]
enum Content {
    Original,
    Empty,
}

struct Graph {
    raw: String,
    blobs: Vec<Vec<u8>>,
    bundle: bool,
    payload: usize,
}
impl Graph {
    fn new(bundle: bool, reverse: bool, content: Content, optional: bool) -> Self {
        let (config, chart) = if bundle {
            (
                b"{}".to_vec(),
                bundle_fixture::persisted_bundle()
                    .to_canonical_json()
                    .into_bytes(),
            )
        } else if matches!(content, Content::Empty) {
            (vec![], vec![])
        } else {
            (b"config\0original".to_vec(), b"chart\0original".to_vec())
        };
        let mut config_desc = descriptor(if bundle { EMPTY } else { HELM }, &config);
        if bundle && optional {
            config_desc.insert_str(1, r#""data":"e30=","#);
        }
        let mut blobs = vec![config, chart];
        let mut layers = vec![descriptor(
            if bundle { BUNDLE_LAYER } else { CHART },
            &blobs[1],
        )];
        if !bundle {
            let prov = if matches!(content, Content::Empty) {
                vec![]
            } else {
                b"provenance\0original".to_vec()
            };
            layers.push(descriptor(PROV, &prov));
            blobs.push(prov);
            if reverse {
                layers.reverse();
                blobs[1..].reverse();
            }
        }
        let artifact = if bundle {
            format!(r#", "artifactType":"{BUNDLE}""#)
        } else if optional {
            format!(r#", "artifactType":"{HELM}""#)
        } else {
            String::new()
        };
        let raw = format!(" \r\n{{\"schemaVersion\":2,\"mediaType\":\"{OCI}\"{artifact},\"config\":{config_desc},\"layers\":[{}]}}\t\n", layers.join(","));
        Self {
            raw,
            blobs,
            bundle,
            payload: if !bundle && reverse { 2 } else { 1 },
        }
    }
    fn digest(&self) -> Digest {
        Digest::of_bytes(self.raw.as_bytes())
    }
    fn entry(&self, f: &Fixture) -> PathBuf {
        f.0.join("cache/oci-proof-v1")
            .join(if self.bundle { "bundle" } else { "helm" })
            .join("sha256")
            .join(format!(
                "{}.entry",
                self.digest().as_str().strip_prefix("sha256:").unwrap()
            ))
    }
    fn proof(&self) -> Vec<u8> {
        let mut b = b"ESSOCI1\n".to_vec();
        b.extend((self.raw.len() as u64).to_be_bytes());
        b.extend(self.raw.as_bytes());
        b.extend(u32::try_from(self.blobs.len()).unwrap().to_be_bytes());
        for blob in &self.blobs {
            b.extend((blob.len() as u64).to_be_bytes());
            b.extend(blob);
        }
        b
    }
    fn install(&self, f: &Fixture) {
        std::fs::write(f.0.join("manifest"), &self.raw).unwrap();
        let mut requests = format!("manifest\t{REPO}@{}\tmanifest\n", self.digest());
        for (i, blob) in self.blobs.iter().enumerate() {
            std::fs::write(f.0.join(format!("blob-{i}")), blob).unwrap();
            writeln!(
                requests,
                "blob\t{REPO}@{}\tblob-{i}",
                Digest::of_bytes(blob)
            )
            .unwrap();
        }
        std::fs::write(f.0.join("requests"), requests).unwrap();
    }
    fn warm(&self, f: &Fixture, proof: &[u8]) {
        let entry = self.entry(f);
        std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
        std::fs::write(entry, proof).unwrap();
        std::fs::write(f.0.join("trap"), b"").unwrap();
    }
    fn plan(&self) -> serde_json::Value {
        let d = self.digest().to_string();
        serde_json::json!({"format":"ess-deployment/1","environment":"attack","stack_digest":d,"cluster":"test-cluster","rollout_order":["first"],"releases":{"first":{"service":"first","release_name":"first","namespace":"test","service_account":"default","chart":{"build_output":"chart","kind":"helm_chart","reference":format!("oci://{REPO}"),"digest":d},"images":{"app":{"build_output":"app","kind":"oci_image","reference":"registry.invalid/app","digest":d,"platforms":{"linux/amd64":d}}}}}})
    }
    fn run(&self, f: &Fixture) -> Output {
        let mut c = f.command();
        if self.bundle {
            c.args(["release", "fetch", "--from"])
                .arg(format!("{REPO}@{}", self.digest()))
                .arg("--out")
                .arg(f.0.join("out"));
        } else {
            let path = f.0.join("plan.json");
            std::fs::write(&path, serde_json::to_vec(&self.plan()).unwrap()).unwrap();
            c.args(["deployment", "reconcile", "--path"]).arg(path);
        }
        c.arg("--cache").arg(f.0.join("cache"));
        let output = c.output().unwrap();
        f.save("cli", &output);
        output
    }
    fn refuse(&self, f: &Fixture, reason: &str, existing: bool) -> Output {
        if existing && self.bundle {
            std::fs::write(f.0.join("out"), b"sentinel").unwrap();
        }
        let out = self.run(f);
        assert!(
            !out.status.success(),
            "admitted unsupported input: {}",
            self.raw
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(reason),
            "expected {reason}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        if self.bundle && existing {
            assert_eq!(std::fs::read(f.0.join("out")).unwrap(), b"sentinel");
        } else {
            assert!(!f.0.join("out").exists());
        }
        assert!(!f.0.join("consumed-first").exists());
        out
    }
    fn success(&self, f: &Fixture, out: &Output) {
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            std::fs::read(f.0.join(if self.bundle { "out" } else { "consumed-first" })).unwrap(),
            self.blobs[self.payload]
        );
    }
    fn insert(&mut self, location: &str, fields: &str) {
        let marker = match location {
            "root" => "{\"schemaVersion\"",
            "config" => "\"config\":{",
            "chart" => "\"layers\":[{",
            _ => panic!("location"),
        };
        let pos =
            self.raw.find(marker).unwrap() + if location == "root" { 1 } else { marker.len() };
        self.raw.insert_str(pos, fields);
    }
}

#[test]
fn decoded_optional_duplicates_and_nulls_are_refused_cold_and_warm() {
    for bundle in [false, true] {
        for location in ["root", "config", "chart"] {
            for fields in [
                r#""annotations":{"a":"x","\u0061":"y"},"#,
                r#""annotations":{},"\u0061nnotations":{},"#,
                r#""annotations":null,"#,
                r#""annotations":{"a":null},"#,
                r#""annotations":{"a":{}},"#,
            ] {
                for warm in [false, true] {
                    let mut g = Graph::new(bundle, false, Content::Original, true);
                    g.insert(location, fields);
                    let f = Fixture::new();
                    if warm {
                        g.warm(&f, &g.proof());
                    } else {
                        g.install(&f);
                    }
                    g.refuse(&f, "decoding closed OCI manifest profile", warm);
                    assert_eq!(f.calls().len(), usize::from(!warm));
                }
            }
        }
        for (location, fields) in [
            ("root", r#""artifactType":null,"#),
            ("config", r#""data":null,"#),
            ("chart", r#""data":null,"#),
        ] {
            let mut g = Graph::new(bundle, false, Content::Original, false);
            g.insert(location, fields);
            let f = Fixture::new();
            g.install(&f);
            g.refuse(&f, "decoding closed OCI manifest profile", true);
            assert_eq!(f.calls().len(), 1);
        }
    }
}

#[test]
fn original_profiles_and_decoded_utf8_limits_keep_exact_owned_bytes() {
    for bundle in [false, true] {
        for optional in [false, true] {
            for reverse in [false, true] {
                let g = Graph::new(
                    bundle,
                    reverse,
                    if bundle {
                        Content::Original
                    } else {
                        Content::Empty
                    },
                    optional,
                );
                let f = Fixture::new();
                g.install(&f);
                g.success(&f, &g.run(&f));
                assert_eq!(std::fs::read(g.entry(&f)).unwrap(), g.proof());
                let count = f.calls().len();
                std::fs::write(f.0.join("trap"), b"").unwrap();
                g.success(&f, &g.run(&f));
                assert_eq!(f.calls().len(), count + usize::from(!bundle));
            }
        }
    }
    for location in ["root", "config", "chart"] {
        for excess in [false, true] {
            let mut g = Graph::new(false, true, Content::Original, true);
            let fields = format!(
                "\"annotations\":{{{}:{}}},",
                serde_json::to_string(&"é".repeat(2048 + usize::from(excess))).unwrap(),
                serde_json::to_string(&"😀".repeat(2048)).unwrap()
            );
            g.insert(location, &fields);
            let f = Fixture::new();
            g.install(&f);
            if excess {
                g.refuse(&f, "annotation limit exceeded", false);
                assert_eq!(f.calls().len(), 1);
            } else {
                g.success(&f, &g.run(&f));
            }
        }
    }
}

#[test]
fn warm_frame_extremes_and_cross_profile_entries_never_fetch_or_consume() {
    for bundle in [false, true] {
        let g = Graph::new(bundle, true, Content::Original, true);
        let full = g.proof();
        let count = 16 + g.raw.len();
        let mut offsets = vec![0, 7, 8, 15, 16, count - 1, count, count + 3];
        let mut cursor = count + 4;
        for blob in &g.blobs {
            offsets.extend([cursor, cursor + 7, cursor + 8, cursor + 8 + blob.len() - 1]);
            cursor += 8 + blob.len();
        }
        for n in offsets {
            let f = Fixture::new();
            let bytes = &full[..n];
            g.warm(&f, bytes);
            g.refuse(&f, "truncated OCI proof", n % 2 == 0);
            assert_eq!(std::fs::read(g.entry(&f)).unwrap(), bytes);
            assert!(f.calls().is_empty());
        }
        for n in [0, u32::MAX] {
            let mut b = full.clone();
            b[count..count + 4].copy_from_slice(&n.to_be_bytes());
            let f = Fixture::new();
            g.warm(&f, &b);
            g.refuse(&f, "blob count mismatch", true);
            assert_eq!(std::fs::read(g.entry(&f)).unwrap(), b);
        }
        let mut cursor = count + 4;
        for (i, blob) in g.blobs.iter().enumerate() {
            let limit = if i == 0 || (!bundle && i != g.payload) {
                1024 * 1024
            } else if bundle {
                32 * 1024 * 1024
            } else {
                64 * 1024 * 1024
            };
            for n in [limit + 1, u64::MAX] {
                let mut b = full.clone();
                b[cursor..cursor + 8].copy_from_slice(&n.to_be_bytes());
                let f = Fixture::new();
                g.warm(&f, &b);
                g.refuse(&f, "frame length exceeds local limit", true);
                assert!(f.calls().is_empty());
            }
            cursor += 8 + blob.len();
        }
        let mut other = Graph::new(!bundle, false, Content::Original, true);
        other.bundle = bundle;
        let f = Fixture::new();
        other.warm(&f, &other.proof());
        other.refuse(
            &f,
            if bundle {
                "unsupported bundle artifactType"
            } else {
                "unsupported Helm artifactType"
            },
            false,
        );
        assert!(f.calls().is_empty());
        let mut b = full.clone();
        b.extend(full);
        let f = Fixture::new();
        g.warm(&f, &b);
        g.refuse(&f, "trailing bytes", true);
        assert!(f.calls().is_empty());
    }
}

#[test]
fn integer_layer_tokens_refuse_before_any_fetch_of_a_blob() {
    for bundle in [false, true] {
        for token in [
            "-0",
            "-1",
            "0.0",
            "0e0",
            "18446744073709551616",
            "9007199254740993.0",
            "null",
            "true",
        ] {
            let mut g = Graph::new(bundle, true, Content::Original, true);
            let pos = g.raw.find("\"layers\":[{").unwrap();
            let rest = &g.raw[pos..];
            let size = rest.find("\"size\":").unwrap() + pos + 7;
            let end = g.raw[size..].find(',').unwrap() + size;
            g.raw.replace_range(size..end, token);
            let f = Fixture::new();
            g.install(&f);
            g.refuse(&f, "decoding closed OCI manifest profile", true);
            assert_eq!(f.calls().len(), 1);
        }
    }
}

#[test]
fn late_blob_outputs_must_be_regular_complete_and_successful() {
    for bundle in [false, true] {
        for mode in ["exit", "missing", "directory", "symlink", "truncated"] {
            for existing in [false, true] {
                let g = Graph::new(bundle, false, Content::Original, true);
                let f = Fixture::new();
                g.install(&f);
                let selected = if bundle { "blob-1" } else { "blob-2" };
                std::fs::write(f.0.join("fail"), format!("{selected}\t{mode}")).unwrap();
                let reason = match mode {
                    "exit" => "ORAS fetch failed",
                    "missing" => "No such file",
                    "directory" | "symlink" => "must be a regular file",
                    _ => "descriptor size mismatch",
                };
                g.refuse(&f, reason, existing);
                assert_eq!(f.calls().len(), if bundle { 2 } else { 4 });
                assert!(!g.entry(&f).exists());
                assert_eq!(
                    std::fs::read_dir(g.entry(&f).parent().unwrap())
                        .unwrap()
                        .count(),
                    0
                );
            }
        }
    }
    let g = Graph::new(false, false, Content::Original, true);
    let f = Fixture::new();
    g.install(&f);
    std::fs::write(f.0.join("fail"), b"blob-2\tflood").unwrap();
    g.success(&f, &g.run(&f));
    assert_eq!(f.calls().len(), 5);
}

#[test]
fn a_late_complete_or_corrupt_winner_is_preserved_without_replacement() {
    for bundle in [false, true] {
        for corrupt in [false, true] {
            let mut g = Graph::new(bundle, false, Content::Original, true);
            if !bundle {
                let p = g.raw.rfind(",{\"mediaType\"").unwrap();
                let end = g.raw[p..].find(']').unwrap() + p;
                g.raw.replace_range(p..end, "");
                g.blobs.pop();
            }
            let f = Fixture::new();
            g.install(&f);
            let mut winner = g.proof();
            if corrupt {
                winner[0] ^= 1;
            }
            std::fs::write(f.0.join("winner-bytes"), &winner).unwrap();
            std::fs::write(
                f.0.join("winner-target"),
                g.entry(&f).to_string_lossy().as_bytes(),
            )
            .unwrap();
            if corrupt {
                g.refuse(&f, "unsupported OCI proof magic", true);
            } else {
                g.success(&f, &g.run(&f));
            }
            assert_eq!(std::fs::read(g.entry(&f)).unwrap(), winner);
            assert_eq!(
                std::fs::read_dir(g.entry(&f).parent().unwrap())
                    .unwrap()
                    .count(),
                1
            );
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn a_stall_on_final_provenance_uses_the_original_deadline_and_is_reaped() {
    let g = Graph::new(false, false, Content::Original, true);
    let f = Fixture::new();
    g.install(&f);
    std::fs::write(f.0.join("deadline"), b"").unwrap();
    let start = std::time::Instant::now();
    g.refuse(&f, "owned ORAS child killed and reaped", false);
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() >= 59.5 && elapsed.as_secs() < 66,
        "{elapsed:?}"
    );
    let pid = std::fs::read_to_string(f.0.join("stalled-pid")).unwrap();
    let dead = !Path::new("/proc").join(pid.trim()).exists();
    std::fs::write(
        f.0.join("process-readback"),
        format!(
            "elapsed={elapsed:?}\npid={}\nproc_absent={dead}\n",
            pid.trim()
        ),
    )
    .unwrap();
    assert!(dead, "owned process remained live or zombie");
    assert_eq!(f.calls().len(), 4);
    assert!(!g.entry(&f).exists());
}

#[test]
fn self_consistent_other_original_bytes_cannot_replace_the_requested_identity() {
    for bundle in [false, true] {
        for whitespace in [false, true] {
            for warm in [false, true] {
                for existing in [false, true] {
                    let g = Graph::new(bundle, true, Content::Original, true);
                    let mut other = Graph::new(bundle, true, Content::Original, true);
                    if whitespace {
                        other.raw.push(' ');
                    } else {
                        other.insert(
                            "root",
                            r#""annotations":{"origin":"other self-consistent object"},"#,
                        );
                    }
                    assert_ne!(g.digest(), other.digest());
                    let f = Fixture::new();
                    if warm {
                        g.warm(&f, &other.proof());
                    } else {
                        g.install(&f);
                        std::fs::write(f.0.join("manifest"), &other.raw).unwrap();
                    }
                    g.refuse(&f, "requested OCI manifest digest mismatch", existing);
                    assert_eq!(f.calls().len(), usize::from(!warm));
                    if warm {
                        assert_eq!(std::fs::read(g.entry(&f)).unwrap(), other.proof());
                    } else {
                        assert!(!g.entry(&f).exists());
                    }
                }
            }
        }
    }
}

#[test]
fn offline_helm_consumes_the_owned_snapshot_after_shared_entry_replacement() {
    let g = Graph::new(false, true, Content::Original, true);
    let f = Fixture::new();
    g.warm(&f, &g.proof());
    std::fs::write(
        f.0.join("replace-target"),
        g.entry(&f).to_string_lossy().as_bytes(),
    )
    .unwrap();
    g.success(&f, &g.run(&f));
    assert_eq!(f.calls().len(), 1);
    assert_eq!(f.calls()[0][0], "helm");
    assert_eq!(
        std::fs::read(g.entry(&f)).unwrap(),
        b"changed after verification"
    );
    let snapshot = std::fs::read_to_string(f.0.join("snapshot-first")).unwrap();
    assert!(!Path::new(&snapshot).starts_with(f.0.join("cache")));
    assert!(
        !Path::new(&snapshot).exists(),
        "snapshot outlived executor completion"
    );
    let second = g.run(&f);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("unsupported OCI proof magic"));
    assert_eq!(f.calls().len(), 1);
    assert_eq!(
        std::fs::read(f.0.join("consumed-first")).unwrap(),
        g.blobs[g.payload]
    );
}
