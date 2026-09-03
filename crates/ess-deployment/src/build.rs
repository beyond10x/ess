use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};

/// Human-authored build graph format.
pub const BUILD_FORMAT: &str = "ess-build/1";
/// Compiler-owned build graph format.
pub const BUILD_IR_FORMAT: &str = "ess-build-ir/1";

/// One target platform for a build output.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    /// OCI operating system name.
    pub os: String,
    /// OCI architecture name.
    pub architecture: String,
    /// Optional OCI architecture variant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

impl Platform {
    fn render(&self) -> String {
        self.variant.as_ref().map_or_else(
            || format!("{}/{}", self.os, self.architecture),
            |variant| format!("{}/{}/{variant}", self.os, self.architecture),
        )
    }
}

/// Whether an executable build node may use the network.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    /// No network access.
    #[default]
    None,
    /// Sandboxed `BuildKit` network access.
    Sandbox,
}

/// One typed mount made available only while a run node executes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BuildMount {
    /// Read-only filesystem from another DAG node.
    Input {
        /// Supplying node.
        from: Identifier,
        /// Absolute mount target.
        target: String,
        /// Optional source path inside the supplying node.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    /// Executor-owned persistent cache. Its contents are not an artifact input.
    Cache {
        /// Stable cache identity.
        id: Identifier,
        /// Absolute mount target.
        target: String,
    },
    /// Ephemeral in-memory filesystem.
    Tmpfs {
        /// Absolute mount target.
        target: String,
    },
    /// Authorization-only secret supplied by the executor.
    Secret {
        /// Declared secret identity, never its value.
        secret: Identifier,
        /// Absolute mount target.
        target: String,
    },
}

/// OCI runtime configuration attached to an image node.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageConfig {
    /// Executable and fixed arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entrypoint: Vec<String>,
    /// Default runtime arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    /// Numeric or named runtime user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Default working directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    /// Public, non-secret image environment.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,
}

/// One transformation in the build DAG.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BuildNode {
    /// A repository-relative source tree imported into a scratch stage.
    Source {
        /// Repository-relative source path.
        path: String,
        /// Absolute path in the stage.
        destination: String,
    },
    /// An immutable OCI base image.
    OciBase {
        /// Registry repository without a tag or digest.
        reference: String,
        /// Exact manifest or index digest.
        digest: Digest,
    },
    /// An explicit executable transformation.
    Run {
        /// Root filesystem inherited by this node.
        base: Identifier,
        /// Executable and arguments; never interpreted as a shell string.
        argv: Vec<String>,
        /// Optional working directory.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        workdir: Option<String>,
        /// Public, content-affecting environment.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        environment: BTreeMap<String, String>,
        /// Typed ephemeral mounts.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        mounts: Vec<BuildMount>,
        /// Explicit network policy.
        #[serde(default)]
        network: NetworkMode,
    },
    /// Copy files from one stage onto another root filesystem.
    Copy {
        /// Root filesystem inherited by this node.
        base: Identifier,
        /// Stage supplying the copied content.
        from: Identifier,
        /// Source path.
        source: String,
        /// Destination path.
        destination: String,
    },
    /// Final OCI image configuration over an existing root filesystem.
    Image {
        /// Root filesystem node.
        rootfs: Identifier,
        /// OCI configuration.
        #[serde(default)]
        config: ImageConfig,
    },
    /// A file-shaped artifact exported from a stage.
    Artifact {
        /// Stage supplying the artifact.
        from: Identifier,
        /// Absolute artifact path in that stage.
        path: String,
    },
}

impl BuildNode {
    fn dependencies(&self) -> BTreeSet<Identifier> {
        let mut dependencies = BTreeSet::new();
        match self {
            Self::Source { .. } | Self::OciBase { .. } => {}
            Self::Run { base, mounts, .. } => {
                dependencies.insert(base.clone());
                for mount in mounts {
                    if let BuildMount::Input { from, .. } = mount {
                        dependencies.insert(from.clone());
                    }
                }
            }
            Self::Copy { base, from, .. } => {
                dependencies.insert(base.clone());
                dependencies.insert(from.clone());
            }
            Self::Image { rootfs, .. } => {
                dependencies.insert(rootfs.clone());
            }
            Self::Artifact { from, .. } => {
                dependencies.insert(from.clone());
            }
        }
        dependencies
    }
}

