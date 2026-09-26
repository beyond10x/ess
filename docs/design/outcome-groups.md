# Outcome groups: one outcome declared once for many commands

Status: proposed (beyond10x/ess#105, wave 2 design unit E8d). This page changes no source, IR or
format. A later unit implements exactly what it says. Every claim about current code cites the tree
it was written against (integration base `4ffcb537c40`, branch `integrate/ess-issues-20260926`). A
line marked *inferred* was not measured.

## Behavior and authority

A consumer specification has 28 commands, and a remote service carries out each one on the caller's
behalf. Any of them can end the same way: the service rejects the session credential. Today that
outcome has to be copied into each command:

```yaml
- name: credential-rejected          # copy 1 of 28
  external: the service rejects the session credential
  error: session.Unauthenticated
```

Outcomes can be declared only on a command. `RawActorSpec` has three keys and refuses any other
(`crates/specify/ess-domain/src/actor.rs:117-133`, `deny_unknown_fields`), which is why the issue's
`actors[].outcomes` is refused with `unknown field outcomes`. Domains have no member declarations
of their own either (`RawSpecFile`, `crates/specify/ess-domain/src/spec.rs:43-112`).

The operator settled both of the issue's open questions (wave-2 design brief, E8d row). This page
records the answers and does not reopen them:

1. **Membership is either an explicit command list or a selector, `actor:` or `domain:`.** Both
   forms exist.
2. **A member command that declares an outcome of the same name is refused.** Nothing is silently
   overridden. The one exception is a command the group names in an explicit `except:` list, which
   the group then skips.
3. **Expansion happens before validation.** Synthesis, conformance and every generator see ordinary
   per-command outcomes.

### Where "before validation" is in this tree

The brief says "in the compiler". The pipeline has one production entry point, `ess-cli`'s
`load::specification` (`crates/edge/ess-cli/src/load.rs:26-80`):

1. `RawSpecFile::parse` reads each file (`:39`).
2. `Specification::assemble` absorbs the files and runs `validate_after` (`:57`;
   `spec.rs:219-260`). Absorbing converts each `RawCommandSpec` into a `CommandSpec`
   (`spec.rs:895-921`), and that conversion already runs the outcome-set checks:
   `EmptyDeclaration` (`command.rs:1463`), duplicate outcome names (`command.rs:1487-1503`) and
   branch coverage (`command.rs:1892`).
3. `ess_compiler::compile` validates again and resolves (`load.rs:69`; `compile_locating`,
   `crates/specify/ess-compiler/src/resolve.rs:979-996`).

"Before validation" therefore means **before any `RawCommandSpec` is converted**. That is the start
of `Specification::assemble`, in `ess-domain`, and `ess-compiler` has no earlier stage. **Decision:**
expansion is a pass from raw to raw, run as the first statement of `Specification::assemble`, over
every parsed file at once. An expanded command reaches `CommandSpec::try_from` looking exactly like
one where the author copied the outcome by hand, so every later check is the check the hand copy
would get. No other production caller of `assemble` exists: a search for
`Specification::assemble` outside `tests/` finds only `load.rs:57` and test helpers
(`ess-diff/src/impact.rs:1510`, `ess-synth/src/accessor_output.rs:79`,
`ess-conformance/src/coverage_build.rs:546`). Each of those sits after the file's `#[cfg(test)]` line
(`impact.rs:1316`, `accessor_output.rs:60`, `coverage_build.rs:507`).

## Syntax

`outcome_groups:` is a new top-level key of `RawSpecFile`. It sits above the domains, like
`bindings:` and `components:` (`spec.rs:929-935`), so it can appear in any file, with or without a
`domain:` key.

```yaml
format: ess/12
outcome_groups:
  - name: remote-backed
    actor: calls.Agent              # or  commands: [calls.Hold, calls.Resume]
    except: [calls.Park]            # only beside actor: or domain:
    outcomes:
      - name: credential-rejected
        external: the service rejects the session credential
        error: session.Unauthenticated
        summary: The remote service refused the session.
        refs: [issue:beyond10x/ess#105]
```

| key | type | meaning |
|---|---|---|
| `name` | `OutcomeGroupName`: the same pattern as `BindingName` and `OutcomeName`, `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` (`binding.rs:742-760`) | the group's identity, unique across the specification |
| `commands` | `Option<BTreeSet<QualifiedName>>` | explicit membership, fully qualified names |
| `actor` | `Option<QualifiedName>` | every command in that actor's `may:` |
| `domain` | `Option<QualifiedName>` | every command declared in a file whose `domain:` is this name |
| `except` | `BTreeSet<QualifiedName>`, default empty | commands the selector would include and the group skips |
| `outcomes` | `Vec<RawGroupOutcome>`, default empty | the outcomes each member gains |

`RawGroupOutcome` has exactly five keys: `name` (`OutcomeName`), `external` (`String`, required),
`error` (`QualifiedName`, required), `summary` (`Option<String>`) and `refs` (`Refs`). Both
structs are `deny_unknown_fields`.

Why each shape is what it is:

- **Exactly one of `commands`, `actor` or `domain` per group.** A union of two forms and an
  intersection of two forms are both plausible readings, and the page would have to pick one
  arbitrarily. Two groups with the same outcomes express a union. An intersection has no use case
  in the issue.
- **Sets, not lists**, for the reason `ActorSpec::may` gives (`actor.rs:52-56`): the same member
  written twice means the same thing, and set iteration order is stable.
- **`commands` is an `Option`**, so an empty `commands: []` is a refusal (G9) and not the same
  document as leaving the key out (G2).
- **Qualified names only.** A group belongs to no domain, so there is no namespace in which a
  one-segment name could be resolved.
- **The outcome is always an external refusal: `external:` plus `error:`, and nothing else.** Every
  other outcome key depends on the command it lands in:
  - `when` reads that command's input.
  - `wrong_state`, `when_subject*` and `when_state_changes` read that command's subject and
    transitions.
  - `creates`, `moves`, `updates`, `preserves`, `instance`, `sets`, `payload` and `replays` name
    that command's entities, fields or sibling outcomes.
  - `emits` needs a payload source per event field (ess/4 and later).

  An outcome copied unchanged into 28 commands has to mean the same thing in each. Of the keys that
  exist, only an external cause and a declared error do. It is also the issue's case.
  `external:` is required because without a condition the outcome would be a second default branch
  in every member (`ConflictingDeclaration`, `command.rs:169`). `error:` is required because an
  external outcome with no event and no error is `EmptyChange` (`command.rs:170`), and one with an
  event needs a payload.
- **A shape error is a reader error**, not a validation code: an unknown key, a missing `external:`
  or `error:`, or a malformed name. This is how every other unknown key in the format fails today,
  and it names the key and the admitted keys. It is the `aggregate-views.md:142-147` precedent.
  `outcome_groups:` keeps no key for later growth. A later shape adds its keys under a later
  format.

## Expansion

`outcome_group::expand(files: &mut [(Source, RawSpecFile)], errors: &mut ValidationErrors)` is the
new module `crates/specify/ess-domain/src/outcome_group.rs`. `Specification::assemble` collects its
`files` argument into a `Vec`, calls `expand`, and then absorbs the files as today. `expand` takes
`outcome_groups` out of each file with `std::mem::take`, so `absorb` never sees the key.

**Inputs it reads**, all from the raw files and before any conversion:

- The declared command names, and for each one the `domain:` of the file that declares it.
- The declared error names.
- Each actor's `may:`. When an actor is declared twice, `expand` reads the first declaration in
  source order. Actor conversion cannot fail (`actor.rs:138-146`), so that first declaration is the
  one `record` keeps (`spec.rs:619-623`). The duplicate itself is refused by `declare` later
  (`spec.rs:577-597`).
- The header format: `format:` of the single file that carries `system:`, defaulting to `ess/1` as
  `absorb_header` does (`spec.rs:772`). With no header, or two headers, the merge already
  refuses the specification, and the gate G16 is skipped.

**Membership:**

| form | members |
|---|---|
| `commands: S` | `S` |
| `actor: A` | `A`'s `may:` ∩ declared commands, minus `except` |
| `domain: D` | commands declared in a file whose `domain:` is `D`, minus `except` |

- **`may:` entries that name no command are skipped without a second report.**
  `ActorSpec::validate` already reports each one (`actor.rs:79-110`).
- **`domain:` means the declaring file's `domain:`**, which is ownership as `DomainMembers` records
  it (`spec.rs:627-634`, filled at `:895-921`). It does not mean the name prefix. Domains do not nest
  (`domain.rs:283-298`), and a member must lie inside its domain's namespace (`domain.rs:208-235`),
  so in a valid specification the two readings select the same commands. In an invalid one, the
  misplaced command is refused on its own account either way.

**What each member gains.** Every member command gains, in this order and after its own outcomes,
the group's outcomes in written order. Each becomes a `RawOutcome` with `name`, `external: Some`,
`error: Some`, `summary` and `refs` copied, and every other key at its default (`command.rs:2995-3107`).
When several groups select one command, their outcomes are appended in **ascending group name**
order.

- **Appended, not prepended.** A command's own outcome indices do not move. The IR's replay handle
  is an index into the same command's outcomes (`crates/specify/ess-compiler/src/ir.rs:735`,
  `:995`), and outcomes are an ordered `Vec` all the way to the IR (`ir.rs:910`).
