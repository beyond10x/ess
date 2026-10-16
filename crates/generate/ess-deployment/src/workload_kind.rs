//! The `ess-runtime/2` keys: the scheduled workload and the pod settings every workload kind takes.
//!
//! `docs/design/runtime-workload-kinds.md` is the design. The types are read by
//! [`crate::runtime`], which decides which format may carry them, and rendered by
//! [`crate::environment`]'s Helm projection.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::diagnostic::{Diagnostic, DiagnosticCode, Stage};
use crate::identity::Identifier;
use crate::runtime::{ContainerRole, ProvidedEndpoint, SecretSlot, Workload};

/// The image a process runs.
///
/// Written as one string: an identifier names an OCI image output of the compiled build, which an
/// environment fills in with a repository and digest; anything else is an image reference, which
/// must be pinned by digest (`ess-runtime/2` only) and is rendered verbatim.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcessImage {
    /// An OCI image output of the compiled build.
    Output(Identifier),
    /// An image reference, `<repository>@sha256:<64 lowercase hex>` once validated.
    Reference(String),
}

impl ProcessImage {
    /// The build output this image names, when it names one.
    pub fn output(&self) -> Option<&Identifier> {
        match self {
            Self::Output(output) => Some(output),
            Self::Reference(_) => None,
        }
    }

    /// The written spelling.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Output(output) => output.as_str(),
            Self::Reference(reference) => reference,
        }
    }
}

impl fmt::Display for ProcessImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<Identifier> for ProcessImage {
    fn from(output: Identifier) -> Self {
        Self::Output(output)
    }
}

impl serde::Serialize for ProcessImage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for ProcessImage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(Identifier::new(&value).map_or(Self::Reference(value), Self::Output))
    }
}

/// CPU and memory quantities, in Kubernetes quantity notation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceQuantities {
    /// CPU quantity, such as `100m` or `1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// Memory quantity, such as `128Mi`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

/// A container's resource requests and limits.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    /// What the scheduler reserves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requests: Option<ResourceQuantities>,
    /// What the container may not exceed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<ResourceQuantities>,
}

/// The pod's security context.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PodSecurity {
    /// User every container process runs as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_user: Option<u32>,
    /// Primary group every container process runs as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_group: Option<u32>,
    /// Group that owns mounted volumes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_group: Option<u32>,
    /// Refuse to start a container whose image runs as root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_as_non_root: Option<bool>,
}

/// One key of a secret projected to a file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretItem {
    /// Key inside the environment-owned secret object.
    pub key: String,
    /// Relative path below the mount.
    pub path: String,
}

/// A secret slot mounted read-only as a volume.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretMount {
    /// Absolute path inside the container.
    pub path: String,
    /// File mode of the projected files, four octal digits from `0000` to `0777`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Keys projected to files; absent projects every key under its own name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<SecretItem>,
}

impl SecretMount {
    /// The mode as a number, when it is four octal digits within `0000`–`0777`.
    pub fn mode_bits(&self) -> Option<Option<u32>> {
        self.mode
            .as_deref()
            .map_or(Some(None), |mode| octal_mode(mode).map(Some))
    }
}

/// What a scheduled run does when another is still running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Overlap {
    /// Skip the new run.
    Forbid,
    /// Stop the running one and start the new one.
    Replace,
    /// Run both.
    Allow,
}

impl Overlap {
    /// The Kubernetes `concurrencyPolicy` spelling.
    pub const fn as_kubernetes(self) -> &'static str {
        match self {
            Self::Forbid => "Forbid",
            Self::Replace => "Replace",
            Self::Allow => "Allow",
        }
    }
}

/// What a failed run's container does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobRestart {
    /// A failed container is not restarted; the retry limit starts a new pod.
    Never,
    /// A failed container is restarted in its pod.
    OnFailure,
}

impl JobRestart {
    /// The Kubernetes `restartPolicy` spelling.
    pub const fn as_kubernetes(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::OnFailure => "OnFailure",
        }
    }
}

/// A workload that runs on a schedule instead of continuously.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    /// Five-field cron expression, or one of the `@hourly`-style macros.
    pub cron: String,
    /// IANA time zone the expression is read in; absent reads it in the cluster's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// What a run does while another is still running; absent forbids the overlap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlap: Option<Overlap>,
    /// Finished successful runs kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successful_runs: Option<u32>,
    /// Finished failed runs kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_runs: Option<u32>,
    /// No run starts while suspended.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub suspended: bool,
    /// Seconds after its scheduled time within which a missed run may still start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_deadline_seconds: Option<i64>,
    /// Seconds a run may take before it is stopped. Required: a run without one is unbounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_deadline_seconds: Option<i64>,
    /// Retries of a failed run before the run is failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_limit: Option<i64>,
    /// What a failed container does; absent never restarts it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<JobRestart>,
}

/// Whether spreading a workload's pods is a scheduling requirement or a preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spread {
    /// The scheduler spreads the pods where it can, and places them together where it cannot.
    Preferred,
    /// A pod that cannot be placed apart from the others stays pending.
    Required,
}

