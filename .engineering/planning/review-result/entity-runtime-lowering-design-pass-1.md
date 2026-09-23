---
format: aep.planning-md/1
id: review-result:entity-runtime-lowering-design-pass-1
kind: review-result
status: active
title: Entity Runtime lowering design review pass 1
relations:
- reviews: initiative:ess-evolution
revision: 1
---
needs-revision

Covered proposal: `docs/design/ess-evolution/entity-runtime-lowering.md` at
`sha256:3fc6d46b38529c9dfea60dad58bc0f6ffdecbda1bd095d7704cc4ea69e46ec15`,
`docs/design/models/entity-runtime-lowering/system.yaml` at
`sha256:a88342f8bcfb4a331949b0656ae4865f5c3524b9fd01c39c35f5f91406bd9564`, and
`docs/design/models/entity-runtime-lowering/domains/contract.yaml` at
`sha256:b84da3d243c09f739eb18469b921eca33a3ff4353c3bd582e5c17292ddfb6a06`,
against ESS `be604d874ee9e567ae104e565e7cbddf953885a9` and Entity Runtime
`24d31cf1f97a3744db7e65c5629e056bc7a3a241`.

## Blockers

### 1. Load-before-selection loses a reachable billing refusal

**Requirement.** The complete interface must retain the real billing fixture's behavior while ER
owns entity decisions. The design says the host supplies facts but only ER selects an outcome
(`docs/design/ess-evolution/entity-runtime-lowering.md:336`), and its `/4` flow loads the instance by
the command binding before invoking ER (`docs/design/ess-evolution/entity-runtime-lowering.md:467`).

**Source and target evidence.** `PayInvoice` declares `settled` under the positive-amount guard and
then the subjectless `rejected` default before its `wrong-state` branch
(`examples/billing/domains/invoice.yaml:310`). The approved realization makes the precedence
explicit: a nonpositive payment is `rejected` before the invoice lookup, including when that invoice
does not exist (`examples/billing-realization/src/invoice.rs:271`); the conformance adapter preserves
the same boundary (`examples/billing-realization/tests/conformance.rs:430`). ER cannot perform that
selection without a subject: `RegistryRuntime::decide` requires `&EntityInstance`
(`crates/entity-core/src/runtime.rs:359` at ER `24d31cf1`), and `decide` validates the instance before
normalizing arguments or selecting an outcome (`crates/entity-core/src/runtime.rs:740` at that
revision).

**Why this prevents the claimed handoff.** For a nonpositive payment naming an unknown invoice, the
proposed host cannot load the instance needed to call ER. Returning an unknown-subject result first
changes the declared branch to no outcome; evaluating the amount guard in the host preserves the
fixture but makes the host select `rejected`, contrary to the interface's ownership rule. This is a
reachable source/target ordering gap, separate from `StatelessCommandUnsupported`, and it means the
stated load-then-decide `/4` flow is not a complete realization of even the required invoice-service
boundary.

**Bounded correction.** State and type the pre-load decision boundary explicitly. Either retain a
host-side preliminary selector with exact source ordering and identify that exception to ER outcome
ownership, or record this PayInvoice counterexample as a durable-service blocker requiring a later
admitted ER operation that can select subjectless refusals before instance loading. Do not make the
pure lowerer refuse or omit PayInvoice merely to hide the runtime gap, and add the negative-amount,
unknown-identity fixture check to the `/4` acceptance boundary.

### 2. Omitted optional creation fields are silently changed from host decisions to absence

**Requirement.** Host facts and delegated decisions must remain explicit. The design itself says an
omitted creation field is `Undetermined` (`docs/design/ess-evolution/entity-runtime-lowering.md:199`)
and provides `BoundTarget::EntityField`, `BoundSource::Undetermined`, and
`UndeterminedFieldSupplied` for that case (`docs/design/ess-evolution/entity-runtime-lowering.md:104`).

**Source and fixture evidence.** ESS retains only the fields a branch determines in
`ResolvedOutcome::sets`; an empty or missing entry is a statement that the implementation chooses
the field, exactly as for event payloads (`crates/specify/ess-compiler/src/ir.rs:729`). The real
fixtures reach the optional case: invoice creation does not set optional `note` or `issued_at`
(`examples/billing/domains/invoice.yaml:118` and `:122`), and gatepass registration has no `sets` at
all while `Visit.badge` is optional (`examples/gatepass/domains/visit.yaml:90` and `:172`).

**Why this prevents exact lowering.** The proposed rule supplies slots for missing required fields
but says optional undetermined fields "stay absent"
(`docs/design/ess-evolution/entity-runtime-lowering.md:363`). That is a projector decision where the
source delegates a host decision. It also conflicts with the design's own refusal for a host-owned
output whose presence is conditional (`docs/design/ess-evolution/entity-runtime-lowering.md:429`):
ER cannot reference a missing bound slot without a template error, and `null` is not absence. Billing
and gatepass therefore cannot both be claimed complete under the stated rule merely because their
current sample realizations happen to choose absence.

