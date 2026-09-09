//! The requested OCI manifest identity must survive cache admission and consumption.
use ess_deployment::Digest;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-cache-origin-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
        c.env("PATH", executors()).env("ESS_CACHE_FIXTURE", &self.0);
        c
    }
    fn helm(&self, digest: &Digest) -> Output {
        static NEXT_PLAN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let d = digest.as_str();
        let plan = serde_json::json!({"format":"ess-deployment/1", "environment":"test", "stack_digest":d, "cluster":"test-cluster", "rollout_order":["first"], "releases":{"first":{"service":"first", "release_name":"first", "namespace":"test", "service_account":"default", "images":{"app":{"build_output":"app", "kind":"oci_image", "reference":"example.invalid/app", "digest":d, "platforms":{"linux/amd64":d}}}, "chart":{"build_output":"chart", "kind":"helm_chart", "reference":"oci://example.invalid/chart", "digest":d}}}});
        let path = self.0.join(format!(
            "desired-{}.json",
            NEXT_PLAN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
        self.acquire(&path, true)
    }

    /// Acquires the pinned chart through the driver, with or without an admitted authority.
    ///
    /// `reconcile` is authority-gated now, so a vector that ran the shipped binary without one
    /// would stop at the authority and never reach the cache boundary it exists to decide — the
    /// vacuity C12 forbids. The driver is the test-only Rust adapter the offline qualification
    /// allows: it admits a *synthetic* authority through the production registry scan and then
    /// acquires the payload through the production OCI proof, so every original-byte assertion
    /// below keeps its exact meaning. `authority: false` is the control that proves it.
    fn acquire(&self, plan: &Path, authority: bool) -> Output {
        static NEXT_JOB: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let job_id = NEXT_JOB.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let job = self.0.join(format!("job-{job_id}.json"));
        // Each caller provisions its own synthetic authority before admission.
        // They still race on the same cache and fixture executors; provisioning
        // must not truncate a registry the other caller is already reading.
        let authority_root = self.0.join(format!("authority-{job_id}"));
        std::fs::write(
            &job,
            serde_json::to_vec(&serde_json::json!({
                "root": authority_root, "mode": "cache", "nonces": [],
                "fail": null, "interrupt": null, "open_journal": false,
                "plan": plan, "cache": self.0.join("cache"), "authority": authority
            }))
            .unwrap(),
        )
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_ess-recovery-driver"))
            .env("PATH", executors())
            .env("ESS_CACHE_FIXTURE", &self.0)
            .arg(&job)
            .output()
            .unwrap()
    }
}
fn executors() -> &'static Path {
    static VALUE: OnceLock<PathBuf> = OnceLock::new();
    VALUE
        .get_or_init(|| {
            let f = Fixture::new();
            let output = Command::new("rustc")
                .arg("--edition=2021")
                .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/fake_oci.rs"))
                .arg("-o")
                .arg(f.0.join("oras"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::copy(f.0.join("oras"), f.0.join("helm")).unwrap();
            f.0
        })
        .as_path()
}

#[test]
fn legacy_self_consistent_helm_cache_cannot_claim_requested_origin() {
    let f = Fixture::new();
    let requested = Digest::of_bytes(b"a different manifest");
    let legacy =
        f.0.join("cache/helm/sha256")
            .join(requested.as_str().strip_prefix("sha256:").unwrap());
    std::fs::create_dir_all(&legacy).unwrap();
    let substitute = b"self-consistent substituted chart";
    std::fs::write(legacy.join("chart.tgz"), substitute).unwrap();
    std::fs::write(
        legacy.join("payload.digest"),
        Digest::of_bytes(substitute).as_str(),
    )
    .unwrap();
    std::fs::write(f.0.join("trap"), b"").unwrap();
    let out = f.helm(&requested);
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("parsing"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "self-consistent legacy chart was admitted: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        !f.0.join("helm-chart").exists(),
        "Helm consumed an unproved chart"
    );
    assert_eq!(std::fs::read(legacy.join("chart.tgz")).unwrap(), substitute);
}

#[path = "support/bundle_fixture.rs"]
mod bundle_fixture;
const OCI: &str = "application/vnd.oci.image.manifest.v1+json";
const HELM: &str = "application/vnd.cncf.helm.config.v1+json";
const CHART: &str = "application/vnd.cncf.helm.chart.content.v1.tar+gzip";
const PROV: &str = "application/vnd.cncf.helm.chart.provenance.v1.prov";
const BUNDLE: &str = "application/vnd.beyond10x.ess.release-bundle.v1";
const JSON: &str = "application/vnd.beyond10x.ess.release-bundle.v1+json";
const EMPTY: &str = "application/vnd.oci.empty.v1+json";
fn descriptor(kind: &str, bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({"mediaType":kind, "digest":Digest::of_bytes(bytes).as_str(), "size":bytes.len()})
}
struct Graph {
    manifest: Vec<u8>,
    blobs: Vec<Vec<u8>>,
    bundle: bool,
    payload: usize,
}
impl Graph {
    fn helm(provenance: bool, reversed: bool, artifact_type: bool) -> Self {
        let config = br#"{ "name": "independent-chart", "version": "1.0.0" }"#.to_vec();
        let chart = b"\x1f\x8b independently assembled opaque chart bytes".to_vec();
        let prov = b"opaque provenance, not a verified signature".to_vec();
        let mut layers = vec![descriptor(CHART, &chart)];
        let mut blobs = vec![config.clone(), chart];
        if provenance {
            layers.push(descriptor(PROV, &prov));
            blobs.push(prov);
        }
        if reversed {
            layers.reverse();
            blobs[1..].reverse();
        }
        let mut manifest = serde_json::json!({"schemaVersion":2,"mediaType":OCI,"config":descriptor(HELM,&config),"layers":layers,"annotations":{"org.opencontainers.image.title":"../../ignored.tgz"}});
        if artifact_type {
            manifest["artifactType"] = HELM.into();
        }
        Self {
            manifest: format!(" \n{}\n ", serde_json::to_string_pretty(&manifest).unwrap())
                .into_bytes(),
            blobs,
            bundle: false,
            payload: if reversed && provenance { 2 } else { 1 },
        }
    }
    fn bundle(data: bool) -> Self {
        let payload = bundle_fixture::persisted_bundle()
            .to_canonical_json()
            .into_bytes();
        let mut config = descriptor(EMPTY, b"{}");
        if data {
            config["data"] = "e30=".into();
        }
        let manifest = serde_json::json!({"schemaVersion":2,"mediaType":OCI,"artifactType":BUNDLE,"config":config,"layers":[descriptor(JSON,&payload)]});
        Self {
            manifest: format!(" \n{}\n", serde_json::to_string_pretty(&manifest).unwrap())
                .into_bytes(),
            blobs: vec![b"{}".to_vec(), payload],
            bundle: true,
            payload: 1,
        }
    }
    fn digest(&self) -> Digest {
        Digest::of_bytes(&self.manifest)
    }
    fn edit(&mut self, edit: impl FnOnce(&mut serde_json::Value)) {
        let mut value = serde_json::from_slice(&self.manifest).unwrap();
        edit(&mut value);
        self.manifest = serde_json::to_vec(&value).unwrap();
    }
    fn repository(&self) -> &'static str {
        if self.bundle {
            "example.invalid/bundle"
        } else {
            "example.invalid/chart"
        }
    }
    fn entry(&self, f: &Fixture, requested: &Digest) -> PathBuf {
        f.0.join("cache/oci-proof-v1")
            .join(if self.bundle { "bundle" } else { "helm" })
            .join("sha256")
            .join(format!(
                "{}.entry",
                requested.as_str().strip_prefix("sha256:").unwrap()
            ))
    }
    fn proof(&self) -> Vec<u8> {
        let mut bytes = b"ESSOCI1\n".to_vec();
        bytes.extend((self.manifest.len() as u64).to_be_bytes());
        bytes.extend(&self.manifest);
        bytes.extend(u32::try_from(self.blobs.len()).unwrap().to_be_bytes());
        for blob in &self.blobs {
            bytes.extend((blob.len() as u64).to_be_bytes());
            bytes.extend(blob);
        }
        bytes
    }
    fn install(&self, f: &Fixture, requested: &Digest) {
        std::fs::write(f.0.join("manifest"), &self.manifest).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&self.manifest).unwrap_or(serde_json::Value::Null);
        let mut requests = format!("manifest\t{}@{}\tmanifest\n", self.repository(), requested);
        for (index, blob) in self.blobs.iter().enumerate() {
            let name = format!("blob-{index}");
            std::fs::write(f.0.join(&name), blob).unwrap();
            let d = if index == 0 {
                &value["config"]["digest"]
            } else {
                &value["layers"][index - 1]["digest"]
            };
            if let Some(d) = d.as_str() {
                writeln!(requests, "blob\t{}@{d}\t{name}", self.repository()).unwrap();
            }
        }
        std::fs::write(f.0.join("requests"), requests).unwrap();
    }
    fn run(&self, f: &Fixture, requested: &Digest) -> Output {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let output = if self.bundle {
            f.command()
                .args(["release", "fetch", "--from"])
                .arg(format!("{}@{requested}", self.repository()))
                .arg("--cache")
                .arg(f.0.join("cache"))
                .arg("--out")
                .arg(f.0.join("out.json"))
                .output()
                .unwrap()
        } else {
            f.helm(requested)
        };
        // Retain exact process outputs and fixture bytes under the assigned TMPDIR.
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::fs::write(f.0.join(format!("result-{n}.stdout")), &output.stdout).unwrap();
        std::fs::write(f.0.join(format!("result-{n}.stderr")), &output.stderr).unwrap();
        std::fs::write(
            f.0.join(format!("result-{n}.status")),
            output.status.to_string(),
        )
        .unwrap();
        output
    }
    fn consumed(&self, f: &Fixture) -> PathBuf {
        f.0.join(if self.bundle {
            "out.json"
        } else {
            "helm-chart"
        })
    }
    fn assert_success(&self, f: &Fixture, output: &Output) {
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read(self.consumed(f)).unwrap(),
            self.blobs[self.payload]
        );
    }
    fn assert_refusal(&self, f: &Fixture, requested: &Digest, reason: &str, existing: bool) {
        if existing && self.bundle {
            std::fs::write(self.consumed(f), b"output sentinel").unwrap();
        }
        let output = self.run(f, requested);
        assert!(!output.status.success(), "admitted invalid graph");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(reason),
            "expected boundary {reason:?}, got {error}"
        );
        if existing && self.bundle {
            assert_eq!(std::fs::read(self.consumed(f)).unwrap(), b"output sentinel");
        } else {
            assert!(!self.consumed(f).exists());
        }
        assert!(!std::fs::read_to_string(f.0.join("calls"))
            .unwrap_or_default()
            .lines()
            .any(|l| l.starts_with("helm\t")));
    }
}
fn calls(f: &Fixture) -> Vec<Vec<String>> {
    std::fs::read_to_string(f.0.join("calls"))
        .unwrap_or_default()
        .lines()
        .map(|l| l.split('\t').map(str::to_owned).collect())
        .collect()
}
fn assert_cold_calls(g: &Graph, f: &Fixture, requested: &Digest) {
    let calls = calls(f);
    let oras: Vec<_> = calls.iter().filter(|c| c[0] == "oras").collect();
    let value: serde_json::Value = serde_json::from_slice(&g.manifest).unwrap();
    let mut expected = vec![("manifest", requested.as_str().to_owned())];
    if !g.bundle {
        expected.push((
            "blob",
            value["config"]["digest"].as_str().unwrap().to_owned(),
        ));
    }
    for layer in value["layers"].as_array().unwrap() {
        expected.push(("blob", layer["digest"].as_str().unwrap().to_owned()));
    }
    assert_eq!(oras.len(), expected.len());
    for (actual, (operation, digest)) in oras.iter().zip(expected) {
        assert_eq!(actual.len(), 6);
        assert_eq!(&actual[1..4], [operation, "fetch", "--output"]);
        assert_eq!(actual[5], format!("{}@{digest}", g.repository()));
        assert!(Path::new(&actual[4]).starts_with(f.0.join("cache/oci-proof-v1")));
    }
    assert_eq!(
        calls.iter().filter(|c| c[0] == "helm").count(),
        usize::from(!g.bundle)
    );
}