/// A named build node before duplicate and graph checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NamedBuildNode {
    /// Stable node identity.
    pub id: Identifier,
    /// Typed transformation.
    #[serde(flatten)]
    pub node: BuildNode,
}

/// Kind of immutable output produced by a build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildOutputKind {
    /// OCI image or multi-platform index.
    OciImage,
    /// Executable or library file.
    Binary,
    /// General archive.
    Archive,
    /// Packaged Helm chart.
    HelmChart,
}

/// One named result of the build graph.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildOutput {
    /// Stable output identity.
    pub name: Identifier,
    /// Independently versioned release unit which owns this output.
    pub release_unit: Identifier,
    /// Node providing the result.
    pub node: Identifier,
    /// Artifact kind.
    pub kind: BuildOutputKind,
    /// Optional publication repository, without a mutable tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Human-authored build description.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildSpec {
    format: String,
    /// Stable build identity.
    pub build: Identifier,
    /// Supported target platforms.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub platforms: BTreeSet<Platform>,
    /// Executor-known secret identities available to run nodes.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub secrets: BTreeSet<Identifier>,
    /// Build transformations.
    pub nodes: Vec<NamedBuildNode>,
    /// Published outputs.
    pub outputs: Vec<BuildOutput>,
}

impl BuildSpec {
    /// Reads a strict JSON build specification.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML build specification.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// Validated, canonical build graph.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildIr {
    format: String,
    build: Identifier,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    platforms: BTreeSet<Platform>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    secrets: BTreeSet<Identifier>,
    nodes: BTreeMap<Identifier, BuildNode>,
    order: Vec<Identifier>,
    outputs: BTreeMap<Identifier, BuildOutput>,
}

impl BuildIr {
    /// Reads compiler-owned JSON and rejects incompatible formats or invalid graphs.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Stable build identity.
    pub fn build(&self) -> &Identifier {
        &self.build
    }

    /// Canonically ordered graph nodes.
    pub fn nodes(&self) -> &BTreeMap<Identifier, BuildNode> {
        &self.nodes
    }

    /// Deterministic topological execution order.
    pub fn order(&self) -> &[Identifier] {
        &self.order
    }