/// Where a workload's pods run.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Affinity {
    /// A node label key, such as `kubernetes.io/hostname`: no two pods of the workload share a
    /// value of it (pod anti-affinity against the workload's own pods).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread_across: Option<String>,
    /// Whether `spread_across` is required or preferred; absent prefers it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<Spread>,
    /// Labels the node of every pod must carry, each with exactly this value (node affinity).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub node_labels: BTreeMap<String, String>,
}

/// The pod volume a mounted secret slot occupies.
pub(crate) fn secret_volume_name(slot: &Identifier) -> String {
    format!("secret-{slot}")
}

/// The largest value of a Kubernetes `int32` field.
const INT32_MAX: i64 = 2_147_483_647;

/// Kubernetes caps the total size of one object's annotations at 256 KiB.
const ANNOTATIONS_LIMIT: usize = 256 * 1024;

/// A DNS-1123 label: the name of a container, a pod volume or a port.
pub(crate) fn is_dns_label(name: &str) -> bool {
    let alphanumeric = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    (1..=63).contains(&name.len())
        && name.bytes().all(|byte| alphanumeric(byte) || byte == b'-')
        && name.bytes().next().is_some_and(alphanumeric)
        && name.bytes().last().is_some_and(alphanumeric)
}

/// A DNS-1123 subdomain: dot-separated labels, at most 253 characters.
fn is_dns_subdomain(name: &str) -> bool {
    name.len() <= 253 && name.split('.').all(is_dns_label)
}

/// The name half of a qualified name, and a non-empty label value.
fn is_qualified_segment(text: &str) -> bool {
    let alphanumeric = |byte: u8| byte.is_ascii_alphanumeric();
    (1..=63).contains(&text.len())
        && text
            .bytes()
            .all(|byte| alphanumeric(byte) || matches!(byte, b'-' | b'_' | b'.'))
        && text.bytes().next().is_some_and(alphanumeric)
        && text.bytes().last().is_some_and(alphanumeric)
}

/// A label or annotation key: an optional DNS subdomain prefix and `/`, then a name.
fn is_qualified_name(key: &str) -> bool {
    match key.split_once('/') {
        Some((prefix, name)) => is_dns_subdomain(prefix) && is_qualified_segment(name),
        None => is_qualified_segment(key),
    }
}

/// A label value: empty, or a qualified-name segment.
fn is_label_value(value: &str) -> bool {
    value.is_empty() || is_qualified_segment(value)
}

/// A key of a Secret or `ConfigMap` (`IsConfigMapKey`).
fn is_secret_key(key: &str) -> bool {
    (1..=253).contains(&key.len())
        && key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        && !key.starts_with("..")
        && key != "."
}

/// Whether Helm would read `text` as a template action rather than as text.
pub(crate) fn has_template_action(text: &str) -> bool {
    text.contains("{{")
}

/// Four octal digits from `0000` to `0777`.
fn octal_mode(mode: &str) -> Option<u32> {
    let digits = mode.strip_prefix('0')?;
    if digits.len() != 3 || !digits.bytes().all(|digit| (b'0'..=b'7').contains(&digit)) {
        return None;
    }
    u32::from_str_radix(digits, 8).ok()
}

/// Why `reference` is not an image reference pinned by digest, or `None` when it is one.
fn unpinned(reference: &str) -> Option<&'static str> {
    let Some((repository, digest)) = reference.rsplit_once('@') else {
        return Some("it carries no digest");
    };
    if repository.is_empty()
        || repository.contains('@')
        || repository
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Some("its repository is empty or carries whitespace");
    }
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Some("its digest is not sha256");
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Some("its digest is not 64 lowercase hexadecimal characters");
    }
    None
}

/// A Kubernetes quantity: a decimal number and an optional SI or binary suffix.
fn is_quantity(text: &str) -> bool {
    const SUFFIXES: [&str; 16] = [
        "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "n", "u", "m", "k", "M", "G", "T", "P", "E", "",
    ];
    SUFFIXES.iter().any(|suffix| {
        text.strip_suffix(suffix).is_some_and(|number| {
            let mut parts = number.splitn(2, '.');
            let whole = parts.next().unwrap_or_default();
            let fraction = parts.next();
            let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
            digits(whole)
                && fraction.is_none_or(digits)
                && !(whole.is_empty() && fraction.is_none_or(str::is_empty))
        })
    })
}

