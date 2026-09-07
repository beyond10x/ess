# Predictable authored-input discovery

Status: accepted for implementation on 2026-09-07 under the standing remediation approval.
Story: story:review-authored-discovery.
Source subject: 603dd90f855e994ce41aeab4788ea72f4fa01dbe. The original candidate used dbe78c5b15df478ec2cd4883c67d0012cdf90e17; its independently reviewed contract is preserved below.

## Problem and required result

A directory containing model fragments, nested authored scenarios and generated output must select the same intended sources independently of filesystem enumeration order. Generated files must remain outside that selection unless explicitly named as inputs.

Preserve existing supported layouts and the explicit-empty authored-path refusal. Omitted --scenarios continues to select no authored sources. Committed --suite and --suite-input execution continues to bypass filesystem discovery.

## Current behavior

These statements describe the implementation before this discovery change:

- Model directories require immediate system.yaml and recursively select lowercase .yaml/.yml files. Explicit model files bypass extension filtering. Directory identities are canonicalized to stop repeated traversal; child enumeration is currently unsorted before that decision.
- Model fragments may omit format and system. Domain, component/conversion and topology fragments in examples/billing demonstrate this.
- Both authored loaders select immediate lowercase .yaml/.yml entries, sort them and refuse an explicit empty selection. They do not recurse. An explicit authored file bypasses extension filtering.
- Suite/4 follows supported file/root symlinks. Suite/5 refuses selected symlinks and nonregular files and uses checked root-relative UTF-8 identities.
- RawSpecFile and the authored compiler own document parsing, format support, semantic duplicates and accumulated refusals. Selection is not currently based on successful parsing.

Owners: ess-cli/src/load.rs:25, main.rs:2833, coverage.rs:16; ess-domain/src/spec.rs:45,151; ess-conformance/src/authored.rs:1451 and coverage_build.rs:46.

## Accepted decision

Accepted coordinator choice: introduce optional ess-inputs.yaml directory configuration, with discriminator format: ess-inputs/1 and separate exact specification and scenarios lists.

No new CLI flag is introduced. The explicitly supplied directory and the presence of its immediate ess-inputs.yaml opt into the new contract.

Uniform typed discovery is a viable alternative only after defining a permitted layout or ownership markers. Content-only recognition is insufficient: current fragments have no required discriminator, malformed intended fragments must not disappear because they fail classification, and a generated copy of an ESS-shaped fragment is indistinguishable from an authored fragment by content. Reserved generated directories would instead impose a layout convention and migration rules.

The manifest costs a new versioned configuration reader and model declaration. It buys exact selection without adding classifier exceptions, implicit recursion or arbitrary generated-directory names. The coordinator selects that tradeoff for this story. Uniform content-based classification is not part of the implementation.

## Optionality and active input role

Required contract:

1. Omitted --scenarios returns no authored inputs before looking for any scenario manifest. A scenarios list in the model's manifest never creates an implicit authored selection.
2. An explicit file remains a single source file, independent of extension. Do not search its parent or ancestors for a manifest.
3. An explicit directory checks only its immediate ess-inputs.yaml.
4. With no manifest, use legacy discovery. Do not search ancestors or nested directories for configuration.
5. If the reserved manifest path exists, it must be a readable regular non-symlink file containing one supported manifest. Invalid, unreadable, nonregular or unsupported configuration refuses; never fall back to legacy scanning.
6. A directory supplied for a model selects specification. A directory supplied through --scenarios selects scenarios. The same directory may be supplied for both roles.
7. Parse and structurally validate the entire manifest, including both lists. Only the active list's files are resolved, inspected and read. A structurally valid entry in the inactive list may name a missing file without blocking the active role.
8. A model selection with an empty specification list refuses before compilation. An explicit scenario selection with an empty scenarios list refuses before suite/report/artifact output.
9. Both lists may be empty structurally; such a document cannot satisfy either active input role.
10. Run with --suite or --suite-input retains its current bypass and argument-conflict rules. A poisoned or absent acquisition root cannot affect an execution branch that does not acquire sources.

## Manifest shape

The reader must admit this configuration shape when the active selected files satisfy the contract. The example is not a claim that a reader existed at binding acceptance:

```yaml
format: ess-inputs/1
specification:
  - model/system.yaml
  - model/domains/invoice.yaml
  - model/domains/email.yaml
  - model/components.yaml
  - model/topology.yaml
scenarios:
  - authored/routing/first.yaml
  - authored/e2e/second.scenario
```

Exactly three required fields: format, specification, scenarios. The two lists contain strings; null, omitted lists, unknown fields and duplicate YAML mapping keys refuse. The supported format spelling is exactly ess-inputs/1.