#[test]
fn both_finite_profiles_cold_then_offline_warm_revalidate_original_bytes() {
    for g in [
        Graph::bundle(false),
        Graph::bundle(true),
        Graph::helm(false, false, false),
        Graph::helm(false, false, true),
        Graph::helm(true, false, false),
        Graph::helm(true, true, true),
    ] {
        let f = Fixture::new();
        let requested = g.digest();
        g.install(&f, &requested);
        g.assert_success(&f, &g.run(&f, &requested));
        assert_cold_calls(&g, &f, &requested);
        assert_eq!(std::fs::read(g.entry(&f, &requested)).unwrap(), g.proof());
        let before = calls(&f).len();
        std::fs::write(f.0.join("trap"), b"").unwrap();
        g.assert_success(&f, &g.run(&f, &requested));
        assert_eq!(calls(&f).len(), before + usize::from(!g.bundle));
        if !g.bundle {
            let snapshot = std::fs::read_to_string(f.0.join("helm-path")).unwrap();
            assert!(
                !Path::new(&snapshot).exists(),
                "snapshot lifetime ends after Helm"
            );
        }
    }
}

#[test]
fn both_consumers_reject_wrong_original_manifest_even_whitespace_only() {
    for bundle in [true, false] {
        for whitespace in [true, false] {
            for existing in [true, false] {
                let mut g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(false, false, false)
                };
                let requested = g.digest();
                if whitespace {
                    g.manifest.push(b' ');
                } else {
                    g.edit(|v| {
                        v["annotations"] =
                            serde_json::json!({"changed":"self-consistent other graph"});
                    });
                }
                let f = Fixture::new();
                g.install(&f, &requested);
                g.assert_refusal(
                    &f,
                    &requested,
                    "requested OCI manifest digest mismatch",
                    existing,
                );
                assert_eq!(calls(&f).len(), 1);
                assert!(!g.entry(&f, &requested).exists());
            }
        }
    }
}

