//! Assemble an explicitly selected authored site without crawling its source directory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use ess_gen::document::{Block, Page, PageId, Target};
use ess_gen::Artifact;
use fluent_uri::IriRef;

#[derive(Debug, Default, clap::Args)]
pub struct Options {
    /// An authored Markdown page, written `<page-id>=<path>`. Repeat for multiple pages.
    #[arg(long, value_name = "PAGE=PATH")]
    pub include: Vec<String>,
    /// Override the specification-adjacent README without changing its source directory.
    #[arg(long, value_name = "PATH")]
    pub front_page: Option<PathBuf>,
    /// Publish a declared UTF-8 download verbatim, written `<output-path>=<source-path>`.
    #[arg(long, value_name = "OUTPUT=PATH")]
    pub asset: Vec<String>,
    /// Refuse unpublished local link targets before writing any output.
    #[arg(long)]
    pub strict_links: bool,
}

struct Input {
    source: PathBuf,
    destination: String,
    text: String,
}

impl Input {
    fn page_id(&self) -> &str {
        self.destination
            .strip_suffix(".html")
            .unwrap_or(&self.destination)
    }

    fn line(&self, offset: usize) -> usize {
        self.text[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    }

    fn read(source: &Path, destination: String) -> Result<Self> {
        Ok(Self {
            source: absolute(source)?,
            destination,
            text: fs::read_to_string(source)
                .with_context(|| format!("reading selected UTF-8 source {}", source.display()))?,
        })
    }
}

fn declaration<'a>(argument: &'a str, option: &str) -> Result<(&'a str, &'a Path)> {
    let (destination, source) = argument.split_once('=').with_context(|| {
        format!("`--{option} {argument}` must be `<destination>=<source-path>`")
    })?;
    ess_gen::artifact::validate_path(destination).map_err(anyhow::Error::msg)?;
    Ok((destination, Path::new(source)))
}

pub fn render(
    path: &Path,
    options: &Options,
    ir: &ess_compiler::EssIr,
) -> Result<BTreeMap<String, Artifact>> {
    let Inputs {
        front,
        pages,
        assets,
    } = Inputs::read(path, options)?;

    let mut destinations: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for input in front.iter().chain(&pages).chain(&assets) {
        destinations
            .entry(input.source.clone())
            .or_default()
            .insert(input.destination.clone());
    }
    let mint = ess_gen::provenance::ProvenanceMint::new(ir);
    let mut document = ess_gen::docs::document(ir, &mint);
    let mut errors = Vec::new();
    let mut fragments = Vec::new();
    let mut parse = |input: &Input| {
        ess_gen::authored::titled_with_links(&input.text, input.page_id(), |url, offset| {
            match resolve(url, input, &destinations) {
                Ok(link) => {
                    if let Some((destination, fragment)) = link.fragment {
                        fragments.push((
                            destination,
                            fragment,
                            input.source.clone(),
                            offset,
                            url.to_owned(),
                        ));
                    }
                    Target::External { url: link.url }
                }
                Err(error) => {
                    if options.strict_links {
                        let line = input.line(offset);
                        errors.push(format!(
                            "{}:{line}: link `{url}`: {error}",
                            input.source.display()
                        ));
                    }
                    Target::External {
                        url: url.to_owned(),
                    }
                }
            }
        })
    };
    let introduction = front
        .as_ref()
        .map(|input| parse(input).1)
        .unwrap_or_default();
    for input in &pages {
        let (title, blocks) = parse(input);
        document.pages.push(Page {
            id: PageId(input.page_id().to_owned()),
            title,
            about: None,
            provenance: mint.whole(),
            blocks,
        });
    }
    if options.strict_links {
        let anchors = page_anchors(&document, &introduction);
        for (destination, fragment, source, offset, url) in fragments {
            if let Some(found) = anchors.get(&destination) {
                if !found.contains(&fragment) {
                    let input = front
                        .iter()
                        .chain(&pages)
                        .find(|input| input.source == source)
                        .context("fragment source is not a selected page")?;
                    let line = input.line(offset);
                    errors.push(format!("{}:{line}: link `{url}`: fragment `{fragment}` is absent from `{destination}`", source.display()));
                }
            }
        }
    }
    if !errors.is_empty() {
        bail!("unresolved authored site links:\n{}", errors.join("\n"));
    }
    let mut artifacts = ess_gen::html::Site::new()
        .with_front_page(introduction)
        .try_render(&document, &mint.whole().provenance)
        .map_err(anyhow::Error::msg)?;
    for input in assets {
        artifacts.push(Artifact::new(input.destination, input.text));
    }
    ess_gen::artifact::validate_paths(artifacts.iter().map(|artifact| artifact.path.as_str()))
        .map_err(anyhow::Error::msg)?;
    Ok(artifacts
        .into_iter()
        .map(|artifact| (format!("site/{}", artifact.path), artifact))
        .collect())
}

