# Recovery for finite deployment execution

## C01. Status and binding provenance

This binding consolidates the accepted finite recovery contract for `obligation:review-execution-recovery-implementation`, owed by F11 and the implemented design story. The coordinator adopts the previously reviewed contract and exact named model as preparation for its implementation story. Runtime implementation, deployment authority, execution evidence and obligation discharge remain owed.

Source subject: ESS `a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3`. The motivating finding is [F11](../reviews/2026-09-05-architecture-review.md), lines 394–422. “Current source” below refers to this pinned commit. Normative requirements describe the intended recovery implementation, whose executable proof remains owed.

Consolidation precedence is the original design, authority base, addendum-v2, then addendum-v3. Later inputs replace only their specified clauses. The coordinator's accepted shared-production-code offline-driver qualification governs the construction and limits of executable evidence. This text introduces no further authority, format, model field, test exemption or supported profile.

| Retained contract input | SHA256 |
|---|---|
| Original design | `679482baa7e9584c78c715fed55d201f89b33d95e6488f7d9b318e5fb15e2783` |
| Authority base, `report.md` | `1b0d0cbc32eef5957cb2ccb6d3435a46b3e388e03cd8d528727b28f8e8e93a0e` |
| `addendum-v2.md` | `fac0c4de5bcd6483d935a09324193abdf33116eadb4f373943a62f5b4ae5a87f` |
| `addendum-v3.md` | `845cac69786aecc40de6b4a8dc324c32bc147610087090760bd4f880ecfe9248` |
| Offline-driver coordinator decision | `291717217326896e307f7481566186e4c6e513a04c324f811433042a7ed472a9` |
| Accepted scope adjustment | `9fc6f284993f03afda945e192c637630c48d7dca96152374473426c408e721f0` |

The authority inputs are retained under `target/review-boundaries-14/preparation/execution-recovery-authority-candidate/`; the decision paths are `implementation-scope/coordinator-decision.json` and `implementation-scope/integrated-refresh/root-readback.json`. The adjacent publication-draft clause map and readback bind complete input bytes and each replacement. Historical “unaccepted candidate” and blanket “UNMAPPED” status sentences are not competing current requirements; the concrete contract below replaces them. Declaration validation is structural evidence, not runtime enforcement.

## C02. Finite execution and distinct claims

A finite invocation may compare desired documents, attempt a bounded ordered set of external operations and report what it can establish. It must not infer applied state from equality with a caller's previous desired document. Once an external operation may have started, a missing success record or a failed child process does not establish that nothing changed.

Recovery must preserve uncertainty at the affected release, retain known earlier outcomes, and stop later mutations when an operation or its durable recording is unresolved. A later explicit invocation observes and decides again under the same bound target/ownership policy; it does not blindly resume a remembered list of commands. This is a finite workflow, with no background watch, eventual-convergence promise, automatic endless retry or continuous drift repair. Each selected apply or removal is attempted at most once in one invocation. Helm's own behavior within that call is a separate external contract.

The supported recovery contract is conservative: when identity, authority, ownership or the required observation cannot be established, refuse state-dependent mutation and name the unresolved claim. Refusal leaves the state **unknown**, not absent, rolled back or successfully reconciled. An exact caller-authorized repair pre-state is admitted only under the ownership and observation rules below; no generic force switch upgrades uncertainty into proof.

Per-release `--atomic` does not coordinate multiple releases. ESS issues no compensating calls for previously successful releases when a later call fails. Even the outcome of the failed release remains unproven by ESS: a nonzero Helm exit is not independent proof that a rollback completed, and a process exit of zero is not a perpetual state guarantee.

These terms describe behavior and evidence obligations. They are **not** a proposed serialized enum, receipt schema, field list or lifecycle model.

| Claim | Meaning and limit |
|---|---|
| Desired | Validated intent supplied for this invocation, including exact artifact references/digests and release bindings. It says what is requested, not what exists. A prior desired document is a comparison input only. |
| Observed | A bounded statement acquired from the selected external authority at a particular observation point. It may establish present, absent, matching or differing state only for the part actually observed. The admissible source, content comparison and freshness rules are specified in C05–C09. Observation cannot invent a historical execution attribution. |
| Applied | A historical claim that a particular authorized operation against the bound target completed, supported by admitted durable execution evidence. It is not equivalent to “currently matches desired.” External process success without durable evidence is only an acknowledgement available to the running process. |
| Unknown | The required claim cannot be established: missing, corrupt, stale, conflicting or wrongly scoped evidence; indeterminate process outcome; unresolved identity/ownership; or failed/insufficient observation. Unknown is neither success nor a negative assertion about the target. |

Two documents with the same desired digest do not establish either of the latter two positive claims. A chart in cache proves neither that Helm ran nor which cluster it affected. A valid release manifest proves no deployment attempt. A successful readback may establish that the target now matches, while the identity of the earlier operation remains unknown; do not manufacture an applied-history record to make those facts agree.

### Ordered workflow

1. Admit the complete desired document and any comparison/baseline input. Preserve JSON/YAML unknown-field, duplicate-key and intrinsic-invariant refusals. Resolve the intended target, ownership, baseline authority and concurrency preconditions before a state-dependent mutation. `--allow-removals` authorizes a reviewed removal set; it does not prove ownership or target identity.
2. For dry-run, report an **unverified comparison preview** from admitted local inputs and stop. No external observation, ORAS/Helm/Kubernetes call, cache population or execution-evidence write occurs. A preview cannot establish a live no-op, successful application or absence. The existing local-preview argument behavior is retained.
3. For execution, obtain the observations this contract requires. Equality of desired documents never substitutes for this step. A no-op requires fresh, correctly scoped evidence of the relevant desired state, not an old desired file or an old success record alone. Observation failure leaves the needed state unknown and refuses mutation dependent on it.
4. Before launching each mutation, require that its identity/authority and durable tracking can be established. Refuse before launch when that prerequisite is unavailable. Acquire/validate the chart and prepare values without treating either as target progress. Iterate desired releases in canonical rollout order, and authorized retirements afterward in reverse baseline rollout order.
5. After each external call, distinguish launch refusal, acknowledgement, definite observed outcome and uncertainty. Persist the permitted outcome before later mutations. If a child may have run but the result or its durable recording is uncertain, stop; do not issue later applies, removals or compensating rollback calls. A later local error cannot erase an earlier external effect.
6. A new explicit recovery invocation re-admits desired inputs and durable evidence, re-establishes target/ownership/concurrency authority and obtains required observations. It may acknowledge an observed match/absence without replaying the uncertain mutation, but must retain unknown historical attribution. It may execute a new authorized repair only after the observation/ownership policy establishes that action as safe; no “retry means repeat every command” rule is admitted.
7. A successful completion claim is limited to the bounded work whose required outcomes and durable recording have been established. It is not a claim that the environment will stay unchanged. An unresolved release, recording error or withheld removal prevents an overall recovered/applied-success claim. State the completed prefix and unresolved remainder rather than asserting global rollback or global success.

No all-release rollback is selected. Previously settled releases remain settled historical outcomes when a later release fails. Explicit subsequent repair can change them through a newly authorized plan; this design does not promise compensation, exactly-once external effects, or a global transaction across Helm and local storage.

## C03. Grounded current owners and evidence boundary

In the following source inventory, `main.rs` and `oci_cache.rs` are in `crates/edge/ess-cli/src/`; deployment `environment.rs` and `runtime.rs` are in `crates/generate/ess-deployment/src/`; CLI test paths are under `crates/edge/ess-cli/tests/`. These current owners establish existing behavior and controls, not recovery enforcement.

