//! The cycle shapes and record kinds the declared-field-order and box-only-recursion fixture of
//! beyond10x/ess#661 leaves out, shared by the adversary cases that read the generated text and
//! the `rust_realization` cases that build it.

pub const SOURCE: &str = r"format: ess/32
system: demo
version: v1
domains: [demo.r]
domain: demo.r
types:
  - name: demo.r.Leaf
    kind: struct
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: String}
  - name: demo.r.C1
    kind: struct
    fields:
      - {name: c2, type: demo.r.C2}
      - {name: label, type: String}
  - name: demo.r.C2
    kind: struct
    fields:
      - {name: c3, type: demo.r.C3}
  - name: demo.r.C3
    kind: struct
    fields:
      - {name: back, type: 'Optional<demo.r.C1>'}
  - name: demo.r.Neg
    kind: struct
    fields:
      - {name: inner, type: demo.r.Expr}
  - name: demo.r.Expr
    kind: union
    tag: kind
    variants:
      lit: String
      neg: demo.r.Neg
  - name: demo.r.Dir
    kind: struct
    fields:
      - {name: entries, type: 'Map<String, demo.r.Dir>'}
      - {name: kids, type: 'List<demo.r.Dir>'}
      - {name: parent, type: 'Optional<List<demo.r.Dir>>'}
  - name: demo.r.Entry
    kind: struct
    fields:
      - {name: next, type: 'Optional<demo.r.Entry>'}
      - {name: leaf, type: demo.r.Leaf}
  - name: demo.r.Outside
    kind: struct
    fields:
      - {name: entry, type: demo.r.Entry}
      - {name: also, type: 'Optional<demo.r.Entry>'}
      - {name: expr, type: demo.r.Expr}
  - name: demo.r.Open
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: 'Optional<String>'}
      - {name: renamed, type: String, wire: aaa}
  - name: demo.r.Short
    kind: struct
    shorthand: zeta
    fields:
      - {name: zeta, type: String}
      - {name: alpha, type: 'Optional<String>'}
";

pub const ROOTS: [&str; 11] = [
    "demo.r.Leaf",
    "demo.r.C1",
    "demo.r.C2",
    "demo.r.C3",
    "demo.r.Neg",
    "demo.r.Expr",
    "demo.r.Dir",
    "demo.r.Entry",
    "demo.r.Outside",
    "demo.r.Open",
    "demo.r.Short",
];
