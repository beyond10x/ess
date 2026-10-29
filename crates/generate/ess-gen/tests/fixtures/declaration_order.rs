//! A model whose records declare their fields out of wire-name order, and whose types hold one
//! another by value, through a cycle, and inside a list (beyond10x/ess#661). Shared by ess-gen's
//! `model_types.rs` and schema-contract's `declaration_order.rs` and `rust_realization.rs`; kept
//! in Rust rather than YAML so the synthesis tables, which read every YAML model, do not gain it.

/// `Outer`, the event `Made`, the command `Make`'s input and response and the view `Things`'s
/// row each declare `zeta, alpha, mid`. `Node` holds itself optionally and in a list, `A` and
/// `B` hold each other optionally, and `Holder` holds a `Node`.
pub const MODEL: &str = r"format: ess/22
system: demo
version: v1
domain: demo.g
types:
  - name: demo.g.Inner
    kind: struct
    fields:
      - {name: label, type: String}
  - name: demo.g.Outer
    kind: struct
    fields:
      - {name: zeta, type: demo.g.Inner}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<demo.g.Inner>'}
  - name: demo.g.Node
    kind: struct
    fields:
      - {name: next, type: 'Optional<demo.g.Node>'}
      - {name: kids, type: 'List<demo.g.Node>'}
  - name: demo.g.A
    kind: struct
    fields:
      - {name: b, type: 'Optional<demo.g.B>'}
  - name: demo.g.B
    kind: struct
    fields:
      - {name: a, type: 'Optional<demo.g.A>'}
  - name: demo.g.Holder
    kind: struct
    fields:
      - {name: tree, type: demo.g.Node}
entities:
  - name: demo.g.Thing
    identity: {name: thing_id, type: String}
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<String>'}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
commands:
  - name: demo.g.Make
    input:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<String>'}
    response:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<String>'}
    outcomes:
      - name: made
        returns: true
        emits: [demo.g.Made]
        payload:
          demo.g.Made: {zeta: input.zeta, alpha: input.alpha, mid: input.mid}
events:
  - name: demo.g.Made
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<String>'}
views:
  - name: demo.g.Things
    source: demo.g.Thing
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
      - {name: mid, type: 'Optional<String>'}
components:
  - component: demo-service
    reached_by: network
    owns: {domains: [demo.g]}
    accepts: {commands: [demo.g.Make]}
    publishes: {events: [demo.g.Made]}
";

/// Every root of [`MODEL`]: its types, its event, its command's bodies and its view's row.
pub const ROOTS: [&str; 10] = [
    "demo.g.Inner",
    "demo.g.Outer",
    "demo.g.Node",
    "demo.g.A",
    "demo.g.B",
    "demo.g.Holder",
    "demo.g.Made",
    "demo.g.Make.input",
    "demo.g.Make.response",
    "demo.g.Things.row",
];

/// The definitions that declare `zeta, alpha, mid`.
pub const DECLARED: [&str; 5] = [
    "demo.g.Outer",
    "demo.g.Made",
    "demo.g.Make.input",
    "demo.g.Make.response",
    "demo.g.Things.row",
];
