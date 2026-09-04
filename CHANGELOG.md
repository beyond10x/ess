# Changelog

## [0.13.4] — 2026-09-04

### Fixed

- The component-release composite action invokes its packaged script through Bash, so clean GitHub
  Actions checkouts do not depend on a repository executable bit.

## [0.13.3] — 2026-09-04

### Fixed

- Helm projections now materialize every typed runtime secret slot as a credential-free
  `{name, key}` default and describe that closed shape in the values schema. A newly projected
  chart therefore renders and lints before an environment supplies its Secret object name.

### Added

- The ESS-owned `release-component` action lets any component repository run its own gate, build or
  adopt an exact runtime image, package and sign its chart, publish SBOM/provenance/signature/
  conformance evidence, verify release manifests, and publish the canonical OCI component bundle.
  It has no Service SDK manifest or registry-coordinate assumptions.

## [0.13.2] — 2026-09-04

### Fixed

- Canonical build IR restores an omitted empty secret set while reading, so compiler output
  round-trips through release verification and OCI bundle construction.

## [0.13.1] — 2026-09-04

### Fixed

- Configuration-neutral Helm projections now provide a schema-valid default service account,
  allowing a freshly generated chart to pass `helm lint` before private environment bindings
  replace it.

## [0.13.0] — 2026-09-04

### Added

- `ess-component/1` and canonical `ess-component-ir/1` make the implementation repository the
  owner of its semantic, realization, build, runtime, and independent runtime/chart release units.
- `ess-release-bundle/1` carries the complete verified release chain as one OCI payload. `ess
  generate release publish` uses ORAS at the credential edge; `ess generate release fetch` requires
  a digest-pinned source, revalidates canonical bytes, and caches them by OCI manifest digest.
- `ess generate build execute` retains the deterministic BuildKit projection and invokes Docker
  Buildx Bake. `ess generate deployment reconcile` computes the affected release set, follows the
  rollout DAG, fetches charts by digest, and invokes Helm only for changed releases. It refuses
  implicit removal.
- Runtime models can declare component-owned HTTP endpoints, named persistent volumes, and explicit
  container mounts. Helm projection now emits Services, workload-specific selectors, stateful
  governing Services, claims, and mounts. Named required endpoints can be derived from locked
  component or external-system providers without duplicating URLs in environment bindings.

### Changed

- Side effects are an explicit CLI executor boundary. The deployment compiler and every projection
  remain deterministic and offline while BuildKit, OCI, and Helm execution reuse their exact IR.

## [0.12.0] — 2026-09-04

### Changed

- **`ess --help` lists four commands, one per area, and every verb keeps the spelling it had.**
  The crates moved under `crates/{specify,generate,verify,infra}/` in 0.11.1; the command surface
  now says the same thing, so where a thing is implemented and how it is spelled are one fact
  instead of two. The first level is `specify`, `generate`, `verify`, `infra` and nothing else.
- **Every flat spelling is a hidden alias of its area path.** `ess validate --path .` is
  `ess specify validate --path .`: the same arguments, and when the command runs, the same stdout,
  the same stderr and the same exit status, with no deprecation line anywhere. For a refusal clap
  writes — a missing required argument — and for `--help`, only the `Usage:` line differs, because
  it names the path that was typed, which is what the flat spelling has always printed. The alias
  is left out of `--help` only so the listing stays the four areas. Both spellings are mounted from
  one definition in the derive, so they cannot drift apart, and a test enumerates every leaf of the
  clap tree and asserts the pairing rather than a list somebody keeps up to date. Nothing is
  deprecated and no pinned caller — agentide's gate invokes `ess compile` and `ess generate` by
  name — needs changing.

  | Flat spelling | Area path |
  |---|---|
  | `ess validate` | `ess specify validate` |
  | `ess compile` | `ess specify compile` |
  | `ess compose` | `ess specify compose` |
  | `ess inspect` | `ess specify inspect` |
  | `ess graph` | `ess specify graph` |
  | `ess realization` | `ess specify realization` |
  | `ess runtime` | `ess specify runtime` |
  | `ess generate` | `ess generate generate` |
  | `ess synthesize` | `ess generate synthesize` |
  | `ess project` | `ess generate project` |
  | `ess schema` | `ess generate schema` |
  | `ess build` | `ess generate build` |
  | `ess release` | `ess generate release` |
  | `ess stack` | `ess generate stack` |
  | `ess deployment` | `ess generate deployment` |
  | `ess conform` | `ess verify conform` |
  | `ess diff` | `ess verify diff` |
  | `ess impact` | `ess verify impact` |
  | `ess infra <operation>` | `ess infra infra <operation>` |
  | `ess import` | `ess infra import` |

  Two verbs share the name of their area. `ess generate --path …` is the flat spelling of
  `ess generate generate --path …`, and `ess infra diagnose …` of `ess infra infra diagnose …`.
  The `generate` area therefore offers the verb's five options beside its eight subcommands, and
  refuses the two written together — `ess generate --path X synthesize` says two things at once and
  exits 2, as it did before the area existed.

