//! Structural identity keys shared by equality and ordering in generated stores.
use super::super::{name, Emit};
use ess_compiler::ir::{ResolvedBody, ResolvedTypeRef, TypeHandle};
use ess_domain::{name::QualifiedName, types::Primitive};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(super) struct Keys<'a, 'ir> {
    emit: &'a Emit<'ir>,
    named: BTreeMap<QualifiedName, TypeHandle>,
    json: bool,
    containers: BTreeMap<String, (String, String)>,
}
impl<'a, 'ir> Keys<'a, 'ir> {
    pub fn new(emit: &'a Emit<'ir>) -> Self {
        Self {
            emit,
            named: BTreeMap::new(),
            json: false,
            containers: BTreeMap::new(),
        }
    }
    pub fn expression(&mut self, reference: &ResolvedTypeRef, value: &str) -> String {
        match reference {
            ResolvedTypeRef::Primitive { name } => match name {
                Primitive::Boolean => format!("memoryKey{{kind: 1, flag: {value}}}"),
                Primitive::Integer => format!("memoryKey{{kind: 2, integer: {value}}}"),
                Primitive::Decimal => format!("memoryKey{{kind: 4, text: ({value}).Value()}}"),
                Primitive::String => format!("memoryKey{{kind: 4, text: {value}}}"),
                Primitive::Uuid | Primitive::Timestamp | Primitive::Duration => {
                    format!("memoryKey{{kind: 4, text: ({value}).Value()}}")
                }
                Primitive::Bytes => format!("memoryKey{{kind: 5, text: string({value})}}"),
                Primitive::Json => {
                    self.json = true;
                    format!("memoryJSONKey(({value}).Value())")
                }
                Primitive::Binary64 => unreachable!("native synthesis refuses Binary64"),
            },
            ResolvedTypeRef::Declared { name } => {
                self.named.insert(name.name().clone(), name.clone());
                format!("{}({value})", self.function(name.name()))
            }
            ResolvedTypeRef::Optional { .. }
            | ResolvedTypeRef::List { .. }
            | ResolvedTypeRef::Map { .. } => self.container(reference, value),
        }
    }
    fn container(&mut self, reference: &ResolvedTypeRef, value: &str) -> String {
        let key = reference.to_string();
        if let Some((function, _)) = self.containers.get(&key) {
            return format!("{function}({value})");
        }
        let function = format!("memoryCollection{}", self.containers.len());
        self.containers
            .insert(key.clone(), (function.clone(), String::new()));
        let ty = self.emit.go_type(reference);
        let mut body = format!("\nfunc {function}(values {ty}) memoryKey {{\n");
        match reference {
            ResolvedTypeRef::Optional { of } => {
                let item = self.expression(of, "*values");
                let _ = writeln!(
                    body,
                    "\tif values == nil {{\n\t\treturn memoryKey{{kind: 8}}\n\t}}\n\treturn memoryKey{{kind: 8, items: []memoryKey{{{item}}}}}"
                );
            }
            ResolvedTypeRef::List { of } => {
                let item = self.expression(of, "value");
                let _ = writeln!(body, "\titems := make([]memoryKey, 0, len(values))\n\tfor _, value := range values {{\n\t\titems = append(items, {item})\n\t}}\n\treturn memoryKey{{kind: 6, items: items}}");
            }
            ResolvedTypeRef::Map { key, value: item } => {
                self.emit.import("sort");
                let key = self.expression(&ResolvedTypeRef::Primitive { name: *key }, "key");
                let item = self.expression(item, "value");
                let _ = writeln!(body, "\titems := make([]memoryKey, 0, len(values))\n\tfor key, value := range values {{\n\t\titems = append(items, memoryKey{{kind: 6, items: []memoryKey{{{key}, {item}}}}})\n\t}}\n\tsort.Slice(items, func(i, j int) bool {{ return memoryCompare(items[i], items[j]) < 0 }})\n\treturn memoryKey{{kind: 7, items: items}}");
            }
            _ => unreachable!("only containers request a collection helper"),
        }
        body.push_str("}\n");
        self.containers.insert(key, (function.clone(), body));
        format!("{function}({value})")
    }
    pub fn helpers(mut self) -> String {
        let mut out = String::new();
        let mut rendered = BTreeSet::new();
        while let Some((name, handle)) = self
            .named
            .iter()
            .find(|(name, _)| !rendered.contains(*name))
            .map(|(name, handle)| (name.clone(), handle.clone()))
        {
            rendered.insert(name.clone());
            let ty = self
                .emit
                .go_type(&ResolvedTypeRef::Declared { name: handle });
            let _ = writeln!(
                out,
                "\nfunc {}(value {ty}) memoryKey {{",
                self.function(&name)
            );
            match self.emit.ir.types()[&name].body.clone() {
                ResolvedBody::Newtype { of, .. } => {
                    let expression = self.expression(&of, "value.Value()");
                    let _ = writeln!(out, "\treturn {expression}");
                }
                ResolvedBody::Struct { fields, .. } => {
                    let values = fields
                        .iter()
                        .map(|field| {
                            self.expression(
                                &field.type_ref,
                                &format!("value.{}", name::exported(&field.name)),
                            )
                        })
                        .collect::<Vec<_>>();
                    let _ = writeln!(
                        out,
                        "\treturn memoryKey{{kind: 6, items: []memoryKey{{\n{}\n\t}}}}",
                        values
                            .iter()
                            .map(|v| format!("\t\t{v},"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                }
                ResolvedBody::Enum { variants } => {
                    out.push_str("\tswitch value.(type) {\n");
                    for variant in variants {
                        let variant_ty = self.emit.reference_variant(&name, &variant);
                        let _ = writeln!(
                            out,
                            "\tcase {variant_ty}:\n\t\treturn memoryKey{{kind: 4, text: {:?}}}",
                            variant.to_string()
                        );
                    }
                    out.push_str("\t}\n\treturn memoryKey{}\n");
                }
                ResolvedBody::Union { variants, .. } => {
                    out.push_str("\tswitch value := value.(type) {\n");
                    for (variant, reference) in variants {
                        let ty = self.emit.reference_variant(&name, &variant);
                        let key = self.expression(&reference, "value.Value");
                        let _ = writeln!(out, "\tcase {ty}:\n\t\treturn memoryKey{{kind: 6, items: []memoryKey{{{{kind: 4, text: {variant:?}}}, {key}}}}}");
                    }
                    out.push_str("\t}\n\treturn memoryKey{}\n");
                }
            }
            out.push_str("}\n");
        }
        for (_, body) in self.containers.values() {
            out.push_str(body);
        }
        self.emit.import("strings");
        out.push_str(KEY);
        if self.json {
            self.emit.import("encoding/json");
            out.push_str(JSON);
        }
        out
    }
    fn function(&self, name: &QualifiedName) -> String {
        let index = self
            .emit
            .ir
            .types()
            .keys()
            .position(|candidate| candidate == name)
            .expect("resolved type");
        format!("memoryDeclared{index}")
    }
}

const KEY: &str = r"
// memoryInvalidRaw is disjoint from every successfully decoded model value.
const memoryInvalidRaw = 9

type memoryKey struct {
	kind    int
	flag    bool
	integer int64
	text    string
	items   []memoryKey
}

func memoryCompare(left, right memoryKey) int {
	if left.kind < right.kind {
		return -1
	}
	if left.kind > right.kind {
		return 1
	}
	switch left.kind {
	case 1:
		if left.flag != right.flag {
			if left.flag {
				return 1
			}
			return -1
		}
	case 2:
		if left.integer < right.integer {
			return -1
		}
		if left.integer > right.integer {
			return 1
		}
	case 3, 4, 5, memoryInvalidRaw:
		return strings.Compare(left.text, right.text)
	case 6, 7, 8:
		for i := 0; i < len(left.items) && i < len(right.items); i++ {
			if order := memoryCompare(left.items[i], right.items[i]); order != 0 {
				return order
			}
		}
		if len(left.items) < len(right.items) {
			return -1
		}
		if len(left.items) > len(right.items) {
			return 1
		}
	}
	return 0
}
";

const JSON: &str = r"
func memoryJSONKey(text string) memoryKey {
	// Direct constructors can carry invalid JSON. Keep that raw identity separate from
	// every admitted decoded value; HTTP decoding still refuses malformed JSON.
	invalid := memoryKey{kind: memoryInvalidRaw, text: text}
	if !json.Valid([]byte(text)) {
		return invalid
	}
	value, err := ordered([]byte(text))
	if err != nil {
		return invalid
	}
	key, valid := memoryJSONValue(value)
	if !valid {
		return invalid
	}
	return key
}

func memoryJSONValue(value any) (memoryKey, bool) {
	switch value := value.(type) {
	case nil:
		return memoryKey{}, true
	case bool:
		return memoryKey{kind: 1, flag: value}, true
	case json.Number:
		return memoryKey{kind: 3, text: string(value)}, true
	case string:
		return memoryKey{kind: 4, text: value}, true
	case []any:
		items := make([]memoryKey, 0, len(value))
		for _, item := range value {
			key, valid := memoryJSONValue(item)
			if !valid {
				return memoryKey{}, false
			}
			items = append(items, key)
		}
		return memoryKey{kind: 6, items: items}, true
	case *object:
		items := make([]memoryKey, 0, len(value.order))
		for _, name := range value.order {
			key, valid := memoryJSONValue(value.members[name])
			if !valid {
				return memoryKey{}, false
			}
			items = append(items, memoryKey{kind: 6, items: []memoryKey{{kind: 4, text: name}, key}})
		}
		return memoryKey{kind: 7, items: items}, true
	}
	return memoryKey{}, false
}
";