#[test]
fn descriptor_changes_and_size_disagreement_refuse_at_exact_blob_boundary() {
    for bundle in [true, false] {
        for index in 0..if bundle { 2 } else { 3 } {
            for size in [true, false] {
                let mut g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(true, false, false)
                };
                // Bundle config is fixed by profile; every other descriptor retains a valid raw identity.
                if bundle && index == 0 {
                    g.edit(|v| v["config"]["digest"] = Digest::of_bytes(b"xx").as_str().into());
                    let f = Fixture::new();
                    let requested = g.digest();
                    g.install(&f, &requested);
                    g.assert_refusal(
                        &f,
                        &requested,
                        "bundle config must identify exact empty JSON",
                        true,
                    );
                    assert_eq!(calls(&f).len(), 1);
                    continue;
                }
                if size {
                    g.blobs[index].push(b'x');
                } else {
                    g.blobs[index][0] ^= 1;
                }
                let f = Fixture::new();
                let requested = g.digest();
                g.install(&f, &requested);
                g.assert_refusal(
                    &f,
                    &requested,
                    if size {
                        "OCI descriptor size mismatch"
                    } else {
                        "OCI descriptor digest mismatch"
                    },
                    true,
                );
                assert!(!g.entry(&f, &requested).exists());
            }
        }
    }
}

#[test]
fn legacy_bundle_cache_is_ignored_and_cold_failure_preserves_both_sentinels() {
    for existing in [true, false] {
        let g = Graph::bundle(true);
        let f = Fixture::new();
        let requested = g.digest();
        let legacy =
            f.0.join("cache/sha256")
                .join(requested.as_str().strip_prefix("sha256:").unwrap())
                .join("ess-release-bundle.json");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, &g.blobs[1]).unwrap();
        std::fs::write(f.0.join("trap"), b"").unwrap();
        g.assert_refusal(&f, &requested, "ORAS fetch failed", existing);
        assert_eq!(std::fs::read(legacy).unwrap(), g.blobs[1]);
        assert_eq!(calls(&f).len(), 1);
    }
}

#[test]
fn corrupt_warm_entries_are_not_repaired_and_never_launch_a_client() {
    for bundle in [true, false] {
        for corruption in 0..8 {
            let g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(true, true, true)
            };
            let f = Fixture::new();
            let requested = g.digest();
            let mut bytes = g.proof();
            let reason = match corruption {
                0 => {
                    bytes[0] ^= 1;
                    "unsupported OCI proof magic"
                }
                1 => {
                    bytes.truncate(4);
                    "truncated OCI proof magic"
                }
                2 => {
                    bytes.truncate(12);
                    "truncated OCI proof length"
                }
                3 => {
                    bytes.truncate(bytes.len() - 1);
                    "truncated OCI proof bytes"
                }
                4 => {
                    bytes.push(0);
                    "trailing bytes in OCI proof"
                }
                5 => {
                    bytes[16] ^= 1;
                    "requested OCI manifest digest mismatch"
                }
                6 => {
                    let n = 16 + g.manifest.len();
                    bytes[n..n + 4].copy_from_slice(&99u32.to_be_bytes());
                    "OCI proof blob count mismatch"
                }
                _ => {
                    bytes[8..16].copy_from_slice(&(1024u64 * 1024 + 1).to_be_bytes());
                    "OCI proof frame length exceeds local limit"
                }
            };
            let entry = g.entry(&f, &requested);
            std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
            std::fs::write(&entry, &bytes).unwrap();
            g.assert_refusal(&f, &requested, reason, true);
            assert_eq!(std::fs::read(entry).unwrap(), bytes);
            assert!(calls(&f).is_empty());
        }
    }
}

