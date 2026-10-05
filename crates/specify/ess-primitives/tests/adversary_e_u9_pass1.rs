//! Adversary pass 1 against E-U9 (beyond10x/ess#200): a string operator against `{param: q}` /
//! `{input: q}` decides exactly as the same operator against the literal holding the same text, for
//! every text — empty, regex and SQL metacharacters, quotes, combining marks, multi-byte and
//! supplementary-plane characters, case variants. A fixed-seed property over generated pairs.

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::{Predicate, TextNamespace, TextOp, TextOperand, Truth};

const ALPHABET: [&str; 22] = [
    "a",
    "A",
    "b",
    "%",
    "_",
    "*",
    ".",
    "^",
    "$",
    "'",
    "\"",
    "\\",
    "[",
    "+",
    " ",
    "é",
    "e\u{301}",
    "\u{301}",
    "ß",
    "SS",
    "\u{1F600}",
    "\u{0}",
];

/// xorshift64*, fixed seed: the same pairs on every machine.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn text(&mut self, most: u64) -> String {
        let length = self.next() % (most + 1);
        (0..length)
            .map(|_| {
                let index = usize::try_from(self.next() % ALPHABET.len() as u64).unwrap();
                ALPHABET[index]
            })
            .collect()
    }
}

fn path(text: &str) -> FactPath {
    FactPath::new(text).expect("a fact path")
}

#[test]
fn adv_u9_operand_and_literal_decide_alike_for_every_text() {
    let mut rng = Rng(0x00E9_0200_5EED_0001);
    let mut checked = 0_u32;
    for _ in 0..4000 {
        let fact = rng.text(6);
        // Half the operands are cut from the fact, so matches are frequent too.
        let operand = if rng.next() % 2 == 0 {
            let chars: Vec<char> = fact.chars().collect();
            let from = usize::try_from(rng.next() % (chars.len() as u64 + 1)).unwrap();
            let to =
                from + usize::try_from(rng.next() % ((chars.len() - from) as u64 + 1)).unwrap();
            chars[from..to].iter().collect()
        } else {
            rng.text(3)
        };
        for op in [TextOp::StartsWith, TextOp::EndsWith, TextOp::Contains] {
            for namespace in [TextNamespace::Param, TextNamespace::Input] {
                let typed = Predicate::TextMatch {
                    path: path("note"),
                    op,
                    value: TextOperand::fact(namespace, "q").expect("one segment"),
                };
                let literal = Predicate::TextMatch {
                    path: path("note"),
                    op,
                    value: TextOperand::Literal(FactValue::text(operand.clone())),
                };
                let mut store = FactStore::new();
                store.set(path("note"), FactValue::text(fact.clone()));
                store.set(
                    path(&format!("{}.q", namespace.keyword())),
                    FactValue::text(operand.clone()),
                );
                let expected = Truth::from_bool(match op {
                    TextOp::StartsWith => fact.as_bytes().starts_with(operand.as_bytes()),
                    TextOp::EndsWith => fact.as_bytes().ends_with(operand.as_bytes()),
                    TextOp::Contains => fact.contains(operand.as_str()),
                });
                assert_eq!(
                    (typed.evaluate(&store), literal.evaluate(&store)),
                    (expected, expected),
                    "{op} fact {fact:?} operand {operand:?} ({namespace})"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 24_000);
}
