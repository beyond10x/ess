//! Resolve a finite production declaration graph without borrowing a wire representation.
use anyhow::{bail, Context, Result};
use quote::{quote, ToTokens};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use syn::{
    parse::Parse, parse::ParseStream, Attribute, Fields, GenericArgument, Item, PathArguments,
    Token, Type, UseTree,
};

pub(super) const ROOTS: &[&str] = &[
    "ess_domain::spec::RawSpecFile",
    "ess_domain::spec::Specification",
    "ess_compiler::ir::EssIr",
    "ess_compiler::ir::EssIrParts",
    "ess_compiler::refs::EssSemanticRef",
    "ess_compiler::graph::SemanticDependencyGraph",
];
#[derive(Default)]
struct Graph {
    declarations: BTreeMap<String, Item>,
    imports: BTreeMap<String, BTreeMap<String, String>>,
    modules: BTreeSet<String>,
    macros: BTreeMap<String, String>,
    diagnostics: BTreeMap<String, Value>,
    diagnostic_symbols: BTreeMap<String, String>,
}

#[cfg(test)]
pub(super) fn fixture(text: &str) -> Result<Value> {
    let mut graph = Graph::default();
    graph.items(
        "fixture",
        &file_items(text, "fixture")?,
        &BTreeMap::new(),
        "",
    )?;
    graph.extract(&["fixture::Root"])
}
pub(super) fn extract(sources: &BTreeMap<String, String>) -> Result<Value> {
    let mut graph = Graph::default();
    for name in ["ess-domain", "ess-compiler", "ess-primitives"] {
        let root = format!("crates/specify/{name}/src/lib.rs");
        graph.items(
            &name.replace('-', "_"),
            &file_items(
                sources
                    .get(&root)
                    .with_context(|| format!("missing model root {root}"))?,
                &root,
            )?,
            sources,
            &format!("crates/specify/{name}/src"),
        )?;
    }
    graph.extract(ROOTS)
}
fn file_items(source: &str, owner: &str) -> Result<Vec<Item>> {
    let file = syn::parse_file(source)?;
    if !active(&file.attrs, owner)? {
        return Ok(Vec::new());
    }
    for a in &file.attrs {
        if !matches!(tokens(a.path()).as_str(), "doc" | "allow" | "forbid") {
            bail!("{owner}: unknown file-level attribute {}", tokens(a));
        }
    }
    Ok(file.items)
}
fn tokens(x: &impl ToTokens) -> String {
    x.to_token_stream().to_string()
}
fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Struct(x) => &x.attrs,
        Item::Enum(x) => &x.attrs,
        Item::Type(x) => &x.attrs,
        Item::Mod(x) => &x.attrs,
        Item::Use(x) => &x.attrs,
        Item::Macro(x) => &x.attrs,
        Item::Fn(x) => &x.attrs,
        Item::Impl(x) => &x.attrs,
        Item::Const(x) => &x.attrs,
        Item::Static(x) => &x.attrs,
        Item::Trait(x) => &x.attrs,
        Item::ExternCrate(x) => &x.attrs,
        _ => &[],
    }
}
fn active(attrs: &[Attribute], owner: &str) -> Result<bool> {
    let mut enabled = true;
    for a in attrs {
        if a.path().is_ident("cfg") {
            let condition = a.parse_args::<syn::Meta>()?;
            if matches!(&condition,syn::Meta::Path(p) if p.is_ident("test")) {
                enabled = false;
            } else {
                bail!("{owner}: unhandled production cfg {}", tokens(&condition));
            }
        }
        if a.path().is_ident("cfg_attr") {
            bail!("{owner}: unhandled production cfg_attr");
        }
    }
    Ok(enabled)
}
fn attrs(attrs: &[Attribute], owner: &str) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for a in attrs {
        let path = tokens(a.path());
        match path.as_str() {
            "doc" | "allow" | "must_use" => {}
            "derive" | "serde" | "schemars" | "repr" | "non_exhaustive" | "default" => {
                representation(a, owner)?;
                result.push(tokens(&a.meta));
            }
            _ => bail!("{owner}: unknown representation attribute {path}"),
        }
    }
    Ok(result)
}
fn representation(a: &Attribute, owner: &str) -> Result<()> {
    use syn::{punctuated::Punctuated, Meta};
    let name = tokens(a.path());
    if matches!(name.as_str(), "non_exhaustive" | "default") {
        if !matches!(a.meta, Meta::Path(_)) {
            bail!("{owner}: unsupported {name} shape");
        }
        return Ok(());
    }
    let items = a.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for item in items {
        let key = tokens(item.path());
        let allowed = match (&*name, &item) {
            ("derive", Meta::Path(_)) => matches!(
                key.as_str(),
                "Debug"
                    | "Clone"
                    | "Copy"
                    | "Default"
                    | "PartialEq"
                    | "Eq"
                    | "PartialOrd"
                    | "Ord"
                    | "Hash"
                    | "serde :: Serialize"
                    | "serde :: Deserialize"
                    | "schemars :: JsonSchema"
            ),
            ("serde", Meta::Path(_)) => matches!(
                key.as_str(),
                "default"
                    | "flatten"
                    | "skip"
                    | "skip_serializing"
                    | "skip_deserializing"
                    | "deny_unknown_fields"
                    | "transparent"
                    | "untagged"
            ),
            ("serde", Meta::NameValue(n)) => {
                matches!(
                    key.as_str(),
                    "alias"
                        | "rename"
                        | "rename_all"
                        | "tag"
                        | "content"
                        | "default"
                        | "skip_serializing_if"
                        | "deserialize_with"
                        | "serialize_with"
                        | "with"
                        | "into"
                        | "from"
                        | "try_from"
                ) && matches!(&n.value,syn::Expr::Lit(x) if matches!(x.lit,syn::Lit::Str(_)))
            }
            ("schemars", Meta::NameValue(n)) => {
                matches!(key.as_str(), "with" | "schema_with" | "rename")
                    && matches!(&n.value,syn::Expr::Lit(x) if matches!(x.lit,syn::Lit::Str(_)))
            }
            ("schemars", Meta::List(l)) if key == "regex" => {
                let nested = l.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                nested.len()==1&&nested.iter().all(|x|matches!(x,Meta::NameValue(n) if n.path.is_ident("pattern")&&matches!(&n.value,syn::Expr::Lit(x) if matches!(x.lit,syn::Lit::Str(_)))))
            }
            ("repr", Meta::Path(_)) => matches!(
                key.as_str(),
                "C" | "transparent" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64"
            ),
            _ => false,
        };
        if !allowed {
            bail!(
                "{owner}: unknown representation grammar {name}({})",
                tokens(&item)
            );
        }
    }
    Ok(())
}
fn imports(tree: &UseTree, prefix: &str, out: &mut BTreeMap<String, String>) -> Result<()> {
    match tree {
        UseTree::Path(p) => imports(&p.tree, &format!("{prefix}{}::", p.ident), out),
        UseTree::Name(n) => {
            let (name, target) = if n.ident == "self" {
                let t = prefix.trim_end_matches("::");
                (
                    t.rsplit("::")
                        .next()
                        .context("empty self import")?
                        .to_owned(),
                    t.to_owned(),
                )
            } else {
                (n.ident.to_string(), format!("{prefix}{}", n.ident))
            };
            if out.insert(name.clone(), target).is_some() {
                bail!("ambiguous import {name}");
            }
            Ok(())
        }
        UseTree::Rename(n) => {
            if n.rename == "_" {
                return Ok(());
            }
            let target = if n.ident == "self" {
                prefix.trim_end_matches("::").to_owned()
            } else {
                format!("{prefix}{}", n.ident)
            };
            if out.insert(n.rename.to_string(), target).is_some() {
                bail!("ambiguous alias {}", n.rename);
            }
            Ok(())
        }
        UseTree::Group(g) => {
            for t in &g.items {
                imports(t, prefix, out)?;
            }
            Ok(())
        }
        UseTree::Glob(_) => bail!("unhandled production glob import {prefix}*"),
    }
}
impl Graph {
    fn items(
        &mut self,
        module: &str,
        items: &[Item],
        sources: &BTreeMap<String, String>,
        directory: &str,
    ) -> Result<()> {
        if !self.modules.insert(module.into()) {
            bail!("duplicate or cyclic module owner {module}");
        }
        let mut uses = BTreeMap::new();
        for item in items {
            if !active(item_attrs(item), module)? {
                continue;
            }
            match item {
                Item::Use(u) => {
                    let prefix = if u.leading_colon.is_some() { "::" } else { "" };
                    imports(&u.tree, prefix, &mut uses)
                        .with_context(|| format!("{module}: import"))?;
                }
                Item::Mod(m) => {
                    attrs(&m.attrs, module)?;
                    let child = format!("{module}::{}", m.ident);
                    if let Some((_, items)) = &m.content {
                        self.items(&child, items, sources, &format!("{directory}/{}", m.ident))?;
                    } else {
                        let base = std::path::Path::new(directory);
                        let candidates = [
                            base.join(format!("{}.rs", m.ident)),
                            base.join(m.ident.to_string()).join("mod.rs"),
                        ];
                        let found: Vec<_> = candidates
                            .iter()
                            .filter_map(|p| p.to_str().and_then(|p| sources.get(p).map(|s| (p, s))))
                            .collect();
                        let [(p, s)] = found.as_slice() else {
                            bail!("{child}: absent or ambiguous module file");
                        };
                        self.items(
                            &child,
                            &file_items(s, p)?,
                            sources,
                            &format!("{directory}/{}", m.ident),
                        )?;
                    }
                }
                Item::Struct(s) => self.insert(module, &s.ident.to_string(), item.clone())?,
                Item::Enum(s) => self.insert(module, &s.ident.to_string(), item.clone())?,
                Item::Type(s) => self.insert(module, &s.ident.to_string(), item.clone())?,
                Item::Macro(m) => self.item_macro(module, m)?,
                Item::Fn(_) | Item::Impl(_) | Item::Const(_) | Item::Static(_) | Item::Trait(_) => {
                }
                _ => bail!("{module}: unhandled production item {}", tokens(item)),
            }
        }
        self.imports.insert(module.into(), uses);
        Ok(())
    }
    fn insert(&mut self, module: &str, name: &str, item: Item) -> Result<()> {
        let id = format!("{module}::{name}");
        if self.declarations.insert(id.clone(), item).is_some() {
            bail!("duplicate declaration {id}");
        }
        Ok(())
    }
    fn item_macro(&mut self, module: &str, m: &syn::ItemMacro) -> Result<()> {
        attrs(&m.attrs, module)?;
        if m.mac.path.is_ident("macro_rules") {
            let name = m.ident.as_ref().context("unnamed macro_rules")?.to_string();
            let key = format!("{module}::{name}");
            guard(&key, "definition", &m.mac.tokens)?;
            if self
                .macros
                .insert(
                    key.clone(),
                    super::hash_bytes(tokens(&m.mac.tokens).as_bytes()),
                )
                .is_some()
            {
                bail!("{key}: additional macro definition");
            }
            return Ok(());
        }
        let name = tokens(&m.mac.path);
        let key = format!("{module}::{name}");
        if !self.macros.contains_key(&key) {
            bail!("{key}: unknown production item macro");
        }
        match name.as_str() {
            "handles" | "semantic_refs" => {
                let inv = syn::parse2::<Wrappers>(m.mac.tokens.clone())?;
                for w in inv.0 {
                    if (name == "handles") != w.handle {
                        bail!("{key}: wrong invocation grammar");
                    }
                    let n = &w.name;
                    let ty = &w.ty;
                    let supplied = &w.attrs;
                    attrs(supplied, &key)?;
                    self.insert(module,&n.to_string(),syn::parse2(quote!(#(#supplied)* #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)] #[serde(transparent)] pub struct #n(#ty);))?)?;
                }
            }
            "semantic_ref_from" => {
                syn::parse2::<Conversions>(m.mac.tokens.clone())?;
            }
            // One periodic profile word: a single-variant enum the document writes as one word.
            // Accounted like `handles` — the invocation is the declaration, so the expansion is
            // rebuilt here rather than pinned per call, of which this macro has several.
            "profile_word" => {
                let word = syn::parse2::<ProfileWord>(m.mac.tokens.clone())?;
                let name = &word.name;
                let variant = &word.variant;
                let doc = &word.doc;
                self.insert(
                    module,
                    &name.to_string(),
                    syn::parse2(quote!(
                        #[doc = #doc]
                        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
                        #[serde(rename_all = "snake_case")]
                        pub enum #name { #[doc = #doc] #variant }
                    ))?,
                )?;
            }
            "codes" | "validation_codes" => {
                guard(&key, "invocation", &m.mac.tokens)?;
                let generated = if name == "validation_codes" {
                    vec![format!("{module}::ValidationCode")]
                } else {
                    let names = syn::parse2::<Codes>(m.mac.tokens.clone())?;
                    let mut ids = names
                        .0
                        .into_iter()
                        .map(|n| format!("{module}::{n}"))
                        .collect::<Vec<_>>();
                    ids.push(format!("{module}::ALL"));
                    ids
                };
                for symbol in &generated {
                    self.diagnostic_symbols.insert(symbol.clone(), key.clone());
                }
                let classification = json!({"definition_sha256":self.macros[&key],"invocation_sha256":super::hash_bytes(tokens(&m.mac.tokens).as_bytes()),"generated_symbols":generated,"classification":"guarded diagnostic surface; reaching a generated declaration from a selected model root refuses"});
                if self
                    .diagnostics
                    .insert(key.clone(), classification)
                    .is_some()
                {
                    bail!("{key}: additional diagnostic macro invocation");
                }
            }
            _ => bail!("{key}: unknown production macro"),
        }
        Ok(())
    }
    fn resolve(&self, module: &str, path: &str, visiting: &mut BTreeSet<String>) -> Result<String> {
        let key = format!("{module}|{path}");
        if !visiting.insert(key.clone()) {
            bail!("{module}: cyclic unresolved alias {path}");
        }
        let absolute = path.strip_prefix("::");
        let parts: Vec<_> = absolute.unwrap_or(path).split("::").collect();
        let first = parts[0];
        let suffix = parts[1..].join("::");
        let target = if let Some(external_path) = absolute {
            if !self.modules.contains(first)
                && !(first == "std"
                    && (external(external_path).is_some() || external_namespace(external_path)))
            {
                bail!("{module}: unknown absolute external owner {path}");
            }
            external_path.into()
        } else if first == "crate" {
            format!(
                "{}::{suffix}",
                module.split("::").next().context("crate owner")?
            )
        } else if first == "self" {
            format!("{module}::{suffix}")
        } else if first == "super" {
            format!(
                "{}::{suffix}",
                module.rsplit_once("::").context("super escapes crate")?.0
            )
        } else if let Some(import) = self.imports.get(module).and_then(|u| u.get(first)) {
            let target = self.resolve(module, import, visiting)?;
            if suffix.is_empty() {
                target
            } else {
                format!("{target}::{suffix}")
            }
        } else if self
            .declarations
            .contains_key(&format!("{module}::{first}"))
            || self.modules.contains(&format!("{module}::{first}"))
        {
            format!("{module}::{path}")
        } else if self.declarations.contains_key(path)
            || self.modules.contains(path)
            || self.modules.contains(first)
            || self.diagnostic_symbols.contains_key(path)
            || external(path).is_some()
            || external_namespace(path)
        {
            path.into()
        } else {
            format!("{module}::{path}")
        };
        if let Some(mac) = self.diagnostic_symbols.get(&target) {
            bail!("{module}: selected model traversal reaches diagnostic generated declaration {target} from {mac}; no opaque leaf exemption");
        }
        let result = if self.declarations.contains_key(&target)
            || self.modules.contains(&target)
            || external(&target).is_some()
            || external_namespace(&target)
        {
            target
        } else if let Some((owner, name)) = target.rsplit_once("::") {
            if let Some(import) = self.imports.get(owner).and_then(|u| u.get(name)) {
                self.resolve(owner, import, visiting)?
            } else {
                bail!("{module}: unknown type owner {path} (resolved {target})");
            }
        } else {
            bail!("{module}: unknown type owner {path}");
        };
        visiting.remove(&key);
        Ok(result)
    }
    fn ty(&self, module: &str, ty: &Type, edges: &mut BTreeSet<String>) -> Result<Value> {
        match ty {
            Type::Path(p) if p.qself.is_none() => {
                let mut raw = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                if p.path.leading_colon.is_some() {
                    raw.insert_str(0, "::");
                }
                let mut args = Vec::new();
                for (i, s) in p.path.segments.iter().enumerate() {
                    match &s.arguments {
                        PathArguments::None => {}
                        PathArguments::AngleBracketed(a) if i + 1 == p.path.segments.len() => {
                            for a in &a.args {
                                if let GenericArgument::Type(t) = a {
                                    args.push(self.ty(module, t, edges)?);
                                } else {
                                    bail!("{module}: unhandled generic argument {}", tokens(a));
                                }
                            }
                        }
                        _ => bail!("{module}: unsupported path arguments {}", tokens(ty)),
                    }
                }
                let resolved = self.resolve(module, &raw, &mut BTreeSet::new())?;
                if let Some(arity) = external(&resolved) {
                    if args.len() != arity {
                        bail!("{module}: container {resolved} requires {arity} arguments");
                    }
                } else {
                    if external_namespace(&resolved) {
                        bail!("{module}: external namespace {resolved} is not a type");
                    }
                    if !args.is_empty() {
                        bail!("{module}: unresolved generic declaration {resolved}");
                    }
                    edges.insert(resolved.clone());
                }
                Ok(json!({"path":resolved,"arguments":args}))
            }
            Type::Tuple(t) => Ok(
                json!({"tuple":t.elems.iter().map(|t|self.ty(module,t,edges)).collect::<Result<Vec<_>>>()?}),
            ),
            _ => bail!("{module}: unsupported declaration type {}", tokens(ty)),
        }
    }
    fn fields(
        &self,
        module: &str,
        owner: &str,
        fields: &Fields,
        out: &mut BTreeMap<String, String>,
        edges: &mut BTreeSet<String>,
    ) -> Result<Value> {
        let mut shapes = Vec::new();
        for (i, f) in fields.iter().enumerate() {
            if !active(&f.attrs, owner)? {
                continue;
            }
            let name = f
                .ident
                .as_ref()
                .map_or_else(|| i.to_string(), ToString::to_string);
            let id = format!("{owner}/field/{name}");
            let shape = json!({"type":self.ty(module,&f.ty,edges)?,"visibility":tokens(&f.vis),"attributes":attrs(&f.attrs,&id)?});
            out.insert(format!("rust:{id}"), super::hash_json(&shape));
            shapes.push(json!({"name":name,"shape":shape}));
        }
        Ok(
            json!({"kind":match fields{Fields::Named(_)=>"named",Fields::Unnamed(_)=>"tuple",Fields::Unit=>"unit"},"fields":shapes}),
        )
    }
    fn extract(&self, roots: &[&str]) -> Result<Value> {
        let mut pending: BTreeSet<String> = roots.iter().map(|x| (*x).into()).collect();
        let mut visited = BTreeSet::new();
        let mut out = BTreeMap::new();
        let mut relations = BTreeMap::new();
        while let Some(owner) = pending.pop_first() {
            if !visited.insert(owner.clone()) {
                continue;
            }
            let item = self
                .declarations
                .get(&owner)
                .with_context(|| format!("missing root/declaration {owner}"))?;
            let module = owner.rsplit_once("::").context("declaration owner")?.0;
            let mut edges = BTreeSet::new();
            let shape = match item {
                Item::Struct(s) => {
                    if !s.generics.params.is_empty() {
                        bail!("{owner}: generic declaration unsupported");
                    }
                    json!({"kind":"struct","visibility":tokens(&s.vis),"attributes":attrs(&s.attrs,&owner)?,"body":self.fields(module,&owner,&s.fields,&mut out,&mut edges)?})
                }
                Item::Enum(e) => {
                    if !e.generics.params.is_empty() {
                        bail!("{owner}: generic enum unsupported");
                    }
                    let mut variants = Vec::new();
                    for v in &e.variants {
                        if !active(&v.attrs, &owner)? {
                            continue;
                        }
                        let id = format!("{owner}/variant/{}", v.ident);
                        if v.discriminant.is_some() {
                            bail!("{id}: explicit enum discriminant unsupported");
                        }
                        let shape = json!({"attributes":attrs(&v.attrs,&id)?,"body":self.fields(module,&id,&v.fields,&mut out,&mut edges)?});
                        out.insert(format!("rust:{id}"), super::hash_json(&shape));
                        variants.push(json!({"name":v.ident.to_string(),"shape":shape}));
                    }
                    json!({"kind":"enum","visibility":tokens(&e.vis),"attributes":attrs(&e.attrs,&owner)?,"variants":variants})
                }
                Item::Type(a) => {
                    if !a.generics.params.is_empty() {
                        bail!("{owner}: unresolved generic alias");
                    }
                    json!({"kind":"alias","visibility":tokens(&a.vis),"attributes":attrs(&a.attrs,&owner)?,"type":self.ty(module,&a.ty,&mut edges)?})
                }
                _ => bail!("{owner}: unsupported declaration"),
            };
            out.insert(format!("rust:{owner}"), super::hash_json(&shape));
            pending.extend(edges.iter().cloned());
            relations.insert(owner, edges);
        }
        Ok(
            json!({"obligations":out,"references":relations,"macro_definitions":self.macros,"diagnostic_macros":self.diagnostics}),
        )
    }
}
// Only prefixes of the closed external type table may serve as import namespaces.
// Resolving a namespace never admits it as an opaque model type.
fn external_namespace(path: &str) -> bool {
    matches!(
        path,
        "std"
            | "std::option"
            | "std::boxed"
            | "std::vec"
            | "std::collections"
            | "std::string"
            | "std::num"
    )
}
fn external(path: &str) -> Option<usize> {
    match path {
        "Option"
        | "std::option::Option"
        | "Box"
        | "std::boxed::Box"
        | "Vec"
        | "std::vec::Vec"
        | "std::collections::BTreeSet" => Some(1),
        "std::collections::BTreeMap" => Some(2),
        "String"
        | "std::string::String"
        | "bool"
        | "char"
        | "u8"
        | "u16"
        | "u32"
        | "u64"
        | "u128"
        | "usize"
        | "i8"
        | "i16"
        | "i32"
        | "i64"
        | "i128"
        | "isize"
        | "f32"
        | "f64"
        // A periodic interval is a positive count of units: the same leaf as `u32`, with the
        // zero excluded at the type rather than by a refusal a reader has to find.
        | "NonZeroU32"
        | "std::num::NonZeroU32" => Some(0),
        _ => None,
    }
}
struct Wrapper {
    name: syn::Ident,
    ty: Type,
    handle: bool,
    attrs: Vec<Attribute>,
}
struct Wrappers(Vec<Wrapper>);
impl Parse for Wrappers {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut result = Vec::new();
        while !input.is_empty() {
            let attrs = input.call(Attribute::parse_outer)?;
            let name = input.parse()?;
            let inner;
            syn::parenthesized!(inner in input);
            let ty = inner.parse()?;
            if !inner.is_empty() {
                return Err(inner.error("one underlying type required"));
            }
            let handle = if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                input.parse::<syn::Ident>()?;
                input.parse::<Token![:]>()?;
                input.parse::<syn::Ident>()?;
                input.parse::<Token![in]>()?;
                input.parse::<syn::Ident>()?;
                true
            } else {
                let from = input.parse::<syn::Ident>()?;
                if from != "from" {
                    return Err(input.error("expected from"));
                }
                input.parse::<Type>()?;
                false
            };
            input.parse::<Token![,]>()?;
            input.parse::<syn::LitStr>()?;
            input.parse::<Token![;]>()?;
            result.push(Wrapper {
                name,
                ty,
                handle,
                attrs,
            });
        }
        Ok(Self(result))
    }
}
/// `profile_word!(Name, Variant, "doc")` — the grammar the periodic profile words are written in.
struct ProfileWord {
    name: syn::Ident,
    variant: syn::Ident,
    doc: syn::LitStr,
}
impl Parse for ProfileWord {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<Token![,]>()?;
        let variant = input.parse::<syn::Ident>()?;
        input.parse::<Token![,]>()?;
        let doc = input.parse::<syn::LitStr>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("a profile word is exactly a name, a variant and its doc"));
        }
        Ok(Self { name, variant, doc })
    }
}

