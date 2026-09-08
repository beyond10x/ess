# Generated output ownership and recovery

Status: selected design for `story:review-output-ownership`, against published ESS
`1e618d225f7f60f79c3ace7c0fa60f356cbc44d9`. Implementation, fault witnesses and native platform
validation remain owed. The typed anchor/transaction model is in `models/output-ownership`.

## Selected product behavior

- Keep existing ordinary generator command signatures and emitted artifact bytes. Each tree output defaults to its own output-root anchor; standalone generated files use their parent. The fixed G01–G14 owner-family table from the retained anchor candidate is selected. Model/input/version/content do not identify the owner. All-generation updates its separately collected five projection owners together; a selected generator updates only its own owner.
- Compose gains an explicit enclosing ownership root when it selects any file outputs. Its one anchor-wide owner replaces the complete selected companion/client set; omitting an output retires its formerly owned files. No-output compose remains nonwriting. Ordinary flat routes do not gain wider-anchor configuration in this change.
- Default first generation refuses existing unowned destinations. Legacy adoption is a separate output-management operation from a settled generated reference root. The reference ledger and every selected actual file must agree; target files enroll only when their bytes equal the reference. Missing target files may remain absent and unowned until ordinary generation creates them. Unknown/extra legacy files stay unowned. Pending/corrupt/dirty references, other-owner collisions, overlapping reference/target roots and invalid native paths refuse. The actual source/reference owner key is selected from the fixed family/location table; an arbitrary ownership override is not offered.
- Ordinary regeneration may repair edited/missing files already owned by its selected family; snapshot their actual pre-operation state for rollback. Stale retirement removes only selected owned paths; remove only recorded created directories that are empty. Authored neighbours and authored descendants blocking file/directory transitions are preserved.
- Recovery is explicit and independent of current model inputs. A generation observing pending state refuses and identifies its root. Recovery follows the recorded operation; it never reinterprets changed command destinations. Check/no-output paths create no state or stages and perform no recovery.

## Filesystem contract

Preserve Linux and macOS support declared by the current release matrix. Use safe platform APIs compatible with the current MSRV and unsafe-code prohibition. rustix1.1.4 is a candidate dependency: its source provides Unix directory descriptors/locks and Linux/Apple no-replacement rename. Native Linux execution and native macOS validation remain distinct required evidence; Linux tests do not establish macOS execution. A CI reservation may be needed for native macOS checks before acceptance.

The guarantee covers process interruption and injected I/O failures on one local mounted filesystem with controlled parents and cooperating writers. Preserve descriptor-relative containment, no-follow/type/link checks, top-down nonblocking shared ancestor/exclusive anchor locks, fresh-anchor parent locking, and enrolled ancestor/descendant refusal. No hostile-writer, cross-filesystem transaction, simultaneous cross-directory visibility, or unconditional power-loss guarantee is made. Use actual file/directory synchronization calls and record/refuse failures; document underlying storage assumptions. No empty hardware admission registry or ext4-only rule is selected. Linux same-device bind mounts still require mount-identity checking before mutation. The macOS equivalent and sync behavior must be established from APIs and native validation.

## Durable protocol

Select the prior five-phase checkpoint design with a separate versioned output-state format: Idle (no transaction), Staging, Prepared, Committed and Restored. Paths, owners, digests, ordinary modes, before/after ledgers and exact internal file inventories are closed typed data. Unknown versions/fields, duplicate keys, invalid canonical bytes, unsafe names and contradictory state refuse without deleting evidence. A checksum detects inconsistent bytes and does not authenticate an adversary.

Initialize through a fully written/read-back/synchronized private directory followed by no-replacement publication of the state directory. Unpublished initialization orphans have no output authority and are preserved. Durably record the exact staging inventory before creating any transaction member. Complete and synchronize immutable preimages/new stages before publishing Prepared. Keep preimages independent from output inodes.

Before a durable commit, recovery restores the complete actual old snapshot and then records Restored. After the complete new output and all changed directories are synchronized, publish/synchronize Committed. Ambiguous commit synchronization stops with evidence; it does not guess rollback. Committed/Restored survives cleanup so interruption after deleting backups still has an admitted checkpoint. Cleanup removes only recorded internal entries; Idle is published only after cleanup is synchronized. Repeated recovery is idempotent.

A failed operation returns an error plus a recoverable checkpoint when restoration cannot finish. Never report success or start an external build before publication and finalization succeed. Preserve the distinction between a successfully published semantic report with exit1 and a publication failure.

## Typed home and validation

The model declares Anchor (UUID, native root binding, ledger digest) and its at-most-one owned Transaction (UUID, anchor_id, exact before/after plans). The ownership relation is selected because the transaction cannot outlive or be recovered independently of its enrolled anchor; the anchor is never automatically removed. NativePath, OwnerKey, Digest, Ledger, BlobRef and PathChange are concrete values. Prepare, Commit and Restore command outcomes cause the declared transaction transitions. The model gives identities and decision structure; it does not replace the runtime checkpoint grammar or prove filesystem behavior.