## [0.11.1] — 2026-09-04

### Changed

- **Every crate moved under the area it serves — `crates/<area>/<crate>/` — and no crate or binary
  was renamed.** `specify/` carries an authored system to a validated IR, `generate/` turns that IR
  into artifacts, `verify/` holds an implementation or a later revision to it, `infra/` is the
  separate bounded context over an observed cluster, and `edge/` is the `ess` binary and this
  repository's own tooling. Every `[package] name`, every `[workspace.dependencies]` key and the
  `ess` binary name are the bytes they were: nothing an adopter builds, links or runs changed, and
  `cargo metadata` names the same twenty-two packages. `ess-deployment` is under `generate/`
  because it depends on `ess-compiler` and `ess-realization` and on no `infra-*` crate.
- A workspace test now asserts both halves of that layout: every member sits under one of the five
  areas, and every literal `crates/<crate>/…` path this repository writes down — a fixture argument
  in `Taskfile.yml`, a source path a test reads, a path named in prose — resolves to something that
  exists. The paths that break on a move are the ones no compiler reads.
## [0.11.0] — 2026-09-03

### Added

- **A wrong-state branch that accepts rather than refuses: `refuses: false`.** `wrong_state:` could
  say one thing — the command refuses here, and reports this error — so a command that is
  deliberately idempotent had no way to be written down. ACD's `EndCall` on a call that has already
  ended answers and does nothing, on purpose and documented; its specification claimed a refusal the
  code never makes, and that claim was filed as a defect in ACD before anybody read the code. A key
  rather than "leave `error:` out": a branch missing its error is a mistake the validator has caught
  since `wrong_state:` shipped, and making the omission mean *accepts* would turn every one of those
  mistakes into a silent claim. The generated scenario's id says which claim it carries —
  `…/accepts/EndCall` beside `…/refuses/CancelInvoice`.
- **`params:` on a view, and a `query_view` that carries them.** Most views are one answer the whole
  system shares. ACD's position in a queue is not: it is per queue, and one ranked list of every
  queued call is a different thing from what the implementation can be asked. A parameter is read in
  the filter as `param.<name>`, so the predicate grammar needs nothing new; `params:` and the
  parameters the filter reads must be the same set. Synthesis binds each from what the arrangement
  settled — the scenario asks for the lane it just put the order in — and the runner refuses to send
  a request with a parameter it could not resolve, because a query with a made-up parameter reads a
  different set of rows and every assertion after it is about the wrong thing. `SemanticViewRequest`
  and the Go runner's `ViewRequest` gain `Params`.
- **`task release-status` checks that every pushed version tag is on `origin/main`.** See *Fixed*.

### Changed

- **A view's filter is now decided against what the arrangement settled, not only against the
  state.** The state was the only fact a synthesised scenario knew about the instance it had just
  created; `sets:` made the rest knowable in 0.10.0. Without this, `lane_id == param.lane_id` stays
  undecidable however well the parameter is bound, because the left side is the one nothing had
  answered. A filter over a field the scenario supplied is now decidable, so more views are asserted
  and fewer are refused as undecidable.

### Fixed

- **A release could be tagged on a branch that never reached `main`.** `task release-status` asked
  the remote for the tag list and GitHub for the release list, and neither says which line a commit
  is on — so `0.10.0`, tagged on `feat/outcome-sets-entity-fields` and never merged, reported
  "tagged and published" while `main` did not have a line of it. It is checked now, and `AGENTS.md`
  says to merge before tagging.

## [0.10.0] — 2026-09-03

### Added

