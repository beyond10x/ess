//! Live infrastructure acceptance: one generated service placement, one disposable k3d cluster.
//!
//! The offline gate proves the binding comparison over fixtures. What it cannot prove is that the
//! comparison holds against a cluster somebody else put into a state, so this command builds that
//! situation and tears it down again:
//!
//! 1. the generated service crate is built for static Linux and imported as one container image;
//! 2. a disposable cluster is created, and **this module's own manifests** — never an ESS
//!    projection — put the workload, its namespace and a Secret into it;
//! 3. ESS reads the namespace live (`ess verify bindings --live --observation-out`) and compares it
//!    with the declared placement; the infrastructure projection of the placement intent is
//!    produced twice over the same observation and must be byte-identical;
//! 4. three sensitivity cases (wrong image tag, missing workload, an extra unbound container) must
//!    each fail with their named finding, and the cluster is restored and must pass again after
//!    each;
//! 5. every observation and report is checked for the Secret's value, and the cluster, the images
//!    and every scratch file are removed and their absence is read back.
//!
//! The placement intent asks for resource bounds the manifest deliberately does not declare, and
//! carries a remedy. The projection therefore proposes a patch, and because nothing here applies a
//! projection, the last observation of the run still shows the gap: non-application is observed,
//! not only asserted. The structural half of that property — that no code path in this module can
//! hand a projection file to `kubectl apply` — is `tests::only_harness_manifests_reach_apply`.
//!
//! Needs Docker, `k3d`, `kubectl` and `tar`. It is not part of `task check`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MANIFEST: &str = include_str!("../infra-acceptance/placement.yaml");
const EXTRA_CONTAINER: &str = include_str!("../infra-acceptance/extra-container.yaml");
const INTENT: &str = include_str!("../infra-acceptance/placement.infra-spec.yaml");

const NAMESPACE: &str = "billing";
const WORKLOAD: &str = "billing";
const CONTAINER: &str = "billing";
const SECRET_NAME: &str = "billing-credentials";
const IMPLEMENTATION: &str = "billing-service";
const COMPONENT: &str = "invoice-service";
const KUBECTL: &str = "kubectl";
const MUSL: &str = "x86_64-unknown-linux-musl";

/// Arguments of `cargo xtask infra-acceptance`.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Cargo package directory of the generated service host, built for static Linux.
    #[arg(long)]
    service_crate: PathBuf,
    /// Semantic ESS source the service was generated from.
    #[arg(long)]
    spec: PathBuf,
    /// Run name; the cluster and image repository are both called `ess-<run>`.
    #[arg(long)]
    run: String,
    /// New directory outside every Git checkout for kubeconfig, image root and observations.
    #[arg(long)]
    scratch: PathBuf,
    /// New directory for receipts.
    #[arg(long)]
    evidence: PathBuf,
}

/// Which of this module's own manifests is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Matching,
    WrongTag,
    ExtraContainer,
}

impl Variant {
    const fn label(self) -> &'static str {
        match self {
            Self::Matching => "matching",
            Self::WrongTag => "wrong-tag",
            Self::ExtraContainer => "extra-container",
        }
    }
}

/// A manifest this module authored. The only value `apply` accepts, and it is constructed only by
/// [`HarnessManifest::render`] from the embedded templates above.
struct HarnessManifest {
    label: &'static str,
    bytes: Vec<u8>,
}

impl HarnessManifest {
    fn render(variant: Variant, image: &Image, secret: &Sentinel) -> Self {
        let reference = match variant {
            Variant::Matching | Variant::ExtraContainer => image.pinned(),
            Variant::WrongTag => image.wrong_tag(),
        };
        let extra = match variant {
            Variant::ExtraContainer => EXTRA_CONTAINER.replace("{{IMAGE}}", &reference),
            Variant::Matching | Variant::WrongTag => String::new(),
        };
        let text = MANIFEST
            .replace("{{EXTRA_CONTAINER}}", &extra)
            .replace("{{IMAGE}}", &reference)
            .replace("{{SECRET}}", secret.as_str());
        Self {
            label: variant.label(),
            bytes: text.into_bytes(),
        }
    }
}

/// The imported image and the digest the cluster's runtime resolved for it.
struct Image {
    repository: String,
    digest: Option<String>,
}

impl Image {
    fn local(&self, tag: &str) -> String {
        format!("{}:{tag}", self.repository)
    }
    fn qualified(&self) -> String {
        format!("docker.io/{}", self.repository)
    }
    fn pinned(&self) -> String {
        format!(
            "{}@{}",
            self.qualified(),
            self.digest.as_deref().unwrap_or("sha256:unresolved")
        )
    }
    fn wrong_tag(&self) -> String {
        format!("{}:wrong-tag", self.qualified())
    }
}

/// One row of the story's acceptance table.
#[derive(Debug, serde::Serialize)]
struct Acceptance {
    row: u8,
    statement: &'static str,
    pass: bool,
    evidence: Value,
}