struct Conversions;
impl Parse for Conversions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        while !input.is_empty() {
            input.parse::<syn::Ident>()?;
            input.parse::<Token![=>]>()?;
            input.parse::<syn::Ident>()?;
            input.parse::<Token![;]>()?;
        }
        Ok(Self)
    }
}

fn guard(key: &str, kind: &str, body: &proc_macro2::TokenStream) -> Result<()> {
    let guards: Value = serde_json::from_str(include_str!("macro-guards.json"))?;
    let expected = guards[key][kind]
        .as_str()
        .with_context(|| format!("{key}: unclassified production macro {kind}"))?;
    let actual = super::hash_bytes(tokens(body).as_bytes());
    if actual != expected {
        bail!("{key}: {kind} guard mismatch: expected {expected}, found {actual}");
    }
    Ok(())
}
struct Codes(Vec<String>);
impl Parse for Codes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut out = Vec::new();
        while !input.is_empty() {
            input.call(Attribute::parse_outer)?;
            let name = input.parse::<syn::Ident>()?;
            input.parse::<Token![=]>()?;
            input.parse::<syn::Expr>()?;
            input.parse::<Token![,]>()?;
            input.parse::<syn::Expr>()?;
            input.parse::<Token![;]>()?;
            out.push(name.to_string());
        }
        Ok(Self(out))
    }
}
