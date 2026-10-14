//! `ess-composition/4` contract roles: one imported service's component plays another's.
//!
//! A role relates two pins the composition already holds: the component selected for `service`
//! plays the component selected for `plays`. Its command entries say which local command stands
//! where in that contract, and `stands` is closed:
//!
//! * `implements`: the local command answers the contract command. Both ends are admitted on their
//!   selected components' `accepts` surfaces, and the inputs are compared: every contract input
//!   field is a local input field of the same name, wire name and a conforming type (the field
//!   comparison type conformance applies), and a local input the contract does not declare is
//!   `Optional` and not `null_when_absent`, since the contract's caller never sends it.
//! * `gates`: the local command decides before the contract command succeeds. Both ends are
//!   admitted; the ordering is recorded, not executed, and no inputs are compared.
//!
//! See `docs/design/composition-contract-roles.md`.

use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, ComponentRef, EssSemanticRef};
use ess_compiler::EssIr;
use ess_domain::name::{QualifiedName, Version};

use crate::{
    conformance, Admission, CompositionCode, CompositionDiagnostic, CompositionSpec,
    ResolvedService, ServiceKey, SourceDigest,
};

/// Where a role's local command stands in the contract it plays (`ess-composition/4`).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Stands {
    /// The local command answers the contract command; their inputs are compared.
    Implements,
    /// The local command decides before the contract command succeeds; recorded, not executed.
    Gates,
}

/// One command entry of a [`ContractRole`]: a local command and where it stands in the contract.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct RoleCommand {
    local: CommandRef,
    stands: Stands,
    contract: CommandRef,
}

impl RoleCommand {
    /// Says that `local` stands as `stands` toward the contract's `contract`.
    pub fn new(local: CommandRef, stands: Stands, contract: CommandRef) -> Self {
        Self {
            local,
            stands,
            contract,
        }
    }

    /// A command the role's own service's selected component accepts.
    pub fn local(&self) -> &CommandRef {
        &self.local
    }

    /// Where the local command stands.
    pub const fn stands(&self) -> Stands {
        self.stands
    }

    /// A command the contract service's selected component accepts.
    pub fn contract(&self) -> &CommandRef {
        &self.contract
    }
}

/// An `ess-composition/4` assertion that the component selected for `service` plays the component
/// selected for `plays`, with the commands that stand in it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRole {
    service: ServiceKey,
    plays: ServiceKey,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    commands: Vec<RoleCommand>,
}

impl ContractRole {
    /// Says that `service`'s selected component plays `plays`'s, with `commands` in order.
    pub fn new(service: ServiceKey, plays: ServiceKey, commands: Vec<RoleCommand>) -> Self {
        Self {
            service,
            plays,
            commands,
        }
    }

    /// The service whose selected component plays the role.
    pub fn service(&self) -> &ServiceKey {
        &self.service
    }

    /// The service whose selected component is the contract played.
    pub fn plays(&self) -> &ServiceKey {
        &self.plays
    }

    /// The command entries, in document order.
    pub fn commands(&self) -> &[RoleCommand] {
        &self.commands
    }
}

/// One end of a [`ResolvedRole`]: the pin it was checked at.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RoleParty {
    key: ServiceKey,
    system: QualifiedName,
    version: Version,
    source_digest: SourceDigest,
    component: ComponentRef,
}

impl RoleParty {
    fn of(key: &ServiceKey, service: &ResolvedService) -> Self {
        Self {
            key: key.clone(),
            system: service.system.clone(),
            version: service.version,
            source_digest: service.source_digest.clone(),
            component: service.component.clone(),
        }
    }

    /// The composition-local service key.
    pub fn key(&self) -> &ServiceKey {
        &self.key
    }

    /// The ESS system identity.
    pub fn system(&self) -> &QualifiedName {
        &self.system
    }

    /// The ESS specification version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// The compiled-model digest the role was checked at.
    pub fn source_digest(&self) -> &SourceDigest {
        &self.source_digest
    }

    /// The selected component.
    pub fn component(&self) -> &ComponentRef {
        &self.component
    }
}

/// A checked [`ContractRole`] in compiled composition IR.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedRole {
    service: RoleParty,
    plays: RoleParty,
    commands: Vec<RoleCommand>,
}

impl ResolvedRole {
    /// The service whose component plays the role, at the pin it was checked at.
    pub fn service(&self) -> &RoleParty {
        &self.service
    }

    /// The contract's service, at the pin it was checked at.
    pub fn plays(&self) -> &RoleParty {
        &self.plays
    }

    /// The command entries, in document order.
    pub fn commands(&self) -> &[RoleCommand] {
        &self.commands
    }
}

