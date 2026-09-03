//! Typed, deterministic lowering from ESS systems to immutable deployment intent.
//!
//! This crate does not build artifacts, contact registries, or apply infrastructure. It compiles
//! author-owned descriptions into reviewable IR, projects `BuildKit` and Helm inputs, verifies
//! executor-produced release manifests, resolves exact stack locks from an explicit catalogue,
//! and lowers private environment bindings to an independent Helm release set.

mod build;
mod diagnostic;
mod environment;
mod identity;
mod release;
mod runtime;
mod stack;

pub use build::{
    compile_build, project_buildkit, BuildIr, BuildMount, BuildNode, BuildOutput, BuildOutputKind,
    BuildSpec, BuildkitProjection, ImageConfig, NetworkMode, Platform, BUILD_FORMAT,
    BUILD_IR_FORMAT,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
pub use environment::{
    compile_deployment, project_helm, DeploymentIr, DeploymentRelease, EnvironmentSpec,
    ExternalBinding, HelmProjection, ReleaseBinding, SecretBinding, DEPLOYMENT_FORMAT,
    ENVIRONMENT_FORMAT,
};
pub use identity::{Digest, Identifier};
pub use release::{
    verify_release, Artifact, ArtifactKind, Evidence, EvidenceKind, ReleaseManifest, RELEASE_FORMAT,
};
pub use runtime::{
    compile_runtime, ConfigKind, ConfigSlot, ContainerRole, EndpointSlot, Process, RuntimeIr,
    RuntimeSpec, SecretSlot, Workload, RUNTIME_FORMAT, RUNTIME_IR_FORMAT,
};
pub use stack::{
    resolve_stack, ExternalSystem, LockedSystem, ReleaseCandidate, ReleaseCatalog,
    RuntimeRequirements, StackLock, StackSpec, SystemRequirement, RELEASE_CATALOG_FORMAT,
    STACK_FORMAT, STACK_LOCK_FORMAT,
};