- **By group name, not by file order.** The digest is of the IR, so that "two source trees that
  differ only in comments and file layout mean the same system"
  (`crates/generate/ess-gen/src/provenance.rs:28-33`). File order would let renaming a file reorder
  the outcomes and move the digest.

**The equivalence this buys.** A specification with a group, and the same specification with each
expanded outcome written by hand at the end of each member's `outcomes:` in the order above,
compile to **byte-identical canonical IR** (`EssIr::to_canonical_json`, `ir.rs:2057`). Their suites
are therefore byte-identical too. The IR carries no source format and no group, so nothing else
could differ. This is the unit's deciding check (C1 below).

**A group with any refusal of its own (G1–G13) is not expanded into any command.** A group with an
undeclared `error:` expanded into 28 commands would be refused 28 times more by
`CommandSpec::validate`. Not expanding keeps its own refusal the only one. This is the same
reasoning as `Refused` (`spec.rs:710-727`). G14 and G15 apply per command: the command in question
gets **none** of that group's outcomes, and the group's other members are expanded. G16 does not
stop expansion. A format gate refuses the version and still lets the rest of the document be
checked, as every gate in `primitive_admission.rs` does.

## Validation and stable codes

The codes are `ess-domain` `ValidationCode`s. Every location starts with the head `outcome_groups`.
`family_of` (`resolve.rs:778-798`) gains one arm, `"outcome_group" => codes::family::COMMAND`. The
head `outcome_groups` becomes `outcome_group` after `trim_end_matches('s')`, the same way
`entities` becomes `entitie`. `class_of` (`resolve.rs:804-848`) is unchanged, so each stable code
below follows from the pair.

