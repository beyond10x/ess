---
format: aep.planning-md/3
id: review-result:adversary-w2-gen2-pass-1
kind: review-result
status: active
title: 'W2 GEN2 adversary pass 1: Optional wrap, server stems and shared CLI binaries'
summary: NEEDS-CHANGE, 67 to 80 executed, 2 red; F1 shared-binary first-level word to fix round 1, F2 to backlog
relations:
- reviews: story:go-synthesis-refuses-an-optional-into-required-assignment
- reviews: story:go-synthesis-repairs-a-served-component-named-server
- reviews: story:components-sharing-a-cli-binary-generate-one-tree-with-every-group
revision: 1
---
unit: W2 GEN2 (story:go-synthesis-refuses-an-optional-into-required-assignment, story:go-synthesis-repairs-a-served-component-named-server, story:components-sharing-a-cli-binary-generate-one-tree-with-every-group), e69e2094f5, 28187ad02e, d3baeab30b and 5811b2a6ae merged as cffb7deb3e, reviewed in tree ess-w2-gen2 at 5811b2a6ae
verdict: NEEDS-CHANGE
cases: executed 67→80, red 2
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory gen2/adversary-scratch (cargo log)
needs-coordinator: whether F1 falls inside story 3's Acceptance (ruled: yes)

**F1 (introduced, warning, sent to fix round 1).** `validate_shared_binaries`
(`crates/specify/ess-domain/src/component.rs:1426`) compares group names with group names and
qualified command and view names with each other, but never a group word of one sharing component
with a top-level command or view word of another. `desk record` validates and the merged clap tree
defines `record` twice. Red case
`adv_gen2_a_group_named_like_another_sharing_components_top_level_command_is_refused`. The lead rules
it inside the story's Acceptance (a command path declared by two components is refused at validate).

**F2 (pre-existing, note, out of scope).** Inside one component a group named like its own top-level
command word validates (`validate_command_placement`, `component.rs:1043`). Control case
`adv_gen2_control_one_component_naming_a_group_like_its_own_top_level_command_is_refused`; filed as
the backlog draft story:a-group-named-like-a-top-level-command-of-its-component-is-refused and left
out of the committed file.

**Attacked, not broken.** The Optional wrap over nested Optional, Optional of a list, a struct and a
newtype from another package, beside a proved member and all together (go build and go vet pass);
surface stems cannot reproduce `ess_memory`/`ess_static`, repaired names that collide again are
repaired again and build; control digests in `go.rs` and `clap.rs` hold; tree and dispatch agree for
three components mixing top-level words, views and groups; a command at two paths is refused naming
both components; `insert` refuses a path collision naming the path and both artifacts. Peak target/
6G, lowest / 133G.