- **`sets:` on a command outcome: which fields of the entity the branch determines, and from
  what.** The twin of `payload:`, pointed at the subject instead of an emitted event. Before it,
  nothing in the model related a command's input to the entity state it produced, so a generated
  scenario could find the row it had just created and say nothing about what was in it — an
  implementation that stored somebody else's amount passed. Validated like `payload:`: the field
  must exist on the entity, the source must be an input the command takes, and the types must be
  assignable or have a declared conversion.
- **A generated `contains` names the values the row holds, not only which row it is.** Read from
  `sets:` where the source is an input the scenario chose, the types do not cross a conversion, and
  the view projects the field at the entity's own type; left out rather than guessed at otherwise.
  Measured on the normative billing example, which now declares `sets:` on `CreateInvoice`: the
  deliberate negative-total fault was caught by 6 of 29 scenarios before and is caught by 13 after.
  It is a claim about a row this scenario made, so it holds on a target §8 permits to be shared.
- The documentation projection writes a sentence for `sets:` on every outcome that declares one.
- **The billing example declares an invariant over a top-level field of its own** —
  `reminder_count >= 0`, projected by `InvoiceById` so it can be decided. Nothing shipped here had
  that shape: the entity's other invariant is `total.amount >= 0`, which has a dot and parses the
  same either way, so the Go runner's left-operand defect below was invisible to every fixture.
- `cargo xtask schema [--check]` regenerates `schemas/generated/ess.schema.json` from
  `RawSpecFile`, and `task projection-check` runs it. The file was documented as drift-checked and
  was checked by nothing; the schema an adopter validates against had not moved since the model
  gained its last two constructs.
- **`ess conform synthesize --component <name>`: the suite one component can be held to.** A
  specification with two components obliges two implementations, and an implementation of one
  answers `ErrUnsupported` to every scenario about the other — which the runner reports as a skip,
  and a run with skips in it cannot say it passed. The scoped suite holds exactly the scenarios
  whose every command, event and view the component accepts, publishes or owns; the rest are
  printed as `outside:` with what they need, so they are visible somewhere other than by their
  absence. `SuiteProvenance.component` records the scope and is left out of an unscoped suite, so
  every existing suite is the bytes it was. The refusals are the whole system's, untouched.
  Measured on the ACD ↔ backend model: 44 scenarios for the system, of which the `acd` component
  can be held to the ones its own commands, events and views make answerable.
- **The emitted Go runner writes `ess-conformance-report/1`.** `ESS_REPORT_OUT=<file> go test`
  leaves the same closed document the Rust runner writes — specification, digest, implementation,
  status, counts, the non-passing scenarios by status — so `aep artifact evidence --from <file>`
  records a Go implementation's run without anybody typing a count. Before this the Go suite
  produced a `go test` exit status and nothing a workflow system could read. A skipped scenario
  makes the run `inconclusive`: a target that could not answer has not shown the answer. Recording
  a skip means the runner routes every `Skipf` through one helper that sets the status first, which
  is exercised here for the first time — no shipped fixture skips, so nothing before this release
  ran that path at all.

### Fixed

- **The emitted Go runner read the left-hand side of a comparison as a literal.** `Operand::parse`
  is documented for the *right* side — a bare word is a literal there, which is what makes
  `state == Bridged` name an enum variant — and the Go mirror put both sides through it. So an
  entity invariant over a top-level field of its own compared the *word*: `reschedule_count >= 0`
  asked whether the string `"reschedule_count"` is at least zero, which is Unknown and reported as
  a defect in the view, and `any: [{not: state == Bridged}, defined(agent_id)]` evaluated
  `"state" == "Bridged"`, which is false, so the `not` made the implication vacuously true and the
  invariant checked nothing. The Rust runner never had this: `parse_expression` puts the left side
  through `FactPath::new`. Found by the first run of a generated Go suite against an adopter.
- **`ErrUnsupported` from `ExecuteCommand` failed the scenario instead of skipping it.** Every
  other method's sentinel was honoured; this one was compared and then reported as an execution
  error. A command whose actor is the implementation itself has no caller a target can be, and a
  target saying so was being told its implementation is wrong. Every sentinel comparison now goes
  through `errors.Is`, so a wrapped one carrying the reason is recognised too.

### Changed

- `at` is still not synthesised, and the reason is now a different one. The values are known —
  `sets:` says where each row's ranking value came from — but nothing says a view holds only what
  the scenario running it put there, so naming the *first* row would be a claim about another user
  of the target. The gap table records which half is answered.

