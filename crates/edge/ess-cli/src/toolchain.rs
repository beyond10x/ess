//! `ess` manages its own toolchain (ess#147).
//!
//! The pin is the exact `requires: ess X.Y.Z` of the nearest `ess-inputs.yaml` above the working
//! directory, or `ESS_TOOLCHAIN=X.Y.Z` for one command. When it names another release, the running
//! `ess` execs that release from its cache with the same arguments and environment, adding
//! `ESS_TOOLCHAIN_DELEGATED=1` so the release it runs never delegates again. A minor-line pin or no
//! pin at all runs this release, exactly as before. `docs/design/specification-requires-release.md`
//! states the rules.

use std::{
    ffi::OsString,
    fmt, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::OnceLock,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::{input_discovery, requires};

/// Names one release for this command, overriding any pin.
const OVERRIDE: &str = "ESS_TOOLCHAIN";
/// Set on a delegated release: it runs itself whatever the pin says.
const DELEGATED: &str = "ESS_TOOLCHAIN_DELEGATED";
/// Replaces the cache directory.
const CACHE: &str = "ESS_TOOLCHAIN_DIR";
/// Replaces the release download base; a `file://` URL or a plain path reads a directory.
const BASE_URL: &str = "ESS_TOOLCHAIN_BASE_URL";
const DEFAULT_BASE_URL: &str = "https://github.com/beyond10x/ess/releases/download";
/// A released `ess` archive is tens of megabytes; this bounds a hostile or broken source.
const DOWNLOAD_LIMIT: u64 = 1 << 30;
/// The first release that reads `ess-inputs/2`, and so the first a `requires` pin can name.
const FIRST_PINNABLE: Release = Release(0, 34, 0);

/// `ess specify toolchain`.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    /// Download a released `ess`, verify it against the release's SHA256SUMS, and cache it.
    ///
    /// Only published releases install: building from a git revision or a tag is not supported.
    /// A release already cached is left as it is. Never prompts.
    Install {
        /// The exact release, `X.Y.Z`.
        #[arg(value_parser = parse_release)]
        version: Release,
        /// Also write `requires: ess X.Y.Z` into the nearest `ess-inputs.yaml`.
        #[arg(long)]
        pin: bool,
    },
    /// List the cached releases, oldest first.
    List,
    /// Print the release that would run here, and why: the override, the pin, or this `ess`.
    Which,
}

/// An exact release, `X.Y.Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Release(u64, u64, u64);

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl From<(u64, u64, u64)> for Release {
    fn from((major, minor, patch): (u64, u64, u64)) -> Self {
        Self(major, minor, patch)
    }
}

fn parse_release(text: &str) -> std::result::Result<Release, String> {
    requires::release(text)
        .map(Release::from)
        .ok_or_else(|| format!("`{text}` is not an exact release `X.Y.Z`"))
}

fn this() -> Release {
    requires::this().into()
}

/// Why a release runs here.
enum Reason {
    Delegated,
    Override,
    Pin { manifest: PathBuf, requires: String },
    Unpinned { manifest: PathBuf, detail: String },
    NoManifest { from: PathBuf },
    NoWorkingDirectory,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delegated => write!(f, "self: {DELEGATED} is set, so this ess was delegated to"),
            Self::Override => write!(f, "override: {OVERRIDE} names it"),
            Self::Pin { manifest, requires } => {
                write!(f, "pin: `requires: {requires}` in {}", manifest.display())
            }
            Self::Unpinned { manifest, detail } => {
                write!(f, "self: {} {detail}", manifest.display())
            }
            Self::NoManifest { from } => write!(
                f,
                "self: no ess-inputs.yaml in {} or any directory above it",
                from.display()
            ),
            Self::NoWorkingDirectory => write!(
                f,
                "self: the working directory cannot be read, so no ess-inputs.yaml is found"
            ),
        }
    }
}

struct Choice {
    release: Release,
    reason: Reason,
}

/// The release that runs in the working directory under this environment.
fn choose() -> Result<Choice> {
    let choice = choose_unchecked()?;
    // A release before `ess-inputs/2` cannot read the manifest a pin lives in, and an override
    // is the same pin spelled another way: neither is downloaded nor run.
    if choice.release != this() && choice.release < FIRST_PINNABLE {
        bail!(
            "refused ess {} ({}): it predates ess {FIRST_PINNABLE}, the first release that reads \
             ess-inputs/2, so it cannot run under a pin; name {FIRST_PINNABLE} or later",
            choice.release,
            choice.reason
        );
    }
    Ok(choice)
}