struct Link {
    url: String,
    fragment: Option<(String, String)>,
}

fn resolve(
    url: &str,
    input: &Input,
    destinations: &BTreeMap<PathBuf, BTreeSet<String>>,
) -> Result<Link> {
    let reference = IriRef::parse(url).context("invalid URI reference")?;
    if reference.scheme().is_some() || reference.authority().is_some() {
        return Ok(Link {
            url: url.to_owned(),
            fragment: None,
        });
    }
    let fragment = reference
        .fragment()
        .filter(|fragment| !fragment.is_empty())
        .map(|fragment| {
            fragment
                .decode()
                .to_string()
                .map(std::borrow::Cow::into_owned)
                .map_err(|_| anyhow::anyhow!("fragment is not UTF-8"))
        })
        .transpose()?;
    if reference.path().is_empty() {
        return Ok(Link {
            url: url.to_owned(),
            fragment: fragment.map(|fragment| (input.destination.clone(), fragment)),
        });
    }
    let path = reference
        .path()
        .decode()
        .to_string()
        .map_err(|_| anyhow::anyhow!("path is not UTF-8"))?;
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        bail!("local target must be a relative source path");
    }
    let target = normalize(
        &input
            .source
            .parent()
            .unwrap_or(Path::new("."))
            .join(path.as_ref()),
    );
    let candidates = destinations
        .get(&target)
        .or_else(|| destinations.get(&target.join("README.md")))
        .context("target is not an included page or declared asset")?;
    if candidates.len() != 1 {
        bail!("source has multiple publication destinations");
    }
    let destination = candidates
        .first()
        .context("target has no publication destination")?;
    let mut resolved = relative(&input.destination, destination);
    if let Some(query) = reference.query() {
        resolved.push('?');
        resolved.push_str(query.as_str());
    }
    if let Some(fragment) = reference.fragment() {
        resolved.push('#');
        resolved.push_str(fragment.as_str());
    }
    Ok(Link {
        url: resolved,
        fragment: fragment.map(|fragment| (destination.clone(), fragment)),
    })
}

fn section_anchors(blocks: &[Block], into: &mut BTreeSet<String>) {
    for block in blocks {
        match block {
            Block::Section { anchor, blocks, .. } => {
                into.insert(anchor.clone());
                section_anchors(blocks, into);
            }
            Block::Quote { blocks } => section_anchors(blocks, into),
            Block::List { items, .. } => {
                for item in items {
                    section_anchors(item, into);
                }
            }
            _ => {}
        }
    }
}

fn page_anchors(
    document: &ess_gen::document::Document,
    introduction: &[Block],
) -> BTreeMap<String, BTreeSet<String>> {
    document
        .pages
        .iter()
        .map(|page| {
            let mut found = BTreeSet::new();
            section_anchors(&page.blocks, &mut found);
            if page.id.as_str() == "index" {
                section_anchors(introduction, &mut found);
            }
            (format!("{}.html", page.id.as_str()), found)
        })
        .collect()
}

fn absolute(path: &Path) -> Result<PathBuf> {
    Ok(normalize(&if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    }))
}

fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            component => result.push(component),
        }
    }
    result
}

fn relative(from: &str, to: &str) -> String {
    let mut parent: Vec<_> = from.split('/').collect();
    parent.pop();
    let target: Vec<_> = to.split('/').collect();
    let shared = parent
        .iter()
        .zip(&target)
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts = vec![".."; parent.len() - shared];
    parts.extend_from_slice(&target[shared..]);
    parts.join("/")
}

struct Inputs {
    front: Option<Input>,
    pages: Vec<Input>,
    assets: Vec<Input>,
}

impl Inputs {
    fn read(path: &Path, options: &Options) -> Result<Self> {
        let directory = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(Path::new("."))
        };
        let default_front = directory.join("README.md");
        let front_path = options.front_page.as_deref().unwrap_or(&default_front);
        let front = match Input::read(front_path, "index.html".to_owned()) {
            Ok(input) => Some(input),
            Err(error)
                if options.front_page.is_none()
                    && error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound) =>
            {
                None
            }
            Err(error) => return Err(error),
        };
        let mut pages = Vec::new();
        for argument in &options.include {
            let (id, source) = declaration(argument, "include")?;
            pages.push(Input::read(source, format!("{id}.html"))?);
        }
        let mut assets = Vec::new();
        for argument in &options.asset {
            let (destination, source) = declaration(argument, "asset")?;
            assets.push(Input::read(source, destination.to_owned())?);
        }
        Ok(Self {
            front,
            pages,
            assets,
        })
    }
}