/// An exact quantity: `numerator / 10^scale`, or `None` when it is not one or does not fit.
fn quantity_value(text: &str) -> Option<(u128, u32)> {
    const SUFFIXES: [(&str, i32, u32); 16] = [
        ("Ki", 0, 10),
        ("Mi", 0, 20),
        ("Gi", 0, 30),
        ("Ti", 0, 40),
        ("Pi", 0, 50),
        ("Ei", 0, 60),
        ("n", -9, 0),
        ("u", -6, 0),
        ("m", -3, 0),
        ("k", 3, 0),
        ("M", 6, 0),
        ("G", 9, 0),
        ("T", 12, 0),
        ("P", 15, 0),
        ("E", 18, 0),
        ("", 0, 0),
    ];
    if !is_quantity(text) {
        return None;
    }
    SUFFIXES.iter().find_map(|(suffix, decimal, binary)| {
        let number = text.strip_suffix(suffix)?;
        let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
        if !whole
            .bytes()
            .chain(fraction.bytes())
            .all(|b| b.is_ascii_digit())
            || (whole.is_empty() && fraction.is_empty())
        {
            return None;
        }
        let digits: String = whole.chars().chain(fraction.chars()).collect();
        let significant = digits.trim_start_matches('0');
        let mantissa: u128 = if significant.is_empty() {
            0
        } else {
            significant.parse().ok()?
        };
        let scale = u32::try_from(fraction.len()).ok()?;
        let mut numerator = mantissa.checked_mul(1u128.checked_shl(*binary)?)?;
        let scale = if *decimal >= 0 {
            numerator = numerator.checked_mul(10u128.checked_pow(decimal.unsigned_abs())?)?;
            scale
        } else {
            scale.checked_add(decimal.unsigned_abs())?
        };
        Some((numerator, scale))
    })
}

/// Whether quantity `left` is greater than quantity `right`, when both are exact.
fn exceeds(left: &str, right: &str) -> Option<bool> {
    let (left, left_scale) = quantity_value(left)?;
    let (right, right_scale) = quantity_value(right)?;
    let left = left.checked_mul(10u128.checked_pow(right_scale)?)?;
    let right = right.checked_mul(10u128.checked_pow(left_scale)?)?;
    Some(left > right)
}

/// A relative path that stays below its mount, and does not start with `..`, which the
/// kubelet's atomic writer reserves.
fn is_contained(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with("..")
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

/// An absolute path that does not traverse a parent.
fn is_absolute(path: &str) -> bool {
    path.starts_with('/') && !path.split('/').any(|segment| segment == "..")
}

const CRON_MACROS: [&str; 7] = [
    "@yearly",
    "@annually",
    "@monthly",
    "@weekly",
    "@daily",
    "@midnight",
    "@hourly",
];

const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];
const DAYS: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

/// Why `expression` is not a cron expression Kubernetes reads, or `None` when it is one.
fn cron_error(expression: &str) -> Option<String> {
    if expression.contains("TZ=") {
        return Some("it carries its time zone; declare that as `time_zone`".to_owned());
    }
    if expression.starts_with('@') {
        return (!CRON_MACROS.contains(&expression))
            .then(|| format!("it is not one of the macros {}", CRON_MACROS.join(", ")));
    }
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return Some(format!(
            "it has {} field(s), and a schedule has five: minute, hour, day of month, month and \
             day of week",
            fields.len()
        ));
    }
    let shapes: [(&str, u32, u32, &[&str], u32); 5] = [
        ("minute", 0, 59, &[], 0),
        ("hour", 0, 23, &[], 0),
        ("day of month", 1, 31, &[], 0),
        ("month", 1, 12, &MONTHS, 1),
        ("day of week", 0, 6, &DAYS, 0),
    ];
    for (field, (name, low, high, names, first)) in fields.iter().zip(shapes) {
        let wildcard_question = matches!(name, "day of month" | "day of week");
        let valid = field
            .split(',')
            .all(|part| cron_part(part, low, high, names, first, wildcard_question));
        if !valid {
            return Some(format!(
                "its {name} field {field:?} is not a value, range, list or step within \
                 {low}-{high}"
            ));
        }
    }
    None
}

fn cron_part(
    part: &str,
    low: u32,
    high: u32,
    names: &[&str],
    first: u32,
    wildcard_question: bool,
) -> bool {
    let (base, step) = match part.split_once('/') {
        Some((base, step)) => (base, Some(step)),
        None => (part, None),
    };
    if let Some(step) = step {
        if !step.parse::<u32>().is_ok_and(|step| step > 0) {
            return false;
        }
    }
    let value = |text: &str| -> Option<u32> {
        text.parse::<u32>().ok().or_else(|| {
            names
                .iter()
                .position(|name| name.eq_ignore_ascii_case(text))
                .and_then(|index| u32::try_from(index).ok())
                .map(|index| index + first)
        })
    };
    if base == "*" || (wildcard_question && base == "?") {
        return true;
    }
    let (start, end) = match base.split_once('-') {
        Some((start, end)) => (value(start), value(end)),
        None => (value(base), value(base)),
    };
    matches!((start, end), (Some(start), Some(end)) if low <= start && start <= end && end <= high)
}

/// Whether `name` is a canonical IANA time zone in the bundled database.
fn is_time_zone(name: &str) -> bool {
    jiff_tzdb::get(name).is_some_and(|(canonical, _)| canonical == name)
}

fn refuse(diagnostics: &mut Vec<Diagnostic>, subject: &Identifier, detail: String) {
    diagnostics.push(Diagnostic::new(
        Stage::Runtime,
        DiagnosticCode::InvalidValue,
        Some(subject.clone()),
        detail,
    ));
}