    /// Named immutable outputs.
    pub fn outputs(&self) -> &BTreeMap<Identifier, BuildOutput> {
        &self.outputs
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of the exact canonical build IR.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Compile and validate an authored build graph.
#[allow(clippy::too_many_lines)]
pub fn compile_build(specification: &BuildSpec) -> Result<BuildIr, Diagnostics> {
    let mut diagnostics = Vec::new();
    if specification.format != BUILD_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Build,
            DiagnosticCode::UnsupportedFormat,
            Some(specification.build.clone()),
            format!(
                "build format {:?} is unsupported; expected {BUILD_FORMAT:?}",
                specification.format
            ),
        ));
    }
    if specification.platforms.is_empty() {
        diagnostics.push(Diagnostic::new(
            Stage::Build,
            DiagnosticCode::InvalidValue,
            Some(specification.build.clone()),
            "a build must declare at least one target platform",
        ));
    }
    for platform in &specification.platforms {
        if platform.os.is_empty() || platform.architecture.is_empty() {
            diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::InvalidValue,
                Some(specification.build.clone()),
                "platform os and architecture must not be empty",
            ));
        }
    }

    let mut nodes = BTreeMap::new();
    for named in &specification.nodes {
        if nodes.insert(named.id.clone(), named.node.clone()).is_some() {
            diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::DuplicateIdentifier,
                Some(named.id.clone()),
                format!("build node {} is declared more than once", named.id),
            ));
        }
        validate_node(named, &specification.secrets, &mut diagnostics);
    }

    for (id, node) in &nodes {
        for dependency in node.dependencies() {
            if !nodes.contains_key(&dependency) {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::UnknownReference,
                    Some(id.clone()),
                    format!("build node {id} depends on unknown node {dependency}"),
                ));
            }
        }
    }

    let mut outputs = BTreeMap::new();
    for output in &specification.outputs {
        if outputs
            .insert(output.name.clone(), output.clone())
            .is_some()
        {
            diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::DuplicateIdentifier,
                Some(output.name.clone()),
                format!("build output {} is declared more than once", output.name),
            ));
        }
        match nodes.get(&output.node) {
            None => diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::UnknownReference,
                Some(output.name.clone()),
                format!(
                    "build output {} selects unknown node {}",
                    output.name, output.node
                ),
            )),
            Some(BuildNode::Image { .. }) if output.kind == BuildOutputKind::OciImage => {}
            Some(BuildNode::Artifact { .. }) if output.kind != BuildOutputKind::OciImage => {}
            Some(_) => diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::InvalidValue,
                Some(output.name.clone()),
                "output kind does not match the selected image or artifact node",
            )),
        }
        if output.kind == BuildOutputKind::OciImage && output.repository.is_none() {
            diagnostics.push(Diagnostic::new(
                Stage::Build,
                DiagnosticCode::MissingOutput,
                Some(output.name.clone()),
                "an OCI image output must declare its publication repository",
            ));
        }
    }
    if outputs.is_empty() {
        diagnostics.push(Diagnostic::new(
            Stage::Build,
            DiagnosticCode::MissingOutput,
            Some(specification.build.clone()),
            "a build must publish at least one named output",
        ));
    }

    let order = topological_order(&nodes).unwrap_or_else(|| {
        diagnostics.push(Diagnostic::new(
            Stage::Build,
            DiagnosticCode::DependencyCycle,
            Some(specification.build.clone()),
            "the build graph contains a dependency cycle",
        ));
        Vec::new()
    });

    if diagnostics.is_empty() {
        Ok(BuildIr {
            format: BUILD_IR_FORMAT.to_owned(),
            build: specification.build.clone(),
            platforms: specification.platforms.clone(),
            secrets: specification.secrets.clone(),
            nodes,
            order,
            outputs,
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

#[allow(clippy::too_many_lines)]
fn validate_node(
    named: &NamedBuildNode,
    secrets: &BTreeSet<Identifier>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let invalid_absolute = |value: &str| !value.starts_with('/');
    match &named.node {
        BuildNode::Source { path, destination } => {
            let path = Path::new(path);
            if path.is_absolute()
                || path
                    .components()
                    .any(|component| matches!(component, Component::ParentDir))
            {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::InvalidValue,
                    Some(named.id.clone()),
                    "source paths must be repository-relative and may not traverse parents",
                ));
            }
            if invalid_absolute(destination) {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::InvalidValue,
                    Some(named.id.clone()),
                    "source destinations must be absolute",
                ));
            }
        }
        BuildNode::OciBase { reference, .. } => {
            if reference.contains('@')
                || reference
                    .rsplit('/')
                    .next()
                    .is_some_and(|p| p.contains(':'))
            {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::UnpinnedInput,
                    Some(named.id.clone()),
                    "OCI base reference must be a repository without tag or embedded digest; use the digest field",
                ));
            }
        }
        BuildNode::Run { argv, mounts, .. } => {
            if argv.is_empty() || argv.iter().any(String::is_empty) {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::InvalidValue,
                    Some(named.id.clone()),
                    "run argv must contain a non-empty executable and only non-empty arguments",
                ));
            }
            for mount in mounts {
                let target = match mount {
                    BuildMount::Input { target, .. }
                    | BuildMount::Cache { target, .. }
                    | BuildMount::Tmpfs { target }
                    | BuildMount::Secret { target, .. } => target,
                };
                if invalid_absolute(target) {
                    diagnostics.push(Diagnostic::new(
                        Stage::Build,
                        DiagnosticCode::InvalidValue,
                        Some(named.id.clone()),
                        format!("build mount target {target:?} must be absolute"),
                    ));
                }
                if let BuildMount::Secret { secret, .. } = mount {
                    if !secrets.contains(secret) {
                        diagnostics.push(Diagnostic::new(
                            Stage::Build,
                            DiagnosticCode::UndeclaredSecret,
                            Some(named.id.clone()),
                            format!("run node requires undeclared executor secret {secret}"),
                        ));
                    }
                }
            }
        }
        BuildNode::Copy {
            source,
            destination,
            ..
        } => {
            if invalid_absolute(source) || invalid_absolute(destination) {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::InvalidValue,
                    Some(named.id.clone()),
                    "copy source and destination must be absolute",
                ));
            }
        }
        BuildNode::Image { .. } => {}
        BuildNode::Artifact { path, .. } => {
            if invalid_absolute(path) {
                diagnostics.push(Diagnostic::new(
                    Stage::Build,
                    DiagnosticCode::InvalidValue,
                    Some(named.id.clone()),
                    "artifact path must be absolute",
                ));
            }
        }
    }
}