**Decision: the `COMMAND` family, not a new one.** A group is a declaration of command outcomes. A
reader filtering on `ESS-COMMAND-` finds a group's refusals beside the refusals of the outcomes it
produces. A new family would need a new `ConstructKind` in `ess-primitives`. That enum is
deliberately exhaustive (`crates/specify/ess-primitives/src/error.rs:450-498`) and mirrors
`codes::family::ALL` (`resolve.rs:186-189`). That is a published-vocabulary change the construct
does not need.

**Decision: string locations, not typed sites.** `ValidationError::at` takes a `ConstructRef`,
whose `ConstructKind` has no group kind. Rendering a group as `command.<group>…` would name a
command that does not exist. So the refusals use `ValidationError::new`, and the bridge locates them
through `needles_for`. `STRUCTURAL` (`resolve.rs:854-907`) gains `"except"`. Without it,
`outcome_groups.g.except` would search for `name: g.except` rather than `name: g`. No existing
location uses an `except` segment (search: `rg '\bexcept\b' crates/specify`, doc comments only).
Every location below then ends at the group or at a structural key, so the needle is
`name: <group>` (`needles_from_tokens`, `resolve.rs:935-963`).

| # | written | `ValidationCode` | location | stable code |
|---|---|---|---|---|
| G1 | a group name declared twice, in any files. The message uses `declare`'s wording (`spec.rs:590-591`) and names the second source. The later copy is not expanded | `DuplicateDeclaration` | `outcome_groups.<g>` | `ESS-COMMAND-006` |
| G2 | none of `commands`, `actor` or `domain` | `MissingDeclaration` | `outcome_groups.<g>` | `ESS-COMMAND-005` |
| G3 | more than one of them. The message names the keys written | `ConflictingDeclaration` | `outcome_groups.<g>` | `ESS-COMMAND-004` |
| G4 | a `commands` entry that no source declares as a command. The hint lists the declared commands, as `actor.rs:95-107` does | `UndeclaredReference` | `outcome_groups.<g>.commands` | `ESS-COMMAND-001` |
| G5 | `actor:` names no declared actor | `UndeclaredReference` | `outcome_groups.<g>` | `ESS-COMMAND-001` |
| G6 | `domain:` names a domain no source's `domain:` contributes | `UndeclaredReference` | `outcome_groups.<g>` | `ESS-COMMAND-001` |
| G7 | `except:` beside `commands:`. Hint: "leave the command out of `commands:`" | `ConflictingDeclaration` | `outcome_groups.<g>.except` | `ESS-COMMAND-004` |
| G8a | an `except` entry that no source declares as a command | `UndeclaredReference` | `outcome_groups.<g>.except` | `ESS-COMMAND-001` |
| G8b | an `except` entry the selector does not select. It excludes nothing, and would silently start excluding if the actor were later granted that command | `ConflictingDeclaration` | `outcome_groups.<g>.except` | `ESS-COMMAND-004` |
| G9 | the membership is empty after `except`: `commands: []`, an actor with no declared grant, a domain with no command, or everything excepted | `EmptyDeclaration` | `outcome_groups.<g>` | `ESS-COMMAND-007` |
| G10 | `outcomes` empty or absent | `EmptyDeclaration` | `outcome_groups.<g>.outcomes` | `ESS-COMMAND-007` |
| G11 | two outcomes of one group with the same name | `DuplicateDeclaration` | `outcome_groups.<g>.outcomes.<o>` | `ESS-COMMAND-006` |
| G12 | `external:` empty after `trim`, the same test as `command.rs:1634-1646` | `UnexplainedDecision` | `outcome_groups.<g>.outcomes.<o>.external` | `ESS-COMMAND-012` (`class_of` has no arm for it, the same as the command rule it mirrors) |
| G13 | `error:` names no declared error | `UndeclaredReference` | `outcome_groups.<g>.outcomes.<o>.error` | `ESS-COMMAND-001` |
| G14 | a member command already declares an outcome named `<o>`. One refusal per (group, outcome, command). The message names the command. The hint says "rename one of the two, drop the command's own, or list `<command>` under `except:`". Under `commands:` the last option reads "leave `<command>` out of `commands:`" | `DuplicateDeclaration` | `outcome_groups.<g>.outcomes.<o>` | `ESS-COMMAND-006` |
| G15 | two groups would give one command outcomes of the same name. Reported once, at the group later in name order, naming the earlier group and the command. Neither group expands into that command | `DuplicateDeclaration` | `outcome_groups.<later>.outcomes.<o>` | `ESS-COMMAND-006` |
| G16 | any group in a specification whose header format is below `ess/12`. Message: "outcome groups require specification format ess/12". One refusal per group | `UnsupportedFormatVersion` | `outcome_groups.<g>` | `ESS-COMMAND-009` |