| Owner | Existing behavior and required change |
|---|---|
| `main.rs:882–901,1637–1749` | Currently interprets `--current` as previously applied, selects by IR equality, then invokes Helm sequentially. Replace this execution branch with admitted recovery while preserving whole-input validation, early removal guarding, local dry-run and both command spellings. |
| `main.rs:3150–3158,3184–3253` | Process status and temporary files currently provide neither durable execution evidence nor an authoritative target observation. Keep this IO at the edge; do not add execution to projection libraries. |
| `oci_cache.rs:117–218,283–335,490–578` | Already proves the requested original manifest and each descriptor’s original bytes. Recovery consumes that proof and separately admits the generated chart. Cache presence cannot establish application. |
| `environment.rs:167–283,590–728,732–941` | Owns strict deployment validation, canonical identity and the exact five-file chart projection. Reuse it; the recovery contract does not require changing deployment bytes or projection semantics. |
| `runtime.rs:233–268,395–477` | Owns validated runtime IR and its canonical digest. Read the caller-supplied digest-bound runtime from the admitted runtime directory. Its digest is not an execution receipt. |
| `ess-kubernetes/src/lib.rs:13–14,37–48,61–110`; `ess-kubernetes/src/topology.rs:17–110,120–154,165–214` | Legacy cluster scans now stop on a failed request, with no narrower fallback. The separate `scan_namespace` API checks explicit namespace/object scope, selects referenced nodes and emits the deliberately reduced `infra-observation/2` topology profile. Both still use PATH-selected `kubectl`. Neither supplies this contract's pinned target/principal, Helm ownership or complete live projection evidence. Keep the narrow recovery adapter; scanner output is not authoritative recovery evidence. |
| `crates/edge/ess-cli/src/load.rs:130–166`; `crates/infra/infra-compiler/src/read.rs:62–252`; `docs/design/review-observation-completeness.md:8–50` | The infrastructure loader now admits observation/1 and /2 or persisted infra-ir/1 and /2, preserving qualified coverage through the existing observation/compiler readers. Namespace topology omits annotations, literal environment entries, probes and unmodeled content; its collected membership and model digest do not establish the complete recovery snapshot or authenticated absence predicates in C05–C09. This loading API is read-only reuse evidence, not an added recovery input or write reservation. |
| `persisted_delivery.rs:107–202,204–303` | Retain duplicate-key, whole-plan, order and validation-before-executor controls. Its successful fake currently supplies opaque chart bytes and no independent target. |
| `cache_origin.rs:21–41,250–314,1175–1232`; `cache_origin_adversary_pass1.rs:199–251` | Existing Helm cache controls invoke reconcile directly and assert exact consumed bytes. They must continue testing the cache boundary after authority admission changes. An earlier authority refusal must not make their intended cache assertion vacuous. |

The current cache admits original manifest bytes, the closed descriptor/layer profile and exact referenced blob bytes (`oci_cache.rs:117–218,239–319`). It stages a framed proof, reads it back and publishes without replacement (`378–405`); manifest/blob acquisition and warm selection are at `490–578`. A verified private chart snapshot and values precede the existing Helm apply at `main.rs:3193–3226`. Cache-origin proof is distinct from authenticated release attestations, deployment authority, target observation and execution history. The cache's `flush`/hard-link protocol does not establish a synchronized recovery journal.

Service, namespace, release and digest coordinates remain inputs rather than physical identity or execution authority. The existing strict deployment/release readers own desired intent and release artifact references. Preserve their intrinsic validation and exact canonical bytes; no receipt field is appended to `ess-deployment/1` or `ess-release/1`. A valid conformance attachment is not verified deployment execution.

The existing transient-directory cleanup is best effort. Interruption may leave residue; a missing cleanup or surviving file cannot establish whether the target changed. Recovery retains the original duplicate-key, complete-plan, canonical-order, cache-consumption and CLI controls.

The support-check source row is owned by `crates/edge/ess-xtask/src/support.rs:502`, and its current rendered counterpart by `website/docs/status/where-this-stands.md:67`. The read-only check compares source claims; it never executes a deployment.

## C04. Supported profile and nine authority contracts

The finite profile is `SingleHostGeneratedHelm1`. Its caller preconditions are explicit:

1. **Actual execution requires a future caller-provided authority document.** The implementation must never manufacture that authorization from desired input, observations, journal records or successful tests.
2. **The supported mutation scope is one trusted execution host and exclusively managed release addresses.** Other writers of covered fields must follow the same host exclusion protocol. A file protocol cannot exclude an administrator, another workstation or an uncooperative controller.
3. **The first recovery profile admits verified ESS-generated charts, not arbitrary Helm charts.** Its observation claim covers the complete direct resource inventory and specified live object projection, not arbitrary workload effects or eventual readiness.
4. **Interrupted execution can require an administrative quiescence decision.** Automatic lock expiry, PID-based reclamation and blind retry are excluded. The positive restart cases in the matrix gain this explicit precondition when their predecessor retained an execution lock.

These qualifications resolve the original nine open contracts and preserve all R01–R29 families, including the administrative-input dimension on retained-claim restart cases.

| Concern | Binding contract |
|---|---|
| Invocation identity | `(store_epoch, nonce)`, both UUIDs. The caller provisions the store epoch. The CLI obtains a fresh nonce from the host OS random source and reserves an immutable invocation directory with exclusive creation; collisions retry at most 16 times, then refuse. The retained reservation prevents reuse. A retry is a new invocation with an optional reference to its predecessor. Missing/corrupt store identity refuses; the CLI never silently initializes a replacement store after loss. |
| Physical target and principal | Caller authority pins the API server URL, trusted CA digest, an identity namespace’s UID, every release namespace’s UID, and one ServiceAccount namespace/name/UID. Context names are permitted aliases. Resolve the selected kubeconfig, reject insecure TLS, impersonation and executable credential plugins, then authenticate readback through Kubernetes identity/namespace queries and a self-subject identity response. Helm and observer use that same root-controlled kubeconfig/context. These are explicit deployment requirements; a context string is not physical identity. |
| Baseline admission | Authority pins the exact canonical desired digest and optional canonical baseline digest. `--current` remains desired intent. Its presence/digest must match the authority, including the complete retirement set. Each release’s baseline projection and incarnation are declared by the caller. Neither an old journal nor matching environment names can nominate a baseline. |
| Ownership and cardinality | Each service has exactly one physical `(target, namespace UID, release name)` address; each address belongs to exactly one authority/environment/service and one active incarnation. No cross-environment sharing or address aliases are supported. Helm’s description carries the exact marker `ess-recovery/1:<authority UUID>:<incarnation UUID>` supplied on every managed apply. The marker is useful only within the declared writer/credential trust boundary. Foreign or unmarked existing releases refuse; there is no automatic adoption. |
| Authoritative observation | Fresh authenticated Kubernetes reads cover every expected direct object, plus Helm release metadata and stored manifest membership. Missing objects require authenticated API absence, not Helm error text. Compare every authored rendered field and the full caller-pinned live projection described below. Observations belong to one invocation/read interval; journal readback never refreshes them. |
| Conflicting executors | Atomically publish one durable lock claim per physical target using complete staged bytes, `sync_all`, a no-replacement hard link and directory synchronization. Hold it for the invocation. Competing executors cannot publish another claim. There is no timeout or automatic stealing. Ordinary safe completion removes its own claim; uncertainty retains it. |
| Journal durability and trust | A preprovisioned private local store, one immutable canonical JSON file per journal entry, exact sequence and predecessor digest, file synchronization followed by no-replacement publication and directory synchronization. Durable `Prepared` precedes each mutation. Required outcome records precede subsequent mutations; final completion precedes exit zero. This proves protocol completion under the declared filesystem/host trust assumptions, not independent execution authority. |
| Removal and retry | Preserve the early removal flag guard and reverse baseline order. Before uninstall, require matching ownership/incarnation, exact approved manifest inventory and a fresh permitted pre-state. Run uninstall once, then independently read absence. Finalizers or retained direct objects prevent an absence claim. Retry with fresh complete absence issues no uninstall. A different incarnation always refuses. |
| Completion and compatibility | Keep existing desired/release formats unchanged. Add separate versioned authority/registry/store/lock/journal contracts. Actual `reconcile` without an admitted authority refuses; dry-run retains its existing local-only syntax and behavior. Keep text output and binary success/failure exits initially. Success requires durable completion of the declared bounded work; blocked observation output is explicitly incomplete recovery. |

## C05. Protected authority, registry and physical identity

The future caller provisions the authority and its trust inputs. The production layout is:

```text
/etc/ess/recovery/registry.json
/etc/ess/recovery/registry-history/<generation>.json
/etc/ess/recovery/authorities/<authority-id>/history/<revision>.json
/etc/ess/recovery/host-id
/etc/ess/recovery/runtime/<runtime-digest>.json
<configured-state-root>/store.json
<configured-state-root>/target.lock
<configured-state-root>/invocations/<nonce>/
```

Authority files, retained revisions, runtime inputs and kubeconfig use the protected administrative-file admission rules. The trusted control account is independent of the non-root executor; the registry contract requires root-controlled authority. Readers reject symlinks, unexpected ownership, writable parent components, invalid paths, duplicate keys and unknown fields. Retained authority revisions are immutable, and old journal context must match its corresponding retained authoritative revision.

`registry.json` contains the complete active `Authority` values, sorted by authority UUID. Historical authority files remain immutable audit inputs. Per-authority `current.json` files cease to select execution authority; retaining them for human convenience must not create a second admission path.