Preserve W01–W17 from the retained scoper reports, updated for separate reference adoption and Linux/macOS support. Require meaningful red-before-green tests for interrupted staging, every mutation and cleanup boundary, repeat/recovery, stale ownership, authored sentinels, native names, collisions, changed compose outputs, selected-owner isolation and no-write checks. Keep every existing gate, finite initial consumer eligibility and exact attributed cases. New helper declarations receive explicit classifications; ordinary function bodies still require behavior evidence. Any actual signature/profile change is a new accounting obligation, not permission to expand e005.

Excluded named compile/import/data/execution-report outputs remain outside generated-output ownership, as the fixed route table specifies. Repository projection sync must refuse intersecting reserved ownership state before its existing blanket orphan deletion. No generic hidden-file exclusion, new release or downstream publication is selected.

## Fixed owner families

`@r` means a tree location relative to the anchor; `@f` means an exact native filename.
The owner flag selects an existing fixed key from the reference; it cannot rename an owner or
its paths. Ordinary tree roots use their root location; standalone filenames must match.

| Routes | Recommended family and aliases |
|---|---|
| G01 generate all/selected | `projection:docs@r`, `projection:site@r`, `projection:schema@r`, `projection:openapi@r`, `projection:asyncapi@r`; separately `projection:docs-ir@r`. All updates five owners together; one updates only its selected owner. |
| G02 authored site | `projection:site@r`, including ordinary site. Use actual root-relative Artifact.path, never the site-prefixed map key. Retire withdrawn publication copies, preserve authored inputs. |
| G03 synthesis Rust/Go/Web/Clap | `synthesis@r`; target switches replace this owner's complete artifact/plan/report set. |
| G04 compose | One `compose` owner per explicit anchor, covering both companions and the complete three-file Rust tree. |
| G05 Go conformance | `conformance-go@r`; legacy and coverage suite routes are aliases. |
| G06 browser conformance | `conformance-browser@r`; legacy/coverage alias; preserve unowned skin.js and other authored additions. |
| G07 BuildKit | `buildkit@r`; project and build-execute projection phase alias. External execution follows successful publication only. |
| G08 Helm | `helm@r`, complete five-file chart. |
| G09 Kubernetes | `kubernetes@r`, complete projected artifact set. |
| G10 OpenAPI | `--path` aliases G01 `projection:openapi@r`; recommend explicitly enrolling `--ir`'s named YAML as `openapi-file@f`. |
| G11 bundle types | `types-bundle@r`, all targets under the same owner. |
| G12 model types | `model-types@r`, all targets; deliberately distinct from G11. |
| G13 normalization libraries | `normalization@r`, all targets and complete retained inputs/reports. |
| G14 standalone generation | `typescript-file@f` and `realization-markdown@f`; retain existing check modes. |

Adoption is first enrollment for the selected owner, or an exact idempotent repetition of its
existing inventory. Other owners remain unchanged. It must never replace a selected owner's
previous inventory with a smaller reference and silently lose stale-retirement authority.
Reference and target must be disjoint and non-aliasing; acquire their locks in one deterministic
order. Adoption changes only ownership metadata, so it needs initialization/checkpoint failure
and process-restart witnesses of its own.

## Format and compatibility

The new private envelope is `ess-output-state/1`, canonical sorted-key JSON with one final LF.
A strict reader rejects a byte sequence differing from its canonical reserialization, unknown
fields/variants, duplicate map entries, unsupported versions and contradictory inventories.
`UnixBytes1` encodes native path components as lowercase hexadecimal on the supported Linux and
macOS platforms. Reject empty/dot/dot-dot/NUL/separator components, except an empty component
list representing the root-relative owner location. Digest strings are lowercase SHA-256;
ordinary modes exclude privilege bits. Runtime admission supplies these constraints in addition
to the entity model's abstract types.

The checkpoint contains the anchor UUID, absolute native-root binding and directory identity,
sequence, settled ledger or exact current transaction, and a checksum over the canonical payload
excluding that checksum. Immutable backups and new/restore stages have recorded names and
identities. Publish a synchronized state.next over state.json and synchronize its directory;
state.next is never authoritative during recovery. Reserve .ess-output and the initialization
prefix .ess-output-init- in generated destinations and enrollment discovery. Preserve unknown
reserved entries and unpublished initialization orphans; do not grant cleanup authority from
a filename.

Existing producer artifact bytes and their format identities remain unchanged. Older ESS
versions have no output-state reader or participating lock protocol; their generation commands
are outside the enrolled-root contract. Compatibility tests distinguish unchanged generated
bytes from admission of this new private format. Repository and public format catalogs must
name the new reader/writer and link this contract when the runtime is implemented.

The original W01–W17 definitions are retained with the source-scoping evidence. For this selected
binding, W02 covers reference adoption, W09 covers process interruption rather than hardware
power cuts, W11 covers top-down directory locking, and W14 requires refusal of cross-mount
transactions before destination mutation. The newer scoper's adjacent-stage/cross-filesystem
alternative was considered and not selected.