/// Whether this process was delegated to, read once and then removed from its environment so the
/// processes it starts — a target under conformance, a helper — do not inherit the marker.
fn delegated() -> bool {
    static DELEGATED_HERE: OnceLock<bool> = OnceLock::new();
    *DELEGATED_HERE.get_or_init(|| {
        let set = std::env::var_os(DELEGATED).is_some_and(|value| !value.is_empty());
        // Read before any thread starts: `delegate` is the first thing `main` does.
        std::env::remove_var(DELEGATED);
        set
    })
}

fn choose_unchecked() -> Result<Choice> {
    if delegated() {
        return Ok(Choice {
            release: this(),
            reason: Reason::Delegated,
        });
    }
    // Every toolchain variable reads empty as unset: `ESS_TOOLCHAIN= ess …` is how a shell
    // clears it for one command.
    if let Some(value) = std::env::var_os(OVERRIDE).filter(|value| !value.is_empty()) {
        let text = value.to_string_lossy();
        let release = parse_release(&text).map_err(|error| {
            anyhow::anyhow!("{OVERRIDE}={text}: {error}; unset it or name a release")
        })?;
        return Ok(Choice {
            release,
            reason: Reason::Override,
        });
    }
    // A removed or unreadable working directory has no manifest above it; `ess` ran there before
    // ess#147 and still does.
    let Ok(from) = std::env::current_dir() else {
        return Ok(Choice {
            release: this(),
            reason: Reason::NoWorkingDirectory,
        });
    };
    let Some(pin) = input_discovery::nearest_pin(&from) else {
        return Ok(Choice {
            release: this(),
            reason: Reason::NoManifest { from },
        });
    };
    let manifest = pin.manifest;
    let unpinned = |detail: &str| Choice {
        release: this(),
        reason: Reason::Unpinned {
            manifest: manifest.clone(),
            detail: detail.to_owned(),
        },
    };
    Ok(match pin.requires {
        Err(_) if pin.symlink => {
            unpinned("is a symlink, which every command refuses; no pin is read through it")
        }
        Err(_) => unpinned("does not parse; the command that reads it says why"),
        Ok(None) => unpinned("has no `requires`"),
        Ok(Some(requires)) => match requires::exact(&requires) {
            Some(release) => Choice {
                release: release.into(),
                reason: Reason::Pin {
                    manifest: manifest.clone(),
                    requires,
                },
            },
            None => unpinned(&format!(
                "has `requires: {requires}`, which names no exact release, so this ess decides it"
            )),
        },
    })
}

/// Before clap parses anything: run the chosen release instead of this one, if it is another.
///
/// `None` lets this `ess` run the command. `Some` is the exit status when the chosen release
/// could not be run; on success `exec` does not return.
pub(crate) fn delegate() -> Option<ExitCode> {
    // Before anything else, so the marker leaves this process's environment in every case.
    let _ = delegated();
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    if manages_toolchain(&arguments) {
        return None;
    }
    let outcome = (|| {
        let choice = choose()?;
        if choice.release == this() {
            return Ok(None);
        }
        // The dispatcher names itself before anything can fail, so `--version` always does. Its
        // stdout stays lines ending in a release, which the release smoke check reads with
        // `awk '{print $NF}'`; why it delegates goes to stderr.
        if asks_version(&arguments) {
            println!("ess {}", this());
            std::io::stdout().flush().ok();
            eprintln!(
                "note: ess {} is the dispatcher; delegating to ess {} ({})",
                this(),
                choice.release,
                choice.reason
            );
        }
        let binary = ensure(choice.release, &choice.reason)?;
        run(&binary, &arguments).map(Some)
    })();
    match outcome {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            Some(ExitCode::from(1))
        }
    }
}

/// `ess [--strict-requires] specify toolchain …` and its flat `ess toolchain …` stay here: the
/// dispatcher manages the cache, and a delegated older release may have no such command.
fn manages_toolchain(arguments: &[OsString]) -> bool {
    let mut words = arguments
        .iter()
        .filter(|argument| !argument.to_string_lossy().starts_with('-'));
    match words.next().map(|word| word.to_string_lossy()) {
        Some(word) if word == "toolchain" => true,
        Some(word) if word == "specify" => words.next().is_some_and(|word| word == "toolchain"),
        _ => false,
    }
}