## [0.9.2] — 2026-09-03

### Added

- `ess build graph` renders the exact validated `ess-build/1` DAG and its independent release-unit
  outputs as deterministic Mermaid source, so adopter documentation can show the graph CI actually
  executes and refuse a hand-maintained diagram that drifts from it.

## [0.9.1] — 2026-09-03

### Fixed

- `ess project buildkit` emits source, cross-stage, and artifact Dockerfile `COPY` instructions in
  valid JSON form, so generated multi-output Dockerfiles are accepted by BuildKit.

## [0.9.0] — 2026-09-03

### Added

- Typed `ess-build/1`, `ess-runtime/1`, `ess-release/1`, `ess-stack/1`,
  `ess-stack-lock/1`, `ess-environment/1`, and `ess-deployment/1` formats establish the deterministic
  lowering seam from semantic systems to independently released Helm deployments. ESS projects
  `BuildKit` and chart inputs, verifies immutable executor evidence, and refuses unresolved
  stage-owned obligations; it still never builds, publishes, or applies anything.
- The `ess` CLI compiles build and runtime IR, verifies releases, resolves stacks from an
  explicit offline catalogue, projects `BuildKit` and Helm files, compiles environment deployments,
  and reports the exact component releases changed between two deployment documents.

## [0.8.0] — 2026-09-03

### Added

- **Physical realizations are typed data without changing `EssIr`.** An adopter-authored
  `ess-realization/1` binds one exact ESS system, version, and semantic digest to resolved
  components and actors, immutable implementation artifacts, typed runtime requirements, and local,
  loopback, or network entrypoints. `ess realization compile` emits deterministic
  `ess-realization-ir/1` and rejects stale locks, unresolved or out-of-subset references,
  incomplete implementation coverage, malformed placeholders, and inline secret arguments.
- `ess realization generate` projects the same resolved IR into a deterministic run-mode guide and
  supports `--check` for committed-document drift.

## [0.7.0] — 2026-09-03

### Fixed

- **A declared `order_by` was asserted against nothing.** Synthesis created at most one instance
  per scenario and a `ranked` expectation passes on fewer than two rows by design, so every
  ordering assertion a suite emitted was a no-op: an author declared an order, saw scenarios
  generated, saw them pass, and was covered by nothing. Scenarios that assert an order now arrange
  a second row through declared command outcomes, differing on the ranking keys. Measured on an
  adopter's model: 11 ordering assertions, 0 of them running against two rows before, 11 after.
  Where a second row cannot be arranged the order is refused (`ESS-SYNTH-014`) and not asserted.

### Changed

- **`ess-conformance/2`.** The expectation vocabulary grew, and the emitted Go runner reports an
  expectation it does not know as a *failed scenario* — a wrong verdict about an implementation,
  caused by the age of the tool reading the suite. A reader that checks the format first refuses
  the document instead. Both `1` and `2` are read: a `1` suite means in `2` exactly what it meant.

### Added

- A `counts` view expectation, asserting how many rows a view holds. Synthesis writes only a floor,
  and only from two up: a target may be shared, so "exactly N" is a claim about every other user of
  it, and "at least one" is what `contains` already says. Both bounds are in the vocabulary.
- An `at` view expectation, asserting what sits at a position. Both runners read it and both refuse
  it on a view that declares no order. **Synthesis does not yet emit it**: nothing in the model
  relates a command's input to the field a view ranks by, so choosing a position would be matching
  on a shared field name. The construct that would license one is named in the gap table.

## [0.6.0] — 2026-09-03

### Changed

- **`--kind site` writes a website, not inputs for one.** It used to emit markdown with frontmatter
  and a sidebar list, leaving an adopter to run a static site generator over it — measured on the
  one that did: `npm ci` plus a webpack build peaking at 3931 MiB, killed twice by a CI runner, to
  render seven pages of prose that were already written. It now writes the pages as HTML with
  navigation, a stylesheet and a diagram renderer beside them, so publishing a specification is one
  command and needs no Node. **An adopter consuming the old markdown output must change.**
- The documentation projection is built as an `ess-docs/1` document and rendered, rather than
  written as markdown directly. The markdown a repository reads is unchanged, byte for byte, and
  the documentation of every example is now pinned and compared to keep it so. The point of the
  layer is that the site renderer reads the same document instead of parsing the markdown back —
  which is what it had to do before, scanning for a heading to recover a title it had just written.