#[test]
fn shared_cache_replacement_cannot_change_helm_private_snapshot() {
    let g = Graph::helm(false, false, false);
    let f = Fixture::new();
    let requested = g.digest();
    g.install(&f, &requested);
    std::fs::write(
        f.0.join("replace-entry"),
        g.entry(&f, &requested).to_string_lossy().as_bytes(),
    )
    .unwrap();
    g.assert_success(&f, &g.run(&f, &requested));
    assert_eq!(
        std::fs::read(g.entry(&f, &requested)).unwrap(),
        b"untrusted replacement after admission"
    );
    let snapshot = std::fs::read_to_string(f.0.join("helm-path")).unwrap();
    assert!(!Path::new(&snapshot).starts_with(f.0.join("cache")));
}

#[cfg(unix)]
#[test]
fn symlink_and_directory_entries_refuse_without_following_or_replacing() {
    for bundle in [true, false] {
        for symlink in [true, false] {
            let g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, false)
            };
            let f = Fixture::new();
            let requested = g.digest();
            let entry = g.entry(&f, &requested);
            std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
            let outside = f.0.join("outside");
            std::fs::write(&outside, g.proof()).unwrap();
            if symlink {
                std::os::unix::fs::symlink(&outside, &entry).unwrap();
            } else {
                std::fs::create_dir(&entry).unwrap();
            }
            g.assert_refusal(&f, &requested, "must be a regular file", true);
            assert!(calls(&f).is_empty());
            assert_eq!(std::fs::read(outside).unwrap(), g.proof());
        }
    }
}

#[test]
fn two_real_writers_publish_one_complete_proof_without_replacement() {
    for bundle in [true, false] {
        let g = if bundle {
            Graph::bundle(true)
        } else {
            Graph::helm(true, true, true)
        };
        let f = Fixture::new();
        let requested = g.digest();
        g.install(&f, &requested);
        std::fs::write(f.0.join("barrier"), b"").unwrap();
        let (a, b) = std::thread::scope(|s| {
            let a = s.spawn(|| g.run(&f, &requested));
            let b = s.spawn(|| g.run(&f, &requested));
            (a.join().unwrap(), b.join().unwrap())
        });
        g.assert_success(&f, &a);
        g.assert_success(&f, &b);
        assert_eq!(std::fs::read(g.entry(&f, &requested)).unwrap(), g.proof());
        assert_eq!(
            calls(&f).iter().filter(|c| c[0] == "oras").count(),
            if bundle { 4 } else { 8 }
        );
    }
}

#[test]
fn unsupported_fields_profiles_and_nulls_refuse_before_any_blob_call() {
    for bundle in [true, false] {
        let mut edits: Vec<(&str, serde_json::Value, &str)> = vec![
            (
                "/schemaVersion",
                3.into(),
                "unsupported OCI manifest profile",
            ),
            (
                "/mediaType",
                "application/vnd.oci.image.index.v1+json".into(),
                "unsupported OCI manifest profile",
            ),
            (
                "/artifactType",
                serde_json::Value::Null,
                "decoding closed OCI manifest profile",
            ),
            (
                "/config/mediaType",
                "unsupported".into(),
                if bundle {
                    "bundle config"
                } else {
                    "Helm config"
                },
            ),
            (
                "/config/digest",
                format!("sha256:{}", "A".repeat(64)).into(),
                "decoding closed OCI manifest profile",
            ),
            (
                "/annotations",
                serde_json::Value::Null,
                "decoding closed OCI manifest profile",
            ),
            (
                "/annotations",
                serde_json::json!({"a":3}),
                "decoding closed OCI manifest profile",
            ),
        ];
        if bundle {
            edits.extend([
                (
                    "/artifactType",
                    "other".into(),
                    "unsupported bundle artifactType",
                ),
                (
                    "/config/data",
                    serde_json::Value::Null,
                    "decoding closed OCI manifest profile",
                ),
                (
                    "/config/data",
                    "e30".into(),
                    "unsupported bundle config data",
                ),
                (
                    "/layers/0/mediaType",
                    CHART.into(),
                    "bundle requires one JSON layer",
                ),
            ]);
        } else {
            edits.extend([
                (
                    "/artifactType",
                    "other".into(),
                    "unsupported Helm artifactType",
                ),
                (
                    "/layers/0/mediaType",
                    PROV.into(),
                    "Helm chart layer is missing",
                ),
            ]);
        }
        for (pointer, value, reason) in edits {
            let mut g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, true)
            };
            g.edit(|v| {
                if pointer == "/annotations" {
                    v["annotations"] = value;
                } else {
                    *v.pointer_mut(pointer).unwrap() = value;
                }
            });
            let f = Fixture::new();
            let requested = g.digest();
            g.install(&f, &requested);
            g.assert_refusal(&f, &requested, reason, true);
            assert_eq!(calls(&f).len(), 1);
        }
    }
}