struct Run {
    root: PathBuf,
    cluster: String,
    context: String,
    kubeconfig: PathBuf,
    scratch: PathBuf,
    evidence: PathBuf,
    spec: PathBuf,
    ess: Option<PathBuf>,
    image: Image,
    secret: Sentinel,
    applied: Vec<Value>,
    projection_bytes: Vec<Vec<u8>>,
    /// Every directory a projection was written to; no later command may name a path in one.
    projection_dirs: Vec<PathBuf>,
    commands: Vec<Value>,
    cluster_created: bool,
    api_port: Option<String>,
    images_created: bool,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run(root: &Path, args: &Args) -> Result<String> {
    let (cluster, context) = names(&args.run)?;
    for path in [&args.scratch, &args.evidence] {
        if path.exists() {
            bail!("{} must not exist yet", path.display());
        }
    }
    let scratch_parent = args
        .scratch
        .parent()
        .context("--scratch needs a parent")?
        .canonicalize()
        .context("--scratch parent must exist")?;
    if let Some(checkout) = git_ancestor(&scratch_parent) {
        bail!(
            "--scratch must be outside Git checkouts; {} is one",
            checkout.display()
        );
    }
    fs::create_dir(&args.scratch)?;
    fs::create_dir(&args.evidence)?;
    let scratch = args.scratch.canonicalize()?;
    let evidence = args.evidence.canonicalize()?;
    for dir in [
        "manifests",
        "observations",
        "inputs",
        "image",
        "tmp",
        "kube-cache",
    ] {
        fs::create_dir(scratch.join(dir))?;
    }
    for dir in ["observations", "reports"] {
        fs::create_dir(evidence.join(dir))?;
    }
    let mut run = Run {
        root: root.to_path_buf(),
        context,
        kubeconfig: scratch.join("kubeconfig"),
        image: Image {
            repository: format!("{cluster}/billing"),
            digest: None,
        },
        cluster,
        scratch,
        evidence,
        spec: args.spec.canonicalize().context("--spec must exist")?,
        ess: None,
        secret: sentinel()?,
        applied: Vec::new(),
        projection_bytes: Vec::new(),
        projection_dirs: Vec::new(),
        commands: Vec::new(),
        cluster_created: false,
        api_port: None,
        images_created: false,
    };
    let default_context_before = run.default_context();
    let outcome = run.execute(&args.service_crate);
    let teardown = run.teardown(default_context_before.as_deref());
    let mut rows = match &outcome {
        Ok(rows) => rows.clone_rows(),
        Err(_) => Vec::new(),
    };
    rows.push(teardown);
    let failure = outcome.as_ref().err().map(|e| format!("{e:#}"));
    let passed = failure.is_none() && rows.len() == 7 && rows.iter().all(|r| r.pass);
    let receipt = json!({
        "format": "ess-infra-acceptance-receipt/1",
        "cluster": run.cluster,
        "context": run.context,
        "namespace": NAMESPACE,
        "image": run.image.digest.as_ref().map(|_| run.image.pinned()),
        "secret_sha256": hex(&Sha256::digest(run.secret.as_str().as_bytes())),
        "applied_manifests": run.applied,
        "failure": failure,
        "passed": passed,
        "rows": rows,
    });
    fs::write(
        run.evidence.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let commands = run.commands.iter().fold(String::new(), |mut out, c| {
        let _ = writeln!(out, "{c}");
        out
    });
    fs::write(run.evidence.join("commands.jsonl"), commands)?;
    let mut summary = String::new();
    for row in &rows {
        writeln!(
            summary,
            "row {}: {} — {}",
            row.row,
            if row.pass { "pass" } else { "FAIL" },
            row.statement
        )?;
    }
    if let Some(failure) = &receipt["failure"].as_str() {
        writeln!(summary, "stopped: {failure}")?;
    }
    writeln!(
        summary,
        "receipt: {}",
        run.evidence.join("receipt.json").display()
    )?;
    if passed {
        Ok(summary)
    } else {
        bail!("{summary}infrastructure acceptance failed")
    }
}

trait CloneRows {
    fn clone_rows(&self) -> Vec<Acceptance>;
}
impl CloneRows for Vec<Acceptance> {
    fn clone_rows(&self) -> Vec<Acceptance> {
        self.iter()
            .map(|r| Acceptance {
                row: r.row,
                statement: r.statement,
                pass: r.pass,
                evidence: r.evidence.clone(),
            })
            .collect()
    }
}

impl Run {
    #[allow(clippy::too_many_lines)]
    fn execute(&mut self, service_crate: &Path) -> Result<Vec<Acceptance>> {
        self.preflight()?;
        let ess = self.build_ess()?;
        self.ess = Some(ess);
        let (binary, sdk) = self.build_service(service_crate)?;
        self.build_image(&binary)?;
        self.create_cluster()?;
        self.import_image()?;
        let realization_digest = self.author_inputs(&sdk)?;

        let matching = HarnessManifest::render(Variant::Matching, &self.image, &self.secret);
        self.apply(&matching)?;
        self.rollout()?;
        let (first_exit, first) = self.verify_live("matching")?;

        let observation = self.scratch.join("observations/matching.json");
        // Acceptance 2: the projection of the placement intent, twice over the same observation. The
        // projection refuses a namespace-topology read, which omits what it would patch, so its
        // input is a full sanitized scan of the disposable cluster taken in the same state.
        let full = self.scan_full("matching-full")?;
        let (a_out, a_tree) = self.project(&full, "projection-a")?;
        let (b_out, b_tree) = self.project(&full, "projection-b")?;
        copy_tree(
            &self.scratch.join("projection-a"),
            &self.evidence.join("projection-a"),
        )?;
        copy_tree(
            &self.scratch.join("projection-b"),
            &self.evidence.join("projection-b"),
        )?;
        // The evidence copies are projection outputs too; no later command may name them.
        self.projection_dirs
            .push(self.evidence.join("projection-a"));
        self.projection_dirs
            .push(self.evidence.join("projection-b"));
        fs::write(self.evidence.join("projection-a.json"), &a_out)?;
        fs::write(self.evidence.join("projection-b.json"), &b_out)?;
        // `.ess-output/` is the output-ownership record of the directory a projection was
        // written into, not projection content; it is compared separately and never hidden.
        let (a_artifacts, a_owned) = split_ownership(&a_tree);
        let (b_artifacts, b_owned) = split_ownership(&b_tree);
        let row2 = Acceptance {
            row: 2,
            statement: "the infrastructure projection is produced twice and is byte-identical",
            pass: a_out == b_out && a_artifacts == b_artifacts && !a_artifacts.is_empty(),
            evidence: json!({
                "input": "observations/matching-full.json",
                "stdout_sha256": [hex(&Sha256::digest(&a_out)), hex(&Sha256::digest(&b_out))],
                "artifact_tree_sha256": [tree_digest(&a_artifacts), tree_digest(&b_artifacts)],
                "artifacts": a_artifacts.keys().collect::<Vec<_>>(),
                "ownership_records": a_owned.keys().collect::<Vec<_>>(),
                "ownership_records_identical": a_owned == b_owned,
                "gaps": gap_ids(&a_out),
            }),
        };

        // Sensitivity: each case is its own observation, and the cluster is restored after it.
        let wrong = HarnessManifest::render(Variant::WrongTag, &self.image, &self.secret);
        self.apply(&wrong)?;
        self.rollout()?;
        let wrong_tag = self.verify_live("wrong-tag")?;
        self.apply(&matching)?;
        self.rollout()?;
        let wrong_tag_restored = self.verify_live("wrong-tag-restored")?;

        self.kubectl(&[
            "delete",
            "deployment",
            WORKLOAD,
            "--namespace",
            NAMESPACE,
            "--wait=true",
            "--timeout=180s",
        ])?;
        let missing = self.verify_live("missing-workload")?;
        self.apply(&matching)?;
        self.rollout()?;
        let missing_restored = self.verify_live("missing-workload-restored")?;

        let extra = HarnessManifest::render(Variant::ExtraContainer, &self.image, &self.secret);
        self.apply(&extra)?;
        self.rollout()?;
        let extra_container = self.verify_live("extra-container")?;
        self.apply(&matching)?;
        self.rollout()?;
        let extra_restored = self.verify_live("extra-container-restored")?;

        // A desired declaration offered where an observation belongs must be refused, not read.
        let declaration = self.declaration_as_observation()?;

        // The last observation still shows what the projection proposed: it was never applied.
        let last = self.scan_full("final-full")?;
        let (final_out, final_tree) = self.project(&last, "projection-final")?;

        let row1 = {
            let checks = first["checks"].as_array().cloned().unwrap_or_default();
            let binding = &first["bindings"]["billing-api"];
            Acceptance {
                row: 1,
                statement:
                    "the ess-observed-bindings/1 document for the billing workload validates",
                pass: checks.iter().all(|c| c["code"] != "OBS-BIND-000")
                    && first["binding_digest"].is_string()
                    && first["realization_digest"] == json!(realization_digest)
                    && first["scope"] == json!({"context": self.context, "namespace": NAMESPACE})
                    && binding["workload"] == json!(format!("{NAMESPACE}/deployment/{WORKLOAD}"))
                    && binding["container"] == json!(CONTAINER)
                    && binding["expected_image"] == json!(self.image.pinned()),
                evidence: json!({
                    "bindings": "inputs/bindings.json",
                    "realization_digest": realization_digest,
                    "binding_digest": first["binding_digest"],
                    "scope": first["scope"],
                    "workload": binding["workload"],
                    "container": binding["container"],
                    "image": binding["expected_image"],
                }),
            }
        };

        let final_gaps = gap_ids(&final_out);
        let row3 = Acceptance {
            row: 3,
            statement:
                "the cluster is set up from harness manifests only; no projection output is applied",
            pass: self
                .applied
                .iter()
                .all(|a| a["projection_bytes_equal"] == json!(false))
                && final_tree.keys().eq(a_tree.keys())
                && final_gaps.iter().any(|g| g == "billing-resources"),
            evidence: json!({
                "applied": self.applied,
                "structural_guard": "ess-xtask infra_acceptance::tests::only_harness_manifests_reach_apply",
                "projection_files_first": a_tree.keys().collect::<Vec<_>>(),
                "projection_files_last": final_tree.keys().collect::<Vec<_>>(),
                "gaps_in_last_observation": final_gaps,
            }),
        };

        let named = |report: &Value, code: &str| -> Option<String> {
            report["bindings"]["billing-api"]["checks"]
                .as_array()?
                .iter()
                .find(|c| c["code"] == code)
                .and_then(|c| c["status"].as_str().map(str::to_owned))
        };
        let row4 = Acceptance {
            row: 4,
            statement: "a live namespace read outside Git is compared and passes for the matching placement",
            pass: first_exit == Some(0)
                && first["status"] == "satisfied"
                && first["acquisition"] == "live_namespace_read"
                && first["observation"]["provenance"]["context"] == json!(self.context)
                && declaration.0 == Some(2)
                && declaration.1["checks"][0]["code"] == "OBS-BIND-006",
            evidence: json!({
                "exit": first_exit,
                "status": first["status"],
                "acquisition": first["acquisition"],
                "observation_digest": first["observation"]["digest"],
                "observation_written_to": observation,
                "checks": first["bindings"]["billing-api"]["checks"],
                "declaration_offered_as_observation": {
                    "exit": declaration.0,
                    "finding": declaration.1["checks"][0],
                },
            }),
        };

        let case = |label: &str, (exit, report): &(Option<i32>, Value), code: &str| {
            json!({
                "case": label,
                "exit": exit,
                "status": report["status"],
                "finding": code,
                "finding_status": named(report, code),
                "detail": report["bindings"]["billing-api"]["checks"]
                    .as_array()
                    .and_then(|checks| checks.iter().find(|c| c["code"] == code))
                    .map(|c| c["detail"].clone()),
            })
        };
        let restore_case = |label: &str, (exit, report): &(Option<i32>, Value)| json!({"case": label, "exit": exit, "status": report["status"]});
        let failing = [
            (&wrong_tag, "OBS-BIND-004"),
            (&missing, "OBS-BIND-002"),
            (&extra_container, "OBS-BIND-008"),
        ];
        let restores = [&wrong_tag_restored, &missing_restored, &extra_restored];
        let row5 = Acceptance {
            row: 5,
            statement: "wrong image tag, missing workload and an extra unbound container each fail with a named finding, and pass again once restored",
            pass: failing.iter().all(|((exit, report), code)| {
                *exit == Some(1)
                    && report["status"] == "violated"
                    && named(report, code).as_deref() == Some("violated")
            }) && restores
                .iter()
                .all(|(exit, report)| *exit == Some(0) && report["status"] == "satisfied"),
            evidence: json!([
                case("wrong-tag", &wrong_tag, "OBS-BIND-004"),
                restore_case("wrong-tag-restored", &wrong_tag_restored),
                case("missing-workload", &missing, "OBS-BIND-002"),
                restore_case("missing-workload-restored", &missing_restored),
                case("extra-container", &extra_container, "OBS-BIND-008"),
                restore_case("extra-container-restored", &extra_restored),
            ]),
        };

        let row6 = self.secret_row()?;
        Ok(vec![row1, row2, row3, row4, row5, row6])
    }

    fn preflight(&mut self) -> Result<()> {
        for (tool, args) in [
            (
                "docker",
                &["version", "--format", "{{.Server.Version}}"][..],
            ),
            ("k3d", &["version"][..]),
            (KUBECTL, &["version", "--client"][..]),
            ("tar", &["--version"][..]),
        ] {
            let mut command = Command::new(tool);
            command.args(args);
            self.checked(&mut command, &format!("{tool} is required"))?;
        }
        let mut list = self.k3d();
        list.args(["cluster", "list", "--output", "json"]);
        let clusters: Value =
            serde_json::from_slice(&self.checked(&mut list, "k3d cluster list")?.stdout)?;
        refuse_existing(&clusters, &self.cluster)
    }

    fn build_ess(&mut self) -> Result<PathBuf> {
        let mut command = Command::new(cargo());
        command
            .args([
                "build",
                "--locked",
                "--quiet",
                "--package",
                "ess-cli",
                "--bin",
                "ess",
            ])
            .arg("--message-format=json-render-diagnostics")
            .current_dir(&self.root);
        let output = self.checked(&mut command, "building ess")?;
        single_executable(&output.stdout, "ess")
    }

    fn build_service(&mut self, service_crate: &Path) -> Result<(PathBuf, Value)> {
        let manifest = service_crate.join("Cargo.toml");
        let mut command = Command::new(cargo());
        command
            .args([
                "build",
                "--locked",
                "--release",
                "--target",
                MUSL,
                "--manifest-path",
            ])
            .arg(&manifest)
            .arg("--message-format=json-render-diagnostics");
        let output = self.checked(&mut command, "building the generated service")?;
        let executables = executables(&output.stdout);
        let [binary] = executables.as_slice() else {
            bail!("the service crate must build exactly one executable, built {executables:?}");
        };
        let lock = fs::read_to_string(service_crate.join("Cargo.lock"))
            .context("the service crate must carry a Cargo.lock")?;
        Ok((binary.clone(), json!({"sdk_sources": sdk_sources(&lock)})))
    }

    fn build_image(&mut self, binary: &Path) -> Result<()> {
        let root = self.scratch.join("image");
        fs::copy(binary, root.join("billing"))?;
        let tarball = self.scratch.join("image.tar");
        let mut tar = Command::new("tar");
        tar.arg("-C")
            .arg(&root)
            .args(["--owner=0", "--group=0", "--numeric-owner", "-cf"])
            .arg(&tarball)
            .arg("billing");
        self.checked(&mut tar, "packing the image root")?;
        // `docker import` makes an image from one root without a build, so no build cache is
        // left behind for teardown to find.
        let mut import = Command::new("docker");
        import
            .arg("import")
            .args(["--change", "ENTRYPOINT [\"/billing\"]"])
            .args(["--change", "USER 65532:65532"])
            .args([
                "--change",
                &format!("LABEL ess.infra-acceptance.run={}", self.cluster),
            ])
            .arg(&tarball)
            .arg(self.image.local("accepted"));
        self.images_created = true;
        self.checked(&mut import, "importing the image")?;
        let mut tag = Command::new("docker");
        tag.args([
            "tag",
            &self.image.local("accepted"),
            &self.image.local("wrong-tag"),
        ]);
        self.checked(&mut tag, "tagging the wrong-tag image")?;
        fs::remove_file(tarball)?;
        Ok(())
    }

    fn create_cluster(&mut self) -> Result<()> {
        // Read again immediately before creating: the preflight listing is minutes old by now.
        let mut list = self.k3d();
        list.args(["cluster", "list", "--output", "json"]);
        let clusters: Value =
            serde_json::from_slice(&self.checked(&mut list, "k3d cluster list")?.stdout)?;
        refuse_existing(&clusters, &self.cluster)?;
        self.cluster_created = true;
        let mut create = self.k3d();
        create
            .args(["cluster", "create", &self.cluster])
            .args([
                "--servers",
                "1",
                "--agents",
                "0",
                "--no-lb",
                "--wait",
                "--timeout",
                "240s",
            ])
            .args([
                "--kubeconfig-update-default=false",
                "--kubeconfig-switch-context=false",
            ])
            .args(["--k3s-arg", "--disable=traefik@server:0"])
            .args(["--k3s-arg", "--disable=metrics-server@server:0"])
            .args(["--k3s-arg", "--disable=local-storage@server:0"])
            // The node's filesystem is the host's, which this harness does not own and may find
            // nearly full. The default eviction thresholds would taint a disposable node for disk
            // pressure it cannot relieve, and no new pod would schedule.
            .args([
                "--k3s-arg",
                "--kubelet-arg=eviction-hard=imagefs.available<1%,nodefs.available<1%@server:0",
            ]);
        self.checked(&mut create, "creating the disposable cluster")?;
        let mut config = self.k3d();
        config.args(["kubeconfig", "get", &self.cluster]);
        let bytes = self
            .checked(&mut config, "reading the cluster kubeconfig")?
            .stdout;
        let current = String::from_utf8_lossy(&bytes).lines().find_map(|line| {
            line.trim()
                .strip_prefix("current-context: ")
                .map(str::to_owned)
        });
        if current.as_deref() != Some(self.context.as_str()) {
            bail!(
                "the cluster kubeconfig names context {current:?}, not {}",
                self.context
            );
        }
        write_private(&self.kubeconfig, &bytes)?;
        self.api_port = String::from_utf8_lossy(&bytes)
            .lines()
            .find_map(|line| line.trim().strip_prefix("server: "))
            .and_then(|server| server.rsplit_once(':'))
            .map(|(_, port)| port.to_owned());
        Ok(())
    }

    fn import_image(&mut self) -> Result<()> {
        let mut import = self.k3d();
        import
            .args(["image", "import", "--cluster", &self.cluster])
            .arg(self.image.local("accepted"))
            .arg(self.image.local("wrong-tag"));
        self.checked(&mut import, "importing the image into the cluster")?;
        // An imported archive carries only its tag, so the runtime knows no digest reference for
        // it. The manifest digest containerd recorded on import is the image's identity; naming
        // the image by that digest is what lets the workload template pin it.
        let node = format!("k3d-{}-server-0", self.cluster);
        let tagged = format!("{}:accepted", self.image.qualified());
        let mut list = Command::new("docker");
        list.args([
            "exec",
            &node,
            "ctr",
            "--namespace",
            "k8s.io",
            "images",
            "ls",
        ])
        .arg(format!("name=={tagged}"));
        let listing = self
            .checked(&mut list, "reading the imported image digest")?
            .stdout;
        let digest = String::from_utf8_lossy(&listing)
            .lines()
            .filter(|line| line.split_whitespace().next() == Some(tagged.as_str()))
            .find_map(|line| {
                line.split_whitespace()
                    .find(|field| field.starts_with("sha256:") && field.len() == 71)
                    .map(str::to_owned)
            })
            .context("the cluster runtime lists no manifest digest for the imported image")?;
        self.image.digest = Some(digest);
        let mut tag = Command::new("docker");
        tag.args([
            "exec",
            &node,
            "ctr",
            "--namespace",
            "k8s.io",
            "images",
            "tag",
        ])
        .arg(&tagged)
        .arg(self.image.pinned());
        self.checked(
            &mut tag,
            "naming the image by its digest in the cluster runtime",
        )?;
        let mut inspect = Command::new("docker");
        inspect
            .args(["exec", &node, "crictl", "inspecti", "--output", "json"])
            .arg(self.image.pinned());
        self.checked(
            &mut inspect,
            "resolving the digest reference in the cluster runtime",
        )?;
        Ok(())
    }

    /// Writes the realization and binding documents for this run's exact image.
    fn author_inputs(&mut self, sdk: &Value) -> Result<String> {
        let template: Value = serde_yaml::from_str(&fs::read_to_string(
            self.root.join("examples/realizations/billing-local.yaml"),
        )?)?;
        let digest = self.image.digest.clone().context("image digest")?;
        let realization = json!({
            "type": "ess-realization/2",
            "id": "billing-placement",
            "specification": template["specification"],
            "synthesis": {"target": "rust-linux-x86_64-musl/1", "generator": "service-sdk/service-builder"},
            "components": [COMPONENT],
            "actors": [],
            "implementations": [{
                "id": IMPLEMENTATION,
                "components": [COMPONENT],
                "artifact": {"kind": "container", "locator": self.image.pinned(), "identity": digest},
            }],
            "entrypoints": [],
        });
        let inputs = self.scratch.join("inputs");
        fs::write(
            inputs.join("realization.json"),
            serde_json::to_vec_pretty(&realization)?,
        )?;
        let mut compile = Command::new(self.ess_path()?);
        compile
            .args(["specify", "realization", "compile", "--path"])
            .arg(inputs.join("realization.json"))
            .arg("--spec")
            .arg(&self.spec)
            .args(["--format", "json"]);
        let compiled: Value = serde_json::from_slice(
            &self
                .checked(&mut compile, "compiling the realization")?
                .stdout,
        )?;
        let realization_digest = compiled["realization_digest"]
            .as_str()
            .context("compiled realization has no digest")?
            .to_owned();
        let bindings = json!({
            "format": "ess-observed-bindings/1",
            "id": "billing-placement",
            "realization_digest": realization_digest,
            "scope": {"context": self.context, "namespace": NAMESPACE},
            "bindings": [{
                "id": "billing-api",
                "implementation": IMPLEMENTATION,
                "workload": {"kind": "deployment", "name": WORKLOAD},
                "container": CONTAINER,
                "image": self.image.pinned(),
            }],
        });
        fs::write(
            inputs.join("bindings.json"),
            serde_json::to_vec_pretty(&bindings)?,
        )?;
        fs::write(inputs.join("placement.infra-spec.yaml"), INTENT)?;
        fs::create_dir(self.evidence.join("inputs"))?;
        for name in [
            "realization.json",
            "bindings.json",
            "placement.infra-spec.yaml",
        ] {
            fs::copy(inputs.join(name), self.evidence.join("inputs").join(name))?;
        }
        fs::write(
            self.evidence.join("inputs/service.json"),
            serde_json::to_vec_pretty(sdk)?,
        )?;
        Ok(realization_digest)
    }

    /// The single route by which this module changes cluster state from a manifest.
    fn apply(&mut self, manifest: &HarnessManifest) -> Result<()> {
        let path = self.scratch.join("manifests").join(format!(
            "{:02}-{}.yaml",
            self.applied.len(),
            manifest.label
        ));
        write_private(&path, &manifest.bytes)?;
        let equal = self.projection_bytes.contains(&manifest.bytes);
        if equal {
            bail!("refusing to apply bytes identical to a projection file");
        }
        let target = path.display().to_string();
        self.kubectl(&["apply", "--filename", &target])?;
        self.applied.push(json!({
            "manifest": manifest.label,
            "sha256": hex(&Sha256::digest(&manifest.bytes)),
            "projection_bytes_equal": equal,
        }));
        Ok(())
    }

    fn rollout(&mut self) -> Result<()> {
        let waited = self.kubectl(&[
            "rollout",
            "status",
            &format!("deployment/{WORKLOAD}"),
            "--namespace",
            NAMESPACE,
            "--timeout=240s",
        ]);
        let Err(error) = waited else {
            return Ok(());
        };
        // Say why before teardown takes the cluster away: node conditions and each pod's phase
        // and container waiting reasons. Names and reasons only; no spec or environment.
        let nodes = self.kubectl(&[
            "get",
            "nodes",
            "--output",
            "jsonpath={range .items[*]}{.metadata.name}{range .status.conditions[*]} {.type}={.status}{end}{\"\\n\"}{end}",
        ]);
        let pods = self.kubectl(&[
            "get",
            "pods",
            "--namespace",
            NAMESPACE,
            "--output",
            "jsonpath={range .items[*]}{.metadata.name} {.status.phase} {.status.conditions[*].reason} {.status.containerStatuses[*].state.waiting.reason}{\"\\n\"}{end}",
        ]);
        let text = |read: Result<Output>| {
            read.map_or_else(
                |e| format!("unreadable: {e:#}"),
                |o| String::from_utf8_lossy(&o.stdout).trim().to_owned(),
            )
        };
        bail!("{error:#}\nnodes: {}\npods: {}", text(nodes), text(pods))
    }

    fn kubectl(&mut self, args: &[&str]) -> Result<Output> {
        let mut command = Command::new(KUBECTL);
        command
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECACHEDIR", self.scratch.join("kube-cache"))
            .args(["--context", &self.context])
            .args(args);
        let what = format!("kubectl {}", args.first().copied().unwrap_or_default());
        self.checked(&mut command, &what)
    }

    fn verify_live(&mut self, label: &str) -> Result<(Option<i32>, Value)> {
        let observation = self
            .scratch
            .join("observations")
            .join(format!("{label}.json"));
        let inputs = self.scratch.join("inputs");
        let mut command = Command::new(self.ess_path()?);
        command
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECACHEDIR", self.scratch.join("kube-cache"))
            .args(["verify", "bindings", "--spec"])
            .arg(&self.spec)
            .arg("--realization")
            .arg(inputs.join("realization.json"))
            .arg("--bindings")
            .arg(inputs.join("bindings.json"))
            .args(["--live", "--observation-out"])
            .arg(&observation)
            .args(["--format", "json"]);
        let output = self.record(&mut command)?;
        fs::write(
            self.evidence.join("reports").join(format!("{label}.json")),
            &output.stdout,
        )?;
        let report: Value = serde_json::from_slice(&output.stdout)
            .with_context(|| format!("{label}: the report is not JSON"))?;
        Ok((output.status.code(), report))
    }

    fn project(&mut self, observation: &Path, label: &str) -> Result<(Vec<u8>, Tree)> {
        let out = self.scratch.join(label);
        let mut command = Command::new(self.ess_path()?);
        command
            .args(["generate", "project", "kubernetes", "--spec"])
            .arg(self.scratch.join("inputs/placement.infra-spec.yaml"))
            .arg("--ir")
            .arg(observation)
            .arg("--out")
            .arg(&out)
            .args(["--format", "json"]);
        let stdout = self
            .checked(&mut command, "projecting the placement intent")?
            .stdout;
        let tree = read_tree(&out)?;
        self.projection_bytes.extend(tree.values().cloned());
        self.projection_dirs.push(out);
        Ok((stdout, tree))
    }

    /// A full sanitized live scan of the disposable cluster, the projection's input.
    fn scan_full(&mut self, label: &str) -> Result<PathBuf> {
        let observation = self
            .scratch
            .join("observations")
            .join(format!("{label}.json"));
        let mut command = Command::new(self.ess_path()?);
        command
            .env("KUBECONFIG", &self.kubeconfig)
            .env("KUBECACHEDIR", self.scratch.join("kube-cache"))
            .args(["infra", "import", "kubernetes", "--context", &self.context])
            .arg("--observation-out")
            .arg(&observation)
            .args(["--format", "json"]);
        self.checked(&mut command, "scanning the disposable cluster")?;
        Ok(observation)
    }

    fn declaration_as_observation(&mut self) -> Result<(Option<i32>, Value)> {
        let declared = HarnessManifest::render(Variant::Matching, &self.image, &sentinel()?);
        let text = String::from_utf8(declared.bytes)?;
        let items = text
            .split("\n---\n")
            .map(serde_yaml::from_str::<Value>)
            .collect::<Result<Vec<_>, _>>()?;
        let path = self.scratch.join("inputs/declaration.json");
        fs::write(
            &path,
            serde_json::to_vec_pretty(
                &json!({"apiVersion": "v1", "kind": "List", "items": items}),
            )?,
        )?;
        let inputs = self.scratch.join("inputs");
        let mut command = Command::new(self.ess_path()?);
        command
            .args(["verify", "bindings", "--spec"])
            .arg(&self.spec)
            .arg("--realization")
            .arg(inputs.join("realization.json"))
            .arg("--bindings")
            .arg(inputs.join("bindings.json"))
            .arg("--infra")
            .arg(&path)
            .args(["--format", "json"]);
        let output = self.record(&mut command)?;
        fs::write(
            self.evidence.join("reports/declaration-offered.json"),
            &output.stdout,
        )?;
        Ok((
            output.status.code(),
            serde_json::from_slice(&output.stdout)?,
        ))
    }

    fn secret_row(&mut self) -> Result<Acceptance> {
        let needles = secret_needles(self.secret.as_str());
        let mut files = Vec::new();
        let mut clean = true;
        let mut secret_observed = true;
        for entry in sorted_entries(&self.scratch.join("observations"))? {
            let bytes = fs::read(&entry)?;
            let name = file_name(&entry);
            let leaked = leaks(&bytes, &needles);
            let value: Value = serde_json::from_slice(&bytes)?;
            let has_secret = value["kinds"]["secrets"]["items"]
                .as_array()
                .is_some_and(|items| items.iter().any(|s| s["metadata"]["name"] == SECRET_NAME));
            clean &= leaked.is_empty();
            secret_observed &= has_secret;
            if leaked.is_empty() {
                fs::copy(&entry, self.evidence.join("observations").join(&name))?;
            }
            files.push(json!({"observation": name, "leaked_forms": leaked, "secret_object_observed": has_secret}));
        }
        // Every receipt written so far, reports and projections included, not only observations.
        for (relative, bytes) in read_tree(&self.evidence)? {
            let leaked = leaks(&bytes, &needles);
            clean &= leaked.is_empty();
            files.push(json!({"evidence": relative, "leaked_forms": leaked}));
        }
        // The live check is itself checked: a copy with the value planted must be caught.
        let mut planted = fs::read(self.scratch.join("observations/matching.json"))?;
        planted.extend_from_slice(needles[1].1.as_bytes());
        let detector_fires = leaks(&planted, &needles) == vec!["base64"];
        Ok(Acceptance {
            row: 6,
            statement:
                "the Secret's data and stringData never reach an observation file or a report",
            pass: clean && secret_observed && detector_fires && files.len() > 7,
            evidence: json!({
                "checked_forms": needles.iter().map(|(form, _)| form).collect::<Vec<_>>(),
                "files": files,
                "planted_copy_detected": detector_fires,
                "offline_guard": "ess-kubernetes secret_boundary::namespace_topology_is_bounded_sanitized_and_admitted_with_coverage",
            }),
        })
    }

    #[allow(clippy::too_many_lines)]
    fn teardown(&mut self, default_context_before: Option<&str>) -> Acceptance {
        let mut errors = Vec::new();
        if self.cluster_created {
            let mut delete = self.k3d();
            delete.args(["cluster", "delete", &self.cluster]);
            if let Err(error) = self.checked(&mut delete, "deleting the cluster") {
                errors.push(format!("{error:#}"));
            }
        }
        if self.images_created {
            for tag in ["accepted", "wrong-tag"] {
                let mut remove = Command::new("docker");
                remove.args(["image", "rm", &self.image.local(tag)]);
                if let Err(error) = self.checked(&mut remove, "removing an image") {
                    errors.push(format!("{error:#}"));
                }
            }
        }
        if let Err(error) = fs::remove_dir_all(&self.scratch) {
            errors.push(format!("removing scratch: {error}"));
        }
        let cluster = self.cluster.clone();
        let label = format!("label=k3d.cluster={cluster}");
        let name = format!("name={cluster}");
        let image_label = format!("label=ess.infra-acceptance.run={cluster}");
        let reference = format!("reference={}", self.image.repository);
        let mut read = |program: &str, args: &[&str]| -> String {
            let mut command = Command::new(program);
            command.args(args);
            match self.record(&mut command) {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).trim().to_owned()
                }
                Ok(output) => format!("unreadable: exit {:?}", output.status.code()),
                Err(error) => format!("unreadable: {error}"),
            }
        };
        let clusters = read("k3d", &["cluster", "list", "--output", "json"]);
        let residue = json!({
            "containers_by_label": read("docker", &["ps", "--all", "--quiet", "--filter", &label]),
            "containers_by_name": read("docker", &["ps", "--all", "--quiet", "--filter", &name]),
            "volumes_by_label": read("docker", &["volume", "ls", "--quiet", "--filter", &label]),
            "volumes_by_name": read("docker", &["volume", "ls", "--quiet", "--filter", &name]),
            "networks_by_name": read("docker", &["network", "ls", "--quiet", "--filter", &name]),
            "images_by_label": read("docker", &["image", "ls", "--quiet", "--filter", &image_label]),
            "images_by_reference": read("docker", &["image", "ls", "--quiet", "--filter", &reference]),
        });
        let cluster_listed = serde_json::from_str::<Value>(&clusters).map_or(true, |all| {
            all.as_array()
                .is_none_or(|all| all.iter().any(|c| c["name"] == json!(cluster)))
        });
        let default_context_after = self.default_context();
        let default_config_names_cluster = default_kubeconfig()
            .and_then(|path| fs::read_to_string(path).ok())
            .is_some_and(|text| text.contains(&cluster));
        let scratch_absent = !self.scratch.exists();
        // kubectl caches discovery per API endpoint under the home directory unless told otherwise;
        // every call here is told otherwise, and this reads back that none was left behind.
        let home_cache = self.api_port.as_ref().and_then(|port| {
            let home = std::env::var_os("HOME")?;
            let entry = PathBuf::from(home)
                .join(".kube/cache/discovery")
                .join(format!("0.0.0.0_{port}"));
            entry.exists().then(|| entry.display().to_string())
        });
        let empty = residue
            .as_object()
            .is_some_and(|all| all.values().all(|v| v == ""));
        Acceptance {
            row: 7,
            statement: "the cluster, the images and every owned scratch file are removed, and their absence is read back",
            pass: errors.is_empty()
                && !cluster_listed
                && empty
                && scratch_absent
                && home_cache.is_none()
                && default_context_after.as_deref() == default_context_before
                && !default_config_names_cluster,
            evidence: json!({
                "errors": errors,
                "cluster_listed": cluster_listed,
                "residue": residue,
                "scratch_absent": scratch_absent,
                "api_port": self.api_port,
                "home_kube_cache_entry": home_cache,
                "default_context_before": default_context_before,
                "default_context_after": default_context_after,
                "default_kubeconfig_names_cluster": default_config_names_cluster,
            }),
        }
    }