/// The `ess-runtime/2` keys a container declares.
pub(crate) fn container_v2_keys(container: &ContainerRole) -> Vec<String> {
    let mut keys = Vec::new();
    if container.resources.is_some() {
        keys.push("resources".to_owned());
    }
    if container.read_only_root_filesystem.is_some() {
        keys.push("read_only_root_filesystem".to_owned());
    }
    if container.readiness_tcp_port.is_some() {
        keys.push("readiness_tcp_port".to_owned());
    }
    if container.liveness_tcp_port.is_some() {
        keys.push("liveness_tcp_port".to_owned());
    }
    for slot in &container.secrets {
        if slot.mount.is_some() {
            keys.push(format!("secrets.{}.mount", slot.name));
        }
        if slot.environment.is_none() || slot.key.is_none() {
            keys.push(format!("secrets.{} without environment and key", slot.name));
        }
    }
    keys
}

/// The `ess-runtime/2` keys a workload declares.
pub(crate) fn workload_v2_keys(workload: &Workload) -> Vec<&'static str> {
    let mut keys = Vec::new();
    if workload.schedule.is_some() {
        keys.push("schedule");
    }
    if workload.security.is_some() {
        keys.push("security");
    }
    if workload.service_account_token.is_some() {
        keys.push("service_account_token");
    }
    if workload.annotations.is_some() {
        keys.push("annotations");
    }
    if workload.affinity.is_some() {
        keys.push("affinity");
    }
    if workload.replicas.is_none() {
        keys.push("a workload without replicas");
    }
    keys
}

/// Validates the pod settings of one container role.
pub(crate) fn validate_container_settings(
    container: &ContainerRole,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = &container.name;
    if let Some(resources) = &container.resources {
        if resources.requests.is_none() && resources.limits.is_none() {
            refuse(
                diagnostics,
                name,
                format!("container {name} declares resources with neither requests nor limits"),
            );
        }
        for (list, quantities) in [
            ("requests", &resources.requests),
            ("limits", &resources.limits),
        ] {
            let Some(quantities) = quantities else {
                continue;
            };
            if quantities.cpu.is_none() && quantities.memory.is_none() {
                refuse(
                    diagnostics,
                    name,
                    format!(
                        "container {name} declares resource {list} with neither cpu nor memory"
                    ),
                );
            }
            for (resource, quantity) in [("cpu", &quantities.cpu), ("memory", &quantities.memory)] {
                if let Some(quantity) = quantity {
                    if !is_quantity(quantity) {
                        refuse(
                            diagnostics,
                            name,
                            format!(
                                "container {name} resource {list} {resource} {quantity:?} is not \
                                 a Kubernetes quantity, such as 100m or 128Mi"
                            ),
                        );
                    }
                }
            }
        }
        if let (Some(requests), Some(limits)) = (&resources.requests, &resources.limits) {
            for (resource, request, limit) in [
                ("cpu", &requests.cpu, &limits.cpu),
                ("memory", &requests.memory, &limits.memory),
            ] {
                let (Some(request), Some(limit)) = (request, limit) else {
                    continue;
                };
                if exceeds(request, limit) == Some(true) {
                    refuse(
                        diagnostics,
                        name,
                        format!(
                            "container {name} resource requests {resource} {request:?} exceeds \
                             its limit {limit:?}; a request may not be above its limit"
                        ),
                    );
                }
            }
        }
    }
    validate_tcp_probes(container, diagnostics);
    let mut paths: BTreeSet<&str> = container
        .volume_mounts
        .iter()
        .map(|mount| mount.mount_path.as_str())
        .collect();
    for slot in &container.secrets {
        validate_secret_slot(container, slot, &mut paths, diagnostics);
    }
}

fn validate_tcp_probes(container: &ContainerRole, diagnostics: &mut Vec<Diagnostic>) {
    let name = &container.name;
    for (probe, port, path) in [
        (
            "readiness",
            container.readiness_tcp_port,
            &container.readiness_path,
        ),
        (
            "liveness",
            container.liveness_tcp_port,
            &container.liveness_path,
        ),
    ] {
        let Some(port) = port else {
            continue;
        };
        if port == 0 {
            refuse(
                diagnostics,
                name,
                format!("container {name} {probe}_tcp_port 0 is not a port; use 1-65535"),
            );
        }
        if path.is_some() {
            refuse(
                diagnostics,
                name,
                format!(
                    "container {name} declares both {probe}_path and {probe}_tcp_port; a \
                     container has one {probe} probe"
                ),
            );
        }
    }
}