/// Checks every role in document order; only a role with no refusal is returned.
pub(crate) fn resolve_roles(
    specification: &CompositionSpec,
    admission: &Admission<'_, '_>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> Vec<ResolvedRole> {
    let mut resolved = Vec::new();
    let mut pairs: BTreeMap<(&ServiceKey, &ServiceKey), usize> = BTreeMap::new();
    for (index, role) in specification.roles().iter().enumerate() {
        let at = format!("roles[{index}]");
        let mut sound = true;
        for (field, key) in [("service", &role.service), ("plays", &role.plays)] {
            if !admission.declared_keys.contains(key) {
                diagnostics.push(CompositionDiagnostic::new(
                    CompositionCode::UnknownRoleService,
                    Some(key.clone()),
                    format!(
                        "{at}.{field} names service `{key}`, which the composition does not import"
                    ),
                ));
                sound = false;
            }
        }
        if role.service == role.plays {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::ConflictingRole,
                Some(role.service.clone()),
                format!(
                    "{at}: service `{}` cannot play itself; `plays` names the service whose \
                     component is the contract",
                    role.service
                ),
            ));
            sound = false;
        } else if let Some(first) = pairs.get(&(&role.service, &role.plays)) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::ConflictingRole,
                Some(role.service.clone()),
                format!(
                    "{at}: `{}` already plays `{}` in roles[{first}]; write its commands there",
                    role.service, role.plays
                ),
            ));
            sound = false;
        } else {
            pairs.insert((&role.service, &role.plays), index);
        }

        let ends_declared = admission.declared_keys.contains(&role.service)
            && admission.declared_keys.contains(&role.plays);
        let mut entries: BTreeMap<&RoleCommand, usize> = BTreeMap::new();
        for (position, entry) in role.commands.iter().enumerate() {
            let at = format!("{at}.commands[{position}]");
            if let Some(first) = entries.get(entry) {
                diagnostics.push(CompositionDiagnostic::new(
                    CompositionCode::ConflictingRole,
                    Some(role.service.clone()),
                    format!("{at}: repeats roles[{index}].commands[{first}]"),
                ));
                sound = false;
                continue;
            }
            entries.insert(entry, position);
            if !ends_declared {
                // The unimported key is already refused; its commands have no surface to meet.
                continue;
            }
            let local = admit_command(
                admission,
                &role.service,
                &entry.local,
                &format!("{at}.local"),
                diagnostics,
            );
            let contract = admit_command(
                admission,
                &role.plays,
                &entry.contract,
                &format!("{at}.contract"),
                diagnostics,
            );
            let (Some(local_ir), Some(contract_ir)) = (local, contract) else {
                sound = false;
                continue;
            };
            if entry.stands == Stands::Implements
                && !implements(role, entry, &at, (local_ir, contract_ir), diagnostics)
            {
                sound = false;
            }
        }

        let parties = (
            admission.resolved.get(&role.service),
            admission.resolved.get(&role.plays),
        );
        if let (true, (Some(service), Some(plays))) = (sound, parties) {
            resolved.push(ResolvedRole {
                service: RoleParty::of(&role.service, service),
                plays: RoleParty::of(&role.plays, plays),
                commands: role.commands.clone(),
            });
        }
    }
    resolved
}

/// Admits one command end on its service's selected component, locating every refusal at `at`.
fn admit_command<'ir>(
    admission: &Admission<'_, 'ir>,
    service: &ServiceKey,
    command: &CommandRef,
    at: &str,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> Option<&'ir EssIr> {
    let mut found = Vec::new();
    let ir = admission.admit(service, &EssSemanticRef::from(command.clone()), &mut found);
    for mut diagnostic in found {
        diagnostic.detail = format!("{at}: {}", diagnostic.detail);
        diagnostics.push(diagnostic);
    }
    ir
}

/// Compares an `implements` entry's inputs, recording a drift; `true` when they agree.
fn implements(
    role: &ContractRole,
    entry: &RoleCommand,
    at: &str,
    (local, contract): (&EssIr, &EssIr),
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> bool {
    let local_name = entry.local.name();
    let contract_name = entry.contract.name();
    let drifts = conformance::compare_inputs(
        (local, local_name, &local.commands()[local_name].input),
        (
            contract,
            contract_name,
            &contract.commands()[contract_name].input,
        ),
    );
    if drifts.is_empty() {
        return true;
    }
    diagnostics.push(CompositionDiagnostic::new(
        CompositionCode::RoleInputDrift,
        Some(role.service.clone()),
        format!(
            "{at}: `{}` command `{local_name}` does not implement `{}` command \
             `{contract_name}`: {}",
            role.service,
            role.plays,
            drifts.join("; ")
        ),
    ));
    false
}