    /// `k3d` writes temporary host files; they go into scratch, which teardown removes.
    fn k3d(&self) -> Command {
        let mut command = Command::new("k3d");
        command.env("TMPDIR", self.scratch.join("tmp"));
        command
    }

    /// The default kubeconfig's current context, read without `KUBECONFIG` so it is the real
    /// default. Through `record`, like every other command.
    fn default_context(&mut self) -> Option<String> {
        let mut command = Command::new(KUBECTL);
        command
            .env_remove("KUBECONFIG")
            .args(["config", "current-context"]);
        let output = self.record(&mut command).ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    fn ess_path(&self) -> Result<PathBuf> {
        self.ess.clone().context("ess has not been built")
    }

    fn record(&mut self, command: &mut Command) -> Result<Output> {
        let argv: Vec<String> = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let trusted = [
            cargo(),
            self.ess
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        ];
        guard_argv(&argv, &self.projection_dirs, &self.cluster, &trusted)?;
        let started = Instant::now();
        let output = command
            .output()
            .with_context(|| format!("starting {}", argv[0]))?;
        self.commands.push(json!({
            "argv": argv,
            "exit": output.status.code(),
            "seconds": started.elapsed().as_secs_f64(),
        }));
        Ok(output)
    }

    fn checked(&mut self, command: &mut Command, what: &str) -> Result<Output> {
        let output = self.record(command)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let tail: Vec<&str> = stderr.lines().rev().take(12).collect();
            bail!(
                "{what}: exit {:?}\n{}",
                output.status.code(),
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            );
        }
        Ok(output)
    }
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

fn executables(messages: &[u8]) -> Vec<PathBuf> {
    messages
        .split(|b| *b == b'\n')
        .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
        .filter(|m| m["reason"] == "compiler-artifact")
        .filter_map(|m| m["executable"].as_str().map(PathBuf::from))
        .collect()
}

fn single_executable(messages: &[u8], name: &str) -> Result<PathBuf> {
    executables(messages)
        .into_iter()
        .find(|p| p.file_name().is_some_and(|f| f == name))
        .with_context(|| format!("cargo reported no {name} executable"))
}

/// The service SDK sources the lock file pins, as `source` strings with their exact revision.
fn sdk_sources(lock: &str) -> Vec<String> {
    let mut sources: Vec<String> = lock
        .lines()
        .filter_map(|line| line.strip_prefix("source = \""))
        .filter_map(|s| s.strip_suffix('"'))
        .filter(|s| s.contains("/service-sdk"))
        .map(str::to_owned)
        .collect();
    sources.sort();
    sources.dedup();
    sources
}

fn dns_label(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn git_ancestor(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|a| fs::symlink_metadata(a.join(".git")).is_ok())
        .map(Path::to_path_buf)
}

/// A random value, the one input `render` takes that is not an embedded template. Only
/// [`sentinel`] makes one, so no file's bytes can be passed to `render` in its place.
struct Sentinel(String);

impl Sentinel {
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn sentinel() -> Result<Sentinel> {
    use std::io::Read as _;
    let mut bytes = [0_u8; 16];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(Sentinel(format!("ess-acceptance-{}", hex(&bytes))))
}

/// Every form in which the Secret's value could reach bytes on disk.
fn secret_needles(secret: &str) -> Vec<(&'static str, String)> {
    vec![
        ("plain", secret.to_owned()),
        ("base64", base64(secret.as_bytes())),
    ]
}

fn leaks(bytes: &[u8], needles: &[(&'static str, String)]) -> Vec<&'static str> {
    needles
        .iter()
        .filter(|(_, needle)| {
            bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
        })
        .map(|(form, _)| *form)
        .collect()
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0_u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?
        .write_all(bytes)?;
    Ok(())
}

fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn read_tree(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in sorted_entries(&dir)? {
            if entry.is_dir() {
                pending.push(entry);
            } else {
                let relative = entry.strip_prefix(root)?.to_string_lossy().into_owned();
                files.insert(relative, fs::read(&entry)?);
            }
        }
    }
    Ok(files)
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    for (relative, bytes) in read_tree(from)? {
        let target = to.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, bytes)?;
    }
    Ok(())
}

type Tree = BTreeMap<String, Vec<u8>>;

fn split_ownership(tree: &Tree) -> (Tree, Tree) {
    tree.iter()
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .partition(|(path, _)| !path.starts_with(".ess-output/"))
}

fn tree_digest(tree: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    for (path, bytes) in tree {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(hex(&Sha256::digest(bytes)).as_bytes());
        hasher.update([0]);
    }
    hex(&hasher.finalize())
}

/// Expectation ids the projection reports as not holding, in its own order.
fn gap_ids(projection: &[u8]) -> Vec<String> {
    let Ok(value) = serde_json::from_slice::<Value>(projection) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    collect_gap_ids(&value, &mut ids);
    ids.sort();
    ids.dedup();
    ids
}

fn collect_gap_ids(value: &Value, ids: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let (Some(Value::String(id)), Some(verdict)) =
                (map.get("expectation"), map.get("verdict"))
            {
                if verdict != "holds" {
                    ids.push(id.clone());
                }
            }
            if let Some(Value::String(id)) = map.get("expectation") {
                if map.contains_key("gap") || map.contains_key("decision") {
                    ids.push(id.clone());
                }
            }
            map.values().for_each(|v| collect_gap_ids(v, ids));
        }
        Value::Array(items) => items.iter().for_each(|v| collect_gap_ids(v, ids)),
        _ => {}
    }
}

fn default_kubeconfig() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".kube/config"))
}