#[test]
fn unknown_fields_and_embedded_data_refuse_before_any_blob_call() {
    for bundle in [true, false] {
        for location in ["root", "config", "layer"] {
            for field in ["urls", "subject", "platform", "unknown", "data"] {
                if location == "config" && field == "data" && bundle {
                    continue;
                }
                let mut g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(false, false, true)
                };
                g.edit(|v| {
                    let object = match location {
                        "config" => &mut v["config"],
                        "layer" => &mut v["layers"][0],
                        _ => v,
                    };
                    object[field] = if field == "data" {
                        "e30=".into()
                    } else {
                        serde_json::json!(["https://elsewhere.invalid/../../sentinel"])
                    };
                });
                let f = Fixture::new();
                let requested = g.digest();
                g.install(&f, &requested);
                let reason = if field == "data" && location != "root" {
                    if bundle {
                        "unsupported bundle layer data"
                    } else {
                        "unsupported Helm descriptor data"
                    }
                } else {
                    "decoding closed OCI manifest profile"
                };
                g.assert_refusal(&f, &requested, reason, false);
                assert_eq!(calls(&f).len(), 1);
            }
        }
        for count in [0, 2, 3] {
            let mut g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, true)
            };
            g.edit(|v| v["layers"] = vec![v["layers"][0].clone(); count].into());
            let f = Fixture::new();
            let requested = g.digest();
            g.install(&f, &requested);
            g.assert_refusal(
                &f,
                &requested,
                if bundle {
                    "bundle requires one JSON layer"
                } else if count == 2 {
                    "duplicate Helm layer"
                } else {
                    "Helm requires"
                },
                true,
            );
            assert_eq!(calls(&f).len(), 1);
        }
    }
}

#[test]
fn duplicate_keys_and_noninteger_tokens_keep_valid_requested_identity() {
    for bundle in [true, false] {
        for (find, replace, reason) in [
            (
                "\"schemaVersion\":2",
                "\"schemaVersion\":2,\"schemaVersion\":2",
                "duplicate field",
            ),
            (
                "\"mediaType\":",
                "\"mediaType\":\"ignored\",\"mediaType\":",
                "duplicate field",
            ),
            ("\"config\":{", "\"config\":{\"size\":2,", "duplicate field"),
            (
                "\"layers\":[{",
                "\"layers\":[{\"size\":2,",
                "duplicate field",
            ),
            (
                "\"annotations\":{",
                "\"annotations\":{\"a\":\"1\",\"a\":\"2\",",
                "duplicate annotation key",
            ),
        ] {
            let mut g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, true)
            };
            g.edit(|v| v["annotations"] = serde_json::json!({"safe":"inert"}));
            let original = String::from_utf8(g.manifest.clone()).unwrap();
            g.manifest = original.replacen(find, replace, 1).into_bytes();
            assert_ne!(original.as_bytes(), g.manifest);
            let f = Fixture::new();
            let requested = g.digest();
            g.install(&f, &requested);
            g.assert_refusal(&f, &requested, reason, true);
            assert_eq!(calls(&f).len(), 1);
        }
        for token in [
            "-1",
            "-0",
            "2.0",
            "2e0",
            "2E+0",
            "18446744073709551616",
            "9007199254740993.0",
            "null",
            "\"2\"",
        ] {
            for schema in [true, false] {
                let mut g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(false, false, true)
                };
                g.edit(|_| {});
                let value: serde_json::Value = serde_json::from_slice(&g.manifest).unwrap();
                let (old, new) = if schema {
                    (
                        "\"schemaVersion\":2".to_owned(),
                        format!("\"schemaVersion\":{token}"),
                    )
                } else {
                    (
                        format!("\"size\":{}", value["config"]["size"]),
                        format!("\"size\":{token}"),
                    )
                };
                let original = String::from_utf8(g.manifest.clone()).unwrap();
                g.manifest = original.replacen(&old, &new, 1).into_bytes();
                assert_ne!(original.as_bytes(), g.manifest);
                let f = Fixture::new();
                let requested = g.digest();
                g.install(&f, &requested);
                g.assert_refusal(&f, &requested, "decoding closed OCI manifest profile", true);
                assert_eq!(calls(&f).len(), 1);
            }
        }
    }
}

#[test]
fn annotation_limits_apply_to_every_map_with_boundaries_admitted() {
    for bundle in [true, false] {
        for location in ["root", "config", "layer"] {
            for limit in ["pairs", "key", "value"] {
                for excess in [false, true] {
                    let mut g = if bundle {
                        Graph::bundle(true)
                    } else {
                        Graph::helm(false, false, false)
                    };
                    let annotations = match limit {
                        "pairs" => (0..64 + usize::from(excess))
                            .map(|n| (format!("key-{n}"), serde_json::Value::from("inert")))
                            .collect::<serde_json::Map<String, serde_json::Value>>(),
                        _ => serde_json::Map::new(),
                    };
                    let annotations = if limit == "pairs" {
                        serde_json::Value::Object(annotations)
                    } else if limit == "key" {
                        serde_json::json!({"k".repeat(4096+usize::from(excess)):"v"})
                    } else {
                        serde_json::json!({"k":"v".repeat(8192+usize::from(excess))})
                    };
                    g.edit(|v| match location {
                        "root" => v["annotations"] = annotations,
                        "config" => v["config"]["annotations"] = annotations,
                        _ => v["layers"][0]["annotations"] = annotations,
                    });
                    let f = Fixture::new();
                    let requested = g.digest();
                    g.install(&f, &requested);
                    if excess {
                        g.assert_refusal(&f, &requested, "annotation limit exceeded", false);
                        assert_eq!(calls(&f).len(), 1);
                    } else {
                        g.assert_success(&f, &g.run(&f, &requested));
                    }
                }
            }
        }
    }
}