fn validate_secret_slot<'a>(
    container: &ContainerRole,
    slot: &'a SecretSlot,
    paths: &mut BTreeSet<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = &slot.name;
    if slot.environment.is_some() != slot.key.is_some() {
        refuse(
            diagnostics,
            name,
            format!("secret slot {name} declares one of environment and key without the other"),
        );
    }
    let Some(mount) = &slot.mount else {
        if slot.environment.is_none() {
            refuse(
                diagnostics,
                name,
                format!("secret slot {name} declares neither an environment variable nor a mount"),
            );
        }
        return;
    };
    if !is_absolute(&mount.path) {
        refuse(
            diagnostics,
            name,
            format!(
                "secret slot {name} mount path {:?} must be absolute and may not traverse parents",
                mount.path
            ),
        );
    }
    if !paths.insert(mount.path.as_str()) {
        refuse(
            diagnostics,
            name,
            format!(
                "secret slot {name} mount path {:?} is already mounted in container {}",
                mount.path, container.name
            ),
        );
    }
    if mount.mode_bits().is_none() {
        refuse(
            diagnostics,
            name,
            format!(
                "secret slot {name} mode {:?} is not four octal digits within 0000-0777",
                mount.mode.as_deref().unwrap_or_default()
            ),
        );
    }
    let volume = secret_volume_name(name);
    if !is_dns_label(&volume) {
        refuse(
            diagnostics,
            name,
            format!(
                "secret slot {name} is mounted as the pod volume {volume:?}, which is not a \
                 DNS-1123 label (lowercase letters, digits and '-', at most 63 characters); \
                 rename the slot"
            ),
        );
    }
    validate_secret_items(name, mount, diagnostics);
}

fn validate_secret_items(
    name: &Identifier,
    mount: &SecretMount,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut item_paths = BTreeSet::new();
    for item in &mount.items {
        if item.key.is_empty() {
            refuse(
                diagnostics,
                name,
                format!("secret slot {name} projects an item with an empty key"),
            );
        } else if !is_secret_key(&item.key) {
            refuse(
                diagnostics,
                name,
                format!(
                    "secret slot {name} item key {:?} is not a Secret key: at most 253 letters, \
                     digits, '-', '_' and '.', not '.' and not starting with '..'",
                    item.key
                ),
            );
        }
        if !is_contained(&item.path) {
            refuse(
                diagnostics,
                name,
                format!(
                    "secret slot {name} item path {:?} must be relative, stay below the mount \
                     and not start with '..'",
                    item.path
                ),
            );
        }
        if !item_paths.insert(item.path.as_str()) {
            refuse(
                diagnostics,
                name,
                format!(
                    "secret slot {name} projects two items to the path {:?}",
                    item.path
                ),
            );
        }
    }
}

/// Validates the workload kind and pod settings of one workload.
pub(crate) fn validate_workload_settings(
    workload: &Workload,
    containers: &BTreeMap<Identifier, ContainerRole>,
    provided_endpoints: &BTreeMap<Identifier, ProvidedEndpoint>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = &workload.name;
    if workload
        .security
        .as_ref()
        .is_some_and(|security| *security == PodSecurity::default())
    {
        refuse(
            diagnostics,
            name,
            format!("workload {name} declares security with no setting in it"),
        );
    }
    if let Some(security) = &workload.security {
        for (key, id) in [
            ("run_as_user", security.run_as_user),
            ("run_as_group", security.run_as_group),
            ("fs_group", security.fs_group),
        ] {
            if let Some(id) = id.filter(|id| i64::from(*id) > INT32_MAX) {
                refuse(
                    diagnostics,
                    name,
                    format!(
                        "workload {name} security {key} {id} is above {INT32_MAX}, the largest \
                         id Kubernetes accepts"
                    ),
                );
            }
        }
    }
    if let Some(annotations) = &workload.annotations {
        validate_annotations(name, annotations, diagnostics);
    }
    if let Some(affinity) = &workload.affinity {
        validate_affinity(name, affinity, diagnostics);
    }
    validate_secret_volumes(workload, containers, diagnostics);
    let Some(schedule) = &workload.schedule else {
        if workload.replicas.is_none() {
            refuse(
                diagnostics,
                name,
                format!("workload {name} declares neither replicas nor a schedule"),
            );
        }
        return;
    };
    validate_schedule(name, schedule, diagnostics);
    if workload.replicas.is_some() {
        refuse(
            diagnostics,
            name,
            format!(
                "scheduled workload {name} declares replicas; a scheduled run has no standing \
                 instances, so delete replicas or the schedule"
            ),
        );
    }
    if workload.storage.is_some() {
        refuse(
            diagnostics,
            name,
            format!(
                "scheduled workload {name} declares the legacy storage; declare named volumes, \
                 which every run mounts"
            ),
        );
    }
    for container in &workload.containers {
        if containers
            .get(container)
            .is_some_and(|role| role.readiness_path.is_some() || role.readiness_tcp_port.is_some())
        {
            refuse(
                diagnostics,
                name,
                format!(
                    "scheduled workload {name} runs container {container}, which declares a \
                     readiness probe; nothing routes traffic to a scheduled run"
                ),
            );
        }
    }
    for endpoint in provided_endpoints.values() {
        if endpoint.workload == *name {
            refuse(
                diagnostics,
                name,
                format!(
                    "scheduled workload {name} is selected by provided endpoint {}, which would \
                     give it a Service port; a scheduled run serves no Service",
                    endpoint.name
                ),
            );
        }
    }
}

