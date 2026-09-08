//! Actual bounded subprocesses and independent native images, never imported green receipts.
use super::executor::{Captured, CaseRunner};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::os::unix::{fs::MetadataExt, process::ExitStatusExt};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

#[derive(Serialize)]
struct Native {
    source: PathBuf,
    copy: PathBuf,
    sha256: String,
    size: u64,
    original_device: u64,
    original_inode: u64,
    original_mode: u32,
}
pub(super) struct VerifiedCases {
    cases: BTreeSet<String>,
    pub receipt: Value,
}
impl VerifiedCases {
    pub(super) fn ids(&self) -> &BTreeSet<String> {
        &self.cases
    }
}
struct Session<'a> {
    root: &'a Path,
    directory: &'a Path,
    profile: &'a Value,
    environment: BTreeMap<String, String>,
    sequence: usize,
    observations: Vec<Value>,
}
impl Session<'_> {
    fn authority(&mut self) -> Result<()> {
        let source = super::source_files(self.root)?;
        let invocation = super::invocation(self.root, &self.profile["compiled_build"])?;
        let source_hash = super::hash_json(&json!(source));
        self.observations
            .push(json!({"source_sha256":source_hash,"current_invocation":invocation}));
        super::write_json(
            &self.directory.join("authority-observations.json"),
            &json!(self.observations),
        )?;
        if json!(source) != self.profile["source"]
            || self.profile["source"] != self.profile["compiled_provider_source"]
        {
            super::write_json(&self.directory.join("changed-source.json"), &json!(source))?;
            bail!("source/fixture/manifest/lock changed since provider compilation");
        }
        let current = std::env::current_exe()?;
        if super::hash_bytes(&fs::read(current)?) != self.profile["provider_executable_sha256"] {
            bail!("compiled schema provider native identity changed");
        }
        super::validate_build(
            &self.profile["compiled_build"],
            |key| std::env::var(key).ok(),
            |path| Ok(fs::read(path)?),
        )
    }
    fn command(&mut self, program: &Path, args: &[String]) -> Result<Captured> {
        let directory = self.directory.join(format!("command-{:04}", self.sequence));
        self.sequence += 1;
        fs::create_dir(&directory)?;
        super::write_json(
            &directory.join("launch.json"),
            &json!({"program":program,"argv":args,"cwd":self.root,"environment":self.environment}),
        )?;
        let started = Instant::now();
        let child = Command::new(program)
            .args(args)
            .current_dir(self.root)
            .env_clear()
            .envs(&self.environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let pid = child.id();
        let output = child.wait_with_output()?;
        // Save direct status before later UTF-8 parsing, hashing, or artifact retention can fail.
        super::write_json(
            &directory.join("direct-result.json"),
            &json!({"pid":pid,"exit_code":output.status.code(),"signal":output.status.signal(),"elapsed_nanoseconds":started.elapsed().as_nanos()}),
        )?;
        fs::write(directory.join("stdout"), &output.stdout)?;
        fs::write(directory.join("stderr"), &output.stderr)?;
        super::write_json(
            &directory.join("stream-hashes.json"),
            &json!({"stdout":super::hash_bytes(&output.stdout),"stderr":super::hash_bytes(&output.stderr)}),
        )?;
        Ok(Captured {
            stdout: String::from_utf8(output.stdout).context("non-UTF8 command stdout")?,
            stderr: String::from_utf8(output.stderr).context("non-UTF8 command stderr")?,
            exit: output.status.code(),
        })
    }
    fn retain(&self, source: &Path) -> Result<Native> {
        let source = source.canonicalize()?;
        if !source.starts_with(self.root.join("target").canonicalize()?) {
            bail!(
                "native artifact lies outside this unit target: {}",
                source.display()
            );
        }
        let before = fs::metadata(&source)?;
        if !before.is_file() || before.mode() & 0o111 == 0 {
            bail!("artifact is not a native executable");
        }
        let sha256 = super::hash_bytes(&fs::read(&source)?);
        let copy = self.directory.join("native").join(format!("{sha256}.bin"));
        if !copy.exists() {
            fs::copy(&source, &copy)?;
        }
        let after = fs::metadata(&source)?;
        let retained = fs::metadata(&copy)?;
        if before.ino() != after.ino()
            || before.mtime_nsec() != after.mtime_nsec()
            || before.size() != after.size()
            || super::hash_bytes(&fs::read(&source)?) != sha256
            || super::hash_bytes(&fs::read(&copy)?) != sha256
            || (retained.dev() == before.dev() && retained.ino() == before.ino())
        {
            bail!("native artifact changed or was not independently retained");
        }
        Ok(Native {
            source,
            copy,
            sha256,
            size: before.size(),
            original_device: before.dev(),
            original_inode: before.ino(),
            original_mode: before.mode(),
        })
    }
}
struct ExactCase<'a, 'b> {
    session: &'a mut Session<'b>,
    native: &'a Native,
}
impl CaseRunner for ExactCase<'_, '_> {
    fn authority(&mut self) -> Result<()> {
        self.session.authority()?;
        if super::hash_bytes(&fs::read(&self.native.copy)?) != self.native.sha256 {
            bail!("retained case native image changed");
        }
        Ok(())
    }
    fn command(&mut self, args: &[String]) -> Result<Captured> {
        self.session.command(&self.native.copy, args)
    }
}
fn environment(profile: &Value) -> Result<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "CARGO_HOME",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
        "XDG_RUNTIME_DIR",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.into(), value);
        }
    }
    for (key, value) in [
        ("CARGO_BUILD_JOBS", "2"),
        ("CARGO_NET_OFFLINE", "true"),
        ("CARGO_INCREMENTAL", "0"),
        ("CARGO_PROFILE_DEV_DEBUG", "0"),
        ("CARGO_PROFILE_TEST_DEBUG", "0"),
        ("CARGO_PROFILE_DEV_INCREMENTAL", "false"),
        ("CARGO_PROFILE_TEST_INCREMENTAL", "false"),
        ("RUSTC_WRAPPER", ""),
        ("RUSTC_WORKSPACE_WRAPPER", ""),
        ("CARGO_BUILD_RUSTC_WRAPPER", ""),
        ("CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER", ""),
        ("RUSTFLAGS", "-C link-arg=-fuse-ld=lld"),
        ("TERM", "dumb"),
        ("NO_COLOR", "1"),
    ] {
        env.insert(key.into(), value.into());
    }
    for key in ["CARGO", "RUSTC"] {
        env.insert(
            key.into(),
            profile["compiled_build"]["tools"][key]["path"]
                .as_str()
                .context("compiled selected tool path")?
                .into(),
        );
    }
    if let Some(path) = profile["compiled_build"]["environment"]["RUSTDOC"].as_str() {
        env.insert("RUSTDOC".into(), path.into());
    }
    Ok(env)
}
pub(super) fn execute(
    root: &Path,
    directory: &Path,
    profile: &Value,
    cases: &Value,
    required: &BTreeSet<String>,
    metadata: &Value,
) -> Result<VerifiedCases> {
    fs::create_dir(directory)?;
    fs::create_dir(directory.join("native"))?;
    let mut session = Session {
        root,
        directory,
        profile,
        environment: environment(profile)?,
        sequence: 0,
        observations: Vec::new(),
    };
    session.authority()?;
    let provider = session.retain(&std::env::current_exe()?)?;
    super::write_json(&directory.join("provider-native.json"), &json!(provider))?;
    let mut targets = BTreeMap::<(String, String), Native>::new();
    let mut passed = BTreeSet::new();
    let mut receipts = BTreeMap::new();
    for id in required {
        let entry = cases
            .get(id)
            .with_context(|| format!("missing reviewed execution case {id}"))?;
        let identity: super::proposal::CaseIdentity =
            serde_json::from_value(entry["reviewed"]["identity"].clone())?;
        super::executor::source_contract(&entry["actual_source_candidate"])?;
        if !identity.features.is_empty()
            || identity.target_profile != "x86_64-unknown-linux-gnu/default"
            || identity.nested_runtime != "None claimed; direct Rust assertions only"
            || identity.tool_requirements
                != ["frozen Rust 1.98.1; locked offline owner-target build"]
        {
            bail!("unreviewed executable case contract {id}");
        }
        let key = (identity.package.clone(), identity.target_name.clone());
        if !targets.contains_key(&key) {
            let native = build_target(
                &mut session,
                &identity,
                &entry["reviewed"]["source"],
                metadata,
            )?;
            super::write_json(
                &directory.join(format!("artifact-{}.json", super::hash_json(&json!(key)))),
                &json!(native),
            )?;
            targets.insert(key.clone(), native);
        }
        let native = &targets[&key];
        let mut exact = ExactCase {
            session: &mut session,
            native,
        };
        super::executor::execute_case(&mut exact, &identity.full_name)
            .with_context(|| format!("exact consumer case {id}"))?;
        passed.insert(id.clone());
        receipts.insert(id.clone(),json!({"identity":identity,"native":native,"source_sha256":super::hash_json(&profile["source"]),"provider_profile_sha256":super::hash_json(profile),"executed":1,"passed":1,"failed":0,"ignored":0,"nested_runtime":"none claimed"}));
        super::write_json(&directory.join("executed-cases.json"), &json!(receipts))?;
    }
    session.authority()?;
    Ok(VerifiedCases {
        cases: passed,
        receipt: json!(receipts),
    })
}
fn build_target(
    session: &mut Session<'_>,
    identity: &super::proposal::CaseIdentity,
    source: &Value,
    metadata: &Value,
) -> Result<Native> {
    session.authority()?;
    let package = metadata["packages"]
        .as_array()
        .context("Cargo metadata packages")?
        .iter()
        .find(|p| p["name"] == identity.package)
        .context("owner package missing from current metadata")?;
    let package_id = package["id"].as_str().context("Cargo package identity")?;
    let cargo = PathBuf::from(
        session.profile["compiled_build"]["tools"]["CARGO"]["path"]
            .as_str()
            .context("compiled Cargo path")?,
    );
    let args = [
        "test",
        "--locked",
        "--offline",
        "--no-run",
        "--message-format=json-render-diagnostics",
        "-p",
        &identity.package,
        "--test",
        &identity.target_name,
    ]
    .map(str::to_owned);
    let output = session.command(&cargo, &args)?;
    if output.exit != Some(0) {
        bail!(
            "owner-target compilation failed: {} {} {:?}",
            identity.package,
            identity.target_name,
            output.exit
        );
    }
    session.authority()?;
    let source = session
        .root
        .join(source.as_str().context("reviewed source path")?);
    let artifact = super::executor::artifact(
        &output.stdout,
        package_id,
        &identity.target_name,
        source.to_str().context("source path UTF8")?,
    )?;
    if artifact["manifest_path"] != package["manifest_path"] {
        bail!("Cargo artifact manifest differs from owner metadata");
    }
    session.retain(Path::new(
        artifact["executable"]
            .as_str()
            .context("native Cargo artifact")?,
    ))
}