Why each rule is shaped this way:

- **G14 uses the code the command itself would use.** It is the refusal a hand copy gets for two
  outcomes of one name (`command.rs:1487-1503`, `DuplicateDeclaration`). The group reports it once,
  at the place the author wrote the second copy, and does not also expand. Otherwise the command
  would report it a second time.
- **G7: `except:` is for selectors only.** With an explicit list, a command both listed and
  excepted says two opposite things. Leaving it out of the list is the one way to write that.
- **G8b refuses an exception that excludes nothing.** Otherwise it would silently start excluding
  when the selection grows. That is the same class as a grant that grants nothing
  (`actor.rs:68-75`).
- **G15 refuses both groups for that command** rather than letting name order pick a winner. A
  winner picked by name order is a silent override, which is what decision 2 rules out.
- **The group adds no rule about its members' outcome sets.** A member whose own outcomes are all
  external, or that has none, is refused by the command's own `UnreachableBranch`
  (`command.rs:2038-2056`) at the command's own location. That is what the hand copy gets, and the
  fault is in that command.

**The typed-diagnostics inventory grows.** `crates/specify/ess-compiler/tests/typed_diagnostics.rs`
reads the census on `docs/design/review-typed-diagnostics.md:245-322`. A file with
`ValidationError` sites that is missing from the census fails that test. The unit adds
`outcome_group.rs <n> 0` to the file block and `outcome_groups outcome_group.rs <n>` to the heads
block, with `<n>` being its `ValidationError::new` count, plus the spec.rs line if the count there
changes. The census is the machine check for the class. No hand list is kept here.