fn validate_schedule(name: &Identifier, schedule: &Schedule, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(reason) = cron_error(&schedule.cron) {
        refuse(
            diagnostics,
            name,
            format!(
                "scheduled workload {name} cron {:?} is not a schedule: {reason}",
                schedule.cron
            ),
        );
    }
    if let Some(zone) = &schedule.time_zone {
        if !is_time_zone(zone) {
            refuse(
                diagnostics,
                name,
                format!(
                    "scheduled workload {name} time_zone {zone:?} is not a canonical IANA time \
                     zone name, such as Europe/Berlin"
                ),
            );
        }
    }
    match schedule.run_deadline_seconds {
        None => refuse(
            diagnostics,
            name,
            format!(
                "scheduled workload {name} declares no run_deadline_seconds; a run without one \
                 may never end, so bound it"
            ),
        ),
        Some(seconds) if seconds <= 0 => refuse(
            diagnostics,
            name,
            format!("scheduled workload {name} run_deadline_seconds {seconds} must be positive"),
        ),
        Some(_) => {}
    }
    if let Some(seconds) = schedule
        .start_deadline_seconds
        .filter(|seconds| *seconds <= 0)
    {
        refuse(
            diagnostics,
            name,
            format!("scheduled workload {name} start_deadline_seconds {seconds} must be positive"),
        );
    }
    if let Some(retries) = schedule.retry_limit.filter(|retries| *retries < 0) {
        refuse(
            diagnostics,
            name,
            format!("scheduled workload {name} retry_limit {retries} must not be negative"),
        );
    }
    for (key, count) in [
        ("retry_limit", schedule.retry_limit),
        ("successful_runs", schedule.successful_runs.map(i64::from)),
        ("failed_runs", schedule.failed_runs.map(i64::from)),
    ] {
        if let Some(count) = count.filter(|count| *count > INT32_MAX) {
            refuse(
                diagnostics,
                name,
                format!(
                    "scheduled workload {name} {key} {count} is above {INT32_MAX}, the largest \
                     value Kubernetes reads"
                ),
            );
        }
    }
}

fn validate_annotations(
    name: &Identifier,
    annotations: &BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if annotations.is_empty() {
        refuse(
            diagnostics,
            name,
            format!("workload {name} declares annotations with none in them"),
        );
    }
    let mut size = 0usize;
    for (key, value) in annotations {
        size = size.saturating_add(key.len()).saturating_add(value.len());
        if !is_qualified_name(key) {
            refuse(
                diagnostics,
                name,
                format!(
                    "workload {name} annotation key {key:?} is not a qualified name: an optional \
                     DNS subdomain and '/', then at most 63 letters, digits, '-', '_' and '.', \
                     beginning and ending alphanumeric"
                ),
            );
        }
    }
    if size > ANNOTATIONS_LIMIT {
        refuse(
            diagnostics,
            name,
            format!(
                "workload {name} annotations total {size} bytes, above the {ANNOTATIONS_LIMIT} \
                 Kubernetes accepts"
            ),
        );
    }
}

fn validate_affinity(name: &Identifier, affinity: &Affinity, diagnostics: &mut Vec<Diagnostic>) {
    if affinity.spread_across.is_none() && affinity.node_labels.is_empty() {
        refuse(
            diagnostics,
            name,
            format!("workload {name} declares affinity with neither spread_across nor node_labels"),
        );
    }
    if affinity.spread.is_some() && affinity.spread_across.is_none() {
        refuse(
            diagnostics,
            name,
            format!("workload {name} affinity declares spread without spread_across"),
        );
    }
    if let Some(key) = affinity
        .spread_across
        .as_deref()
        .filter(|key| !is_qualified_name(key))
    {
        refuse(
            diagnostics,
            name,
            format!(
                "workload {name} affinity spread_across {key:?} is not a node label key, such as \
                 kubernetes.io/hostname"
            ),
        );
    }
    for (key, value) in &affinity.node_labels {
        if !is_qualified_name(key) {
            refuse(
                diagnostics,
                name,
                format!("workload {name} affinity node_labels key {key:?} is not a label key"),
            );
        }
        if !is_label_value(value) {
            refuse(
                diagnostics,
                name,
                format!(
                    "workload {name} affinity node_labels {key:?} value {value:?} is not a label \
                     value: at most 63 letters, digits, '-', '_' and '.', beginning and ending \
                     alphanumeric"
                ),
            );
        }
    }
}