fn topological_order(nodes: &BTreeMap<Identifier, BuildNode>) -> Option<Vec<Identifier>> {
    let mut remaining: BTreeMap<Identifier, BTreeSet<Identifier>> = nodes
        .iter()
        .map(|(id, node)| (id.clone(), node.dependencies()))
        .collect();
    let mut ready: BTreeSet<Identifier> = remaining
        .iter()
        .filter(|(_, dependencies)| dependencies.is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        remaining.remove(&id);
        for (candidate, dependencies) in &mut remaining {
            dependencies.remove(&id);
            if dependencies.is_empty() {
                ready.insert(candidate.clone());
            }
        }
    }
    (order.len() == nodes.len()).then_some(order)
}

/// Deterministic files executable by BuildKit-compatible tooling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildkitProjection {
    files: BTreeMap<String, String>,
}

impl BuildkitProjection {
    /// Projected files by relative path.
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
}

/// Project a validated build graph to a Dockerfile frontend and Bake targets.
#[allow(clippy::too_many_lines)]
pub fn project_buildkit(build: &BuildIr) -> BuildkitProjection {
    let mut dockerfile = String::from("# syntax=docker/dockerfile:1.12\n");
    for id in &build.order {
        let node = build.nodes.get(id).expect("topological nodes are total");
        let stage_alias = alias(id);
        match node {
            BuildNode::Source { path, destination } => {
                writeln!(&mut dockerfile, "FROM scratch AS {stage_alias}").unwrap();
                writeln!(
                    &mut dockerfile,
                    "COPY {}",
                    json_array([path.as_str(), destination.as_str()])
                )
                .unwrap();
            }
            BuildNode::OciBase { reference, digest } => {
                writeln!(
                    &mut dockerfile,
                    "FROM {reference}@{digest} AS {stage_alias}"
                )
                .unwrap();
            }
            BuildNode::Run {
                base,
                argv,
                workdir,
                environment,
                mounts,
                network,
            } => {
                writeln!(&mut dockerfile, "FROM {} AS {stage_alias}", alias(base)).unwrap();
                if let Some(workdir) = workdir {
                    writeln!(&mut dockerfile, "WORKDIR {workdir}").unwrap();
                }
                for (key, value) in environment {
                    writeln!(&mut dockerfile, "ENV {key}={}", docker_token(value)).unwrap();
                }
                write!(
                    &mut dockerfile,
                    "RUN --network={}",
                    match network {
                        NetworkMode::None => "none",
                        NetworkMode::Sandbox => "default",
                    }
                )
                .unwrap();
                for mount in mounts {
                    write!(&mut dockerfile, " {}", render_mount(mount)).unwrap();
                }
                writeln!(
                    &mut dockerfile,
                    " {}",
                    json_array(argv.iter().map(String::as_str))
                )
                .unwrap();
            }
            BuildNode::Copy {
                base,
                from,
                source,
                destination,
            } => {
                writeln!(&mut dockerfile, "FROM {} AS {stage_alias}", alias(base)).unwrap();
                writeln!(
                    &mut dockerfile,
                    "COPY --from={} {}",
                    alias(from),
                    json_array([source.as_str(), destination.as_str()])
                )
                .unwrap();
            }
            BuildNode::Image { rootfs, config } => {
                writeln!(&mut dockerfile, "FROM {} AS {stage_alias}", alias(rootfs)).unwrap();
                if !config.entrypoint.is_empty() {
                    writeln!(
                        &mut dockerfile,
                        "ENTRYPOINT {}",
                        json_array(config.entrypoint.iter().map(String::as_str))
                    )
                    .unwrap();
                }
                if !config.command.is_empty() {
                    writeln!(
                        &mut dockerfile,
                        "CMD {}",
                        json_array(config.command.iter().map(String::as_str))
                    )
                    .unwrap();
                }
                if let Some(user) = &config.user {
                    writeln!(&mut dockerfile, "USER {user}").unwrap();
                }
                if let Some(workdir) = &config.workdir {
                    writeln!(&mut dockerfile, "WORKDIR {workdir}").unwrap();
                }
                for (key, value) in &config.environment {
                    writeln!(&mut dockerfile, "ENV {key}={}", docker_token(value)).unwrap();
                }
            }
            BuildNode::Artifact { from, path } => {
                writeln!(&mut dockerfile, "FROM scratch AS {stage_alias}").unwrap();
                writeln!(
                    &mut dockerfile,
                    "COPY --from={} {}",
                    alias(from),
                    json_array([path.as_str(), "/out/"])
                )
                .unwrap();
            }
        }
        dockerfile.push('\n');
    }

    let mut bake = String::new();
    writeln!(&mut bake, "group \"default\" {{").unwrap();
    write!(&mut bake, "  targets = [").unwrap();
    for (index, output) in build.outputs.values().enumerate() {
        if index > 0 {
            bake.push_str(", ");
        }
        write!(&mut bake, "\"{}\"", output.name).unwrap();
    }
    bake.push_str("]\n}\n\n");
    for output in build.outputs.values() {
        writeln!(&mut bake, "target \"{}\" {{", output.name).unwrap();
        bake.push_str("  context = \".\"\n  dockerfile = \"Dockerfile.ess\"\n");
        writeln!(&mut bake, "  target = \"{}\"", alias(&output.node)).unwrap();
        write!(&mut bake, "  platforms = [").unwrap();
        for (index, platform) in build.platforms.iter().enumerate() {
            if index > 0 {
                bake.push_str(", ");
            }
            write!(&mut bake, "\"{}\"", platform.render()).unwrap();
        }
        bake.push_str("]\n");
        if output.kind != BuildOutputKind::OciImage {
            writeln!(
                &mut bake,
                "  output = [\"type=local,dest=out/{}\"]",
                output.name
            )
            .unwrap();
        }
        bake.push_str("}\n\n");
    }

    let files = BTreeMap::from([
        ("Dockerfile.ess".to_owned(), dockerfile),
        ("docker-bake.hcl".to_owned(), bake),
        ("ess-build-ir.json".to_owned(), build.to_canonical_json()),
    ]);
    BuildkitProjection { files }
}

fn alias(identifier: &Identifier) -> String {
    identifier
        .as_str()
        .chars()
        .map(|character| match character {
            '.' | '-' => '_',
            other => other,
        })
        .collect()
}

fn json_array<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    serde_json::to_string(&values.into_iter().collect::<Vec<_>>())
        .expect("a string array always serializes")
}

fn docker_token(value: &str) -> String {
    serde_json::to_string(value).expect("a string always serializes")
}

fn render_mount(mount: &BuildMount) -> String {
    match mount {
        BuildMount::Input {
            from,
            target,
            source,
        } => format!(
            "--mount=type=bind,from={},source={},target={},ro",
            alias(from),
            source.as_deref().unwrap_or("/"),
            target
        ),
        BuildMount::Cache { id, target } => {
            format!("--mount=type=cache,id={id},target={target}")
        }
        BuildMount::Tmpfs { target } => format!("--mount=type=tmpfs,target={target}"),
        BuildMount::Secret { secret, target } => {
            format!("--mount=type=secret,id={secret},target={target}")
        }
    }
}