/// Every cluster this harness creates, every context it reads and every node it enters carries
/// this prefix. Nothing else on the machine does, so a name outside it is refused before it is used.
const CLUSTER_PREFIX: &str = "ess-m8-";

/// The disposable cluster and context a run name selects, or a refusal.
fn names(run: &str) -> Result<(String, String)> {
    let cluster = format!("ess-{run}");
    if !dns_label(run)
        || run.len() > 40
        || !cluster.starts_with(CLUSTER_PREFIX)
        || cluster.len() <= CLUSTER_PREFIX.len()
    {
        bail!("--run must be a short lowercase DNS label starting with `m8-`, naming a cluster `{CLUSTER_PREFIX}*`");
    }
    let context = format!("k3d-{cluster}");
    Ok((cluster, context))
}

/// Refuses a cluster name the listing already carries, and a listing that cannot be read.
fn refuse_existing(clusters: &Value, name: &str) -> Result<()> {
    let all = clusters
        .as_array()
        .context("the k3d cluster listing is not a list, so absence cannot be established")?;
    if all.iter().any(|c| c["name"] == json!(name)) {
        bail!("cluster {name} already exists; it is not this run's");
    }
    Ok(())
}

/// The programs the harness runs by bare name; `cargo` and the built `ess` are passed in by path.
const HARNESS_PROGRAMS: &[&str] = &["k3d", KUBECTL, "docker", "tar"];

