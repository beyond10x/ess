# Retrofit wave 1 — proposal (stage 1)

Epic: `epic:retrofit-findings-20260927` (beyond10x/ess#132–#157). Base: `origin/main` `be44a3365`
(0.36.0). Status: **approved 2026-09-27** ("do it /aep:wave"). Story moved draft -> proposed -> active after `serves vision:O2`.

## Proposed unit (N = 1)

| unit | issues | objective | scope | package |
|---|---|---|---|---|
| `story:synthesis-kills-connective-and-source-mutants` | #154, #155, #132 | `serves vision:O2` (to be added before `proposed`) | cited: `crates/verify/ess-conformance/src/synthesize.rs`, `src/synthesize/subject_fact.rs`; inferred: `src/witness.rs`, `tests/mutation_audit.rs`, `ess-compiler` `ir.rs`/`resolve.rs` | `ess-conformance` |

Why one: it is the only retrofit story that needs no source format, no suite format, no Go or
TypeScript runtime change and no network, and its cited scope is one crate.

## Left out

| story | why |
|---|---|
| `story:field-names-wire-and-value-types` | six issues; #143/#138 fan out over ~30 `TypeRef::Map` and 13 `Primitive` match sites; #142 is mostly built (`types.rs:311-322`). Split before a wave. |
| `story:external-mutation-explorer-and-toolchain` | three unrelated tools; #147 downloads releases (network, not implementable offline tonight). Split before a wave. |
| `story:outcome-shapes-beyond-ess-14` | five constructs, none designed; new source format; `ResolvedEffect` ripples to 16 files. Design page first. |
| `story:subject-guard-input-and-case-folding` | new source and suite format; Go and TypeScript evaluators; collides with the unit on `synthesize.rs`/`subject_fact.rs`. |
| `story:aggregates-over-optional-fields` | format question open (`skip_absent`); collides on `witness.rs`. |

## `aep plan artifact waves --kind story --status draft --format json`

Selection path: computed by the verb (aep 0.60.0 installed; `aep --version`), then judged. The
verb covers every draft story in the store; the full output is `retrofit-wave-1.waves.json` beside
this page. It placed the six retrofit stories in waves 2 (`field-names-wire-and-value-types`,
`external-mutation-explorer-and-toolchain`), 3 (`aggregates-over-optional-fields`), 4
(`outcome-shapes-beyond-ess-14`), 5 (`subject-guard-input-and-case-folding`) and 6
(`synthesis-kills-connective-and-source-mutants`). Cycles: none.

Unassessed (verbatim): `story:a-branch-may-clear-the-field-it-owns`,
`story:a-report-says-why-a-scenario-was-skipped`, `story:adopter-reviewed-delta`,
`story:change-fragment-upgrade-obligation`, `story:count-guards-above-one-are-synthesized`,
`story:deleting-a-scratch-tmpdir-breaks-sccache-for-every-other-agent`,
`story:specify-upgrade-command`.

Collisions involving the retrofit stories (verbatim from the verb, 73 rows):

| a | b | path | confidence |
|---|---|---|---|
| `story:a-field-constrained-by-a-charset-publishes-that-charset` | `story:aggregates-over-optional-fields` | `schemas/generated/ess.schema.json` | inferred |
| `story:a-field-constrained-by-a-charset-publishes-that-charset` | `story:field-names-wire-and-value-types` | `schemas/generated/ess.schema.json` | inferred |
| `story:a-field-constrained-by-a-charset-publishes-that-charset` | `story:outcome-shapes-beyond-ess-14` | `schemas/generated/ess.schema.json` | inferred |
| `story:a-field-constrained-by-a-charset-publishes-that-charset` | `story:subject-guard-input-and-case-folding` | `schemas/generated/ess.schema.json` | inferred |
| `story:a-refusal-records-the-document-it-was-read-from` | `story:aggregates-over-optional-fields` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:a-refusal-records-the-document-it-was-read-from` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:a-refusal-records-the-document-it-was-read-from` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:a-wrong-trailing-key-guess-is-reported-as-a-line` | `story:aggregates-over-optional-fields` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:a-wrong-trailing-key-guess-is-reported-as-a-line` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:a-wrong-trailing-key-guess-is-reported-as-a-line` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:field-names-wire-and-value-types` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:field-names-wire-and-value-types` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:field-names-wire-and-value-types` | `crates/verify/ess-conformance/src/witness.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:field-names-wire-and-value-types` | `schemas/generated/ess.schema.json` | inferred |
| `story:aggregates-over-optional-fields` | `story:interpreted-eventual-views` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:interpreted-eventual-views` | `crates/specify/ess-domain/src/view.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:outcome-shapes-beyond-ess-14` | `crates/verify/ess-diff/src/diff.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:outcome-shapes-beyond-ess-14` | `schemas/generated/ess.schema.json` | inferred |
| `story:aggregates-over-optional-fields` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:subject-guard-input-and-case-folding` | `crates/verify/ess-conformance/src/witness.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:subject-guard-input-and-case-folding` | `schemas/generated/ess.schema.json` | inferred |
| `story:aggregates-over-optional-fields` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:aggregates-over-optional-fields` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/witness.rs` | inferred |
| `story:crosswalk-verb-external-names-held-to-declarations` | `story:external-mutation-explorer-and-toolchain` | `crates/edge/ess-cli/Cargo.toml` | inferred |
| `story:crosswalk-verb-external-names-held-to-declarations` | `story:external-mutation-explorer-and-toolchain` | `crates/edge/ess-cli/src/main.rs` | inferred |
| `story:crosswalk-verb-external-names-held-to-declarations` | `story:field-names-wire-and-value-types` | `crates/specify/ess-domain/src/name.rs` | inferred |
| `story:external-mutation-explorer-and-toolchain` | `story:integrate-source-driven-realizations` | `crates/edge/ess-cli/src/main.rs` | cited |
| `story:external-mutation-explorer-and-toolchain` | `story:interpreted-trust-gate` | `crates/edge/ess-cli/src/main.rs` | inferred |
| `story:external-mutation-explorer-and-toolchain` | `story:outcome-shapes-beyond-ess-14` | `crates/verify/ess-conformance/src/go/explore.go` | cited |
| `story:external-mutation-explorer-and-toolchain` | `story:outcome-shapes-beyond-ess-14` | `crates/verify/ess-conformance/src/ts/explore.ts` | cited |
| `story:external-mutation-explorer-and-toolchain` | `story:subject-guard-input-and-case-folding` | `crates/verify/ess-conformance/src/go/runtime.go` | inferred |
| `story:external-mutation-explorer-and-toolchain` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/tests/mutation_audit.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:integrate-source-driven-realizations` | `CHANGELOG.md` | inferred |
| `story:field-names-wire-and-value-types` | `story:interpreted-eventual-views` | `crates/specify/ess-compiler/src/ir.rs` | cited |
| `story:field-names-wire-and-value-types` | `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:outcome-decided-by-environment` | `crates/specify/ess-domain/src/command.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:outcome-shapes-beyond-ess-14` | `CHANGELOG.md` | inferred |
| `story:field-names-wire-and-value-types` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-domain/src/command.rs` | cited |
| `story:field-names-wire-and-value-types` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:outcome-shapes-beyond-ess-14` | `schemas/generated/ess.schema.json` | inferred |
| `story:field-names-wire-and-value-types` | `story:subject-guard-input-and-case-folding` | `CHANGELOG.md` | inferred |
| `story:field-names-wire-and-value-types` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-domain/src/primitive_admission.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-domain/src/system.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:subject-guard-input-and-case-folding` | `crates/verify/ess-conformance/src/witness.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:subject-guard-input-and-case-folding` | `schemas/generated/ess.schema.json` | inferred |
| `story:field-names-wire-and-value-types` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:field-names-wire-and-value-types` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/witness.rs` | inferred |
| `story:integrate-source-driven-realizations` | `story:outcome-shapes-beyond-ess-14` | `CHANGELOG.md` | inferred |
| `story:integrate-source-driven-realizations` | `story:subject-guard-input-and-case-folding` | `CHANGELOG.md` | inferred |
| `story:interpreted-eventual-views` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:interpreted-eventual-views` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-primitives/src/predicate.rs` | inferred |
| `story:interpreted-eventual-views` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:interpreted-scenario-supplied-facts` | `story:subject-guard-input-and-case-folding` | `crates/verify/ess-conformance/src/scenario.rs` | cited |
| `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-domain/src/system.rs` | cited |
| `story:one-name-held-by-two-kinds-is-refused-whether-or-not-it-converts` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-domain/src/system.rs` | cited |
| `story:outcome-decided-by-environment` | `story:outcome-shapes-beyond-ess-14` | `crates/specify/ess-domain/src/command.rs` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:subject-guard-input-and-case-folding` | `CHANGELOG.md` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:subject-guard-input-and-case-folding` | `crates/generate/ess-entity-runtime/src/lib.rs` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:subject-guard-input-and-case-folding` | `crates/specify/ess-domain/src/system.rs` | cited |
| `story:outcome-shapes-beyond-ess-14` | `story:subject-guard-input-and-case-folding` | `crates/verify/ess-conformance/src/synthesize.rs` | cited |
| `story:outcome-shapes-beyond-ess-14` | `story:subject-guard-input-and-case-folding` | `schemas/generated/ess.schema.json` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/ir.rs` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:synthesis-kills-connective-and-source-mutants` | `crates/specify/ess-compiler/src/resolve.rs` | inferred |
| `story:outcome-shapes-beyond-ess-14` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/synthesize.rs` | cited |
| `story:subject-guard-input-and-case-folding` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/synthesize.rs` | cited |
| `story:subject-guard-input-and-case-folding` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` | cited |
| `story:subject-guard-input-and-case-folding` | `story:synthesis-kills-connective-and-source-mutants` | `crates/verify/ess-conformance/src/witness.rs` | inferred |

## Execution on approval

| item | value |
|---|---|
| implementor | `aep:implementor` |
| integration checkout | managed `ess-expressions-20260927`, branch `integrate/retrofit-wave-1` |
| unit worktree | managed `ess-wave1-synthesis`, `<worktrees>/ess-wave1-synthesis`, branch `impl/synthesis-kills-connective-and-source-mutants` |
| unit scratch | `<cache>/ess-wave1/synthesis/scratch` |
| unit stage | dispatched |
| adversary | `aep:adversary` after green |
| unit build dir | `<cache>/b10x-target/ess-expr` (warm; used by this unit only while it runs, then by the integration gate) |
| gate | `task check` once on the integration branch, then CI |
| branch | `integrate/retrofit-wave-1` |

Commits approval authorises: this page and the scope records (1 plan commit), 1 unit commit, the
merge into `integrate/retrofit-wave-1`, 1 closing store commit, the merge to `main`. No tag, no
release.