The reader accepts one YAML document. Lists are sets for selection purposes: declaration order has no semantic meaning, but repeated entries are errors rather than silently deduplicated.

No glob expansion, directory entries, includes, environment substitution, remote sources, content-based exclusion, default filenames or inherited manifests are introduced.

This bounded change introduces no custom source-count or byte-count limit. Existing selected-reader limits remain in force; this is not a claim of general parser-resource hardening.

## Recursion, headers and generated output

Manifest mode performs no directory walk. Nested files are selected by explicit relative paths, with no extension filtering.

The manifest root need not contain immediate system.yaml. Its selected specification list must collectively provide a valid system header under the existing assembler's rules. That header can be model/system.yaml, another filename, or part of a single selected combined document.

Headerless ESS fragments remain valid selected inputs. Do not require format: ess/1 on each fragment and do not use successful full parsing as a selection predicate.

Every unlisted file is outside acquisition. This includes generated OpenAPI/AsyncAPI YAML, malformed YAML, copied ESS-shaped YAML, nested configuration, symlinks and unrelated files. Do not enumerate, classify or open them.

A generated or foreign document explicitly listed in a role is an intended input to that role's existing reader. Wrong kind, unknown format, conflicting discriminators and malformed content remain visible refusals; never silently discard the entry or retry it under another role.

A copied ESS-shaped file explicitly listed as specification is intentionally selected. The manifest does not authenticate authorship or infer whether a selected file was generated.

Document the mixed layout using the sample above with additional unlisted generated/openapi, generated/asyncapi and output subtrees. Repeated validation after generating those files must select the same listed input identities.

## Paths and deterministic selection

The manifest entry grammar matches the existing suite/5 SourceIdentity lexical rules:

- UTF-8, nonempty, /-separated root-relative segments.
- No empty, . or .. segment.
- No backslash, colon or control character.
- No absolute path.
- Preserve Unicode and case exactly; do not normalize either.
- Characters such as * have no pattern meaning. If allowed by the filesystem they are literal filename characters.

Validate entry spelling before filesystem access. Reject duplicate spelling within either list and the same spelling assigned to both roles.

For the active role:

1. Establish the canonical directory root. Reject a directly selected root symlink in manifest mode.
2. Check each selected relative path component below that root; directory components and the selected file must not be symlinks.
3. Require a regular selected file. Refuse missing files, directories, broken links and unsupported file types.
4. Canonicalize the selected file, require it to remain below the canonical root, and reject repeated canonical targets within the active list.
5. Read entries in ascending, case-sensitive order of their original relative UTF-8 identity.
6. Read each selected file's original UTF-8 bytes without newline conversion or reserialization.

Unselected entries receive structural checks only, so canonical-target checks across inactive roles are not promised.

Hardlinks with different relative paths remain distinct source identities. They are not collapsed by inode or content hash. Identical copies also remain distinct. Repeated semantic declarations/scenario IDs retain the existing library refusals and deterministic first-source ordering. This choice avoids silently deleting an explicitly requested source from coverage.

For legacy recursive model discovery, preserve the canonical-directory visited set but sort every directory's children and traverse child directories in ascending lexical order before deciding first visitation. Continue sorting the final file list. This makes the chosen spelling for directory aliases deterministic; it does not introduce physical-file deduplication. Existing file-link and semantic-duplicate behavior remains.

## Source identities and byte compatibility

Keep separate the checked input identity, the physical file opened and the human origin label.

- Specification Source/SourceMap labels in manifest mode are the listed root-relative identities.
- Suite/5 receives those exact identities and original source text through CoverageSource. It retains its current SourceIdentity validation and digest behavior.
- Suite/4 receives the original text and a readable joined root/relative-path origin, consistent with its existing origin-as-diagnostic-label contract.
- Reordering manifest lists, manifest whitespace changes and root relocation with the same relative entries do not change selected identities or source bytes.
- Changing a selected source's newline bytes changes suite/5 source evidence as it does today.
- Changing the acquisition root can change root-relative identities. Moving a.yaml to nested/a.yaml is an identity change even when its contents are unchanged.
- The manifest is acquisition configuration. Do not add it to EssIr, suite provenance or suite/5 authored_sources, or invent a new manifest digest in those formats.
- Coverage remains an inventory of the explicitly requested sources; it is not a claim that every scenario anywhere below the root was discovered.

## Refusals and migration

Discovery refusals identify the requested root, manifest/list/entry when applicable, and an actionable repair. Empty scenarios says that the explicit scenarios list selected no authored files and tells the caller to list the intended files or select a file/legacy child directory.