/// Refuses a command before it starts if it could reach a projection output, deliver files into a
/// node, or act on any cluster, context or node that is not this run's own.
///
/// Paths are compared after lexical normalisation, and a `..` component is refused outright: the
/// guard runs before any file exists, so it cannot resolve links and does not pretend to.
#[allow(clippy::too_many_lines)]
fn guard_argv(
    argv: &[String],
    projection_dirs: &[PathBuf],
    cluster: &str,
    trusted: &[String],
) -> Result<()> {
    // An allowlist, not a denylist: a program that runs another program (a shell, `env`, `xargs`)
    // would carry any argv past every rule below. Named exactly, so no path can stand in for one.
    let started = argv.first().map(String::as_str).unwrap_or_default();
    if !(HARNESS_PROGRAMS.contains(&started)
        || trusted.iter().any(|t| !t.is_empty() && t == started))
    {
        bail!("refusing `{started}`: not one of the programs this harness runs");
    }
    let context = format!("k3d-{cluster}");
    let node = format!("k3d-{cluster}-server-0");
    let program = argv
        .first()
        .and_then(|p| Path::new(p).file_name())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let operands = argv.get(1..).unwrap_or_default();
    for arg in operands {
        let value = arg.split_once('=').map_or(arg.as_str(), |(_, v)| v);
        for candidate in [arg.as_str(), value] {
            let path = Path::new(candidate);
            if path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                bail!("refusing `{program}`: {candidate} climbs with `..`");
            }
            let normal: PathBuf = path
                .components()
                .filter(|c| *c != std::path::Component::CurDir)
                .collect();
            if projection_dirs.iter().any(|dir| normal.starts_with(dir)) {
                bail!("refusing `{program}`: {candidate} is a projection output");
            }
        }
    }
    let flag_value = |long: &str, short: Option<&str>| -> Vec<String> {
        let mut values = Vec::new();
        for (index, arg) in operands.iter().enumerate() {
            if let Some(inline) = arg.strip_prefix(&format!("{long}=")) {
                values.push(inline.to_owned());
            } else if arg == long || short.is_some_and(|s| arg == s) {
                values.push(operands.get(index + 1).cloned().unwrap_or_default());
            }
        }
        values
    };
    for named in flag_value("--context", None) {
        if named != context {
            bail!("refusing `{program}`: context {named} is not this run's ({context})");
        }
    }
    let words: Vec<&str> = operands
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with('-'))
        .collect();
    match program.as_str() {
        "kubectl" => {
            for flag in ["-k", "--kustomize", "-R", "--recursive"] {
                if operands
                    .iter()
                    .any(|a| a == flag || a.starts_with(&format!("{flag}=")))
                {
                    bail!("refusing `kubectl {flag}`: only rendered manifest files are applied");
                }
            }
        }
        "k3d" => {
            if operands.iter().any(|a| a == "--all" || a == "-a") {
                bail!("refusing `k3d --all`: only this run's cluster is touched");
            }
            if operands
                .iter()
                .any(|a| a == "--volume" || a == "-v" || a.starts_with("--volume="))
            {
                bail!("refusing `k3d --volume`: nothing is mounted into a node");
            }
            for named in flag_value("--cluster", Some("-c")) {
                if named != cluster {
                    bail!("refusing `k3d`: cluster {named} is not this run's ({cluster})");
                }
            }
            // `k3d <noun> <verb> <name…>`: the names run from the verb to the first flag, and every
            // one must be this run's own.
            if let [noun, verb, rest @ ..] = operands {
                let names: Vec<&str> = rest
                    .iter()
                    .take_while(|a| !a.starts_with('-'))
                    .map(String::as_str)
                    .collect();
                let own = |name: &str| match noun.as_str() {
                    "node" => name.starts_with(&format!("k3d-{cluster}-")),
                    _ => name == cluster,
                };
                match (noun.as_str(), verb.as_str()) {
                    ("cluster" | "node", "list") | ("image", "import") => {}
                    ("cluster", "create" | "delete") | ("kubeconfig", "get") => {
                        if names.is_empty() || !names.iter().all(|n| own(n)) {
                            bail!("refusing `k3d {noun} {verb}`: {names:?} is not only this run's");
                        }
                    }
                    _ => bail!("refusing `k3d {noun} {verb}`: not a verb this harness uses"),
                }
            }
        }
        "docker" => {
            // The verb may follow global flags or a management command: `docker container cp`.
            if words.contains(&"cp") {
                bail!("refusing `docker cp`: nothing is copied into a node");
            }
            if let Some(position) = operands.iter().position(|a| a == "exec") {
                let mut rest = operands[position + 1..].iter();
                let mut target = None;
                while let Some(arg) = rest.next() {
                    if [
                        "-e",
                        "--env",
                        "-u",
                        "--user",
                        "-w",
                        "--workdir",
                        "--env-file",
                    ]
                    .contains(&arg.as_str())
                    {
                        rest.next();
                    } else if !arg.starts_with('-') {
                        target = Some(arg.as_str());
                        break;
                    }
                }
                if target != Some(node.as_str()) {
                    bail!("refusing `docker exec`: {target:?} is not this run's node ({node})");
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::visit::Visit;

    const SOURCE: &str = include_str!("infra_acceptance.rs");

    /// What one function of this module does, as far as the guard is concerned.
    #[derive(Default, Debug)]
    struct Function {
        programs: Vec<String>,
        literals: Vec<String>,
        kubectl_verbs: Vec<String>,
        reads: Vec<String>,
        manifest_literals: usize,
        sentinel_literals: usize,
        inputs: Vec<String>,
        /// `.output()`, `.status()`, `.spawn()` and `CommandExt::exec` calls: the ways a
        /// `Command` runs, found in expressions and in macro token streams alike.
        process_starts: usize,
        /// First arguments of calls to the local `read` helper, which runs its argument.
        read_programs: Vec<String>,
        /// First elements of tuple expressions (preflight's `(tool, args)` list).
        tuple_heads: Vec<String>,
        /// Every method this function calls by name.
        methods: Vec<String>,
    }

    /// Every name `std::process::Command` goes by in this module: itself, a `use … as` rename and
    /// a `type` alias. A start is then found by the type it names, not by how the path is spelled.
    fn command_names(file: &syn::File) -> std::collections::BTreeSet<String> {
        fn walk(tree: &syn::UseTree, names: &mut std::collections::BTreeSet<String>) {
            match tree {
                syn::UseTree::Path(path) => walk(&path.tree, names),
                syn::UseTree::Group(group) => group.items.iter().for_each(|t| walk(t, names)),
                syn::UseTree::Rename(rename) if names.contains(&rename.ident.to_string()) => {
                    names.insert(rename.rename.to_string());
                }
                _ => {}
            }
        }
        struct Collect(std::collections::BTreeSet<String>);
        impl<'ast> Visit<'ast> for Collect {
            fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
                walk(&item.tree, &mut self.0);
            }
            fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
                let ty = &item.ty;
                let text = quote::quote!(#ty).to_string();
                if text
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|t| self.0.contains(t))
                {
                    self.0.insert(item.ident.to_string());
                }
            }
        }
        let mut names = Collect(["Command".to_owned()].into_iter().collect());
        // Twice, so an alias of an alias declared in either order is found.
        names.visit_file(file);
        names.visit_file(file);
        names.0
    }

    /// Every function outside this test module, keyed by name; item-level code is `<item>`.
    #[derive(Default)]
    struct Scan {
        command_names: std::collections::BTreeSet<String>,
        impl_type: Vec<String>,
        current: Vec<String>,
        functions: BTreeMap<String, Function>,
    }

    impl Scan {
        fn here(&mut self) -> &mut Function {
            let name = self
                .current
                .last()
                .cloned()
                .unwrap_or_else(|| "<item>".to_owned());
            self.functions.entry(name).or_default()
        }
        fn enter(&mut self, sig: &syn::Signature) {
            self.current.push(sig.ident.to_string());
            let inputs = sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    syn::FnArg::Typed(typed) => Some(quote::quote!(#typed).to_string()),
                    syn::FnArg::Receiver(_) => None,
                })
                .collect::<Vec<_>>();
            self.here().inputs.extend(inputs);
        }
        /// String literals inside macro invocations are tokens, not expressions; read them too,
        /// so `vec!["kubectl", …]` or `json!` keys cannot hide an argument from the scan.
        fn tokens(&mut self, stream: proc_macro2::TokenStream) {
            let trees: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();
            for (index, tree) in trees.iter().cloned().enumerate() {
                if let proc_macro2::TokenTree::Ident(ident) = &tree {
                    let name = ident.to_string();
                    if self.command_names.contains(&name) {
                        self.here().programs.push(format!("{name}-in-macro"));
                    }
                    let after_dot = index > 0
                        && matches!(&trees[index - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '.');
                    let called = matches!(trees.get(index + 1), Some(proc_macro2::TokenTree::Group(g)) if g.delimiter() == proc_macro2::Delimiter::Parenthesis);
                    if after_dot
                        && called
                        && ["output", "status", "spawn", "exec"].contains(&name.as_str())
                    {
                        self.here().process_starts += 1;
                    }
                }
                match tree {
                    proc_macro2::TokenTree::Group(group) => self.tokens(group.stream()),
                    proc_macro2::TokenTree::Literal(literal) => {
                        if let Ok(syn::Lit::Str(text)) =
                            syn::parse_str::<syn::Lit>(&literal.to_string())
                        {
                            self.here().literals.push(text.value());
                        }
                    }
                    proc_macro2::TokenTree::Ident(ident) => {
                        if ident == "KUBECTL" {
                            self.here().programs.push("KUBECTL-in-macro".to_owned());
                        }
                    }
                    proc_macro2::TokenTree::Punct(_) => {}
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Scan {
        fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
            if item.ident != "tests" {
                syn::visit::visit_item_mod(self, item);
            }
        }
        fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
            let ty = &item.self_ty;
            self.impl_type.push(quote::quote!(#ty).to_string());
            syn::visit::visit_item_impl(self, item);
            self.impl_type.pop();
        }
        fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
            self.enter(&item.sig);
            syn::visit::visit_impl_item_fn(self, item);
            self.current.pop();
        }
        fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
            self.enter(&item.sig);
            syn::visit::visit_item_fn(self, item);
            self.current.pop();
        }
        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            self.tokens(mac.tokens.clone());
            syn::visit::visit_macro(self, mac);
        }
        fn visit_expr_tuple(&mut self, tuple: &'ast syn::ExprTuple) {
            if let Some(head) = tuple.elems.first() {
                let text = quote::quote!(#head).to_string().replace(' ', "");
                self.here().tuple_heads.push(text);
            }
            syn::visit::visit_expr_tuple(self, tuple);
        }
        fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
            self.here().literals.push(lit.value());
        }
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            let func = &call.func;
            let callee = quote::quote!(#func).to_string().replace(' ', "");
            let names_command = callee
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|token| self.command_names.contains(token));
            if names_command {
                let program = call
                    .args
                    .first()
                    .map(|arg| quote::quote!(#arg).to_string().replace(' ', ""))
                    .unwrap_or_default();
                self.here().programs.push(program);
            }
            if callee == "read" {
                let program = call
                    .args
                    .first()
                    .map(|arg| quote::quote!(#arg).to_string().replace(' ', ""))
                    .unwrap_or_default();
                self.here().read_programs.push(program);
            }
            let in_sentinel = self.impl_type.last().is_some_and(|t| t == "Sentinel");
            if callee == "Sentinel" || (callee == "Self" && in_sentinel) {
                self.here().sentinel_literals += 1;
            }
            syn::visit::visit_expr_call(self, call);
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            let method = call.method.to_string();
            if ["output", "status", "spawn", "exec"].contains(&method.as_str()) {
                self.here().process_starts += 1;
            }
            self.here().methods.push(method);
            if call.method == "kubectl" {
                let verb = call
                    .args
                    .first()
                    .and_then(|arg| match arg {
                        syn::Expr::Reference(r) => match &*r.expr {
                            syn::Expr::Array(a) => a.elems.first().cloned(),
                            _ => None,
                        },
                        _ => None,
                    })
                    .and_then(|first| match first {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) => Some(s.value()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "<not a literal verb>".to_owned());
                self.here().kubectl_verbs.push(verb);
            }
            syn::visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            let text = quote::quote!(#path).to_string().replace(' ', "");
            if text.ends_with("read_to_string")
                || text.ends_with("fs::read")
                || text.ends_with("File::open")
                || text.ends_with("read_dir")
                || text.ends_with("read_tree")
            {
                self.here().reads.push(text);
            }
            syn::visit::visit_expr_path(self, path);
        }
        fn visit_expr_struct(&mut self, expr: &'ast syn::ExprStruct) {
            let in_manifest = self
                .impl_type
                .last()
                .is_some_and(|t| t == "HarnessManifest");
            let in_sentinel = self.impl_type.last().is_some_and(|t| t == "Sentinel");
            if expr.path.is_ident("HarnessManifest") || (expr.path.is_ident("Self") && in_manifest)
            {
                self.here().manifest_literals += 1;
            }
            if expr.path.is_ident("Sentinel") || (expr.path.is_ident("Self") && in_sentinel) {
                self.here().sentinel_literals += 1;
            }
            syn::visit::visit_expr_struct(self, expr);
        }
    }

    fn scan_source(source: &str) -> syn::Result<Scan> {
        let file = syn::parse_file(source)?;
        let mut scan = Scan {
            command_names: command_names(&file),
            ..Scan::default()
        };
        scan.visit_file(&file);
        Ok(scan)
    }

    /// Which program each function may start. Anything else — a new `Command::new`, or a known
    /// program started from a function not listed for it — fails the guard.
    const PROGRAMS: &[(&str, &[&str])] = &[
        ("\"k3d\"", &["k3d"]),
        ("KUBECTL", &["kubectl", "default_context"]),
        ("tool", &["preflight"]),
        ("program", &["teardown"]),
        ("\"docker\"", &["build_image", "import_image", "teardown"]),
        ("\"tar\"", &["build_image"]),
        ("cargo()", &["build_ess", "build_service"]),
        (
            "self.ess_path()?",
            &[
                "author_inputs",
                "verify_live",
                "project",
                "scan_full",
                "declaration_as_observation",
            ],
        ),
    ];

    /// Arguments that could deliver bytes into the cluster by a route other than `apply`: a
    /// kubectl run through `docker exec`, a `docker cp` into a node, a k3d volume onto the node's
    /// manifest directory, or kubectl's other writing verbs.
    const FORBIDDEN: &[&str] = &[
        "cp",
        "--volume",
        "-v",
        "replace",
        "patch",
        "edit",
        "set",
        "run",
        "--filename=-",
    ];

    /// Programs `read` (teardown) and `tool` (preflight) may carry: the same allowlist
    /// `guard_argv` enforces at run time, spelled as source.
    const ALLOWED_PROGRAM_VALUES: &[&str] = &["\"docker\"", "\"k3d\"", "KUBECTL", "\"tar\""];

    /// Every rule the harness source must satisfy, as findings rather than panics, so a mutated
    /// copy of the source can be shown to fail.
    #[allow(clippy::too_many_lines)]
    fn violations(source: &str) -> Vec<String> {
        let Ok(scan) = scan_source(source) else {
            return vec!["the source does not parse".to_owned()];
        };
        let mut found = Vec::new();
        let mut check = |ok: bool, message: String| {
            if !ok {
                found.push(message);
            }
        };
        let mut programs = 0;
        for (function, facts) in &scan.functions {
            for program in &facts.programs {
                programs += 1;
                let allowed = PROGRAMS
                    .iter()
                    .find(|(p, _)| p == program)
                    .is_some_and(|(_, fns)| fns.contains(&function.as_str()));
                check(
                    allowed,
                    format!(
                        "{function} starts `{program}`, which this harness may not start there"
                    ),
                );
            }
            check(
                facts.process_starts == 0 || function == "record",
                format!(
                    "{function} runs a process itself; every command runs through Run::record, \
                     where guard_argv sees it first"
                ),
            );
            for value in &facts.read_programs {
                check(
                    ALLOWED_PROGRAM_VALUES.contains(&value.as_str()),
                    format!("{function} hands `{value}` to a program-running helper"),
                );
            }
            if function == "preflight" {
                for head in &facts.tuple_heads {
                    check(
                        ALLOWED_PROGRAM_VALUES.contains(&head.as_str()),
                        format!("preflight runs the tool `{head}`, outside the allowlist"),
                    );
                }
            }
            // The argument guard spells the verbs it refuses; it starts nothing and returns no text.
            if function == "guard_argv" {
                check(
                    facts.programs.is_empty(),
                    "the argument guard starts a program".to_owned(),
                );
                for called in ["record", "checked", "kubectl", "k3d"] {
                    check(
                        !facts.methods.iter().any(|m| m == called),
                        format!("the argument guard calls {called}, which runs a process"),
                    );
                }
                continue;
            }
            for literal in &facts.literals {
                let text = literal.as_str();
                check(
                    !FORBIDDEN.contains(&text),
                    format!(
                        "{function} passes `{text}`, a route into the cluster other than apply"
                    ),
                );
                check(
                    text != "kubectl" || function == "<item>",
                    format!(
                        "{function} names kubectl directly instead of going through Run::kubectl"
                    ),
                );
                check(
                    !text.contains("rancher") && !text.contains("server/manifests"),
                    format!("{function} names the node's manifest directory: {text}"),
                );
                for (verb, only) in [
                    ("apply", "apply"),
                    ("create", "create_cluster"),
                    ("exec", "import_image"),
                ] {
                    check(
                        text != verb || function == only,
                        format!("{function} passes `{verb}`; only {only} may"),
                    );
                }
                check(
                    !text.starts_with("projection-")
                        || ["execute", "project"].contains(&function.as_str()),
                    format!("{function} names a projection output ({text})"),
                );
            }
            for verb in &facts.kubectl_verbs {
                check(
                    ["apply", "delete", "rollout", "get"].contains(&verb.as_str()),
                    format!(
                        "{function} runs kubectl {verb}, outside the verbs this harness may use"
                    ),
                );
                check(
                    verb != "apply" || function == "apply",
                    format!("kubectl apply is called from {function}"),
                );
            }
            if function == "apply" || function == "render" {
                check(
                    facts.reads.is_empty(),
                    format!(
                        "{function} reads a file ({:?}), so its bytes are no longer only the templates",
                        facts.reads
                    ),
                );
            }
        }
        check(
            programs >= 10,
            format!("the scan found {programs} program starts; it is not reading this module"),
        );
        let function = |name: &str| scan.functions.get(name);
        check(
            function("guard_argv").is_some_and(|f| {
                f.inputs
                    == [
                        "argv : & [String]",
                        "projection_dirs : & [PathBuf]",
                        "cluster : & str",
                        "trusted : & [String]",
                    ]
            }),
            "guard_argv's signature changed".to_owned(),
        );
        check(
            function("record").is_some_and(|f| f.process_starts == 1),
            "record runs exactly one command, after guard_argv".to_owned(),
        );
        check(
            function("teardown").is_some_and(|f| !f.read_programs.is_empty()),
            "the scan found no program handed to teardown's reader".to_owned(),
        );
        check(
            function("apply").is_some_and(|f| {
                f.inputs == ["manifest : & HarnessManifest"]
                    && f.kubectl_verbs == ["apply"]
                    && f.programs.is_empty()
            }),
            "apply must take only a HarnessManifest and run only `kubectl apply`".to_owned(),
        );
        let built: Vec<&str> = scan
            .functions
            .iter()
            .filter(|(_, f)| f.manifest_literals > 0)
            .map(|(n, _)| n.as_str())
            .collect();
        check(
            built == ["render"],
            format!("a HarnessManifest is built in {built:?}"),
        );
        let sentinels: Vec<&str> = scan
            .functions
            .iter()
            .filter(|(_, f)| f.sentinel_literals > 0)
            .map(|(n, _)| n.as_str())
            .collect();
        check(
            sentinels == ["sentinel"],
            format!("a Sentinel is made outside sentinel(): {sentinels:?}"),
        );
        check(
            function("render")
                .is_some_and(|f| f.inputs.contains(&"secret : & Sentinel".to_owned())),
            "render's secret must be a Sentinel".to_owned(),
        );
        found
    }

    /// Acceptance 3's structural half: a projection file has no route into the cluster.
    ///
    /// Every program this module starts is one it is allowed to start from that function, and
    /// every process runs through `record`, which `guard_argv` sees first; no argument anywhere
    /// names kubectl except the one wrapper, a writing verb, a copy into a node, a volume onto its
    /// manifest directory or a projection output; `apply` is reached only from `fn apply`, which
    /// takes only a `HarnessManifest`; a `HarnessManifest` is built only in `render`, whose secret
    /// is a `Sentinel` only `sentinel()` can make; and neither `apply` nor `render` reads a file.
    /// So the only bytes that can reach the cluster are the embedded templates and a random value.
    #[test]
    fn only_harness_manifests_reach_apply() {
        let found = violations(SOURCE);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    /// Every bypass an adversary has shown, applied to a copy of this module's own source. Each
    /// must be found; a fixture whose anchor has moved fails too, so none goes stale silently.
    /// One known bypass: a name, the textual edits that make it, and the finding that must appear.
    type Bypass = (
        &'static str,
        &'static [(&'static str, &'static str)],
        &'static str,
    );

    #[test]
    #[allow(clippy::too_many_lines)]
    fn the_scanner_finds_every_known_bypass() {
        let fixtures: &[Bypass] = &[
            (
                "round 2: docker exec kubectl apply from project",
                &[(
                    "        self.projection_dirs.push(out);\n",
                    "        self.projection_dirs.push(out);\n        let mut smuggle = Command::new(\"docker\");\n        smuggle.args([\"exec\", \"node\", \"kubectl\", \"apply\"]);\n        self.checked(&mut smuggle, \"x\")?;\n",
                )],
                "project starts `\"docker\"`",
            ),
            (
                "round 2: kubectl create from project",
                &[(
                    "        self.projection_dirs.push(out);\n",
                    "        let mut direct = Command::new(KUBECTL);\n        direct.args([\"create\", \"--filename\"]).arg(&out);\n        self.checked(&mut direct, \"x\")?;\n        self.projection_dirs.push(out);\n",
                )],
                "project starts `KUBECTL`",
            ),
            (
                "round 2: projection bytes smuggled through the secret",
                &[(
                    "        let matching = HarnessManifest::render(Variant::Matching, &self.image, &self.secret);\n",
                    "        let smuggled = Sentinel(fs::read_to_string(\"p\")?);\n        let matching = HarnessManifest::render(Variant::Matching, &self.image, &smuggled);\n",
                )],
                "a Sentinel is made outside sentinel()",
            ),
            (
                "round 2: docker cp from build_image",
                &[(
                    "        fs::remove_file(tarball)?;\n",
                    "        fs::remove_file(tarball)?;\n        let mut copy = Command::new(\"docker\");\n        copy.args([\"cp\", \"p\", \"node:/p\"]);\n        self.checked(&mut copy, \"x\")?;\n",
                )],
                "build_image passes `cp`",
            ),
            (
                "round 2: k3d volume onto the manifest directory",
                &[(
                    "            .args([\"--k3s-arg\", \"--disable=local-storage@server:0\"])\n",
                    "            .args([\"--k3s-arg\", \"--disable=local-storage@server:0\"])\n            .args([\"--volume\", \"/p:/var/lib/rancher/k3s/server/manifests/p@server:0\"])\n",
                )],
                "create_cluster passes `--volume`",
            ),
            (
                "round 3 M1: a type alias of Command",
                &[
                    (
                        "use std::process::{Command, Output};\n",
                        "use std::process::{Command, Output};\ntype Launcher = Command;\n",
                    ),
                    (
                        "    fn rollout(&mut self) -> Result<()> {\n",
                        "    fn rollout(&mut self) -> Result<()> {\n        let mut launched = Launcher::new(self.ess_path()?);\n        self.checked(&mut launched, \"x\")?;\n",
                    ),
                ],
                "rollout starts `self.ess_path()?`",
            ),
            (
                "round 3 M2: a `use … as` rename of Command",
                &[
                    (
                        "use std::process::{Command, Output};\n",
                        "use std::process::{Command, Output};\nuse std::process::Command as Proc;\n",
                    ),
                    (
                        "    fn rollout(&mut self) -> Result<()> {\n",
                        "    fn rollout(&mut self) -> Result<()> {\n        let mut launched = Proc::new(\"docker\");\n        self.checked(&mut launched, \"x\")?;\n",
                    ),
                ],
                "rollout starts `\"docker\"`",
            ),
            (
                "round 3 M3: a process started outside record",
                &[(
                    "        self.checked(&mut command, &what)\n",
                    "        let _ = &what;\n        Ok(command.output()?)\n",
                )],
                "kubectl runs a process itself",
            ),
            (
                "round 3 M4: a process started inside guard_argv",
                &[(
                    "    let context = format!(\"k3d-{cluster}\");\n    let node",
                    "    let _ = std::process::Command::new(\"docker\").status();\n    let context = format!(\"k3d-{cluster}\");\n    let node",
                )],
                "guard_argv starts `\"docker\"`",
            ),
            (
                "round 4: a shell handed to teardown's reader",
                &[(
                    "read(\"k3d\", &[\"cluster\", \"list\", \"--output\", \"json\"])",
                    "read(\"sh\", &[\"-c\", \"k3d cluster list --output json\"])",
                )],
                "teardown hands `\"sh\"` to a program-running helper",
            ),
            (
                "round 4: a Command built and run inside macro_rules!",
                &[
                    (
                        "use std::process::{Command, Output};\n",
                        "use std::process::{Command, Output};\nmacro_rules! launch {\n    ($p:expr) => {\n        std::process::Command::new($p).output()\n    };\n}\n",
                    ),
                    (
                        "    fn rollout(&mut self) -> Result<()> {\n",
                        "    fn rollout(&mut self) -> Result<()> {\n        let _ = launch!(\"docker\");\n",
                    ),
                ],
                "<item> runs a process itself",
            ),
            (
                "round 4: CommandExt::exec in default_context",
                &[
                    (
                        "use std::process::{Command, Output};\n",
                        "use std::process::{Command, Output};\nuse std::os::unix::process::CommandExt as _;\n",
                    ),
                    (
                        "        let output = self.record(&mut command).ok()?;\n",
                        "        let _replaced = command.exec();\n        let output = self.record(&mut command).ok()?;\n",
                    ),
                ],
                "default_context runs a process itself",
            ),
        ];
        for (name, edits, expected) in fixtures {
            let mut mutated = SOURCE.to_owned();
            for (from, to) in *edits {
                let at = mutated
                    .find(from)
                    .unwrap_or_else(|| panic!("{name}: anchor moved: {from}"));
                mutated.replace_range(at..at + from.len(), to);
            }
            let found = violations(&mutated);
            assert!(
                found.iter().any(|v| v.contains(expected)),
                "{name}: expected a finding containing {expected:?}, got {found:#?}"
            );
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn a_command_naming_a_projection_output_or_a_foreign_cluster_is_refused_before_it_runs() {
        let dirs = [PathBuf::from("/scratch/run/projection-a")];
        let argv = |args: &[&str]| args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
        let own = "ess-m8-x";
        let trusted = ["/usr/bin/cargo".to_owned(), "/target/debug/ess".to_owned()];
        for admitted in [
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--filename",
                "/scratch/run/manifests/00-matching.yaml",
            ]),
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "rollout",
                "status",
                "deployment/billing",
            ]),
            argv(&["k3d", "cluster", "create", "ess-m8-x", "--servers", "1"]),
            argv(&["k3d", "cluster", "delete", "ess-m8-x"]),
            argv(&["k3d", "cluster", "list", "--output", "json"]),
            argv(&["k3d", "kubeconfig", "get", "ess-m8-x"]),
            argv(&[
                "k3d",
                "image",
                "import",
                "--cluster",
                "ess-m8-x",
                "ess-m8-x/billing:accepted",
            ]),
            argv(&[
                "docker",
                "exec",
                "k3d-ess-m8-x-server-0",
                "ctr",
                "--namespace",
                "k8s.io",
                "images",
                "ls",
            ]),
            argv(&["docker", "image", "rm", "ess-m8-x/billing:accepted"]),
            argv(&[
                "docker",
                "ps",
                "--all",
                "--quiet",
                "--filter",
                "name=ess-m8-x",
            ]),
        ] {
            guard_argv(&admitted, &dirs, own, &trusted)
                .unwrap_or_else(|e| panic!("refused own {admitted:?}: {e:#}"));
        }
        for refused in [
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--filename",
                "/scratch/run/projection-a",
            ]),
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--filename=/scratch/run/projection-a/patches/p.json",
            ]),
            argv(&[
                "docker",
                "cp",
                "/scratch/run/projection-a/objects",
                "k3d-ess-m8-x-server-0:/tmp",
            ]),
            argv(&[
                "kubectl",
                "--context",
                "k3d-production",
                "delete",
                "deployment",
                "billing",
            ]),
            argv(&["kubectl", "--context=k3d-production", "get", "pods"]),
            argv(&["kubectl", "--context", "k3d-ess-m8-other", "get", "pods"]),
            argv(&["k3d", "cluster", "delete", "production"]),
            argv(&[
                "k3d",
                "image",
                "import",
                "--cluster",
                "production",
                "image:tag",
            ]),
            argv(&[
                "docker",
                "exec",
                "k3d-production-server-0",
                "ctr",
                "images",
                "ls",
            ]),
        ] {
            assert!(
                guard_argv(&refused, &dirs, own, &trusted).is_err(),
                "admitted {refused:?}"
            );
        }
    }

    /// The adversary's second pass: ten spellings the first guard admitted.
    #[test]
    fn case_b_every_spelling_that_reaches_a_projection_or_a_foreign_target_is_refused() {
        let dirs = [PathBuf::from("/scratch/run/projection-a")];
        let argv = |args: &[&str]| args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
        let own = "ess-m8-x";
        let trusted = ["/usr/bin/cargo".to_owned(), "/target/debug/ess".to_owned()];
        for refused in [
            // a path that only reaches the projection after `..` is resolved
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--filename",
                "/scratch/run/manifests/../projection-a/patches",
            ]),
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--filename",
                "/scratch/run/./projection-a",
            ]),
            // kustomize and recursive application read directories the harness did not render
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "-k",
                "/scratch/run/manifests",
            ]),
            argv(&[
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "--recursive",
                "--filename",
                "/scratch/run/manifests",
            ]),
            // short and inline cluster spellings
            argv(&["k3d", "image", "import", "-c", "production", "image:tag"]),
            argv(&[
                "k3d",
                "image",
                "import",
                "--cluster=production",
                "image:tag",
            ]),
            // the docker verb behind a management command or a global flag
            argv(&[
                "docker",
                "container",
                "cp",
                "/scratch/run/projection-a",
                "k3d-ess-m8-x-server-0:/var/tmp",
            ]),
            argv(&[
                "docker",
                "--log-level",
                "error",
                "container",
                "exec",
                "k3d-production-server-0",
                "ctr",
                "images",
                "ls",
            ]),
            // a node carrying the prefix that is not this run's own server
            argv(&["docker", "exec", "k3d-ess-m8-x-serverlb", "sh"]),
            argv(&[
                "docker",
                "exec",
                "k3d-ess-m8-other-server-0",
                "ctr",
                "images",
                "ls",
            ]),
            // stopping or deleting what is not this run's
            argv(&["k3d", "cluster", "stop", "ess-m8-other"]),
            argv(&["k3d", "node", "delete", "k3d-ess-m8-other-server-0"]),
            argv(&["k3d", "cluster", "delete", "--all"]),
        ] {
            assert!(
                guard_argv(&refused, &dirs, own, &trusted).is_err(),
                "admitted {refused:?}"
            );
        }
    }

    /// Only the programs the harness runs may start, named exactly: a wrapper that runs another
    /// program — a shell, `env`, `xargs` — would carry any argv past every other rule.
    #[test]
    fn a_program_outside_the_allowlist_is_refused_however_it_is_wrapped() {
        let dirs: [PathBuf; 0] = [];
        let argv = |args: &[&str]| args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
        let trusted = ["/usr/bin/cargo".to_owned(), "/target/debug/ess".to_owned()];
        for admitted in [
            argv(&["/usr/bin/cargo", "build", "--locked"]),
            argv(&["/target/debug/ess", "verify", "bindings"]),
            argv(&["tar", "--version"]),
        ] {
            guard_argv(&admitted, &dirs, "ess-m8-x", &trusted)
                .unwrap_or_else(|e| panic!("refused own {admitted:?}: {e:#}"));
        }
        for refused in [
            argv(&[
                "sh",
                "-c",
                "kubectl --context k3d-ess-m8-x apply -f /scratch/run/projection-a",
            ]),
            argv(&[
                "env",
                "kubectl",
                "--context",
                "k3d-ess-m8-x",
                "apply",
                "-f",
                "/p",
            ]),
            argv(&["xargs", "kubectl", "apply", "-f"]),
            argv(&[
                "/usr/bin/env",
                "docker",
                "cp",
                "/p",
                "k3d-ess-m8-x-server-0:/p",
            ]),
            argv(&["bash", "-c", "docker cp /p k3d-ess-m8-x-server-0:/p"]),
            argv(&[
                "/usr/local/bin/kubectl",
                "--context",
                "k3d-ess-m8-x",
                "get",
                "pods",
            ]),
            argv(&["cargo", "run"]),
        ] {
            assert!(
                guard_argv(&refused, &dirs, "ess-m8-x", &trusted).is_err(),
                "admitted {refused:?}"
            );
        }
    }

    #[test]
    fn a_run_name_outside_the_disposable_prefix_is_refused() {
        assert_eq!(
            names("m8-4b91").unwrap(),
            ("ess-m8-4b91".to_owned(), "k3d-ess-m8-4b91".to_owned())
        );
        for refused in [
            "prod",
            "m9-x",
            "m8-",
            "M8-x",
            "m8-x_y",
            "-m8-x",
            &"m8-".repeat(20),
        ] {
            assert!(names(refused).is_err(), "admitted run name {refused:?}");
        }
    }

    #[test]
    fn a_cluster_that_already_exists_is_never_taken_over() {
        let listed = json!([{"name": "ess-m8-x"}, {"name": "other"}]);
        assert!(refuse_existing(&listed, "ess-m8-x").is_err());
        refuse_existing(&listed, "ess-m8-y").expect("an unused name is admitted");
        assert!(
            refuse_existing(&json!({"unexpected": true}), "ess-m8-y").is_err(),
            "a listing that cannot be read cannot prove absence"
        );
    }

    #[test]
    fn every_manifest_renders_without_an_unresolved_placeholder() {
        let image = Image {
            repository: "ess-test/billing".to_owned(),
            digest: Some(format!("sha256:{}", "a".repeat(64))),
        };
        for variant in [
            Variant::Matching,
            Variant::WrongTag,
            Variant::ExtraContainer,
        ] {
            let manifest = HarnessManifest::render(variant, &image, &sentinel().unwrap());
            let text = String::from_utf8(manifest.bytes).unwrap();
            assert!(!text.contains("{{"), "{variant:?} left a placeholder");
            for document in text.split("\n---\n") {
                serde_yaml::from_str::<Value>(document)
                    .unwrap_or_else(|e| panic!("{variant:?} is not YAML: {e}"));
            }
        }
        let extra = String::from_utf8(
            HarnessManifest::render(Variant::ExtraContainer, &image, &sentinel().unwrap()).bytes,
        )
        .unwrap();
        let deployment: Value =
            serde_yaml::from_str(extra.split("\n---\n").nth(2).unwrap()).unwrap();
        let names: Vec<&str> = deployment["spec"]["template"]["spec"]["containers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["billing", "sidecar"]);
        let wrong = String::from_utf8(
            HarnessManifest::render(Variant::WrongTag, &image, &sentinel().unwrap()).bytes,
        )
        .unwrap();
        assert!(wrong.contains("docker.io/ess-test/billing:wrong-tag"));
        assert!(!wrong.contains("@sha256:"));
    }

    #[test]
    fn a_secret_value_is_found_in_plain_and_base64_form_and_nowhere_else() {
        let needles = secret_needles("ess-acceptance-0123");
        assert_eq!(needles[1].1, "ZXNzLWFjY2VwdGFuY2UtMDEyMw==");
        assert!(leaks(br#"{"token":"[redacted]"}"#, &needles).is_empty());
        assert_eq!(leaks(b"x ess-acceptance-0123 y", &needles), ["plain"]);
        assert_eq!(
            leaks(b"data: ZXNzLWFjY2VwdGFuY2UtMDEyMw==", &needles),
            ["base64"]
        );
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
    }

    #[test]
    fn the_placement_intent_is_a_valid_infra_spec_naming_the_harness_workload() {
        let intent: Value = serde_yaml::from_str(INTENT).unwrap();
        assert_eq!(intent["format"], "infra-spec/1");
        let exists = &intent["expectations"][0]["expect"]["workload_exists"];
        assert_eq!(exists["namespace"], NAMESPACE);
        assert_eq!(exists["name"], WORKLOAD);
        assert!(intent["expectations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["id"] == "billing-resources" && e["remedy"].is_object()));
    }
}