The trusted control account publishes a fully checked snapshot atomically, with durable publication, during an execution-disabled administrative window. There is no registration controller or automatic authority writer. An implementation must never fabricate entries or automatically approve an authority because its fields pass validation.

For the smallest single-host profile, the authoritative snapshot covers all enrolled authorities on that host. Every active entry must name the same independently provisioned host identity and executor UID. Supporting a registry of remotely hosted executors would require additional coordination and is outside this profile.

Before choosing an authority, the executor must:

1. Read and validate the complete root-controlled snapshot: protected path components, regular files, no symlinks, closed fields, duplicate-key rejection and canonical bytes. Match the snapshot to its immutable generation archive.
2. Reject duplicate authority IDs, multiple active revisions of one authority, or invalid entries anywhere in the snapshot. Each embedded authority must equal its retained immutable authority revision.
3. Build the indexes below over **every active entry**, including every release’s baseline, desired and permitted repair inventory.
4. Inspect every referenced local store and kubeconfig needed by those indexes. An unreadable or invalid active entry prevents admission; it is not skipped.
5. Only after the full scan succeeds, select the requested authority and authenticate its live target/principal bindings.

| Index/check | Required admitted relationship |
|---|---|
| Host | Every active `host_id` equals the independently provisioned local host ID; every `executor_uid` equals the executing non-root UID. All entries use the same admitted Helm binary contract. |
| Physical cluster | Fix `TargetPin.identity_namespace.name` to `kube-system`. Its authenticated UID is the physical-cluster key. One such UID maps to exactly one admitted canonical API endpoint and CA digest. |
| Endpoint | One canonical API endpoint cannot map to conflicting identity-namespace UIDs or CA digests. A second endpoint for the same cluster UID is rejected as an unsupported endpoint alias. |
| Namespace | Within a physical cluster, namespace name and namespace UID form a one-to-one mapping across identity and release namespace pins. |
| Target exclusion | One physical cluster maps to exactly one `(host_id, executor_uid, store_epoch, canonical state_root)` and therefore one `target.lock` namespace. Different authorities on that cluster share it. |
| Store | One store epoch/root cannot map to different physical clusters. Reject equal or nested distinct store roots, symlink aliases and different path strings resolving to the same directory/header file identity. |
| Environment | `(physical cluster, environment)` maps to one active authority UUID. |
| Service | Within an authority/environment, a service has one release permit. Duplicate permits are rejected even when their contents agree. |
| Release address | `(physical cluster, namespace UID, release_name)` maps to one authority/environment/service/incarnation. Baseline and desired projections within that one permit do not constitute competing owners. |
| Direct object address | Each address in the union of baseline, desired and `repair_from` inventories maps to one release permit. Two different permits cannot claim the same `(physical cluster, namespace UID, kind, name)`. |
| Context alias | Every admitted context resolves through its protected kubeconfig to the authority’s exact endpoint, TLS trust and principal configuration. A reused kubeconfig/context binding cannot point to conflicting targets or principals. |

For endpoint indexing, admit only canonical HTTPS API roots: no user information, query, fragment or proxy path; normalize hostname case and the default HTTPS port. Resolve filesystem paths through non-symlink components and compare physical file identities as well as canonical path strings. A lexical-path comparison alone is insufficient for store or kubeconfig aliases.

Checking the entire local registry is deliberately strict. Two unrelated active entries cannot silently name different stores for one target, even if the selected authority’s own fields appear valid.

Capture the admitted snapshot’s generation and whole-byte digest in both `Opened` context and `LockClaim`. Re-read the authoritative snapshot after claim acquisition and immediately before each mutation; changed generation or bytes stops launch rather than adopting a new policy mid-invocation. Reject a generation older than retained execution evidence, or different bytes for an already retained generation.

These checks prevent collisions **within the admitted registry**. Completeness of that registry, uniqueness of the enrolled host identity, absence of cloned cluster identities, and exclusion of writers outside this mechanism remain trusted caller assertions. Re-reading a file cannot fence a malicious administrator or prove that an omitted workstation does not exist.

The authority explicitly attests:

- The enrolled host identity is unique and has not been cloned with its store.
- All writers of the covered desired fields and Helm release state obey the target lock protocol.
- The executor credential is not available to another mutation path.
- The Kubernetes API and TLS trust configuration are trusted.
- The local filesystem and storage honor synchronization and atomic publication.
- The approved expected object projections correspond to the intended deployed resources.

These are **trusted administrative assertions**, not properties the CLI can discover completely. The CLI checks their concrete bindings and refuses unsupported profiles; it must not describe these assertions as independently verified infrastructure policy.

Manual changes **between invocations** remain supported observations. Administrators must use an exclusive maintenance window or the same target exclusion protocol. Controllers that independently modify covered specs, such as an HPA changing a covered replica count, are outside this profile.

## C06. Exact Helm artifact and closed executable contract

`HostPolicy.helm` is required and contains:

- An absolute binary path.
- The SHA-256 digest of the entire executable.
- Its exact version string.
- The closed protocol `Helm3Recovery1`.

For this protocol, `version` must be a canonical stable Helm 3 version of the form `v3.M.P`, with decimal components and no prerelease/build suffix. There is no range, `latest` or PATH fallback. The future caller must supply actual reviewed bytes and an exact version; publication of this contract admits **no existing executable** and does not claim that every Helm 3 release satisfies the contract.

Require the installation path:

```text
/opt/ess/recovery-tools/helm/<sha256-hex>/helm
```

The binary and parent chain must be controlled by the trusted control account. Reject wrappers, symlinks, nonregular files, executor/group/other writability and setuid/setgid executables. The filename digest, configured digest and complete binary hash must agree.

The same absolute artifact performs all permitted Helm operations. Recheck its identity and bytes before mutation. Stability between checking and execution relies on the trusted account’s immutable-installation/maintenance-window contract; it is not established by hashing alone. No inherited descriptor or unsafe `pre_exec` mechanism is proposed.

The closed capability contract is:

| Operation | Required admitted capability and restriction |
|---|---|
| Version | `version --template '{{.Version}}'` returns the configured version, allowing only its final newline. |
| Render | `template` renders the admitted private chart with the exact private values and namespace; no dependency update, network chart resolution or added rendering facility. |
| Observe Helm state | `status --output json`, `get manifest --revision …` and `get hooks --revision …` support the pinned namespace/kubeconfig/context. Decode complete outputs, bind reads to the observed revision and reject any hooks or inconsistent release state. |
| Apply | `upgrade --install --atomic --wait --timeout … --description … --no-hooks --skip-crds`, using explicit namespace, kubeconfig, context and private values. The exact ownership marker must be preserved. |
| Remove | `uninstall --no-hooks --wait --timeout …`, using the explicit namespace/kubeconfig/context. Do not retain release history or convert an arbitrary error into absence. |
| Storage/process semantics | Helm 3 Secret-backed release storage and the admitted output shapes; the restricted successful invocation finishes its own mutation work before returning success. This last property is a trusted executable contract, not a consequence of `spawn` or `wait`. |

Do not pass `--create-namespace`: the admitted namespace must already exist with its pinned UID. Do not pass `--keep-history` on removal, because this contract's absence predicate includes Helm release storage. Do not use `--ignore-not-found` to bypass authoritative absence observations.

The child environment must be constructed explicitly. Clear inherited Helm, kubeconfig, proxy and executable-injection settings; set `HELM_DRIVER=secret`, private Helm configuration/cache/data locations and an empty controlled plugin directory. Supply kubeconfig/context/namespace explicitly. Credential execution/auth-provider plugins, configured proxies, arbitrary additional flags and post-renderers remain unsupported. Trusted operating-system and loader behavior remains an explicit platform assumption; a binary hash does not authenticate every platform dependency.

After hashing an already caller-admitted artifact, the future executor performs bounded read-only compatibility probes: exact version and the operation-specific help outputs. Limits are five seconds and 64 KiB per output stream per probe. Parse required flag names structurally. Missing flags, mismatched version, unsupported output or failed probes refuse before target mutation.

Those probes detect incompatibility; they do not create the trust decision. The independently controlled authority’s exact digest/version/protocol declaration asserts that this reviewed artifact satisfies the protocol’s semantics, including successful-call completion. A binary that prints suitable help but lacks those semantics is not admitted.

All started nonzero, timed-out or uncertain calls retain the previous conservative handling. This admission contract does not establish descendant quiescence after interruption.

## C07. Chart admission, complete projections and operation predicates

The current pure projector is unusually helpful: `project_helm` emits exactly five files (`environment.rs:637–728`), and its templates produce Deployments, StatefulSets and Services (`732–940`).