All discovery and acquisition refusals occur before output creation/replacement or runner invocation. Existing document-level semantic refusal behavior, including retained partial/incomplete conformance evidence where a caller already emits it, remains owned by that caller.

Legacy recursive model input still does not infer exclusions. Its diagnostics/documented guidance should direct mixed-root users to an exact manifest or a separate model input directory.

The new reserved filename is a compatibility boundary. A legacy source occupying immediate ess-inputs.yaml with another shape receives an actionable refusal to rename that source or adopt the manifest. Do not silently reinterpret it as a legacy fragment after a manifest refusal.

The manifest is directory configuration; direct-file arguments are not another way to activate it. Document passing its containing directory.

Selected symlinks allowed by suite/4 legacy mode remain supported there. Manifest mode uses the stricter rule above; tell affected users to list real contained files or retain the supported legacy invocation.

Model-types and normalization input/output protection is unchanged. In particular, a manifest does not authorize model-type output inside its specification input directory.

## Typed model before implementation

Accepted declaration paths:

- docs/design/models/authored-discovery/system.yaml
- docs/design/models/authored-discovery/domains/discovery.yaml

These paths hold the accepted named-value model. They establish this story's typed contract, not a general location convention.

The model declares these value/document concepts:

| Concept | Shape | Authority or limit |
|---|---|---|
| InputManifest | format plus specification and scenarios lists | Accepted coordinator choice; the named model below declares these fields |
| ManifestFormat | closed value ess-inputs/1 | Accepted new configuration format |
| RelativeInputPath | String representation with the lexical rules above | Existing coverage SourceIdentity establishes the lexical behavior; selection adds filesystem checks |

Model the selected representation using ESS named types before implementing the new reader; validate the declaration and record its exact files/result in the existing story. The collections represent explicitly supplied path values, as existing acquisition APIs already consume multiple sources.

Do not invent an independently identified Manifest entity, File entity, lifecycle, deletion behavior or owns relation merely to produce a validating model. A manifest listing a path does not establish ownership of that file.

Entity identity, lifecycle, ownership and relation cardinality are outside this named-value contract. No such semantics are asserted by a path list. Adding them would require a separate concrete model decision.

The ESS declaration must report constraints it cannot express, including filesystem containment, symlink checks and canonical-target equality. A String-backed type or projected schema alone is not evidence those constraints were validated. The package-local Rust reader remains responsible for those concrete checks.

The named model has validated and compiled as recorded below. Those results do not validate a manifest instance or implement its reader.

## Implementation owners and completion boundary

Use one shared package-local acquisition/manifest owner, proposed as ess-cli/src/input_discovery.rs, and route these three acquisition implementations through it:

- load::specification_files / specification;
- main::authored_sources;
- coverage::sources.

Keep role-specific adapters for the existing domain and conformance Source types. No discovery filesystem effects move into the pure libraries.

The existing complete model caller set must acquire through the shared loader: specify validate/compile/inspect/graph; compose service paths; realization specification input; runtime system input; generate/project/synthesize; generate types; normalization --model inputs; diff/impact revisions; and fresh conformance acquisition.

The authored matrix covers synthesize IR, synthesize Go, author, web and fresh run, for suite/4 and suite/5, including flat and area aliases. Committed-suite branches remain separate.

The smallest complete unit is the manifest model/reader, all three acquisition adapters, deterministic legacy model-directory traversal, compatibility controls and the documented mixed-layout matrix. Implementing only one authored helper would leave suite/5 or model acquisition inconsistent and would not complete this story.

## Complete required acceptance matrix