#[test]
fn all_descriptor_limits_refuse_before_fetch_and_returned_reads_are_bounded() {
    for bundle in [true, false] {
        for index in 0..if bundle { 2 } else { 3 } {
            if bundle && index == 0 {
                continue;
            }
            let base = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(true, false, true)
            };
            let limit: u64 = if bundle {
                32 * 1024 * 1024
            } else if index == 1 {
                64 * 1024 * 1024
            } else {
                1024 * 1024
            };
            let mut declared = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(true, false, true)
            };
            declared.edit(|v| {
                let d = if index == 0 {
                    &mut v["config"]
                } else {
                    &mut v["layers"][index - 1]
                };
                d["size"] = (limit + 1).into();
            });
            let f = Fixture::new();
            let requested = declared.digest();
            declared.install(&f, &requested);
            declared.assert_refusal(
                &f,
                &requested,
                "OCI descriptor size exceeds local limit",
                false,
            );
            assert_eq!(calls(&f).len(), 1);
            let f = Fixture::new();
            let requested = base.digest();
            base.install(&f, &requested);
            let out = std::fs::OpenOptions::new()
                .write(true)
                .open(f.0.join(format!("blob-{index}")))
                .unwrap();
            out.set_len(limit + 1).unwrap();
            base.assert_refusal(
                &f,
                &requested,
                "OCI returned bytes exceed local limit",
                true,
            );
            assert!(!base.entry(&f, &requested).exists());
        }
    }
}

#[test]
fn manifest_limit_accepts_exact_bound_and_refuses_one_more_original_byte() {
    for bundle in [true, false] {
        for excess in [false, true] {
            let mut g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, true)
            };
            g.manifest.resize(1024 * 1024 + usize::from(excess), b' ');
            let f = Fixture::new();
            let requested = g.digest();
            g.install(&f, &requested);
            if excess {
                g.assert_refusal(
                    &f,
                    &requested,
                    "OCI returned bytes exceed local limit",
                    true,
                );
                assert_eq!(calls(&f).len(), 1);
            } else {
                g.assert_success(&f, &g.run(&f, &requested));
            }
        }
    }
}