/// The `ess-runtime/2` rules for keys `ess-runtime/1` already reads: an `/1` document compiles
/// exactly as it did, and a `/2` document is held to what Kubernetes and Helm accept.
pub(crate) fn validate_version_2(
    processes: &BTreeMap<Identifier, crate::runtime::Process>,
    containers: &BTreeMap<Identifier, ContainerRole>,
    workloads: &BTreeMap<Identifier, Workload>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for process in processes.values() {
        let name = &process.name;
        if let ProcessImage::Reference(reference) = &process.image {
            template(diagnostics, "process", name, "image", reference);
        }
        for argument in &process.entrypoint {
            template(diagnostics, "process", name, "entrypoint", argument);
        }
        for argument in &process.arguments {
            template(diagnostics, "process", name, "arguments", argument);
        }
    }
    for container in containers.values() {
        validate_version_2_container(container, diagnostics);
    }
    for workload in workloads.values() {
        validate_version_2_workload(workload, diagnostics);
    }
}

/// Refuses `text`, a string the chart renders as written, when Helm would evaluate it.
fn template(
    diagnostics: &mut Vec<Diagnostic>,
    owner: &str,
    subject: &Identifier,
    field: &str,
    text: &str,
) {
    if has_template_action(text) {
        refuse(
            diagnostics,
            subject,
            format!(
                "{owner} {subject} {field} {text:?} carries a Helm template action \"{{{{\", \
                 which the chart would evaluate at install time instead of stating it"
            ),
        );
    }
}

fn validate_version_2_container(container: &ContainerRole, diagnostics: &mut Vec<Diagnostic>) {
    let name = &container.name;
    if !is_dns_label(name.as_str()) {
        refuse(
            diagnostics,
            name,
            format!(
                "container {name} is not a DNS-1123 label (lowercase letters, digits and '-', \
                     at most 63 characters), which Kubernetes requires of a container name"
            ),
        );
    }
    if container.http_port == Some(0) {
        refuse(
            diagnostics,
            name,
            format!("container {name} http_port 0 is not a port; use 1-65535"),
        );
    }
    for (field, text) in [
        ("readiness_path", &container.readiness_path),
        ("liveness_path", &container.liveness_path),
    ] {
        if let Some(text) = text {
            template(diagnostics, "container", name, field, text);
        }
    }
    for slot in &container.config {
        template(
            diagnostics,
            "container",
            name,
            "config environment",
            &slot.environment,
        );
        if let Some(value) = &slot.value {
            template(diagnostics, "container", name, "config value", value);
        }
    }
    for slot in &container.endpoints {
        template(
            diagnostics,
            "container",
            name,
            "endpoint environment",
            &slot.environment,
        );
    }
    for mount in &container.volume_mounts {
        template(
            diagnostics,
            "container",
            name,
            "volume mount_path",
            &mount.mount_path,
        );
    }
    for slot in &container.secrets {
        let slot_name = &slot.name;
        if let Some(environment) = &slot.environment {
            template(
                diagnostics,
                "secret slot",
                slot_name,
                "environment",
                environment,
            );
        }
        if let Some(mount) = &slot.mount {
            template(
                diagnostics,
                "secret slot",
                slot_name,
                "mount path",
                &mount.path,
            );
            for item in &mount.items {
                template(diagnostics, "secret slot", slot_name, "item key", &item.key);
                template(
                    diagnostics,
                    "secret slot",
                    slot_name,
                    "item path",
                    &item.path,
                );
            }
        }
    }
}

fn validate_version_2_workload(workload: &Workload, diagnostics: &mut Vec<Diagnostic>) {
    let name = &workload.name;
    if !is_dns_label(name.as_str()) {
        refuse(
            diagnostics,
            name,
            format!(
                "workload {name} is not a DNS-1123 label (lowercase letters, digits and '-', \
                     at most 63 characters); its objects and Services are named after it"
            ),
        );
    }
    if let Some(replicas) = workload
        .replicas
        .filter(|replicas| i64::from(*replicas) > INT32_MAX)
    {
        refuse(
            diagnostics,
            name,
            format!(
                "workload {name} replicas {replicas} is above {INT32_MAX}, the largest value \
                     Kubernetes reads"
            ),
        );
    }
    for volume in &workload.volumes {
        if !is_dns_label(volume.name.as_str()) {
            refuse(
                diagnostics,
                name,
                format!(
                    "workload {name} volume {} is not a DNS-1123 label (lowercase letters, \
                         digits and '-', at most 63 characters), which Kubernetes requires of a \
                         pod volume name",
                    volume.name
                ),
            );
        }
    }
    for value in workload.annotations.iter().flat_map(BTreeMap::values) {
        template(diagnostics, "workload", name, "annotation", value);
    }
}

/// One pod volume per mounted secret slot: two containers mounting one slot agree on it, and no
/// persistent volume takes its name.
fn validate_secret_volumes(
    workload: &Workload,
    containers: &BTreeMap<Identifier, ContainerRole>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut mounts: BTreeMap<&Identifier, &SecretMount> = BTreeMap::new();
    for container in workload
        .containers
        .iter()
        .filter_map(|container| containers.get(container))
    {
        for slot in &container.secrets {
            let Some(mount) = &slot.mount else {
                continue;
            };
            if let Some(first) = mounts.insert(&slot.name, mount) {
                if first.mode != mount.mode || first.items != mount.items {
                    refuse(
                        diagnostics,
                        &workload.name,
                        format!(
                            "workload {} mounts secret slot {} twice with different items or mode",
                            workload.name, slot.name
                        ),
                    );
                }
            }
            let volume = secret_volume_name(&slot.name);
            if workload
                .volumes
                .iter()
                .any(|persistent| persistent.name.as_str() == volume)
            {
                refuse(
                    diagnostics,
                    &workload.name,
                    format!(
                        "workload {} declares the volume {volume}, which secret slot {} occupies",
                        workload.name, slot.name
                    ),
                );
            }
        }
    }
}