fn asks_version(arguments: &[OsString]) -> bool {
    arguments
        .iter()
        .all(|argument| argument.to_string_lossy().starts_with('-'))
        && arguments
            .iter()
            .any(|argument| argument == "--version" || argument == "-V")
}

#[cfg(unix)]
fn run(binary: &Path, arguments: &[OsString]) -> Result<ExitCode> {
    use std::os::unix::process::CommandExt;
    let error = std::process::Command::new(binary)
        .args(arguments)
        .env(DELEGATED, "1")
        .exec();
    Err(error).with_context(|| format!("running {}", binary.display()))
}

#[cfg(not(unix))]
fn run(binary: &Path, arguments: &[OsString]) -> Result<ExitCode> {
    let status = std::process::Command::new(binary)
        .args(arguments)
        .env(DELEGATED, "1")
        .status()
        .with_context(|| format!("running {}", binary.display()))?;
    Ok(ExitCode::from(
        u8::try_from(status.code().unwrap_or(1)).unwrap_or(1),
    ))
}

/// The cached `ess` for `release`, installing it first when it is not cached yet.
fn ensure(release: Release, reason: &Reason) -> Result<PathBuf> {
    let root = cache_root()?;
    let binary = root.join(release.to_string()).join("ess");
    if usable(&binary) {
        return Ok(binary);
    }
    eprintln!("note: installing ess {release} ({reason})");
    install(&root, release)
}

/// A cache entry counts only when its `ess` is a nonempty executable file; anything else — a
/// directory emptied by hand, a crash's leftovers — is not cached and is replaced by an install.
fn usable(binary: &Path) -> bool {
    let Ok(metadata) = fs::metadata(binary) else {
        return false;
    };
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = true;
    metadata.is_file() && metadata.len() > 0 && executable
}

/// Every candidate must be absolute: a relative one would give each directory of one project
/// its own cache, so it is ignored like an unset one.
fn cache_root() -> Result<PathBuf> {
    if let Some(directory) = std::env::var_os(CACHE)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(directory);
    }
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|home| home.is_absolute())
                .map(|home| home.join(".cache"))
        })
        .with_context(|| format!("neither XDG_CACHE_HOME nor HOME is set; set {CACHE}"))?;
    Ok(cache.join("ess").join("toolchains"))
}

/// Cached releases, oldest first.
fn cached(root: &Path) -> Vec<(Release, PathBuf)> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut releases: Vec<_> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let release = parse_release(entry.file_name().to_str()?).ok()?;
            let binary = entry.path().join("ess");
            usable(&binary).then_some((release, binary))
        })
        .collect();
    releases.sort();
    releases
}

/// The release target this `ess` was built for, as the release workflow names its archives.
fn target() -> Option<&'static str> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Some("x86_64-unknown-linux-gnu"),
        ("aarch64", "linux") => Some("aarch64-unknown-linux-gnu"),
        ("x86_64", "macos") => Some("x86_64-apple-darwin"),
        ("aarch64", "macos") => Some("aarch64-apple-darwin"),
        _ => None,
    }
}

/// Download, verify and cache `release`; a refusal names the newest release already cached.
fn install(root: &Path, release: Release) -> Result<PathBuf> {
    fetch_and_cache(root, release).map_err(|error| {
        let fallback = cached(root).pop().map_or_else(
            || "no release is cached".to_owned(),
            |(newest, _)| {
                format!(
                    "the newest cached release is {newest}: `{OVERRIDE}={newest}` runs it, and \
                     `ess specify toolchain install --pin {newest}` pins it"
                )
            },
        );
        anyhow::anyhow!("refused to install ess {release}: {error:#}; {fallback}")
    })
}