| ID | Cases | Required evidence |
|---|---|---|
| A1 | Existing billing five-file directory; combined explicit model file; headerless fragments | Existing semantic output and intended source labels retained |
| A2 | Existing shallow `.yaml`/`.yml` scenario directory; explicit `.json`, extensionless and other-extension files | Current selection remains; nested malformed YAML stays unselected |
| A3 | Omitted scenarios, including poisoned working-directory `scenarios/` and a model manifest containing scenario entries | No implicit authored inputs; current generated/authored-only behavior retained |
| A4 | Empty, nested-only and nonmatching legacy scenario directories | Existing nonzero actionable refusal; no output/runner activity |
| A5 | Manifest mixed root with nested model/scenario inputs and generated OpenAPI/AsyncAPI, malformed and ESS-shaped copies | Exact listed identities selected; all unlisted files ignored |
| A6 | Same mixed tree created in reverse order; manifest lists reordered; root relocated | Identical selected identities, model semantics and suite/5 bytes |
| A7 | Add, remove and corrupt unlisted generated files between runs | Acquisition and successful outputs remain unchanged |
| A8 | Manifest root without immediate `system.yaml`; selected nested/renamed header; selected combined model file | Header location independent of filename; assembler still requires valid header semantics |
| A9 | Missing header, contradictory headers, duplicate declarations and bad selected fragments | Existing semantic refusals remain visible; no parse-success filtering |
| A10 | Required fields omitted/null/wrong type; unknown key; duplicate mapping key; multiple YAML documents; unknown version | Closed manifest refusal, no legacy fallback |
| A11 | Empty active list; inactive empty list; inactive missing source; malformed inactive list | Active empty refuses; inactive files unopened; structural validation applies to whole configuration |
| A12 | Listed generated/foreign document; unknown selected ESS/scenario version; malformed selected YAML | Appropriate existing reader refuses; nothing silently changes role or disappears |
| A13 | Duplicate list entry; same textual path across roles; repeated canonical target | Actionable acquisition refusal before writes |
| A14 | Distinct copied/hardlinked files carrying duplicate semantic identities | Both requested identities retained for processing; deterministic semantic duplicate refusal |
| A15 | Absolute/dot/empty segments, backslash, colon, controls; case/Unicode-preservation controls | Exact path grammar; no normalization |
| A16 | Selected root/file/intermediate-directory symlink; escape; broken link; directory/nonregular entry | Manifest acquisition refuses; unlisted equivalents remain uninspected |
| A17 | Legacy valid file/root links, ignored nonmatching links and canonical directory aliases created in opposite orders | Preserve existing link cases; deterministic legacy alias selection |
| A18 | Selected LF versus CRLF source | Same semantics when applicable; changed suite/5 source bytes/digest |
| A19 | Each discovery failure with absent, existing and omitted outputs | No file/directory creation, sentinel alteration, suite/report output or runner invocation |
| A20 | Committed `--suite` with poisoned scenarios/missing model; `--suite-input` bypass/conflicts | Existing bypass and argument admission retained |
| A21 | All three loaders and the complete caller matrix above; aliases and text/JSON/YAML presentation | Same acquired set or corresponding refusal through every actual route |
| A22 | Manifest filename collision; unsupported new manifest under the frozen old CLI/readers | New CLI supplies actionable migration; old readers refuse rather than silently accepting configuration |
| A23 | Model-types/normalization input protection | Existing output-containment constraints remain |
| A24 | Proposed ESS model, runtime reader and manifest fixture | Actual model validation and agreement checks; explicitly report model-expression gaps |

A22 must not assert that every old conformance command leaves outputs untouched: existing semantic-refusal paths can emit incomplete evidence. The old-reader requirement is non-success and no silently accepted manifest.


## Model validation and explicit reader limits

The two named model files were independently revalidated and compiled using the retained ESS 0.20.0 CLI from the current source on 2026-09-07. Both direct commands returned zero; validation printed `discovery v1 — 2 file(s), valid`, and compilation returned 2,192 bytes. The retained validator SHA-256 is `94983b7227d8b5c3c69cdb448cf4dfe7a4d8c65b086ae7ce54d1db75c607ebf8`. The declaration is not a claim of filesystem, manifest-instance or acquisition behavior.

| Constraint | Existing ESS support | Explicit reader obligation |
|---|---|---|
| Exactly `format`, `specification`, `scenarios`; all required | Struct fields without `Optional`; existing schema projection produces required properties and `additionalProperties: false` | Implement the same closed shape in the manifest parser, including refusal of duplicate YAML keys |
| Exact string `ess-inputs/1` | Singleton enum; enforcing schema projection exists | Check the marker when admitting a manifest; declaring the model does not install a reader |
| Each role contains a list of strings | `List<RelativeInputPath>` and the string representation | Refuse nulls, scalar/list substitutions and nonstring entries |
| Path is nonempty | Newtype invariant `'value != ""'` | Enforce at admission; the current schema projection only annotates this invariant |
| Whole-string exclusions such as `"."` and `".."` | Additional inequality invariants are expressible | Full segment checking is still required; the minimal draft avoids presenting a partial enumeration as the complete grammar |
| Complete relative-path grammar | No string splitting, regex, character classification or filesystem operations in the existing predicate language | Apply the complete grammar below |
| Empty role lists | `List` permits empty lists; `.count` supports cardinality predicates | Preserve empty inactive roles. Require nonempty selection only for an active role |
| Duplicate paths within either list and lexical overlap between lists | No `Set`, `unique` or `distinct` construct; no complete general constraint for these two scalar lists in the current authored expression syntax | Check both lists together before opening selected inputs |
| Canonical target uniqueness, containment, file kind and symlink policy | Filesystem facts are absent from this value contract | Enforce while resolving the active role |
| Deterministic selection and reading exact source bytes | A `List` is an ordered sequence; it does not impose canonical ordering or reading behavior | Sort selected identities and preserve original source bytes |