## Formats

| family | moves? | why |
|---|---|---|
| `ess/` (authored) | **yes, to `ess/12`**: `FormatVersion::V12` beside `V11` (`crates/specify/ess-domain/src/system.rs:89`), `SUPPORTED_FORMATS` gains `12` (`system.rs:53`), gate G16 | a new top-level key. A build older than `ess/12` refuses the header version. The same build reading `outcome_groups:` under an older header fails with `unknown field outcome_groups` |
| document schema | regenerated: `schemas/generated/ess.schema.json` gains `outcome_groups`, `RawOutcomeGroup` and `RawGroupOutcome`. Run `cargo xtask schema`; `projection-check` is the only thing that notices a stale schema (`AGENTS.md`, "A change to `RawSpecFile`…") | — |
| compiled IR | no number exists, no change | expansion leaves nothing behind. A model without groups keeps its bytes and `source_digest` |
| `ess-conformance/` (suite) | **no** | the suite sees ordinary external outcomes, `ScenarioId::Outcome` (`crates/verify/ess-conformance/src/scenario.rs:537-546`) |
| `ess-diff/` | **no** | the diff compares IR. Adding a group reads as `outcome-added` on each member (`crates/verify/ess-diff/src/change.rs:2068`, `:2209`). A group and its hand copy diff empty |
| `infra-spec/1` | no | does not read `RawSpecFile` |

**Numbers taken:** `ess/12` only. At base, the maxima are `ess/10` (`system.rs:53`),
`ess-conformance/17` (`scenario.rs:373-374`) and `ess-diff/7` (`crates/verify/ess-diff/src/delta.rs:13`).
Unit E6 took `ess/11` (string alphabets, input examples and `.count` on text) in the same wave, so
this construct took the next free source number, `ess/12`, and nothing else moved.

**Version tables and literals that move:**

- `FORMAT_RELEASES` gains `("ess", 12, None)` (`crates/edge/ess-xtask/src/docs.rs:89-99`).
  Otherwise the docs lane refuses the grown constant.
- `website/docs/reference/formats.md` gains a row after `:108`, and
  `website/docs/reference/spec-versions.md` gains a paragraph after `:74`. The newest-version
  examples at `:9` and `:25` move to `ess/12`.
- `system.rs:1730` (`a_specification_reports_every_problem_in_one_run`) uses `ess/11` as "a version
  this build cannot read". It moves to `ess/99`, so the next bump does not touch it again.
  `system.rs:1978` gains `ess/12` among the accepted ones.
- The unit runs `rg -n --pcre2 'ess/1[1-9]\b' crates website` and reads each hit before moving it.
  The two lines above are the only hits at base.

## Witness and conformance

**No new witness.** An expanded outcome is `OutcomeCondition::External { cause }`
(`command.rs:3229`), with `TestStrategy::InjectFault` (`command.rs:492`). Synthesis already produces
one scenario per member: `ConfigureExternalOutcome { force: OutcomeRef(command, outcome) }`, then
the command, then the expected outcome (`crates/verify/ess-conformance/src/synthesize.rs:1492-1497`,
`:2439-2452`). The Rust, Go (`src/go/runtime.go:1914`, `:4100`) and TypeScript
(`src/ts/runtime.ts:2713`, `:5037`) lanes all run that step today. So **a group of 28 still adds 28
scenarios** and moves every recorded baseline, exactly as the hand copy does. The group removes the
copying. It does not remove the re-measurement the issue calls expensive. See *Out of scope*.