For each desired or baseline release, the caller supplies a digest-bound `ess-runtime-ir/1` document, chart name and version. The profile:

1. Admits that runtime with the existing `RuntimeIr` reader and intrinsic validation (`runtime.rs:230–268,424–476`).
2. Recomputes the exact `project_helm` five-file projection.
3. Reads the already verified chart payload through a bounded Rust gzip/TAR reader. It admits only regular members under one chart root, with no links, duplicate names, traversal, extra members or differing content. Each member must equal the corresponding projection file.
4. Rejects template delimiters introduced through runtime string data. The admitted template actions must come from the fixed projector, not injected runtime text.
5. Renders with the verified private chart snapshot and the same merged values used for apply. Parse the complete rendered document stream and reject duplicate addresses, unsupported kinds, hooks, `generateName`, keep policies, extra documents or cross-namespace objects.
6. Require the rendered inventory to equal the authority's complete inventory for the particular projection being admitted: baseline against baseline, desired against desired. A baseline-only retirement has no desired projection and remains admissible.
7. For an existing release, require its stored manifest inventory to equal the admitted baseline or desired inventory, and require no hooks. A manifest mismatch or unreadable release state refuses.
8. On live readback, compare every authored rendered field recursively. Also require each complete live projection digest to match its caller-approved fingerprint. Thus a caller fingerprint cannot silently override a differing authored image, selector, replica count or secret reference.

The live projection is precisely:

- `apiVersion`, `kind`;
- metadata name, namespace, labels, annotations, owner references and finalizers, with missing collections normalized to empty;
- all other top-level object content except `status`;
- excluding server bookkeeping metadata such as UID, resource version, generation, timestamps and managed fields.

UID and resource version are recorded separately in observations. Object maps use sorted keys; arrays preserve order. Server-defaulted spec fields are included in the fingerprint. The caller must provide the intended post-defaulting fingerprints; the CLI never learns a newly acceptable fingerprint by blessing whatever it finds.

This is intentionally restrictive. It rejects third-party/different chart templates, dependencies, CRDs, hooks, post-renderers, executable credential plugins, unsupported resource kinds and incomplete inventories. Use `--no-hooks`; skip CRD installation defensively. Unsupported tool output/capabilities refuse.

The claim covers **direct chart resources and their declared object content**. It does not prove Pod behavior, readiness over time, application side effects, Secret contents, PVC/PV cleanup, or convergence of controller-created descendants. Generated StatefulSet volume-claim declarations are compared; retained PVCs are not described as removed. This is the supported observation/removal scope; it makes no whole-cluster equivalence claim.

An apply acknowledgement still cannot substitute for this live comparison.

### Projection-specific operation predicates

- A baseline projection is rendered from its baseline runtime/chart/values and compared with its
  complete baseline authority inventory.
- A desired projection is rendered from its desired runtime/chart/values and compared with its
  complete desired authority inventory.
- A baseline-only retirement has no desired projection to render or compare. That absence is
  permitted by ReleasePermit and must not prevent baseline admission or a removal observation.

Every observation covers the union of baseline and desired direct-object addresses for that
permit. No address is silently omitted when inventories differ. The existing authored-field and
caller-approved complete live-projection comparisons apply to the projection being compared.

The finite operation predicates are:

1. Desired already matches: authenticated Helm ownership/incarnation and stored manifest match the
   desired projection, every desired object matches, and every baseline-only object is absent.
   Record a fresh observation and skip mutation; do not fabricate an earlier application fact.
2. Apply from baseline: Helm ownership/incarnation and stored manifest match the baseline, every
   baseline object matches, and desired-only object addresses are absent. A differing observed
   pre-state requires the separately admitted exact repair_from decision; it cannot waive foreign
   ownership/incarnation or competing object ownership. First creation still requires may_create,
   complete release/object absence and the original retained-history checks.
3. After acknowledged apply: require the complete desired predicate in item 1 before a later
   mutation. Old baseline-only objects left behind prevent claiming the desired projection holds.
4. Retirement: admit the baseline projection and matching retained target/ownership history.
   Fresh absence of both Helm release storage and every baseline direct object produces observed
   already absent with zero uninstall calls. Otherwise require the admitted baseline pre-state or
   a separately authorized exact repair_from pre-state, plus matching ownership/incarnation, before
   one uninstall. Partial/drifting state without that authority remains refused.
5. After acknowledged retirement: require authenticated absence of Helm release storage and every
   baseline direct object. Finalizers or retained direct objects prevent the absence claim. The
   previously declared exclusion of controller-created descendants and retained PVC/PV cleanup
   remains explicit; this does not turn direct-object absence into a whole-cluster claim.

A failed/unknown observation cannot satisfy any predicate. The early allow-removals guard,
canonical apply order, reverse baseline removal order, uncertainty and retained-claim/quiescence
rules remain unchanged. R23–R26 must include baseline-only permits; upgrades must include changed
baseline/desired object inventories and foreign occupants at newly desired addresses.

## C08. Durable exclusion, interruption and caller quiescence

The durable target lock avoids the inherited-file-descriptor problem completely:

- The lock claim exists independently of the parent process.
- It is fully written and synchronized before publication.
- Parent death does not remove it.
- The CLI does not infer quiescence from PID disappearance, process exit timestamps, lock age, an empty journal tail, `Child::wait`, or a lease timeout.
- Timeout handling may kill and reap the owned direct child; it does not claim to have terminated every descendant or drained every already-issued API request.
- Any started call with nonzero exit, timeout, lost acknowledgement or interrupted outcome recording retains the claim and stops the invocation.

For ordinary successful completion, the profile trusts the admitted stock Helm executable and restricted invocation to complete its bounded calls before reporting success. If that executable contract cannot be admitted, mutation is unsupported. That is an explicit trusted executable assumption, not a consequence of `spawn`.

The next invocation may read and report current observations while an old claim remains, but it cannot mutate or report completed recovery. It reports the old invocation and the quiescence requirement.

The future caller’s quiescence procedure must independently establish that the previous executor, its descendants and outstanding mutation requests can issue no further writes. During an execution-disabled administrative window, the caller archives the exact retained lock claim, installs a new authority revision containing a decision naming that claim, removes the old lock, then re-enables execution. **The CLI never reclaims another invocation’s claim.** This avoids a stale-observer/unlink race between competing reclaimers.

The decision is authoritative because it comes from the independently controlled authority source. A copied journal record cannot generate it. If the caller cannot establish quiescence, the supported outcome is continued mutation refusal.

This profile requires an administrative precondition after uncertain interruption. No cgroup, supervisor, unsafe `pre_exec`, inherited descriptor, or perpetual controller is selected. The filesystem operations also avoid relying on a newer standard-library locking API than the repository’s declared Rust 1.85 minimum (`Cargo.toml:42,81–83`).

`Q` means that a retained predecessor lock requires the independently established caller quiescence procedure before mutation or completed recovery. Without Q, available live readings remain explicitly observational, historical attribution remains unknown where applicable, and mutation stays blocked. The CLI cannot generate Q from a journal, successful spawn, direct-child exit or passage of time.

## C09. Preparation, observation and durable Prepared ordering

For each selected operation, finish artifact and local preparation before beginning its mutation-authorizing observation. Artifact and local preparation includes verified cache/chart admission, private chart and values materialization, restricted rendering, complete inventory checks and command construction. It does **not** mean publishing the journal’s `Prepared` fact.

Under the invocation’s exclusive target claim, obtain a fresh, complete `Before` observation after that preparation. Persist its `Observed` journal entry durably. Only then may a mutation decision publish `Prepared`, whose `observation_sequence` names that exact earlier `Observed` entry. `Prepared` must itself be durable before launching the mutating process.

Immediately before launch, require the observation’s invocation-local monotonic age, measured from `started_ms`, to be at most 30,000 milliseconds. This budget includes observation acquisition, observation publication, `Prepared` publication and final precondition checks. Expiry or changed preparation bytes refuses that operation; it does not start an automatic observation/retry loop.

The exact sequence is:

```text
artifact/local preparation completes
    → Before observation starts and finishes
    → Observed(sequence = s) becomes durable
    → mutation predicate is satisfied
    → Prepared(observation_sequence = s, sequence = p > s) becomes durable
    → final freshness, registry, binary, target/principal and claim checks
    → at most one mutating launch
    → durable ProcessOutcome
    → fresh, durable After observation
    → any later operation that mutates
```

Additional reader/executor obligations:

- `Prepared.observation_sequence` must resolve within the same invocation to an earlier `Observed` fact with the same operation index and phase `Before`. An `After` observation or predecessor invocation’s observation cannot authorize launch.
- Require `started_ms <= finished_ms <= launch_check_ms`, with all three values measured by the same invocation’s monotonic clock. Persisted timestamps from another invocation are never fresh.
- The private chart, values, selected authority, admitted executable and constructed mutation arguments must remain those checked for that observation. Any change stops the operation.
- A fresh desired match or authenticated already-absent result is recorded as an observation and skips mutation. It does not require `Prepared`.
- If final checks fail after durable `Prepared` but before any mutating process starts, a durable `ProcessOutcome(NotLaunched)` may establish that fact before `Stopped`. `NotLaunched` covers both definite spawn refusal and definite prelaunch cancellation. A process that started cannot receive that disposition.
- Interruption after `Prepared` and before durable disposition remains indeterminate, including the interval before actual launch. Restart cannot reconstruct `NotLaunched` from an empty journal tail. The retained claim and independent quiescence requirement remain.
- Acknowledgement remains historical process evidence. Required post-acknowledgement observation must complete and become durable before later mutation.

## C10. Journal grammar, retention and durability

Canonical files use UTF-8 JSON, recursively sorted object keys, preserved array order, no duplicate keys or floating-point fields, and one trailing newline. Invocation records are:

```text
invocations/<nonce>/00000000000000000000.json
invocations/<nonce>/00000000000000000001.json
...
```

Each entry is fully serialized into a newly created private stage, synchronized, read back and checked, published without replacement, then its parent directory synchronized. Only after all required steps succeed is the entry durable under the stated storage model. Stages are never journal entries. Their cleanup is optional and cannot create authority.

No automatic history deletion is admitted. Retain reservations, journal entries, authority revisions and archived uncertain lock claims for the store epoch’s lifetime. Store retirement/reinitialization is an explicit caller administrative operation with a new epoch and authority, not an implicit retry path.

A prepared operation lacking admitted durable disposition remains indeterminate. Corrupt entries, gaps, unsupported formats, wrong target/store identity or inconsistent predecessor digests refuse and remain available for diagnosis. Hash chains detect inconsistent bytes; they are not protection against a malicious trusted account that can rewrite the entire store.

After each acknowledged call, persist the acknowledgement and required fresh readback before later mutation. Final `Completed` is the selected count, with full coverage derived and checked from the journal. A failed final publication prevents success even when all external calls acknowledged.

### Valid incomplete prefixes and closed journals

- A valid nonempty journal starts with exactly one Opened at sequence zero. Every published entry
  is a complete admitted canonical record, with contiguous sequence and the required predecessor
  digest. Per-operation ordering and identity rules apply to every prefix.
- A valid incomplete prefix has no terminal fact. Missing terminal is incompleteness, not by
  itself corruption. Preserve it, retain uncertainty for Prepared without a durable disposition,
  and require independent fresh observation and administrative quiescence where the lock remains.
- A closed journal contains exactly one final Stopped or Completed fact and no following entries.
  Completed additionally requires complete selected-operation accounting and its durable final
  publication. Stopped is a settled stopping decision, not evidence that every target effect is known.
- A newly reserved invocation directory with no Opened is an incomplete reservation, not a
  completed journal. It authorizes no mutation. If it has a retained target claim, the CLI does
  not reclaim that claim or infer quiescence merely from the empty directory.

Unpublished private stage files are not journal entries. A partial stage can remain diagnostic
residue without constructing a fact. A malformed or torn file at a published sequence name,
a sequence gap, invalid predecessor digest, wrong identity/scope, duplicated terminal or entry
after a terminal is corrupt/inconsistent evidence and follows R18 refusal. Quiescence does not
repair it or permit overwriting it. Valid interrupted prefixes instead follow R12/R15–R16/R25/R29
recovery handling; missing acknowledgement/finalization never invents a historical success.

Historical admission is independent of the optional retry_of annotation. Omitting that reference
must not bypass retained store, authority-revision, generation, ownership-history or journal
validation. The reader considers all relevant retained invocation reservations/histories in the
selected store; an explicitly named predecessor is additional context, not a selector that hides
other evidence. No automatic historical deletion or store reinitialization is introduced.

Required controls separately cover: no Opened, Opened-only, Observed-before-Prepared, durable
Prepared without disposition, acknowledged disposition without After observation, successful
per-operation records without Completed, valid Stopped/Completed closures, partial unpublished
stages, corrupt published entries, missing sequence members and entries after closure.

### Reservation and directory publication barriers

All fixed store directories, including invocations/, must already
be durably provisioned under the admitted store header and protected filesystem ownership model.
The CLI neither silently creates a replacement store nor treats a missing store as first use.

Before publishing a target claim or allowing any Prepared record for a new invocation:

1. Exclusively create invocations/<nonce>/ beneath the admitted parent.
2. Open and validate the new directory through the protected parent and synchronize the new
   directory itself, then synchronize invocations/ to durably publish its directory entry.
3. Only after both barriers succeed may the invocation reservation be admitted. Any failure
   stops before claim publication and target mutation; retained residue is never reused as a
   fresh successful reservation. A new retry receives a new invocation identity.

For every dynamically created directory used by the protocol, apply the same rule before relying
on any descendant: synchronize its initialization and the parent directory that publishes its
name. Preprovisioned ancestors must already satisfy their own provisioning/durability contract.
Synchronizing an entry file and its immediate directory alone is not the reservation barrier.
The existing complete-stage sync, no-replacement publication and destination-directory sync
remain required for each journal entry and lock claim. Lock removal on ordinary safe completion
also requires synchronizing the lock's parent before reporting that exclusion was released.

Fault controls must fail each directory-creation, new-directory sync and parent-publication sync
boundary independently, then exercise interruption and restart with retained filesystem/target
state. No target mutation may precede the complete reservation/lock/Prepared durability chain.
R12/R16/R29 retain every existing per-entry and finalization boundary as well. These guarantees
remain conditional on the declared filesystem/storage honoring synchronization and atomic
publication; no distributed transaction or power-loss experiment is claimed by this contract.

## C11. Exact named model and mandatory Rust reader rules

The declaration consists of exactly these retained files, published without byte changes:

| Intended path | Bytes | SHA256 |
|---|---:|---|
| `models/execution-recovery/system.yaml` | 75 | `c981b332cc9bdb08690c3f4f44f732138dc66bf725bcef930ce3ed4836825f28` |
| `models/execution-recovery/domains/execution.yaml` | 10578 | `e03ca97fc429ab20950c2c75e0c8567e6174cc4c93aca6ca9c6860338c27e294` |

All named types below have the namespace `recovery.execution`. The files declare 44 named types, with no entities, commands, components or lifecycle. The retained model-v2 replaces the superseded declaration blocks in the authority base and v2 prose.

| Contract group | Declared types |
|---|---|
| Values and identity | `Text`, `Digest`, `Index`, `InvocationId` |
| Formats/profile | `AuthorityFormat`, `StoreFormat`, `LockFormat`, `JournalFormat`, `RegistryFormat`, `Profile` |
| Host and target | `NamespacePin`, `TargetPin`, `PrincipalPin`, `HostPolicy` |
| Projection/ownership | `ChartSource`, `ObjectKind`, `ObjectAddress`, `ObjectFingerprint`, `ReleaseProjection`, `HelmIdentity`, `PresentObject`, `ObjectRead`, `ReleaseSnapshot`, `ReleasePermit` |
| Admission/exclusion | `LockClaim`, `QuiescenceStatement`, `QuiescenceDecision`, `Authority`, `StoreHeader`, `AuthorityRegistry`, `RegistryRef` |
| Invocation/evidence | `InvocationContext`, `ObservationPhase`, `Observation`, `Prepared`, `ProcessDisposition`, `ProcessOutcome`, `RefusalCode`, `Stopped`, `JournalFact`, `JournalEntry` |
| Executable admission | `HelmVersion`, `HelmProtocol`, `HelmBinary` |