**Bounded correction.** Apply one rule to every undetermined optional entity, event, and response
output. If current ER cannot represent the host's per-invocation present/absent decision, return
`OptionalBoundOutputUnsupported` with the exact field path and report the affected billing/gatepass
acceptance as blocked. Otherwise specify a concrete typed conditional-presence mechanism and its
kernel compatibility before claiming the fixtures lower. Do not silently specialize the source to
absence.

### 3. The validated companion model contradicts the public Rust contract

**Requirement.** The three frozen files jointly define one implementable typed interface. New nouns
in the companion ESS model must retain the closed alternatives and payload fields that the design
requires, rather than introduce a second, lossy representation.

**Evidence.** The Rust contract defines `BindingRequirement` as a closed enum whose variants have
different payloads: for example `IdentitySupplied` carries `value` and `observed_at`,
`ScaleDeclaration` carries `entity`, `name`, and `values`, and `RelationExistence` carries a typed
`RelationCoordinate` (`docs/design/ess-evolution/entity-runtime-lowering.md:137`). The validated
model instead defines every `BindingRequirement` as one struct requiring `command`, `outcome`,
`kind`, and an untyped string `target`
(`docs/design/models/entity-runtime-lowering/domains/contract.yaml:89`). It cannot encode those three
examples without inventing meaningless fields and drops their required data. The same drift recurs
where `CommandBinding.entrypoint` and `.instance` retain only kind enums and lose the operation name,
input field, logical identity, and observed event coordinate (`domains/contract.yaml:113` versus the
design at `:58`), while the model's `LoweredService` omits definition bodies and
`source_capabilities` (`domains/contract.yaml:129` versus the design at `:40`).

**Why this prevents implementation.** Both artifacts use the same public noun names but specify
incompatible shapes. An implementor or `/4` consumer cannot tell whether the validated model is an
authority, a persistence projection, or a non-normative inventory; following it loses the exact
coordinates and obligations the Rust interface says are load-bearing.

**Bounded correction.** Make the companion model exact for every new interface noun, using closed
unions with variant-specific payload structs and typed coordinates, or explicitly remove/rename the
lossy duplicate nouns and state that the Rust interface alone is normative. Keep existing ER
definition and ESS resolved types referenced by their established typed homes as the model's opening
comment intends.

## External-effect boundary established

The billing notification binding does not require a stateless ER kernel feature in order for ER to
own the invoice decisions. `invoice-service` publishes `InvoiceCreated`, and the selected
`ServiceIr` includes bindings reacting to a selected publication while the exact
`ResolvedBinding` remains reachable through the bound `EssIr`
(`crates/specify/ess-service-contract/src/lib.rs:348` and
`crates/specify/ess-compiler/src/ir.rs:1600`). The real linkage keeps `SendEmail` as a hand-written
provider obligation and keeps escalation outside entity decision execution
(`examples/billing-realization/src/linker.rs:421` and `examples/billing-realization/src/email.rs:1`).
The lowerer's preserved `source_capabilities`, together with recompilation from the exact source
digest, can retain that seam. Directly lowering the separate `email-service` still correctly reaches
`StatelessCommandUnsupported`; whole-billing acceptance remains open, and no singleton entity or
stateless platform follows merely from this binding.

## Examination boundary

Read the fixed design and validated model, the complete public interface and ten-dimension mapping,
actual billing/gatepass specifications and realizations, extraction selection and binding types, and
the accepted ER definition, registry, validation, identity, relation, outcome, event, and runtime
interfaces. Entity closure, exact type/null rules, nominal invariants, relations and registry-wide
validation, identity/address separation, ordered duplicate events, bound response/generated values,
version/scales admission, deterministic digests, old-format preservation, and the direct stateless
refusal yielded no additional finite finding. No Cargo command, code/test edit, planning write,
publication, or external integration invocation was performed. The repository-required Connectors
use was limited to a local readiness diagnostic. `git diff --stat` remained empty; the three
assigned untracked proposal files remained at their fixed hashes. The sole outside-worktree write
was `~/beyond10x/.ess-evolution/waves/0009-service-convergence/er-lowering-design-review-1.md`.

```findings
- file: docs/design/ess-evolution/entity-runtime-lowering.md
  line: 467
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The load-then-decide /4 flow cannot preserve PayInvoice's reachable input-guarded rejection for an unknown identity because ER requires and validates an EntityInstance before selecting an outcome, so the design must type a pre-load selection boundary or record durable fixture acceptance as blocked.
- file: docs/design/ess-evolution/entity-runtime-lowering.md
  line: 363
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Omitted optional creation fields are source-delegated host choices but the design silently fixes them absent, including billing note/issued_at and gatepass badge, so it must preserve conditional presence or refuse those exact fields instead of claiming complete fixture lowering.
- file: docs/design/models/entity-runtime-lowering/domains/contract.yaml
  line: 89
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The validated companion model gives BindingRequirement, CommandBinding, and LoweredService lossy shapes incompatible with the closed Rust interface, so the model must become exact or stop presenting duplicate public contract nouns.
```