### Deciding checks for the implementation unit

The fixture is `crates/specify/ess-domain/tests/fixtures/outcome-groups/`: a `calls` domain with
four commands (`Hold`, `Resume`, `Park`, `Dial`), actors `calls.Agent` (`may: [Hold, Resume,
Park]`) and `calls.Supervisor` (`may: [Dial]`), a second domain `session` declaring
`session.Unauthenticated` and `session.Expired`, and two groups declared in two files in
**reverse** name order: `remote-backed` (actor selector, `except: [calls.Park]`) and `audited`
(`commands: [calls.Hold, calls.Dial]`, outcome `session-expired`). `calls.Park` declares its own
`credential-rejected`. The hand-copied twin is the same tree with the groups removed and the
outcomes written out.

| # | check | where |
|---|---|---|
| C1 | fixture and twin compile to byte-identical `to_canonical_json()`. Both synthesize byte-identical suites | `crates/specify/ess-compiler/tests/outcome_groups.rs`; suite half in `crates/verify/ess-conformance/tests/outcome_groups.rs` |
| C2 | `calls.Hold`'s outcomes end with `session-expired` (from `audited`) and then `credential-rejected` (from `remote-backed`), because groups append in name order. `calls.Resume` gains only `credential-rejected`, and `calls.Dial` only `session-expired`. `calls.Park` keeps only its own `credential-rejected` (its own `error:`), and no refusal is raised | ess-domain test over `Specification::commands()` |
| C3 | one case per G1–G16, each asserting the `ValidationCode`, the location, the count (exactly one refusal where the table says once) and, through `diagnose_locating`, the `ESS-COMMAND-00N` code and the group's `name:` line | `crates/specify/ess-domain/tests/outcome_groups.rs`; codes in the compiler test |
| C4 | G13 on a group of three members yields **one** refusal, not four | same |
| C5 | an `ess/10` header with a group: G16, and the members are still expanded (C2's shape holds) | same |
| C6 | `ess specify validate` on the issue's reproduction rewritten as a group passes. The original `actors[].outcomes` form is still `unknown field outcomes` | `crates/edge/ess-cli/tests/`, beside the existing validate cases |
| C7 | schema, docs lane and projection check pass. A model without groups keeps its IR bytes | `cargo xtask schema`; `xtask generate/schema/whats-changed --check`; `xtask docs` |

### The mutants it must kill

The unit breaks each condition, observes the named check fail, and restores it.

| mutant | killed by |
|---|---|
| `expand` not called | C1: IR differs, and C2 |
| every group applied to every command | C1, C2: `calls.Dial` gains `credential-rejected` |
| actor selector reads the wrong actor, or the union of all actors | C2: `calls.Dial` |
| `except` ignored | C2: `calls.Park` would collide, so a G14 refusal appears where the test expects none |
| collision resolved silently in favour of the group, or of the command | the G14 case: no refusal. The group-wins variant also fails C2's `calls.Park` error |
| G14 still expands into the colliding command | the G14 case's refusal count: the command's own `DuplicateDeclaration` appears as well |
| outcomes prepended, not appended | C1: outcome order in the IR |
| groups in file order, not name order | C1, C2: the groups are declared in reverse name order |
| `summary`/`refs` not copied | C1 |
| a group with its own refusal still expanded | C4 |
| G16 removed, or G16 stopping expansion | C5 |
| `except` admitted beside `commands` | the G7 case |
| `STRUCTURAL` without `except` | the G7/G8 cases' cited line: the needle becomes `name: <g>.except` and no line is found |
| `family_of` arm removed | C3: the code becomes `ESS-SPEC-…` |

**Not killed:** the `domain:` selector reading the name prefix instead of the declaring file. The
two readings select the same commands in every specification that validates (see *Expansion*), so
the mutant is equivalent wherever a test could observe it.

## Every projection site

Found by searching for `RawSpecFile`, `Specification::assemble` and the IR key lists under
`crates/`. Expansion leaves nothing behind, so every consumer downstream of `assemble` renders the
expanded outcomes as ordinary outcomes and needs no change. The table lists the sites that read
the construct itself, and the ones a reader might expect to.

| site | today | change |
|---|---|---|
| `RawSpecFile` (`spec.rs:43-112`) | top-level keys | **render**: `outcome_groups` |
| `Specification::assemble` (`spec.rs:219-224`) | absorbs files | calls `outcome_group::expand` first |
| `crates/specify/ess-domain/src/outcome_group.rs` | — | new: the raw types, `OutcomeGroupName`, `expand`, G1–G16 |
| `crates/specify/ess-domain/src/lib.rs:58-76` | module list | `pub mod outcome_group` (the raw types are public, as `RawActorSpec` is) |
| `system.rs:53`, `:89` | `ess/10` newest | `V12`, `SUPPORTED_FORMATS` |
| `resolve.rs:778-798` `family_of`, `:854-907` `STRUCTURAL` | — | `outcome_group` → `COMMAND`; `except` |
| `schemas/generated/ess.schema.json` | projection of `RawSpecFile` | regenerated |
| `docs/design/review-typed-diagnostics.md:252-322` | census | two lines (see *Validation*) |
| `ess specify inspect <name>` (`crates/edge/ess-cli/src/main.rs:2243-2266`) | looks names up in IR families | **refuses** a group name with its existing "`<name>` is not a resolved declaration". A group is not a resolved declaration. No change |
| OpenAPI, docs, native plan, Rust/Go/web synthesis, Entity Runtime lowering, service contract, conformance catalog, `ess verify diff` (`diff.rs:1860-1880` residual families) | read resolved commands | unchanged: they see the expanded outcomes, which is the page's decision 3 |
| `website/docs/guides/write-a-specification.md` | "An outcome the input cannot decide says that too" (`:277`) | a new section after it, "One outcome for many commands", with the syntax example above and one sentence per rule G7, G14 and G15. The example is written at `format: ess/12` |
| `website/docs/reference/formats.md`, `spec-versions.md` | version rows | see *Formats* |
| `crates/edge/ess-xtask/src/consumer_coverage/*` | parked (`AGENTS.md`, "Consumer coverage is opt-in"); `metadata.rs:23` names `assemble` | untouched |
| agent plugin grammar (`beyond10x/agentplugins`) | — | after the release, not in this repository |

## Out of scope

- **One scenario for the whole group.** The issue's cost argument is that 28 outcomes are 28
  scenarios. A shared scenario would need a conformance notion of "the same branch in many
  commands", a suite id for it and a new suite format. This construct is defined as sugar that
  leaves no trace, and it keeps the 28.
- **`outcomes:` on actors or domains.** An actor says who may invoke which command "and nothing
  else" (`actor.rs:14-22`). A domain is a namespace. Neither is where a failure mode belongs, and
  the brief's selector form already covers both.
- **Group outcomes other than an external refusal:** guarded (`when:`), wrong-state, emitting or
  subject-changing outcomes. Each depends on its command (*Syntax*), and each is a later format.
- **Union or intersection of selectors, wildcard or prefix selectors**, and groups that select
  other groups.
- **Provenance of an expanded outcome** in the IR, the docs or diagnostics raised after expansion.
  A refusal at a member command (for example `UnreachableBranch`) cites the command, not the group.
- **`ess specify inspect` for groups.** A source-level view of a group is a separate feature.

## What the implementation unit checks before building

These lines are inferred. Confirm each with one read or one run, and report the result:

1. No test pins the full set of `family_of` heads, or the `STRUCTURAL` list, as a literal. If one
   does, it gains the new entry. This is not a reason to change the decision.
2. The heads census counts a head by the first path segment of each string literal
   (`review-typed-diagnostics.md:275-280`). If it counts `outcome_groups` differently, the census
   line follows the test.
3. C1's suite half: a member with subject-state guards reaches its expanded outcome through
   `reach_external` (`synthesize.rs:2915-2917`), the same as the hand copy does. C1 decides it, and
   the fixture's commands carry no subject guards unless the unit adds one to test it.