| Owner | Cardinality/constraint |
|---|---|
| Authority | Exactly one target, principal, host/store and environment; one desired digest; zero or one baseline digest; one or more context aliases. |
| Release permit | Exactly one service/address/incarnation. Zero or one baseline projection and desired projection, with at least one present. Zero or one exact repair pre-state. |
| Physical release address | Exactly one active authority/environment/service owner. The authority registry rejects conflicting active entries and requires the same target to use one host/store/lock namespace. |
| Release projection | Complete ordered object inventory. Duplicate addresses, omitted rendered objects, extras, unsupported kinds and cross-namespace entries refuse. |
| Snapshot | Zero or one authenticated Helm identity; exactly one read result for every member of the union of admitted baseline/desired inventories. An empty or partial list cannot prove absence. |
| Quiescence decision | Exactly one complete retained lock claim and its digest. It refers to historical execution; it grants no ownership over another release and does not rewrite historical outcome evidence. |
| Invocation | Exactly one new identity and context; zero or one predecessor; complete ordered selected service list. Desired releases first, retirements afterward in reverse baseline order. |
| Journal | A nonempty valid prefix has one `Opened` at sequence zero, contiguous immutable sequence and predecessor digest absent only at zero. An incomplete prefix has no terminal. A closed history has one final `Stopped` or `Completed` and no following entry; at most one `Prepared` and one process disposition per selected index. An empty reservation is incomplete and grants no mutation authority. |
| Observation | One operation and phase, complete snapshot, invocation-local monotonic interval. Positive absence requires successful authenticated API reads; unavailable/forbidden/malformed responses cannot construct absence. |

The registry additionally owns the complete list of active `Authority` values and one generation. `RegistryRef` carries exactly that generation and whole-byte digest in `LockClaim` and `InvocationContext`. `HostPolicy` owns exactly one `HelmBinary` with path, digest, exact `HelmVersion` and closed `HelmProtocol`. The full active-registry index and cardinality rules in C05 are mandatory. An AuthorityRegistry has zero or more complete active authorities; an empty registry authorizes no execution. RegistryRef identifies immutable registry bytes; it embeds no registry and introduces no recursive authority structure.

Additional mechanical rules:

- Reuse the existing `Digest` grammar and canonical document hashing.
- Validate identifiers, semantic versions, Kubernetes names, paths, UIDs and integer bounds explicitly.
- Match current/baseline inputs and release membership against authority before execution.
- Admit a create only when `may_create` is true, the required namespace already exists, Helm release storage is authoritatively absent and all managed addresses are absent.
- An existing matching desired projection produces a skip. A matching admitted baseline may permit the requested change.
- Other drift refuses unless a new caller authority revision supplies the **exact independently reviewed** `repair_from` snapshot. It cannot override a foreign incarnation.
- A previously established owned history cannot be discarded by omitting the baseline to regain first-deployment behavior.
- Apply C09's complete preparation/observation/Prepared ordering and freshness budget; no historical or wrong-phase observation authorizes mutation.
- Before a mutation, recheck the current authority revision, target/principal pins and ownership of the target claim.
- Post-acknowledgement readback must establish the required current projection before continuing. Acknowledgement alone remains historical process evidence.

ESS expresses shapes, closed choices, required/optional fields, lists and simple invariants. Cross-record ordering, uniqueness, byte verification, authority admission and OS/network semantics remain explicit Rust reader/executor obligations. Generated schemas carry invariants as annotations (`crates/generate/ess-gen/src/types.rs:654–660`); model validation alone does not enforce these runtime facts.

The model's `Digest` and `HelmVersion` declarations constrain nonempty strings; their complete digest/version grammar is a reader obligation. `Index` is nonnegative, `HostPolicy.executor_uid > 0`, and `Authority.contexts.count > 0`. `ReleasePermit`'s optional fields do not themselves enforce “at least one baseline or desired”; the reader does. Lists do not automatically enforce cardinality, uniqueness or full inventory coverage. The singleton format variants are `ess-execution-authority/1`, `ess-execution-registry/1`, `ess-execution-store/1`, `ess-execution-lock/1` and `ess-execution-evidence/1`.

Retained validate/compile receipts establish declaration validity only. Root binds the exact published declaration to that evidence before implementation. A necessary new persisted noun, field or relationship must be explicitly resolved and declared before code depends on it; otherwise record the newly unresolved point as UNMAPPED. No AEP dependency enters ESS.

## C12. Complete executable R01–R29 matrix

Every indexed row expands at each selected apply index and each reverse-order removal index, preserving the earlier prefix. “Stop” fails the invocation without launching subsequent mutations; it does not imply that external effects are known. The matrix is an executable obligation, not a claim that these tests have run.

Use a fixed fixture containing three desired releases and three baseline-only retirements, with a nontrivial rollout order. For every indexed boundary, inject at each applicable apply and removal index. Also run dedicated empty/equal, first-create, all-absent and changed-inventory fixtures. These are families with multiple concrete vectors, not a claim that 29 tests suffice (`docs/design/review-execution-recovery.md:64–100`).

| Family | Required executable vectors and assertions |
|---|---|
| **R01** | Invalid desired/current JSON and YAML, duplicate keys and invalid later releases. No external call, cache change, recovery reservation or claim. Preserve existing whole-plan controls. |
| **R02** | Wrong/unresolved target or principal, context aliases/remapping, unrelated baseline, duplicate release address and registry/store/object aliases. Reject before acquisition or mutation; an invalid unselected registry entry also rejects selection. |
| **R03** | First/equal/changed plans and removal previews, with differing synthetic target and call traps. Local unverified preview only; cache/evidence/target bytes unchanged. |
| **R04** | Missing execution evidence with unavailable observation on first run and restart. Preserve an independently populated target; never infer empty state. |
| **R05** | Acquisition-directory creation and ORAS spawn failure at every apply index. No Helm mutation at that index; settled prefix remains. |
| **R06** | Fail/interruption during manifest or each referenced blob fetch, including partial output. No chart admission or target progress; restart must reacquire/re-admit. |
| **R07** | Completed fetched output followed by interruption before bounded read, proof assembly or chart preparation. Staged residue is neither a cache hit nor application evidence. |
| **R08** | Wrong/missing/nonregular output; invalid manifest/layer cardinality; descriptor read failure; malformed chart archive or generated-member inventory. No Helm mutation; retain exact completed prefix. |
| **R09** | Fail proof staging write/readback/no-replacement publication boundaries; retain incomplete stages. Restart ignores unpublished stages and preserves existing entries. This tests current proof publication, not the removed archive/checksum pair. |
| **R10** | Corrupt warm framing, original manifest identity, descriptor size/digest, payload and read errors. Refuse without overwrite or silent repair; no Helm mutation. |
| **R11** | Valid warm original-byte proof with ORAS trapped, fresh authorized observation requiring apply. Revalidate and reuse bytes, then perform the required mutation; cache presence does not imply applied state. |
| **R12** | Values serialization/write and each pre-mutation storage failure; cut after values, after `Observed`, and after `Prepared` before launch. Verify preparation/freshness ordering and conservative restart classification. |
| **R13** | Definite Helm apply spawn refusal at every apply index. `NotLaunched` only where absence of launch is established; retain prefix and stop later work. |
| **R14** | Started nonzero, timeout and lost acknowledgement, each with no target effect and effect-before-failure variants. Same indeterminate classification; retained claim; no later apply/removal or fabricated rollback. |
| **R15** | Apply effect and successful return, then fail/cut before durable process outcome or required readback. No overall success; restart observes without automatic replay or invented acknowledgement. |
| **R16** | Cut after complete per-operation evidence and before next action/final output. After required quiescence, fresh matching observation skips duplicate mutation and permits only authorized remaining work. |
| **R17** | Partial multi-release sequence at each index, including middle operation effect then failure. Preserve earlier settled state/evidence; identify unknown release; later releases and all retirements remain untouched. |
| **R18** | Unsupported/wrongly scoped/corrupt published entries, gaps, wrong predecessor hash, duplicate terminal or entries after closure. Refuse and preserve bytes. Separately admit all specified valid incomplete prefixes and ignore unpublished partial stages. |
| **R19** | Valid historical evidence with unavailable fresh observation; monotonic expiry at launch; wrong invocation/operation/phase references. Stale evidence cannot establish no-op or authority. |
| **R20** | Equal desired/baseline bytes with manual target drift between processes. Observe despite equality, refuse implicit repair, then separately test an independently supplied exact `repair_from` revision. |
| **R21** | Effect then lost acknowledgement; restart observes exact desired match. No duplicate mutation or invented historical applied attribution. Without required Q, remain observation-only and mutation-blocked. |
| **R22** | Retirement present, desired apply present, removal flag absent. Reject before both phases and before acquisition/recovery writes that would imply execution began. |
| **R23** | Baseline-only retirement with no desired projection; uninstall spawn refusal at each reverse-order retirement index. Preserve completed prefix and leave later removals untouched. |
| **R24** | Started uninstall failure/timeout/lost acknowledgement with still-present and removed-before-failure variants. Remain uncertain; never infer absence from status or message text. |
| **R25** | Successful uninstall effect followed by evidence/readback failure or process interruption. Restart with retained target and history; no unconditional repeat uninstall or fabricated removal attribution. |
| **R26** | Explicit repeated baseline-only removal with valid owned history and authoritative absence of both Helm storage and every baseline direct object. Complete as observed already absent without another uninstall, subject to required Q. |
| **R27** | Foreign incarnation at old release address; foreign occupant at baseline-only or newly desired object address. Refuse; neither removal flag nor repair authority may adopt/delete it. |
| **R28** | Competing real driver processes, retained claim after parent death, live descendant, changed registry generation/bytes, target/principal/Helm/claim mismatch between observation and launch. No claim theft or subsequent mutation; independently supplied Q is a separate lane. |
| **R29** | Fail final `Completed` staging/readback/publication/directory synchronization after all calls succeed. No complete-success claim; retained valid prefix survives and a later invocation observes instead of replaying all calls. |

