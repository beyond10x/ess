//! Concrete source identities are candidates; no source assertion is an executed case.
use anyhow::{bail, Context, Result};
use quote::{quote, ToTokens};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};
use syn::{
    visit::{self, Visit},
    Attribute, Item,
};

#[derive(Default)]
struct Inventory {
    entries: BTreeMap<String, Value>,
    cases: BTreeMap<String, Value>,
    visited: BTreeSet<String>,
}
#[derive(Clone)]
struct Scope<'a> {
    owner: &'a str,
    module: &'a str,
    test: bool,
    profile: &'a [String],
    file: &'a str,
    sources: &'a BTreeMap<String, String>,
    directory: PathBuf,
}
#[cfg(test)]
pub(super) fn fixture(source: &str) -> Result<Value> {
    let mut out = Inventory::default();
    out.file(
        Scope {
            owner: "fixture",
            module: "",
            test: false,
            profile: &[],
            file: "fixture.rs",
            sources: &BTreeMap::new(),
            directory: PathBuf::new(),
        },
        source,
    )?;
    Ok(json!({"entries":out.entries,"cases":out.cases}))
}
pub(super) fn extract(
    root: &Path,
    sources: &BTreeMap<String, String>,
    metadata: &Value,
) -> Result<Value> {
    let mut packages = BTreeMap::new();
    for package in metadata["packages"]
        .as_array()
        .context("Cargo package inventory")?
    {
        let name = package["name"].as_str().context("Cargo package name")?;
        let mut out = Inventory::default();
        for target in package["targets"].as_array().context("Cargo targets")? {
            let path = Path::new(target["src_path"].as_str().context("Cargo target path")?)
                .strip_prefix(root)?
                .to_str()
                .context("UTF-8 target path")?;
            let source = sources
                .get(path)
                .with_context(|| format!("{name}: missing target source {path}"))?;
            let kind = target["kind"][0].as_str().context("Cargo target kind")?;
            if !matches!(
                kind,
                "lib" | "bin" | "test" | "example" | "custom-build" | "cdylib" | "rlib"
            ) {
                bail!("{name}: unknown Cargo target kind {kind}");
            }
            let target_name = target["name"].as_str().context("Cargo target name")?;
            let owner = format!("{}::{kind}({target_name})", name.replace('-', "_"));
            let mut target_out = Inventory::default();
            target_out.file(
                Scope {
                    owner: &owner,
                    module: "",
                    test: kind == "test",
                    profile: &[],
                    file: path,
                    sources,
                    directory: Path::new(path).parent().context("target parent")?.into(),
                },
                source,
            )?;
            for (id, mut value) in target_out.cases {
                value["package"] = json!(name);
                value["target_kind"] = json!(kind);
                value["target_name"] = json!(target_name);
                value["required_features"] = target["required-features"].clone();
                out.cases
                    .insert(format!("{name}:{kind}:{target_name}:{id}"), value);
            }
            out.entries.extend(target_out.entries);
        }
        packages.insert(name.to_owned(),json!({"manifest":package["manifest_path"],"features":package["features"],"targets":package["targets"],"entries":out.entries,"cases":out.cases}));
    }
    Ok(json!({"packages":packages,"evidence_status":"SOURCE_CANDIDATES_UNEXECUTED"}))
}
fn text(x: &impl ToTokens) -> String {
    x.to_token_stream().to_string()
}
fn without_docs(stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    use proc_macro2::{Delimiter, Group, TokenTree};
    let tokens = stream.into_iter().collect::<Vec<_>>();
    let mut out = proc_macro2::TokenStream::new();
    let mut index = 0;
    while index < tokens.len() {
        let mut next = index + 1;
        if matches!(tokens.get(next),Some(TokenTree::Punct(p)) if p.as_char()=='!') {
            next += 1;
        }
        if matches!(&tokens[index],TokenTree::Punct(p) if p.as_char()=='#')
            && matches!(tokens.get(next),Some(TokenTree::Group(g)) if g.delimiter()==Delimiter::Bracket&&matches!(g.stream().into_iter().next(),Some(TokenTree::Ident(i)) if i=="doc"))
        {
            index = next + 1;
            continue;
        }
        let token = match &tokens[index] {
            TokenTree::Group(group) => {
                TokenTree::Group(Group::new(group.delimiter(), without_docs(group.stream())))
            }
            other => other.clone(),
        };
        out.extend([token]);
        index += 1;
    }
    out
}
fn declaration(item: &Item) -> String {
    let stream = if let Item::Fn(function) = item {
        let attrs = &function.attrs;
        let visibility = &function.vis;
        let signature = &function.sig;
        quote!(#(#attrs)* #visibility #signature)
    } else {
        item.to_token_stream()
    };
    super::hash_bytes(without_docs(stream).to_string().as_bytes())
}
fn attributes(item: &Item) -> &[Attribute] {
    match item {
        Item::Fn(x) => &x.attrs,
        Item::Mod(x) => &x.attrs,
        Item::Impl(x) => &x.attrs,
        Item::Struct(x) => &x.attrs,
        Item::Enum(x) => &x.attrs,
        Item::Type(x) => &x.attrs,
        Item::Use(x) => &x.attrs,
        Item::Trait(x) => &x.attrs,
        Item::Macro(x) => &x.attrs,
        Item::Const(x) => &x.attrs,
        Item::Static(x) => &x.attrs,
        Item::ExternCrate(x) => &x.attrs,
        _ => &[],
    }
}
fn conditions(attrs: &[Attribute]) -> Result<(bool, Vec<String>)> {
    let mut test = false;
    let mut out = Vec::new();
    for a in attrs {
        if a.path().is_ident("cfg") {
            let m = a.parse_args::<syn::Meta>()?;
            let s = text(&m);
            match s.as_str() {
                "test" => test = true,
                "unix"
                | "not (unix)"
                | "target_os = \"linux\""
                | "feature = \"go-typecheck\""
                | "feature = \"typescript-typecheck\"" => out.push(s),
                _ => bail!("unhandled consumer cfg {s}"),
            }
        }
        if a.path().is_ident("cfg_attr") {
            bail!("unhandled consumer cfg_attr {}", text(a));
        }
    }
    Ok((test, out))
}
fn normalize(path: &Path) -> Result<String> {
    let mut segments = Vec::new();
    for p in path.components() {
        match p {
            Component::Normal(x) => segments.push(x.to_str().context("UTF-8 module path")?),
            Component::CurDir => {}
            Component::ParentDir => {
                segments
                    .pop()
                    .context("module path escapes source authority")?;
            }
            _ => bail!("absolute module path"),
        }
    }
    Ok(segments.join("/"))
}
impl Inventory {
    fn file(&mut self, scope: Scope<'_>, source: &str) -> Result<()> {
        let file = syn::parse_file(source)?;
        let (test, mut profile) = conditions(&file.attrs)?;
        profile.extend_from_slice(scope.profile);
        self.walk(
            &Scope {
                test: scope.test || test,
                profile: &profile,
                ..scope
            },
            &file.items,
        )
    }
    fn walk(&mut self, scope: &Scope<'_>, items: &[Item]) -> Result<()> {
        let Scope {
            owner,
            module,
            test,
            profile,
            file,
            ..
        } = scope.clone();
        let module_owner = if module.is_empty() {
            owner.into()
        } else {
            format!("{owner}::{module}")
        };
        if !self.visited.insert(module_owner.clone()) {
            bail!("duplicate consumer module {module_owner}");
        }
        if !test {
            self.entry(format!("{module_owner}::module"),json!({"source":file,"classification":"owned-module-boundary","reason":"All concrete production items are separately inventoried; this module grants no inherited support or exemption.","profile_conditions":profile}))?;
        }
        for item in items {
            let (attrs_test, local_profile) =
                conditions(attributes(item)).with_context(|| format!("{file} {module_owner}"))?;
            let test = test || attrs_test;
            let mut conditions = profile.to_vec();
            conditions.extend(local_profile);
            let common = json!({"source":file,"source_item_sha256":super::hash_bytes(text(item).as_bytes()),"declaration_sha256":declaration(item),"profile_conditions":conditions,"classification":"bounded-owner-entry-unproven","reason":"This exact source entry participates in the owned boundary; its semantic effects require attributed execution. No parent-package support is inferred."});
            match item {
                Item::Mod(m) => self.module(
                    &Scope {
                        test,
                        profile: &conditions,
                        ..scope.clone()
                    },
                    m,
                )?,
                Item::Fn(f) => self.function(
                    &Scope {
                        test,
                        profile: &conditions,
                        ..scope.clone()
                    },
                    f,
                    common,
                )?,
                Item::Impl(i) if !test => self.implementation(&module_owner, i, &common)?,
                Item::Struct(s) if !test => {
                    self.entry(format!("{module_owner}::struct::{}", s.ident), common)?;
                }
                Item::Enum(e) if !test => {
                    self.entry(format!("{module_owner}::enum::{}", e.ident), common.clone())?;
                    for v in &e.variants {
                        let mut row = common.clone();
                        row["variant_ast_sha256"] = json!(super::hash_bytes(text(v).as_bytes()));
                        self.entry(
                            format!("{module_owner}::enum::{}/variant/{}", e.ident, v.ident),
                            row,
                        )?;
                    }
                }
                Item::Type(a) if !test => {
                    self.entry(format!("{module_owner}::type::{}", a.ident), common)?;
                }
                Item::Trait(t) if !test => {
                    self.trait_declaration(&module_owner, t, &common)?;
                }
                Item::Const(c) if !test => {
                    self.entry(format!("{module_owner}::const::{}", c.ident), common)?;
                }
                Item::Static(s) if !test => {
                    self.entry(format!("{module_owner}::static::{}", s.ident), common)?;
                }
                Item::Macro(m) if !test => self.item_macro(&module_owner, m, common)?,
                Item::Use(u) if !test && !matches!(u.vis, syn::Visibility::Inherited) => {
                    self.reexports(&module_owner, u, &common)?;
                }
                Item::Use(_)
                | Item::ExternCrate(_)
                | Item::Impl(_)
                | Item::Struct(_)
                | Item::Enum(_)
                | Item::Type(_)
                | Item::Trait(_)
                | Item::Const(_)
                | Item::Static(_)
                | Item::Macro(_) => {}
                _ => bail!("{file}: unhandled consumer item {}", text(item)),
            }
        }
        Ok(())
    }
    fn reexports(&mut self, owner: &str, item: &syn::ItemUse, common: &Value) -> Result<()> {
        let mut names = Vec::new();
        export_names(&item.tree, "", &mut names)?;
        for (name, target) in names {
            let mut row = common.clone();
            row["reexport_target"] = json!(target);
            self.entry(format!("{owner}::reexport::{name}"), row)?;
        }
        Ok(())
    }
    fn item_macro(&mut self, module_owner: &str, m: &syn::ItemMacro, common: Value) -> Result<()> {
        let id = if let Some(name) = &m.ident {
            format!("definition/{name}")
        } else {
            let anchor = if matches!(
                text(&m.mac.path).as_str(),
                "checked_deserialize" | "crate :: validation :: checked_deserialize"
            ) {
                let invocation = syn::parse2::<CheckedDeserialize>(m.mac.tokens.clone())
                    .with_context(|| {
                        format!("{module_owner}: checked_deserialize invocation grammar")
                    })?;
                format!("/for/{}", invocation.0)
            } else {
                String::new()
            };
            format!("invocation/{}{anchor}", text(&m.mac.path))
        };
        let mut row = common;
        row["classification"] =
            json!("production-macro-requires-explicit-definition-classification");
        self.entry(format!("{module_owner}::macro::{id}"), row)?;

        Ok(())
    }
    fn module(&mut self, scope: &Scope<'_>, m: &syn::ItemMod) -> Result<()> {
        let Scope {
            module,
            test,
            profile: conditions,
            file,
            sources,
            ref directory,
            ..
        } = scope.clone();

        let next = if module.is_empty() {
            m.ident.to_string()
        } else {
            format!("{module}::{}", m.ident)
        };
        if let Some((_, items)) = &m.content {
            self.walk(
                &Scope {
                    module: &next,
                    test,
                    profile: conditions,
                    directory: directory.join(m.ident.to_string()),
                    ..scope.clone()
                },
                items,
            )?;
        } else {
            let explicit = m
                .attrs
                .iter()
                .find(|a| a.path().is_ident("path"))
                .map(|a| match &a.meta {
                    syn::Meta::NameValue(n) => match &n.value {
                        syn::Expr::Lit(x) => match &x.lit {
                            syn::Lit::Str(s) => Ok(s.value()),
                            _ => bail!("module path must be string"),
                        },
                        _ => bail!("module path must be literal"),
                    },
                    _ => bail!("module path attribute shape"),
                })
                .transpose()?;
            let path = Path::new(file);
            let parent = path.parent().context("module parent")?;
            let base = directory;
            let possible = if let Some(p) = explicit {
                vec![normalize(&parent.join(p))?]
            } else {
                vec![
                    normalize(&base.join(format!("{}.rs", m.ident)))?,
                    normalize(&base.join(m.ident.to_string()).join("mod.rs"))?,
                ]
            };
            let found: Vec<_> = possible
                .iter()
                .filter_map(|p| sources.get(p).map(|s| (p, s)))
                .collect();
            let [(p, s)] = found.as_slice() else {
                bail!("{file} {next}: absent or ambiguous module source {possible:?}");
            };
            let next_directory = if Path::new(p).file_name().is_some_and(|n| n == "mod.rs") {
                Path::new(p)
                    .parent()
                    .context("module parent")?
                    .to_path_buf()
            } else {
                Path::new(p).with_extension("")
            };
            self.file(
                Scope {
                    module: &next,
                    test,
                    profile: conditions,
                    file: p,
                    directory: next_directory,
                    ..scope.clone()
                },
                s,
            )?;
        }

        Ok(())
    }
    fn function(&mut self, scope: &Scope<'_>, f: &syn::ItemFn, common: Value) -> Result<()> {
        let Scope {
            owner,
            module,
            test,
            profile: conditions,
            file,
            ..
        } = scope.clone();
        let module_owner = if module.is_empty() {
            owner.into()
        } else {
            format!("{owner}::{module}")
        };

        if f.attrs.iter().any(|a| a.path().is_ident("test")) {
            let name = if module.is_empty() {
                f.sig.ident.to_string()
            } else {
                format!("{module}::{}", f.sig.ident)
            };
            let mut body = Body::default();
            body.visit_block(&f.block);
            let value = json!({"source":file,"full_name":name,"case_ast_sha256":super::hash_bytes(text(f).as_bytes()),"evidence_status":"SOURCE_CANDIDATE_UNEXECUTED","ignored":f.attrs.iter().any(|a|a.path().is_ident("ignore")),"profile_conditions":conditions,"contains_early_return":body.early_return,"contains_catch_unwind":body.catch_unwind,"nested_command_candidates":body.commands,"assertion_candidates":body.assertions,"attribution":"unreviewed; source assertions and command candidates do not establish actual execution"});
            if self.cases.insert(name.clone(), value).is_some() {
                bail!("duplicate source case {name}");
            }
        } else if !test {
            self.entry(format!("{module_owner}::fn::{}", f.sig.ident), common)?;
        }

        Ok(())
    }
    fn implementation(
        &mut self,
        module_owner: &str,
        i: &syn::ItemImpl,
        common: &Value,
    ) -> Result<()> {
        let implementation = format!(
            "impl<{};{}>",
            text(&i.self_ty),
            i.trait_
                .as_ref()
                .map_or_else(String::new, |(_, p, _)| text(p))
        );
        let context = implementation_context(i);
        for member in &i.items {
            let (test, profile) = conditions(implementation_attributes(member))
                .with_context(|| format!("{module_owner}::{implementation}"))?;
            if test {
                continue;
            }
            let (name, declaration) = match member {
                syn::ImplItem::Fn(f) => {
                    let attrs = &f.attrs;
                    let visibility = &f.vis;
                    let signature = &f.sig;
                    let defaultness = &f.defaultness;
                    let associated = associated_contract(i, signature)?;
                    (f.sig.ident.to_string(), quote!(#context #(#attrs)* #visibility #defaultness #signature #associated))
                }
                syn::ImplItem::Const(c) => (format!("const::{}", c.ident), quote!(#context #c)),
                syn::ImplItem::Type(t) => (format!("type::{}", t.ident), quote!(#context #t)),
                _ => bail!("{module_owner}::{implementation}: unsupported active associated declaration {}", text(member)),
            };
            let mut row = common.clone();
            row["source_item_sha256"] = json!(super::hash_bytes(text(member).as_bytes()));
            row["declaration_sha256"] = json!(super::hash_bytes(
                without_docs(declaration).to_string().as_bytes()
            ));
            row["member_profile_conditions"] = json!(profile);
            self.entry(format!("{module_owner}::{implementation}::{name}"), row)?;
        }

        Ok(())
    }
    fn trait_declaration(
        &mut self,
        module_owner: &str,
        item: &syn::ItemTrait,
        common: &Value,
    ) -> Result<()> {
        let owner = format!("{module_owner}::trait::{}", item.ident);
        // Preserve the existing full parent declaration, including its methods.
        self.entry(owner.clone(), common.clone())?;
        let mut context = item.clone();
        context.items.clear();
        for member in &item.items {
            let attrs: &[Attribute] = match member {
                syn::TraitItem::Fn(f) => &f.attrs,
                syn::TraitItem::Const(c) => &c.attrs,
                syn::TraitItem::Type(t) => &t.attrs,
                syn::TraitItem::Macro(m) => &m.attrs,
                _ => &[],
            };
            let (test, profile) = conditions(attrs).with_context(|| owner.clone())?;
            if test {
                continue;
            }
            let name = match member {
                syn::TraitItem::Fn(_) => continue,
                syn::TraitItem::Const(c) => format!("const::{}", c.ident),
                syn::TraitItem::Type(t) => format!("type::{}", t.ident),
                _ => bail!(
                    "{owner}: unsupported active associated declaration {}",
                    text(member)
                ),
            };
            let mut row = common.clone();
            row["source_item_sha256"] = json!(super::hash_bytes(text(member).as_bytes()));
            row["declaration_sha256"] = json!(super::hash_bytes(
                without_docs(quote!(#context #member))
                    .to_string()
                    .as_bytes()
            ));
            row["member_profile_conditions"] = json!(profile);
            self.entry(format!("{owner}::{name}"), row)?;
        }
        Ok(())
    }
    fn entry(&mut self, id: String, value: Value) -> Result<()> {
        if self.entries.contains_key(&id) {
            bail!("duplicate concrete consumer entry {id}");
        }
        self.entries.insert(id, value);
        Ok(())
    }
}
fn implementation_context(i: &syn::ItemImpl) -> proc_macro2::TokenStream {
    let attrs = &i.attrs;
    let unsafety = &i.unsafety;
    let defaultness = &i.defaultness;
    let generics = &i.generics;
    let where_clause = &i.generics.where_clause;
    let self_type = &i.self_ty;
    let trait_path = i.trait_.as_ref().map(|(_, path, _)| path);
    let negative = i
        .trait_
        .as_ref()
        .and_then(|(negative, _, _)| negative.as_ref());
    quote!(#(#attrs)* #unsafety #generics #where_clause #self_type #negative #trait_path #defaultness)
}
fn implementation_attributes(member: &syn::ImplItem) -> &[Attribute] {
    match member {
        syn::ImplItem::Fn(f) => &f.attrs,
        syn::ImplItem::Const(c) => &c.attrs,
        syn::ImplItem::Type(t) => &t.attrs,
        syn::ImplItem::Macro(m) => &m.attrs,
        _ => &[],
    }
}
// A callable contract includes the associated declarations actually selected by its
// signature, recursively. Method bodies and unrelated associated members remain
// checkpoint evidence rather than eligibility identity.
fn associated_contract(
    i: &syn::ItemImpl,
    signature: &syn::Signature,
) -> Result<proc_macro2::TokenStream> {
    let mut names = AssociatedNames {
        trait_path: i.trait_.as_ref().map(|(_, path, _)| text(path)),
        ..AssociatedNames::default()
    };
    names.visit_signature(signature);
    let mut visited = BTreeSet::new();
    let mut declarations = BTreeMap::new();
    while let Some(name) = names.names.pop_first() {
        if !visited.insert(name.clone()) {
            continue;
        }
        for member in &i.items {
            let selected = match member {
                syn::ImplItem::Type(t) => t.ident == name,
                syn::ImplItem::Const(c) => c.ident == name,
                _ => false,
            };
            if selected && !conditions(implementation_attributes(member))?.0 {
                // Visiting the declaration discovers transitive Self:: references.
                names.visit_impl_item(member);
                declarations
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(member);
            }
        }
        if !declarations.contains_key(&name) {
            bail!(
                "{}::{}: unresolved associated contract Self::{name}",
                text(&i.self_ty),
                signature.ident
            );
        }
    }
    if let Some(error) = names.error {
        bail!("{}::{}: {error}", text(&i.self_ty), signature.ident);
    }
    let declarations = declarations.values().flatten();
    Ok(quote!(#(#declarations)*))
}
#[derive(Default)]
struct AssociatedNames {
    names: BTreeSet<String>,
    trait_path: Option<String>,
    error: Option<String>,
    references_self: bool,
}
impl AssociatedNames {
    fn qualified(&mut self, qself: Option<&syn::QSelf>, path: &syn::Path) {
        let Some(qself) = qself else { return };
        if !matches!(&*qself.ty, syn::Type::Path(p) if p.qself.is_none() && p.path.is_ident("Self"))
        {
            let mut receiver = Self::default();
            receiver.visit_type(&qself.ty);
            if receiver.references_self {
                self.error = Some(format!(
                    "unresolved associated contract through receiver {}",
                    text(&qself.ty)
                ));
            }
            return;
        }
        if qself.position > 0 {
            let mut owner = path.clone();
            owner.segments = path.segments.iter().take(qself.position).cloned().collect();
            if self.trait_path.as_deref() != Some(text(&owner).as_str()) {
                self.error = Some(format!(
                    "unresolved qualified associated contract <Self as {}>",
                    text(&owner)
                ));
                return;
            }
        }
        if let Some(member) = path.segments.iter().nth(qself.position) {
            self.names.insert(member.ident.to_string());
        }
    }
}
impl<'ast> Visit<'ast> for AssociatedNames {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none() && path.segments.first().is_some_and(|s| s.ident == "Self")
        {
            self.references_self = true;
            if let Some(member) = path.segments.iter().nth(1) {
                self.names.insert(member.ident.to_string());
            }
        }
        visit::visit_path(self, path);
    }
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        self.qualified(path.qself.as_ref(), &path.path);
        visit::visit_type_path(self, path);
    }
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        self.qualified(path.qself.as_ref(), &path.path);
        visit::visit_expr_path(self, path);
    }
}
struct CheckedDeserialize(syn::Ident);
impl syn::parse::Parse for CheckedDeserialize {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        let fields;
        syn::braced!(fields in input);
        while !fields.is_empty() {
            fields.call(Attribute::parse_outer)?;
            fields.parse::<syn::Visibility>()?;
            fields.parse::<syn::Ident>()?;
            fields.parse::<syn::Token![:]>()?;
            fields.parse::<syn::Type>()?;
            fields.parse::<syn::Token![,]>()?;
        }
        Ok(Self(name))
    }
}
#[derive(Default)]
struct Body {
    early_return: bool,
    catch_unwind: bool,
    commands: BTreeSet<String>,
    assertions: Vec<String>,
}
impl<'ast> Visit<'ast> for Body {
    fn visit_expr_return(&mut self, node: &'ast syn::ExprReturn) {
        self.early_return = true;
        visit::visit_expr_return(self, node);
    }
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*node.func {
            let path = text(&p.path);
            if path.ends_with("catch_unwind") {
                self.catch_unwind = true;
            }
            if path.ends_with("Command :: new") {
                if let Some(syn::Expr::Lit(x)) = node.args.first() {
                    if let syn::Lit::Str(s) = &x.lit {
                        self.commands.insert(s.value());
                    }
                }
            }
        }
        visit::visit_expr_call(self, node);
    }
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.segments.last().is_some_and(|s| {
            matches!(
                s.ident.to_string().as_str(),
                "assert" | "assert_eq" | "assert_ne" | "panic"
            )
        }) {
            self.assertions.push(text(node));
        }
        visit::visit_macro(self, node);
    }
}

fn export_names(tree: &syn::UseTree, prefix: &str, out: &mut Vec<(String, String)>) -> Result<()> {
    match tree {
        syn::UseTree::Path(p) => export_names(&p.tree, &format!("{prefix}{}::", p.ident), out)?,
        syn::UseTree::Name(n) => {
            let name = if n.ident == "self" {
                prefix
                    .trim_end_matches("::")
                    .rsplit("::")
                    .next()
                    .context("empty public self re-export")?
                    .into()
            } else {
                n.ident.to_string()
            };
            out.push((name, format!("{prefix}{}", n.ident)));
        }
        syn::UseTree::Rename(n) => out.push((n.rename.to_string(), format!("{prefix}{}", n.ident))),
        syn::UseTree::Group(g) => {
            for child in &g.items {
                export_names(child, prefix, out)?;
            }
        }
        syn::UseTree::Glob(_) => out.push((format!("glob/{prefix}*"), format!("{prefix}*"))),
    }
    Ok(())
}