/// Validates a process image: an output, or a reference pinned by digest.
pub(crate) fn validate_image(
    process: &Identifier,
    image: &ProcessImage,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let ProcessImage::Reference(reference) = image {
        if let Some(reason) = unpinned(reference) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::UnpinnedInput,
                Some(process.clone()),
                format!(
                    "process {process} image {reference:?} is neither a build output nor an image \
                     pinned by digest ({reason}); write <repository>@sha256:<64 lowercase hex>"
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cron_error, exceeds, is_contained, is_dns_label, is_qualified_name, is_quantity,
        is_secret_key, is_time_zone, octal_mode, unpinned,
    };

    #[test]
    fn cron_expressions_kubernetes_reads_are_admitted() {
        for admitted in [
            "2-59/10 * * * *",
            "0 0 * * *",
            "*/5 1,2,3 1-15 JAN-jun mon-FRI",
            "0 0 ? * 6",
            "5/10 * * * *",
            "@hourly",
        ] {
            assert_eq!(cron_error(admitted), None, "{admitted}");
        }
        for refused in [
            "60 * * * *",
            "* 24 * * *",
            "* * 0 * *",
            "* * * 13 *",
            "* * * * 7",
            "0 0 * * 5-7",
            "*/0 * * * *",
            "5-1 * * * *",
            "? * * * *",
            "* * * *",
            "@every 5m",
            "TZ=UTC 0 0 * * *",
        ] {
            assert!(cron_error(refused).is_some(), "{refused}");
        }
    }

    #[test]
    fn quantities_modes_references_and_zones() {
        for quantity in ["100m", "1", "0.5", "128Mi", "1Gi", "2k"] {
            assert!(is_quantity(quantity), "{quantity}");
        }
        for quantity in ["", "lots", "1.2.3", "-1", "Mi", ".", "1 Gi"] {
            assert!(!is_quantity(quantity), "{quantity}");
        }
        assert_eq!(octal_mode("0400"), Some(0o400));
        assert_eq!(octal_mode("0777"), Some(0o777));
        assert_eq!(octal_mode("0000"), Some(0));
        for mode in ["400", "0800", "1777", "04000", "0x40"] {
            assert_eq!(octal_mode(mode), None, "{mode}");
        }
        let hex = "0123456789abcdef".repeat(4);
        assert_eq!(
            unpinned(&format!("example.invalid/runner@sha256:{hex}")),
            None
        );
        assert!(unpinned("example.invalid/runner:latest").is_some());
        assert!(unpinned(&format!("@sha256:{hex}")).is_some());
        assert!(is_time_zone("Europe/Berlin"));
        assert!(is_time_zone("UTC"));
        assert!(!is_time_zone("europe/berlin"));
        assert!(!is_time_zone("Europe/Atlantis"));
    }

    #[test]
    fn quantities_compare_exactly_across_suffixes() {
        for (request, limit, above) in [
            ("1500m", "1.5", false),
            ("1.5", "1499m", true),
            ("1Gi", "1024Mi", false),
            ("1025Mi", "1Gi", true),
            ("1G", "1Gi", false),
            ("1Gi", "1G", true),
            ("0.5n", "1n", false),
            ("100m", "0.1", false),
        ] {
            assert_eq!(exceeds(request, limit), Some(above), "{request} > {limit}");
        }
        assert_eq!(exceeds("lots", "1"), None);
    }

    #[test]
    fn names_keys_and_paths_kubernetes_accepts() {
        for label in ["a", "secret-db", &"a".repeat(63)] {
            assert!(is_dns_label(label), "{label}");
        }
        for label in ["", "-a", "a-", "a_b", "a.b", "A", &"a".repeat(64)] {
            assert!(!is_dns_label(label), "{label}");
        }
        for key in [
            "team",
            "example.invalid/team",
            "app.kubernetes.io/name",
            "A_b.c",
        ] {
            assert!(is_qualified_name(key), "{key}");
        }
        for key in ["", "-team", "Example/team", "a/b/c", "a b", "/team"] {
            assert!(!is_qualified_name(key), "{key}");
        }
        for key in ["token", "token.json", "a-b_c", ".hidden"] {
            assert!(is_secret_key(key), "{key}");
        }
        for key in ["", ".", "..", "..data", "a/b", "a b"] {
            assert!(!is_secret_key(key), "{key}");
        }
        assert!(is_contained("a/b"));
        for path in ["..data", "..token", "a/../b", "/a", "a//b"] {
            assert!(!is_contained(path), "{path}");
        }
    }
}