Supplement the family table with these finite, named dimensions:

- **Directory durability:** exclusive reservation creation, new-directory sync and parent publication sync fail independently; repeat for every dynamically introduced directory. No claim or mutation before the required barriers.
- **Journal grammar:** reserved directory without `Opened`; `Opened` only; `Observed` without `Prepared`; `Prepared` without disposition; acknowledged disposition without `After`; all per-operation facts without `Completed`; valid `Stopped`/`Completed`; staged partials; corrupt published entries. Omitting `retry_of` never hides relevant history.
- **Inventory changes:** baseline and desired address sets differ. Before apply, newly desired addresses must be absent; after apply, baseline-only addresses must be absent. Retirement admits baseline-only inventory. Retained direct objects/finalizers prevent absence.
- **Generated-chart refusal:** links, traversal, duplicate/extra/missing archive members, mismatched projected bytes, injected template actions, hooks, CRDs, dependencies, post-renderers, cross-namespace objects, `generateName`, keep-policy and unsupported object kinds.
- **Projection fidelity:** authored rendered fields and complete approved live projection, including unknown additional fields; distinguish UID/resourceVersion evidence from the projection digest. Cover Deployment, StatefulSet and Service.
- **Executable admission:** binary/path/hash/version/protocol mismatch, writable/linked installation, absent flags, malformed probe output, output limit and timeout, environment injection and unsupported credential/proxy configuration.
- **Secret containment:** credential or Secret-value sentinels never appear in stdout, stderr, durable evidence, temporary serialized observations or retained test reports.

The cache controls must retain both original-byte transport profiles and their malformed-input, warm-cache, concurrent-winner and private-snapshot assertions. Opaque historical chart fixtures can remain cache-layer vectors; they cannot be relabeled as admissible generated recovery charts. Add genuine projected-chart vectors for the complete recovery lane, and explicitly account for that changed test boundary. The current fake attack client hard-codes old arguments including `--create-namespace` (`support/cache_origin_attack_client.rs:22–35`), so that fixture needs deliberate adaptation.

The supported result remains direct-resource observation at a point in time. Tests must not claim application behavior, descendants, retained PVC/PV cleanup, real admission-controller policy or perpetual convergence.

Additional inherited controls and the authority addenda remain required:

- Snapshot the fake target, cache and evidence paths around R01/R03. Exercise the local preview with and without current/removal flags. R01/R03 return before registry resolution, executable probes and every external call or write.
- Inject after values and after durable preparation separately in R12. A known prelaunch cut has no child effect; a retained Prepared without durable disposition remains historically indeterminate on restart and can require Q.
- Cover definite final-check cancellation as well as spawn refusal in R13/R23. Only a durably recorded NotLaunched settles nonlaunch; failed disposition persistence retains uncertainty.
- Preserve loss of stdout after settled per-operation history in R16; a retry observes afresh and does not replay merely to recreate output.
- For R19, charge observation acquisition, Observed publication, Prepared publication and final checks against the same 30,000-ms window. Use an injected monotonic clock or deterministic observation boundary; no wall-clock sleep or nondeterministic race is required.
- Add the independent administrative Q input to every retained-claim recovery case. R14–R18/R24–R25/R29 remain blocked where it is required. Fresh matching/absent observations in R21/R26 never fabricate historical mutation facts. Q cannot waive R18 corruption.
- R02/R28 cover unselected invalid entries, registry collision indexes, rebound endpoint/context/store aliases and binary replacement. A changed claim/authority or invalidated target/principal stops launch. Unsupported outside writers refuse the profile; an administrative assertion is not a fencing token.
- Distinguish read-only Helm/observer traces from upgrade/uninstall mutation traces. Existing cache controls retain their intended boundary; acquire valid synthetic authority when required rather than letting an earlier refusal make a later assertion pass.
- Failure reports identify the unresolved release and settled prefix. No global rollback, global success or generic Helm “not found means absent” conclusion is permitted.

These families retain the original observable requirements with current cache proof names and the accepted authority preconditions. Actual future results must name each concrete vector, executable identity, frozen implementation subject, command, exit and result; 29 family labels are not a runner count.

## C13. Accepted offline execution qualification

The production profile expects protected administrative files and a nonzero executor UID. An ordinary unprivileged temporary directory cannot honestly instantiate that complete authority arrangement.

The coordinator accepts the test-only Rust process-driver construction under the following mandatory qualification. It must:

- Link the same production argument parser, handler, admission algorithms, reconciliation engine, chart preparation, journal code and process adapter.
- Inject host-evidence/transport/storage controls through Rust interfaces in the test target. The shipped CLI must expose no bypass flag, environment switch or permissive fixture profile.
- Run at least two separate driver processes against retained synthetic target state and retained recovery storage. The independently controlled fake target must outlive the first driver.
- Record exact operation, release, arguments, namespace/context, chart/values bytes, mutation index and process outcome.
- Keep target mutation controls independent from journal/storage fault controls. Provide barriers for “not launched,” “launched without effect,” “effect then failed/lost acknowledgement,” acknowledged success, and parent termination while a child remains active.
- Preserve target state and original failed-run artifacts across retries. Tests must not reconstruct the target from the executor’s journal.

A concrete layout is the three new test paths in the scope: `execution_recovery.rs` orchestrates vectors and retained evidence; `support/recovery_driver.rs` supplies the test-only executable entry into shared production code; `support/fake_recovery.rs` supplies the stateful Rust child/target fixture. The exact Cargo linkage is an implementation prerequisite; it must not require a production runtime escape hatch.

Distinguish the following test evidence:

| Real adapter tests | Explicitly injected or trusted evidence |
|---|---|
| Real regular-file/symlink/directory handling, canonical path and inode alias checks, permission-bit rejection, create-new/no-replacement behavior, readback, hash verification and preservation of corrupt files. | Positive administrative UID ownership under an ordinary unprivileged fixture. Metadata injection exercises the decision algorithm; it does not prove deployment of root-owned files. |
| Actual filesystem invocation directories, claims, immutable entry publication and process restart; real operation failures where practicable, plus deterministic faults at every synchronization boundary. | Simulated loss of unsynchronized publication after a crash. This proves the protocol’s ordering and restart response, not the storage hardware’s power-loss behavior. |
| Loopback Rust TLS server with test CA: certificate/hostname/CA mismatch, endpoint restrictions, redirects/proxies, bounded responses, malformed/duplicate data, HTTP errors, unavailable reads, and finite request selection. | Synthetic namespace/ServiceAccount UIDs and API responses. Successful local TLS does not establish a real cluster’s identity or caller’s authority. |
| Real hashing/version/help parsing and child argv/environment, output bounds, timeout/kill/reap behavior against a Rust executable fixture. | Stock Helm’s semantic correctness, lack of remaining asynchronous effects after successful return, and administrative immutable-installation guarantees. |
| Shipped `ess` binary: flat/grouped help and parse behavior, invalid-input refusal, local dry-run, missing/untrusted authority refusal and no-effect sentinels. | Positive recovery execution through the shared-code test driver. Label it accordingly; do not report it as a deployment by the shipped binary on an admitted host. |

At least one real TLS/HTTP fixture must exercise the production transport rather than replacing every API call with an already decoded snapshot. Secret data must not reach logs, serialized evidence or fixture artifacts: use metadata-only release-storage reads and retain redaction controls. The engine must distinguish authenticated absence from authorization errors, malformed output, transport failure and generic Helm errors.

Use `target/` scratch and TMPDIR owned by the assigned implementation worktree. Do not introduce shell/Python executors, real credentials, a real registry/cluster requirement or a shipped test bypass. Retain exact vector and executable identities, original failed-run artifacts and independent target state across at least two real process invocations.

This qualification accepts offline test construction. It does not establish deployment authority, real root-owned host provisioning, stock Helm semantics, real-cluster authentication, storage hardware behavior or a perpetual state guarantee. The production caller must independently provide the admitted host/Helm contract.