#[test]
fn client_failures_missing_output_and_bounded_diagnostics_preserve_admission() {
    for bundle in [true, false] {
        for mode in ["nonzero", "missing-output", "diagnostic-flood"] {
            let g = if bundle {
                Graph::bundle(true)
            } else {
                Graph::helm(false, false, false)
            };
            let f = Fixture::new();
            let requested = g.digest();
            g.install(&f, &requested);
            std::fs::write(f.0.join(mode), b"").unwrap();
            g.assert_refusal(
                &f,
                &requested,
                if mode == "missing-output" {
                    "No such file"
                } else {
                    "ORAS fetch failed"
                },
                true,
            );
            assert!(!g.entry(&f, &requested).exists());
            assert_eq!(calls(&f).len(), 1);
            if mode == "diagnostic-flood" {
                let errors = std::fs::read_dir(&f.0)
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_name().to_string_lossy().ends_with(".stderr"))
                    .map(|e| std::fs::read(e.path()).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(errors.len(), 1);
                assert!(errors[0].len() < 132_000);
                let text = String::from_utf8_lossy(&errors[0]);
                let stdout = text
                    .split("stdout: ")
                    .nth(1)
                    .unwrap()
                    .split("; stderr: ")
                    .next()
                    .unwrap();
                let stderr = text.split("; stderr: ").nth(1).unwrap().trim_end();
                assert_eq!(stdout.as_bytes(), vec![b'o'; 65536]);
                assert_eq!(stderr.as_bytes(), vec![b'e'; 65536]);
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn actual_stalled_client_is_killed_and_reaped_at_the_shared_deadline() {
    // Both acquisitions run concurrently, each retaining its own real 60-second deadline.
    std::thread::scope(|s| {
        for bundle in [true, false] {
            s.spawn(move || {
                let g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(false, false, false)
                };
                let f = Fixture::new();
                let requested = g.digest();
                g.install(&f, &requested);
                std::fs::write(
                    f.0.join(if bundle {
                        "stall"
                    } else {
                        "stall-after-manifest"
                    }),
                    b"",
                )
                .unwrap();
                let start = std::time::Instant::now();
                g.assert_refusal(&f, &requested, "owned ORAS child killed and reaped", true);
                assert!(start.elapsed().as_secs_f64() >= 59.5 && start.elapsed().as_secs() < 65);
                let pid = std::fs::read_to_string(f.0.join("child-pid")).unwrap();
                assert!(
                    !Path::new("/proc").join(pid.trim()).exists(),
                    "owned child remained alive or a zombie"
                );
                assert!(!g.entry(&f, &requested).exists());
                assert_eq!(calls(&f).len(), if bundle { 1 } else { 2 });
            });
        }
    });
}

#[test]
fn warm_substitution_rechecks_every_blob_and_requested_manifest_identity() {
    for bundle in [true, false] {
        for index in 0..if bundle { 2 } else { 3 } {
            for self_consistent in [false, true] {
                let mut g = if bundle {
                    Graph::bundle(true)
                } else {
                    Graph::helm(true, false, true)
                };
                let f = Fixture::new();
                let requested = g.digest();
                g.blobs[index][0] ^= 1;
                if self_consistent {
                    let digest = Digest::of_bytes(&g.blobs[index]);
                    g.edit(|v| {
                        if index == 0 {
                            v["config"]["digest"] = digest.as_str().into();
                        } else {
                            v["layers"][index - 1]["digest"] = digest.as_str().into();
                        }
                    });
                }
                let entry = g.entry(&f, &requested);
                std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
                let proof = g.proof();
                std::fs::write(&entry, &proof).unwrap();
                g.assert_refusal(
                    &f,
                    &requested,
                    if self_consistent {
                        "requested OCI manifest digest mismatch"
                    } else {
                        "OCI descriptor digest mismatch"
                    },
                    true,
                );
                assert!(calls(&f).is_empty());
                assert_eq!(std::fs::read(entry).unwrap(), proof);
            }
        }
    }
}

#[test]
fn legacy_entries_and_interrupted_stages_are_retained_while_cold_proof_is_published() {
    for bundle in [true, false] {
        let g = if bundle {
            Graph::bundle(true)
        } else {
            Graph::helm(false, false, false)
        };
        let f = Fixture::new();
        let requested = g.digest();
        let entry = g.entry(&f, &requested);
        std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
        let abandoned = entry.parent().unwrap().join(".proof-dead-process-0");
        std::fs::write(&abandoned, b"incomplete stage").unwrap();
        let legacy =
            f.0.join(if bundle {
                "cache/sha256"
            } else {
                "cache/helm/sha256"
            })
            .join(requested.as_str().strip_prefix("sha256:").unwrap());
        std::fs::create_dir_all(&legacy).unwrap();
        let old = legacy.join(if bundle {
            "ess-release-bundle.json"
        } else {
            "chart.tgz"
        });
        std::fs::write(&old, b"legacy bytes").unwrap();
        std::fs::write(
            legacy.join("payload.digest"),
            Digest::of_bytes(b"legacy bytes").as_str(),
        )
        .unwrap();
        g.install(&f, &requested);
        g.assert_success(&f, &g.run(&f, &requested));
        assert_cold_calls(&g, &f, &requested);
        assert_eq!(std::fs::read(old).unwrap(), b"legacy bytes");
        assert_eq!(std::fs::read(abandoned).unwrap(), b"incomplete stage");
        assert_eq!(std::fs::read(entry).unwrap(), g.proof());
    }
}

#[test]
fn descriptor_proof_does_not_admit_noncanonical_or_invalid_bundle_semantics() {
    for canonical in [false, true] {
        let mut g = Graph::bundle(true);
        if canonical {
            let mut value: serde_json::Value = serde_json::from_slice(&g.blobs[1]).unwrap();
            value["format"] = "future/99".into();
            g.blobs[1] = serde_json::to_vec(&value).unwrap();
        } else {
            g.blobs[1].push(b' ');
        }
        let d = descriptor(JSON, &g.blobs[1]);
        g.edit(|v| v["layers"][0] = d);
        let f = Fixture::new();
        let requested = g.digest();
        g.install(&f, &requested);
        g.assert_refusal(
            &f,
            &requested,
            if canonical {
                "parsing verified OCI bundle JSON"
            } else {
                "not canonical release-bundle JSON"
            },
            true,
        );
        assert_eq!(calls(&f).len(), 2);
        assert!(!g.entry(&f, &requested).exists());
    }
}

#[test]
fn failed_later_chart_stops_its_helm_call_after_preserving_earlier_release() {
    let g = Graph::helm(false, false, false);
    let setup = Fixture::new();
    let requested = g.digest();
    g.install(&setup, &requested);
    g.assert_success(&setup, &g.run(&setup, &requested));
    let source = std::fs::read_dir(&setup.0)
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().starts_with("desired-"))
        .unwrap()
        .path();
    let mut plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source).unwrap()).unwrap();
    let bad = Digest::of_bytes(b"other requested chart manifest");
    plan["releases"]["last"] = plan["releases"]["first"].clone();
    plan["releases"]["last"]["service"] = "last".into();
    plan["releases"]["last"]["release_name"] = "last".into();
    plan["releases"]["last"]["chart"]["digest"] = bad.as_str().into();
    plan["rollout_order"] = serde_json::json!(["first", "last"]);
    let f = Fixture::new();
    g.install(&f, &requested);
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(f.0.join("requests"))
            .unwrap(),
        "manifest\t{}@{bad}\tmanifest",
        g.repository()
    )
    .unwrap();
    let desired = f.0.join("sequence.json");
    std::fs::write(&desired, serde_json::to_vec(&plan).unwrap()).unwrap();
    let output = f.acquire(&desired, true);
    std::fs::write(f.0.join("sequence.stdout"), &output.stdout).unwrap();
    std::fs::write(f.0.join("sequence.stderr"), &output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requested OCI manifest digest mismatch")
    );
    assert_eq!(std::fs::read(f.0.join("helm-chart")).unwrap(), g.blobs[1]);
    let calls = calls(&f);
    assert_eq!(
        calls.iter().map(|c| c[0].as_str()).collect::<Vec<_>>(),
        ["oras", "oras", "oras", "helm", "oras"]
    );
    assert_eq!(calls[3][3], "first");
    assert_eq!(calls[4][5], format!("{}@{bad}", g.repository()));
    assert!(!g.entry(&f, &bad).exists());
}