fn fetch_and_cache(root: &Path, release: Release) -> Result<PathBuf> {
    let target = target().with_context(|| {
        format!(
            "ess publishes no archive for {}-{}",
            std::env::consts::ARCH,
            std::env::consts::OS
        )
    })?;
    let package = format!("ess-{release}-{target}");
    let archive_name = format!("{package}.tar.gz");
    let sums = fetch(release, "SHA256SUMS")?;
    let sums = String::from_utf8(sums).context("SHA256SUMS is not UTF-8")?;
    let published = stated_digest(&sums, &archive_name)
        .with_context(|| format!("SHA256SUMS lists no {archive_name}: no asset for {target}"))?;
    let archive = fetch(release, &archive_name)?;
    let actual = hex(&Sha256::digest(&archive));
    if actual != published {
        bail!("{archive_name} has SHA-256 {actual}, and SHA256SUMS states {published}");
    }
    let binary = extract(&archive, &format!("{package}/ess"))?;

    fs::create_dir_all(root).with_context(|| format!("creating {}", root.display()))?;
    let staging = root.join(format!(".{release}.partial-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging).with_context(|| format!("clearing {}", staging.display()))?;
    }
    fs::create_dir(&staging).with_context(|| format!("creating {}", staging.display()))?;
    let staged = staging.join("ess");
    fs::write(&staged, &binary).with_context(|| format!("writing {}", staged.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("making {} executable", staged.display()))?;
    }
    let destination = root.join(release.to_string());
    if destination.exists() && !usable(&destination.join("ess")) {
        // A stale entry is moved aside rather than deleted in place: a rename is atomic, so a
        // usable entry another process renamed in meanwhile is recognised and put back.
        let stale = root.join(format!(".{release}.stale-{}", std::process::id()));
        if fs::rename(&destination, &stale).is_ok() {
            if usable(&stale.join("ess")) {
                fs::rename(&stale, &destination).ok();
            }
            fs::remove_dir_all(&stale).ok();
        }
    }
    if let Err(error) = fs::rename(&staging, &destination) {
        fs::remove_dir_all(&staging).ok();
        // Another process installing the same release at the same time is not a failure.
        if !usable(&destination.join("ess")) {
            return Err(error).with_context(|| format!("moving into {}", destination.display()));
        }
    }
    Ok(destination.join("ess"))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        use fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// The digest `sha256sum` output states for `name`.
fn stated_digest(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (digest, file) = line.split_once(char::is_whitespace)?;
        let file = file.trim_start();
        let file = file.strip_prefix('*').unwrap_or(file);
        (file == name && digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| digest.to_ascii_lowercase())
    })
}

/// The one regular file at `member` in a gzipped tar archive.
fn extract(archive: &[u8], member: &str) -> Result<Vec<u8>> {
    let mut entries = tar::Archive::new(flate2::read::GzDecoder::new(archive));
    for entry in entries.entries().context("reading the archive")? {
        let entry = entry.context("reading the archive")?;
        if entry.header().entry_type().is_file()
            && entry.path().context("reading the archive")?.as_ref() == Path::new(member)
        {
            let mut bytes = Vec::new();
            entry
                .take(DOWNLOAD_LIMIT)
                .read_to_end(&mut bytes)
                .context("reading the archive")?;
            return Ok(bytes);
        }
    }
    bail!("the archive holds no {member}")
}

/// One release asset, from an `https://` base or a local directory.
fn fetch(release: Release, name: &str) -> Result<Vec<u8>> {
    let base = std::env::var(BASE_URL)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
    let base = base.trim_end_matches('/');
    let local = base.strip_prefix("file://");
    if local.is_none() && !base.starts_with("https://") && base.contains("://") {
        bail!(
            "{BASE_URL}={base} is neither https:// nor a local directory: a release is fetched \
             over https or read from a file:// URL or path, never over plain http"
        );
    }
    if base.starts_with("https://") {
        let url = format!("{base}/{release}/{name}");
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(600)))
            .build()
            .into();
        let mut response = agent
            .get(&url)
            .call()
            .with_context(|| format!("downloading {url}"))?;
        return response
            .body_mut()
            .with_config()
            .limit(DOWNLOAD_LIMIT)
            .read_to_vec()
            .with_context(|| format!("downloading {url}"));
    }
    let directory = PathBuf::from(local.unwrap_or(base));
    let path = directory.join(release.to_string()).join(name);
    fs::read(&path).with_context(|| format!("reading {}", path.display()))
}