- The site's stylesheet and its vendored Mermaid bundle stay out of the committed projection
  sample. Neither derives from a specification, and the bundle alone is 3.5 MB.

### Added

- `--kind docs-ir` writes the `ess-docs/1` document as JSON, so a presentation layer of your own
  needs nothing from this crate.
- `--kind site` opens on the `README.md` beside the specification, and takes
  `--include <page-id>=<path>` for any number of pages beside the generated ones — a plan board
  another tool rendered, a runbook. Both are markdown somebody wrote, read into the document and
  styled like every other page. Raw HTML in them is dropped rather than passed through: a generated
  site that embedded it would inherit that markdown's scripts and its styling.

## [0.5.1] — 2026-09-03

### Fixed

- **The browser lab runs again.** `CreateInvoice` took an `account_id` from 0.5.0 and the lab's
  script did not send one, so the module refused the first command of the run —
  `{"kind":"undecodable","at":"input.account_id","expected":"a value","found":"nothing"}` — and
  `task site-build` failed after the tag was cut. The script now names the account its invoices
  belong to, which is the caller's to name because `billing.invoice.Account` declares it `owns`
  them `via account_id`.
- The lab read the identity for the next command out of the *first* entity in the catalogue, which
  was the invoice only for as long as the invoice was the only entity. `billing.invoice.Account` is
  declared above it, so `InvoiceCreated`'s `account_id` was taken for the new invoice's id and the
  next command was issued against an account. It now reads the identity of the entity the taken
  outcome `creates:`.
- `website/src/pages/lab/_source.ts`, the lab's committed copy of `examples/billing/domains/invoice.yaml`,
  was not refreshed for 0.5.0, so every line the left panel highlighted was off by the relation's
  own lines. Refreshed — and a stale copy is now a red gate rather than a wrong panel:
  `task site-lab` compares the two.
- `examples/billing-web/smoke.mjs` asserted the pre-relation input list and sent the pre-relation
  input; it is run by no task, which is why nothing caught it. Same fix, and it passes.
- The documentation pages that quote the command were carrying its old shape:
  `specification-to-contracts.md` said three values are chosen where there are now four and quoted
  a JSON Schema "in full" that was missing `account_id`, and `write-a-specification.md` showed the
  payload block without its first line. Both are the generated files again.

## [0.5.0] — 2026-09-03

- **An entity declares what it owns and what it references.** `relations:` on an entity names a
  relation, its kind (`owns` or `references`), the entity at the other end, how many of it there
  are (`one` or `many`), and the field that carries it. It is declared on the source and nowhere
  else: the reverse direction is a lookup, and a second declaration is a second thing to keep in
  step. Previously an ownership was a typed id field on the child plus an invariant somebody
  remembered to write, which is prose to every projection and refusable by nothing.
- `ess validate` refuses five things a relation can get wrong: a target that is not a declared
  entity, a `via` field the carrying entity does not have, a `via` field typed as anything but the
  identity the design requires — `Optional<…>` or `List<…>` where the cardinality says so — a second
  entity claiming to own one that is already owned, and two relations claiming one field. Each is an
  existing `ValidationCode` with a hint naming what to write instead, and each has a test that
  breaks it on purpose.
- **One extension key, `x-ess-relation`, carries a relation into every projection**, on the property
  that carries the field. The JSON Schema projection gains a document per entity,
  `schema/entities/<name>.schema.json`, because an entity was previously rendered by the
  documentation and by nothing a tool reads; the `OpenAPI` projection publishes the same shapes under a root
  extension, `x-ess-entities` — no path, no method, no query parameter — and adds a `$ref` to the
  schema of what the property identifies, which that document has and a self-contained schema
  document does not. An extension rather than `components.schemas` because that table is what
  `ess import openapi` reads back, and an entity's shape reaches a `Map` and a tagged union, which
  the adapter's subset does not carry: publishing them there made this repository's own adapter
  refuse the document it had generated. The synthesised Rust names the relation in the carrying field's doc comment rather than as a
  typed field: the specification describes, and nothing generated here has a store to navigate with.
- `examples/billing` carries one of them — `billing.invoice.Account` owns many
  `billing.invoice.Invoice`, by the invoice's `account_id` — so the pattern an adopter is pointed at
  is validated, compiled and projected rather than described. `CreateInvoice` takes the account and
  `InvoiceCreated` announces it, because an owner nobody named is an owner the implementation
  invented.