/// The control that keeps every vector above from passing on an earlier refusal.
///
/// Each case in this file now reaches the cache through a driver that admits a *synthetic*
/// authority first. That is only sound if the authority is what lets it through — so this runs an
/// otherwise identical vector with the authority withheld and requires it to stop **before** the
/// first ORAS call, before the cache directory exists and before Helm. Without this, "the cache
/// refused" and "the authority refused" would be indistinguishable from the outside, and every
/// original-byte assertion above would be vacuous.
#[test]
fn the_cache_vectors_reach_their_boundary_only_because_an_authority_admitted_them() {
    let g = Graph::helm(false, false, false);
    let requested = g.digest();

    let admitted = Fixture::new();
    g.install(&admitted, &requested);
    let plan = admitted.0.join("control.json");
    write_plan(&admitted, &plan, &requested);
    let allowed = admitted.acquire(&plan, true);
    assert!(
        allowed.status.success(),
        "with an admitted authority the vector reaches and completes the cache boundary: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    assert!(admitted.0.join("cache/oci-proof-v1").exists());
    assert!(admitted.0.join("helm-chart").exists());
    assert!(
        calls(&admitted).iter().any(|call| call[0] == "oras"),
        "the admitted run really fetched"
    );

    let withheld = Fixture::new();
    g.install(&withheld, &requested);
    let plan = withheld.0.join("control.json");
    write_plan(&withheld, &plan, &requested);
    let refused = withheld.acquire(&plan, false);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("authority"),
        "the refusal is the authority's: {}",
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(
        !withheld.0.join("cache").exists(),
        "no cache is populated before an authority admits"
    );
    assert!(!withheld.0.join("helm-chart").exists());
    assert!(
        calls(&withheld).is_empty(),
        "no external client is invoked: {:?}",
        calls(&withheld)
    );
}

/// Writes the one-release desired document the control vectors use.
fn write_plan(f: &Fixture, path: &Path, digest: &Digest) {
    let d = digest.as_str();
    let plan = serde_json::json!({"format":"ess-deployment/1", "environment":"test", "stack_digest":d, "cluster":"test-cluster", "rollout_order":["first"], "releases":{"first":{"service":"first", "release_name":"first", "namespace":"test", "service_account":"default", "images":{"app":{"build_output":"app", "kind":"oci_image", "reference":"example.invalid/app", "digest":d, "platforms":{"linux/amd64":d}}}, "chart":{"build_output":"chart", "kind":"helm_chart", "reference":"oci://example.invalid/chart", "digest":d}}}});
    let _ = f;
    std::fs::write(path, serde_json::to_vec(&plan).unwrap()).unwrap();
}

/// R07: staged residue from a completed fetch is neither a cache hit nor application evidence.
///
/// A fetch that finished writing its outputs and was then interrupted before the bounded read,
/// the proof assembly or the chart preparation leaves an acquisition directory behind. It looks
/// exactly like a successful fetch and it establishes nothing: the next run fetches again, and the
/// residue is still there afterwards because nothing here repairs or consumes it.
#[test]
fn r07_completed_fetch_residue_is_neither_a_cache_hit_nor_application_evidence() {
    let g = Graph::helm(false, false, false);
    let requested = g.digest();
    let f = Fixture::new();
    g.install(&f, &requested);

    // The residue: an acquisition directory holding exactly what a completed fetch writes.
    let parent = g.entry(&f, &requested).parent().unwrap().to_path_buf();
    let residue = parent.join("acquire-interrupted");
    std::fs::create_dir_all(&residue).unwrap();
    for (name, bytes) in [
        ("manifest", g.manifest.clone()),
        ("blob-0", g.blobs[0].clone()),
        ("blob-1", g.blobs[1].clone()),
    ] {
        std::fs::write(residue.join(name), bytes).unwrap();
    }
    assert!(!g.entry(&f, &requested).exists(), "and no published entry");

    let plan = f.0.join("r07.json");
    write_plan(&f, &plan, &requested);
    let output = f.acquire(&plan, true);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Not a cache hit: the client really ran again.
    assert!(
        calls(&f).iter().any(|call| call[0] == "oras"),
        "staged residue is not a warm entry: {:?}",
        calls(&f)
    );
    // Not application evidence either: the consumer ran once, after the proof was published.
    assert_eq!(
        std::fs::read(g.entry(&f, &requested)).unwrap(),
        g.proof(),
        "the published proof comes from the fresh acquisition"
    );
    assert_eq!(std::fs::read(f.0.join("helm-chart")).unwrap(), g.blobs[1]);
    assert_eq!(
        calls(&f).iter().filter(|call| call[0] == "helm").count(),
        1,
        "the residue caused no extra consumer call"
    );
    // Preserved, untouched, for diagnosis.
    assert_eq!(std::fs::read(residue.join("manifest")).unwrap(), g.manifest);
}

/// R09: an unpublished stage is ignored, and publication never replaces an existing entry.
///
/// Two halves of the same rule. A partial file beside the entry name is not an entry, so a cold
/// run ignores it, publishes properly and leaves it exactly where it was. And an entry that is
/// already published is not replaced by a later run: the warm path revalidates the bytes that are
/// there instead of writing over them.
#[test]
fn r09_an_unpublished_stage_is_ignored_and_publication_never_replaces_an_entry() {
    let g = Graph::helm(false, false, false);
    let requested = g.digest();
    let f = Fixture::new();
    g.install(&f, &requested);

    let entry = g.entry(&f, &requested);
    let parent = entry.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(&parent).unwrap();
    let stage = parent.join(format!(
        "{}.entry.stage",
        requested.as_str().strip_prefix("sha256:").unwrap()
    ));
    let partial = b"ESSOCI1\n\x00\x00\x00".to_vec();
    std::fs::write(&stage, &partial).unwrap();

    let plan = f.0.join("r09.json");
    write_plan(&f, &plan, &requested);
    let cold = f.acquire(&plan, true);
    assert!(
        cold.status.success(),
        "{}",
        String::from_utf8_lossy(&cold.stderr)
    );
    assert_eq!(
        std::fs::read(&entry).unwrap(),
        g.proof(),
        "the entry is published from the fresh acquisition, not from the stage"
    );
    assert_eq!(
        std::fs::read(&stage).unwrap(),
        partial,
        "an incomplete stage is retained exactly as it was"
    );
    let cold_calls = calls(&f).len();
    assert!(cold_calls > 1, "the cold run really fetched");

    // The no-replacement half: with the client trapped, the warm run revalidates and consumes the
    // bytes that are already published, and does not write over them.
    std::fs::write(f.0.join("trap"), b"").unwrap();
    std::fs::remove_file(f.0.join("helm-chart")).unwrap();
    let warm = f.acquire(&plan, true);
    assert!(
        warm.status.success(),
        "{}",
        String::from_utf8_lossy(&warm.stderr)
    );
    assert_eq!(
        std::fs::read(&entry).unwrap(),
        g.proof(),
        "the published entry's bytes are unchanged"
    );
    assert_eq!(std::fs::read(f.0.join("helm-chart")).unwrap(), g.blobs[1]);
    assert_eq!(
        calls(&f).iter().filter(|call| call[0] == "oras").count(),
        cold_calls - 1,
        "no client ran for the warm entry"
    );
    assert_eq!(std::fs::read(&stage).unwrap(), partial);
}