/// `ess specify toolchain …`.
pub(crate) fn run_command(command: &Command) -> Result<ExitCode> {
    let root = cache_root()?;
    match *command {
        Command::Install { version, pin } => {
            if pin && version < FIRST_PINNABLE {
                bail!(
                    "refused to pin ess {version}: `requires` exists only in ess-inputs/2, which \
                     ess {FIRST_PINNABLE} introduced, so ess {version} would refuse the pinned \
                     manifest; pin {FIRST_PINNABLE} or later, or install without --pin"
                );
            }
            let cached_binary = root.join(version.to_string()).join("ess");
            if usable(&cached_binary) {
                println!(
                    "ess {version} is already cached at {}",
                    cached_binary.display()
                );
            } else {
                let binary = install(&root, version)?;
                println!("installed ess {version} at {}", binary.display());
            }
            if pin {
                let from = std::env::current_dir().context("reading the working directory")?;
                let nearest = input_discovery::nearest_pin(&from).with_context(|| {
                    format!(
                        "ess {version} is cached but not pinned: no ess-inputs.yaml in {} or any \
                         directory above it",
                        from.display()
                    )
                })?;
                let manifest = nearest.manifest;
                if nearest.symlink {
                    bail!(
                        "ess {version} is cached but not pinned: the nearest manifest {} is a \
                         symlink, which every command refuses, and --pin does not write through \
                         it or past it; replace it with the real file, then pin again",
                        manifest.display()
                    );
                }
                input_discovery::write_pin(&manifest, &version.to_string())?;
                println!("pinned {}: requires: ess {version}", manifest.display());
            }
        }
        Command::List => {
            let releases = cached(&root);
            if releases.is_empty() {
                eprintln!("no release is cached in {}", root.display());
            }
            for (release, binary) in releases {
                println!("{release}\t{}", binary.display());
            }
        }
        Command::Which => {
            let choice = choose()?;
            println!("ess {}", choice.release);
            println!("reason: {}", choice.reason);
            if choice.release == this() {
                let binary = std::env::current_exe().context("locating this ess")?;
                println!("binary: {} (this ess)", binary.display());
            } else {
                let binary = root.join(choice.release.to_string()).join("ess");
                let state = if usable(&binary) {
                    "cached"
                } else {
                    "not cached; the next command here installs it"
                };
                println!("binary: {} ({state})", binary.display());
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(arguments: &[&str]) -> Vec<OsString> {
        arguments.iter().map(OsString::from).collect()
    }

    #[test]
    fn only_toolchain_commands_stay_in_the_dispatcher() {
        for kept in [
            &["specify", "toolchain", "which"][..],
            &["--strict-requires", "specify", "toolchain", "list"],
            &["toolchain", "install", "1.2.3"],
        ] {
            assert!(manages_toolchain(&words(kept)), "{kept:?}");
        }
        for delegated in [
            &["specify", "validate", "--path", "toolchain"][..],
            &["validate", "toolchain"],
            &["--version"],
            &[],
        ] {
            assert!(!manages_toolchain(&words(delegated)), "{delegated:?}");
        }
    }

    #[test]
    fn version_is_asked_only_without_a_command() {
        assert!(asks_version(&words(&["--version"])));
        assert!(asks_version(&words(&["--strict-requires", "-V"])));
        assert!(!asks_version(&words(&["validate", "--version"])));
        assert!(!asks_version(&words(&[])));
    }

    #[test]
    fn sha256sums_is_read_as_sha256sum_writes_it() {
        let digest = "a".repeat(64);
        let sums = format!(
            "{}  other.tar.gz\n{digest}  ess-1.2.3-x.tar.gz\n{} *binary.tar.gz\n",
            "b".repeat(64),
            "C".repeat(64)
        );
        assert_eq!(stated_digest(&sums, "ess-1.2.3-x.tar.gz"), Some(digest));
        assert_eq!(stated_digest(&sums, "binary.tar.gz"), Some("c".repeat(64)));
        assert_eq!(stated_digest(&sums, "missing.tar.gz"), None);
        assert_eq!(stated_digest("xyz  short.tar.gz\n", "short.tar.gz"), None);
    }

    /// The only unit case touching this variable, and `delegated` is otherwise first reached from
    /// `main`, so the once-only read is this case's.
    #[test]
    fn the_delegated_marker_is_read_once_and_leaves_the_environment() {
        std::env::set_var(DELEGATED, "1");
        assert!(delegated());
        assert_eq!(std::env::var_os(DELEGATED), None);
        assert!(delegated(), "the answer outlives the variable");
    }

    #[test]
    fn releases_order_numerically() {
        let mut releases = [Release(0, 10, 0), Release(0, 9, 12), Release(1, 0, 0)];
        releases.sort();
        assert_eq!(
            releases.map(|release| release.to_string()),
            ["0.9.12", "0.10.0", "1.0.0"]
        );
    }
}