Two language limits deserve particular care:

- Quantifiers exist, but they do not supply a distinct-element count or bindable positional identity. Moreover, the current comparison parser recognizes a right-hand fact reference only when it contains a dot. Thus `left != right` compares `left` with literal text `"right"`; adding `.value` to a scalar newtype binder is also incorrect because newtypes resolve transparently. Do not insert such an invariant to claim uniqueness or disjointness.
- A cardinality invariant such as `specification.count > 0` is expressible, but would strengthen this contract incorrectly: role activation comes from the caller and is absent from these three fields.

Those conclusions follow from [available type constructors](../../crates/specify/ess-domain/src/types.rs#L126), [predicate variants and quantifiers](../../crates/specify/ess-primitives/src/predicate.rs#L327), [right-hand operand parsing](../../crates/specify/ess-primitives/src/predicate.rs#L255), and [typed path resolution](../../crates/specify/ess-domain/src/expression.rs#L362).

The declaration must accompany these explicit reader obligations:

1. **Path admission:** use the existing `SourceIdentity` lexical contract: split on `/`; reject empty, `.` or `..` segments; reject backslash, colon and every control character. Preserve spelling without trimming or normalizing Unicode, case or segments. This also excludes leading/trailing `/` and absolute slash paths. Treat accepted paths as literal filenames, with no glob, include or remote expansion. The existing owner is [SourceIdentity::new](../../crates/verify/ess-conformance/src/coverage.rs#L67).
2. **Whole-manifest structural admission:** check both required lists, every path’s lexical validity, duplicates within each list and overlap across lists, including the inactive role. Only the active role is resolved or opened on the filesystem.
3. **Filesystem admission:** retain the contract’s regular-file, no-symlink and canonical-root-containment checks for the manifest and selected inputs; refuse duplicate canonical targets among active inputs. Do not introduce inode or content deduplication for hardlinks or copied files.
4. **Selection behavior:** retain optional immediate-directory manifest discovery, refusal without fallback when that manifest is invalid, explicit-file bypass, active-role nonempty checks, deterministic identity sorting and exclusion of unlisted files from enumeration and reads.
5. **Caller compatibility:** omitted `--scenarios` continues to mean no authored scenario selection, even when a model manifest has a scenarios list. Preserve the retained contract’s legacy behavior, headerless selected fragments, downstream document refusals, source identities, provenance and output timing.

## Current source and additional caller obligations

The accepted contract preserves all A1–A24 families from the retained candidate, reviewed by
review-result:authored-discovery-binding-pass1 with an empty findings list. That was a design
review, not a runtime test. The current story-scoper refresh at 603dd90 adds the following actual
callers and effect boundaries without changing the manifest shape or model:

- `observed_bindings::run` loads `--spec` through `load::specification` before realization and
  binding admission, live collection or output creation. Exercise its actual offline `--infra`
  route and discovery refusal before any collection. Its existing diagnostic/refusal report on
  stdout remains valid; A19 does not require universally empty stdout. There is no flat bindings alias.
- `release_evidence::Options::read` loads the explicit model for qualified bundle `publish`,
  `check-conformance` and `publish-conformance`, including existing flat aliases. Preserve one
  model load and owned report/selection buffers. Test A19 refusal before ORAS/publication and A21
  successful manifest acquisition with valid downstream fixtures.
- Committed conformance run/select branches bypass acquisition as already documented. Impact still
  acquires both model revisions with a committed suite; release qualification still acquires its
  explicit model. Do not extend the run bypass to those commands.
- OpenAPI `--ir`, BuildKit/Helm projections, infrastructure observation/intent readers and ordinary
  consistency-only release routes do not acquire authored specifications through this configuration.
- `load.rs` specification acquisition is unchanged from the earlier candidate refresh. Preserve its
  independent infrastructure observation/IR `/2` dispatch. Preserve browser replay qualifications:
  replay establishes neither specification coherence nor implementation execution evidence.

The source-current scope has nine reservations: the CLI package, this binding, both named model
files, the two public guides, CLI/formats references and `review-format-catalog.md`. Add only a cited
acquisition-configuration row to that engineering catalog. The existing xtask schema gate projects
RawSpecFile; this edge configuration does not belong in the authored-specification schema and
introduces no universal format registry, library parser or dependency change.