## C14. Implementation owners, CLI seams and compatibility

These are the existing obligation's exact scoped implementation reservations. They do not create a story, select a wave or authorize editing as part of this publication draft.

| Module | Complete responsibility |
|---|---|
| `recovery/model.rs` | Strict Rust representation of all 44 declared types; closed fields/variants, canonical encoding and explicit reader constraints. It owns no Kubernetes calls or administrative decisions. |
| `recovery/authority.rs` | Read the protected active registry and every active authority before selecting one; verify retained revisions, registry generation, host/store/principal/target bindings, aliases, physical release addresses and object-address unions. Admit the future caller’s authority; never manufacture it. |
| `recovery/journal.rs` | Store admission, random invocation reservation, new-directory and parent synchronization, durable no-replacement claim, immutable journal publication/readback, complete relevant-history admission, valid-prefix classification and finalization. No lease, automatic claim theft or automatic history reset. |
| `recovery/chart.rs` | Current OCI proof acquisition; private chart/values preparation; bounded archive decoding; exact five-file `project_helm` comparison; admitted Helm rendering; baseline/desired-specific complete inventories. |
| `recovery/process.rs` | Absolute admitted Helm artifact, restricted environment/argv, bounded compatibility probes, spawn/acknowledgement/timeout boundaries and conservative uncertainty. Direct-child reaping does not establish descendant or outstanding-request quiescence. |
| `recovery/observe.rs` | Join bounded authenticated API reads with consistent Helm metadata/manifest/hooks; compare complete admitted direct-object projections; enforce freshness, ownership and absence predicates. |
| `recovery/mod.rs` | Shared reconcile arguments/handler and finite operation sequence. Validate everything required before execution; observe equal desired plans; execute at most one admitted mutation per operation; stop at uncertainty; account for every selected operation before `Completed`. |
| `ess-kubernetes/src/recovery.rs` | Credential and TLS boundary for the finite API requests. Return bounded sanitized data suitable for the recovery engine, without creating an `InfraIr`/`EssIr` shared envelope. |

Prefer a direct Rust TLS/API transport in `ess-kubernetes` for this bounded adapter. Reusing external `kubectl` would introduce a second executable-admission contract absent from the 44-type candidate. The existing dependency files contain no such transport or archive implementation; dependency selection and MSRV compatibility therefore remain implementation work, not an already available capability.

The CLI surface retains `--path`, `--current`, `--cache`, `--allow-removals`, `--dry-run` and the timeout surface. A proposed `--authority <UUID>` selects an entry from the fixed protected registry; it does not accept a self-authorizing arbitrary file. Any exposed `--retry-of` spelling must decode the existing `InvocationId`; omission never bypasses retained-history admission. Move “previously applied” wording to “admitted baseline desired deployment.”

Dry-run remains a local preview and must return before registry probes, target calls, cache writes or recovery writes. Normal execution without usable authority refuses. No partial-service selector, authority-provisioning command, quiescence command, controller or real deployment is required by this unit.

### Exact 34 write reservations

- `crates/edge/ess-cli/src/main.rs` — cited.
- `crates/edge/ess-cli/src/oci_cache.rs` — cited.
- `crates/edge/ess-cli/src/lib.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/mod.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/model.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/authority.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/journal.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/chart.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/observe.rs` — inferred.
- `crates/edge/ess-cli/src/recovery/process.rs` — inferred.
- `crates/infra/ess-kubernetes/src/lib.rs` — cited.
- `crates/infra/ess-kubernetes/Cargo.toml` — cited.
- `crates/infra/ess-kubernetes/src/recovery.rs` — inferred.
- `crates/infra/ess-kubernetes/tests/recovery_adapter.rs` — inferred.
- `crates/edge/ess-cli/Cargo.toml` — cited.
- `Cargo.lock` — cited.
- `crates/edge/ess-cli/tests/persisted_delivery.rs` — cited.
- `crates/edge/ess-cli/tests/cache_origin.rs` — cited.
- `crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs` — cited.
- `crates/edge/ess-cli/tests/command_surface.rs` — cited.
- `crates/edge/ess-cli/tests/support/fake_delivery.rs` — cited.
- `crates/edge/ess-cli/tests/support/fake_oci.rs` — cited.
- `crates/edge/ess-cli/tests/support/cache_origin_attack_client.rs` — cited.
- `crates/edge/ess-cli/tests/execution_recovery.rs` — inferred.
- `crates/edge/ess-cli/tests/support/recovery_driver.rs` — inferred.
- `crates/edge/ess-cli/tests/support/fake_recovery.rs` — inferred.
- `models/execution-recovery/system.yaml` — inferred.
- `models/execution-recovery/domains/execution.yaml` — inferred.
- `docs/design/review-execution-recovery.md` — cited.
- `website/docs/concepts/component-delivery.md` — cited.
- `website/docs/reference/cli.md` — cited.
- `website/docs/status/where-this-stands.md` — cited.
- `website/docs/status/limitations.md` — cited.
- `crates/edge/ess-xtask/src/support.rs` — cited.

The fixed layout and scope preserve `crates/generate/ess-deployment/src/{lib,environment,runtime}.rs`, `Cargo.toml`, `AGENTS.md`, `Taskfile.yml` and `crates/edge/ess-xtask/src/main.rs` as read-only reuse. Process and credential IO stay at their existing edge/infra owners; pure deployment projection gains no executor or AEP dependency. The listed new modules and concrete Cargo linkage are implementation choices within these reservations. No MSRV increase or unsafe code is selected.

Canonical intent bytes, release artifact semantics and intrinsic validation remain unchanged. Replace the unsupported “previously applied” description with an admitted baseline desired-input description. Normal execution without admitted authority refuses; dry-run preserves its local preview. Text output and binary success/failure exits remain the initial boundary. No separate serialized public result envelope, partial-service selector, authority/Q provisioning command, controller, cache redesign or real deployment is introduced.

Update only affected reconcile/help/support statements in the four reserved public pages and the matching Explicit executors row in `ess-xtask/src/support.rs`. Preserve the complete 20-row block, passive `[ess-source-support-begin]: #` and `[ess-source-support-end]: #` markers with their required blank lines, source/release distinction and dated release observation. The deterministic support lane makes no executor calls and is not runtime recovery evidence.

Raw kubeconfig, credential and Secret values never enter evidence, logs, serialized observations or fixture artifacts; secret references remain references. No private environment is needed for the acceptance matrix. If the future persisted contract changes bytes verified in another repository, root first inventories those readers and coordinates version/order through Atlas. No external receipt reader or completed migration is established by this source.

CLI/help, cache consumers/tests, Kubernetes credential edge, Cargo.lock, the four public pages and support.rs are collision boundaries. Use one implementation worktree with owned temporary/cache/fault fixtures and a serialized full gate. Root alone owns planning and lifecycle mutations. Newly discovered semantic prerequisites are reported before changing this scope or model; they are not inferred as accepted merely to make a check pass.

## C15. Verification and completion boundary

Root publishes and validates the exact two model files and consolidated binding before runtime edits. During implementation run targeted meaningful vectors, then affected-package formatting/tests/strict Clippy. Integration must run every lane of task check, including support-check, and task site-build. The affected packages are ess-cli, ess-kubernetes and ess-xtask. Preserve generated-byte and dependency-boundary controls. Record the actual runner counts rather than claiming 29 tests.

```bash
cargo fmt --package ess-cli --package ess-kubernetes --package ess-xtask -- --check
cargo test --locked -p ess-cli -p ess-kubernetes -p ess-xtask
cargo clippy --locked -p ess-cli -p ess-kubernetes -p ess-xtask --all-targets -- -D warnings
task check
task site-build
```

These are required future gates, not executed results. Record integrated implementation evidence against the story and use it to discharge obligation:review-execution-recovery-implementation; neither model validation, candidate review, story creation nor an unrelated passing source gate is sufficient. Root alone performs lifecycle/evidence actions through AEP.

Preparing or adopting this design is not an executable red/green result. The declaration, consolidated contract and offline-driver qualification must precede implementation. Ordinary implementation verification, adversarial review, integration gates and publication remain required; mechanical consolidation of the already reviewed contract requests no third candidate attack.

The obligation remains open until every original family, indexed variant and supplemental dimension has executed against the recorded integrated implementation, with retained failures, restart evidence and unchanged controls. Package or unrelated source-gate success alone cannot discharge it. Root records actual commands, exits and runner counts and performs governed artifact/evidence/lifecycle actions. No test, build, model validation, deployment or network call is performed by preparing this draft.