- The committed `generated/` tree is regenerated, which also lands 0.4.0's documentation filenames:
  `docs/index.md` and `docs/domains/billing-invoice.md` replace the `README.md` and dotted names the
  tree still carried. `schemas/generated/ess.schema.json` is regenerated from the Rust types, which
  additionally publishes the `order_by:` and quantifier constructs 0.4.0 added and the file had not
  caught up with.

## [0.4.0] — 2026-09-02

- **The documentation projection writes different filenames.** `--kind docs` now writes `index.md`
  rather than `README.md`, and `domains/acd-routing.md` rather than `domains/acd.routing.md`: a dot
  in a path segment makes a static file server read the name as having an extension, so every one
  of those pages was unservable behind GitLab Pages and an adopter carried a rename pass of their
  own. Committed output moves once; the links inside it move with it.
- Add `forall:` and `exists:` to the predicate language, quantifying an invariant over a `List` or
  a `Map`. A collection publishes its size as `<path>.count`, so no existing fact source changes to
  gain them, and one nobody observed evaluates to `unknown` rather than to the vacuous truth an
  empty one gives. Quantifying over anything that is not a collection is refused as
  `type_mismatch`.
- Add `order_by:` to a view, so a view named for a position says something about position.
  Generated conformance scenarios assert it on adjacent rows; a key the view does not project is
  refused, because a rank over a field nobody can read is a promise nothing can check.
- Add `ess conform synthesize --target go`, which writes a Go test package — the runner, a
  three-valued predicate evaluator and the suite — so an implementation in Go is held to the
  specification by `go test`. Previously `conform run` reached only the reference targets in this
  workspace, and an adopter's suite was regenerated on every model change and never executed.
- Add `refs:` to commands, outcomes, bindings and components, written `provider:key`, so a
  construct can name the ticket or incident that explains it. `Conversion.because` was the only
  prose field in the model, and everything else went into YAML comments no projection reads.
- Add `ess generate --kind site`: the documentation with frontmatter and a sidebar, for publishing
  as a static site.
- Print the refusals `conform synthesize` finds instead of counting them. A count says something is
  unchecked without saying what.
- Introduce `ess-docs/1`, an internal document representation between the model and the pages, with
  the markdown projection as one renderer of it. No output changes; the documentation of every
  example is pinned byte for byte and compared.

## [0.3.0] — 2026-09-01

- Add reusable named struct `shape` declarations for views, preserving expanded checked fields in
  compiler IR while OpenAPI reuses one component schema through `$ref`.
- Expose the compiler-owned semantic source digest on `EssIr`, so downstream builders and
  provenance records bind to one canonical implementation.
- Expose the exact versioned browser catalog through `ess_synth::web::browser_catalog`, so service
  generators and documentation hosts consume the same document as the ESS browser target.

## [0.2.1] — 2026-09-01

- Restore the extracted `schema-contract` command surface under `ess schema`: offline validation
  against an explicit schema registry and deterministic TypeScript projection with byte-check mode.
- Document and gate both commands so consumers no longer depend on the retired repository's CLI.

## [0.2.0] — 2026-09-01

- Make `ess-conformance-report/1` the only ESS conformance handoff and remove the legacy
  workflow-shaped evidence, producer, provenance, and verifier API. AEP adaptation now lives only
  in AEP's optional adapter.
- Use the canonical `ess` command in newly generated provenance guidance and current engineering
  documentation. Persisted ESS and infrastructure IR envelopes remain version 1.

## [0.1.1] — 2026-09-01

- Publish the extracted ESS adopter documentation and browser lab at the standalone ESS Pages
  site, with commands, links, and regeneration guidance written for the canonical `ess` CLI.

## [0.1.0] — 2026-09-01

- Extract ESS, infrastructure modeling, schema contracts, generators, conformance, examples, and
  suites into a standalone repository with no AEP dependency.
- Add the canonical `ess` command and explicit import/project adapter contract.
- Add typed OpenAPI service-interface import, deterministic projection, coverage reporting, and
  semantic round-trip checks for the adapter's declared subset.
- Import the Kubernetes credential-edge scanner with pre-write Secret sanitization.
- Publish `ess-conformance-report/1` for optional workflow-side evidence adapters.
