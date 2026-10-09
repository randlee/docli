//! HTML template packs.
//!
//! The bundled `default` pack is compiled into the binary with [`include_dir`].
//! [`resolve_pack`] also loads a pack directory from disk. [`render_pack`]
//! fills `page.html.j2` and `style.css.j2` with MiniJinja.
//!
//! Embedded-default failures are `DOCLI.INTERNAL` (the pack is not read from
//! disk). A filesystem read failure is `DOCLI.IO`. `DOCLI.TEMPLATE_*` codes
//! belong to later sprints.
//!
//! # Examples
//!
//! ```
//! use docli::schema::CliModel;
//! use docli::templates::{render_pack, resolve_pack, TemplateRef, ThemeMap};
//!
//! let model: CliModel = serde_json::from_str(r#"{"name":"demo"}"#)?;
//! let pack = resolve_pack(&TemplateRef::bundled("default"))?;
//! let theme = ThemeMap::defaults(&pack);
//! let html = render_pack(&pack, &model, &theme)?;
//! assert!(html.contains("demo CLI Reference"));
//! Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod page;

use std::backtrace::Backtrace;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use include_dir::{include_dir, Dir};
use minijinja::syntax::SyntaxConfig;
use minijinja::value::{Serde, Value};
use minijinja::{context, Environment, UndefinedBehavior};
use serde::{Deserialize, Serialize};

use crate::schema::CliModel;
use crate::search::search_index;

const DEFAULT_PACK_ID: &str = "default";
const MANIFEST_FILE: &str = "template.toml";
const PAGE_FILE: &str = "page.html.j2";
const STYLE_FILE: &str = "style.css.j2";
const SCRIPT_FILE: &str = "script.js";

static DEFAULT_PACK: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/templates/html/default");

/// Bundled pack id or a filesystem pack directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRef {
    /// Pack id compiled into this binary. Phase B embeds `default` only.
    Bundled(String),
    /// Directory that contains `template.toml`, `page.html.j2`, `style.css.j2`,
    /// and `script.js`.
    Dir(PathBuf),
}

impl TemplateRef {
    /// A bundled pack id such as `default`.
    pub fn bundled(id: impl Into<String>) -> Self {
        Self::Bundled(id.into())
    }

    /// A pack directory on disk.
    pub fn dir(path: impl Into<PathBuf>) -> Self {
        Self::Dir(path.into())
    }
}

/// One `theme_schema` entry from `template.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeKeySpec {
    /// Schema type, for example `color` or `string`.
    #[serde(rename = "type")]
    pub value_type: String,
    /// Value used when the caller does not override this key.
    pub default: String,
    /// Human description of the key.
    #[serde(default)]
    pub description: String,
}

/// Parsed `template.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateManifest {
    /// Stable pack id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Pack version string.
    pub version: String,
    /// One-line description of the pack.
    pub description: String,
    /// Oldest docli version this pack claims to support.
    #[serde(default)]
    pub min_docli: Option<String>,
    /// Theme keys in AUTHOR.md object form.
    pub theme_schema: BTreeMap<String, ThemeKeySpec>,
}

/// A loaded pack ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    /// Parsed manifest.
    pub manifest: TemplateManifest,
    /// `page.html.j2` source.
    pub page_template: String,
    /// `style.css.j2` source.
    pub style_template: String,
    /// `script.js` source, inserted verbatim.
    pub script: String,
}

/// Theme values passed to `style.css.j2`.
///
/// Keys match `theme_schema`. Missing keys are filled from schema defaults
/// when the pack is rendered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ThemeMap(BTreeMap<String, String>);

impl ThemeMap {
    /// An empty override map. Render fills every key from the pack defaults.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Defaults declared by `pack`'s `theme_schema`.
    pub fn defaults(pack: &Pack) -> Self {
        Self(
            pack.manifest
                .theme_schema
                .iter()
                .map(|(key, spec)| (key.clone(), spec.default.clone()))
                .collect(),
        )
    }

    /// Set one theme key.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Borrow the value for `key` when the caller set it.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }
}

impl Default for ThemeMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Failure loading a pack.
///
/// Filesystem reads are `DOCLI.IO`. A damaged embedded pack is
/// `DOCLI.INTERNAL`. Unknown bundled ids and invalid manifests stay
/// unclassified here so later sprints can map them to `DOCLI.TEMPLATE_*`.
#[derive(Debug)]
pub struct PackResolveError {
    kind: PackResolveKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
enum PackResolveKind {
    NotBundled {
        id: String,
    },
    Embedded {
        cause: String,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Invalid {
        path: PathBuf,
        cause: String,
    },
}

impl PackResolveError {
    fn not_bundled(id: impl Into<String>) -> Self {
        Self {
            kind: PackResolveKind::NotBundled { id: id.into() },
            backtrace: Backtrace::capture(),
        }
    }

    fn embedded(cause: impl Into<String>) -> Self {
        Self {
            kind: PackResolveKind::Embedded {
                cause: cause.into(),
            },
            backtrace: Backtrace::capture(),
        }
    }

    fn io(path: PathBuf, source: std::io::Error) -> Self {
        Self {
            kind: PackResolveKind::Io { path, source },
            backtrace: Backtrace::capture(),
        }
    }

    fn invalid(path: PathBuf, cause: impl Into<String>) -> Self {
        Self {
            kind: PackResolveKind::Invalid {
                path,
                cause: cause.into(),
            },
            backtrace: Backtrace::capture(),
        }
    }

    /// Whether this failure is a filesystem read.
    pub fn is_io(&self) -> bool {
        matches!(self.kind, PackResolveKind::Io { .. })
    }

    /// Whether `id` is not one of the packs compiled into the binary.
    pub fn is_not_bundled(&self) -> bool {
        matches!(self.kind, PackResolveKind::NotBundled { .. })
    }

    /// `DOCLI.IO` for a filesystem read, `DOCLI.INTERNAL` for a damaged
    /// embedded pack.
    ///
    /// Returns `None` for an unknown bundled id or an invalid on-disk
    /// manifest. Those become `DOCLI.TEMPLATE_*` in later sprints.
    pub fn machine_code(&self) -> Option<&'static str> {
        match self.kind {
            PackResolveKind::Io { .. } => Some("DOCLI.IO"),
            PackResolveKind::Embedded { .. } => Some("DOCLI.INTERNAL"),
            PackResolveKind::NotBundled { .. } | PackResolveKind::Invalid { .. } => None,
        }
    }

    /// Captured backtrace from the point of failure.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// One-line cause suitable for an envelope `details.cause`.
    pub fn cause(&self) -> String {
        match &self.kind {
            PackResolveKind::NotBundled { id } => format!("bundled template pack not found: {id}"),
            PackResolveKind::Embedded { cause } => cause.clone(),
            PackResolveKind::Invalid { path, cause } => {
                format!("{}: {cause}", path.display())
            }
            PackResolveKind::Io { path, source } => {
                format!("failed to read {}: {source}", path.display())
            }
        }
    }
}

impl Display for PackResolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cause())
    }
}

impl std::error::Error for PackResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PackResolveKind::Io { source, .. } => Some(source),
            PackResolveKind::NotBundled { .. }
            | PackResolveKind::Embedded { .. }
            | PackResolveKind::Invalid { .. } => None,
        }
    }
}

/// Failure rendering a pack that already resolved.
#[derive(Debug)]
pub struct RenderError {
    cause: String,
    backtrace: Backtrace,
}

impl RenderError {
    fn new(cause: impl Into<String>) -> Self {
        Self {
            cause: cause.into(),
            backtrace: Backtrace::capture(),
        }
    }

    /// Render failures of a resolved pack are `DOCLI.INTERNAL` on the
    /// embedded-default path.
    pub fn machine_code(&self) -> &'static str {
        "DOCLI.INTERNAL"
    }

    /// Template engine message.
    pub fn cause(&self) -> &str {
        &self.cause
    }

    /// Captured backtrace from the point of failure.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cause)
    }
}

impl std::error::Error for RenderError {}

/// Embedded-default render failure.
///
/// The bundled pack does not touch the filesystem, so this is always
/// `DOCLI.INTERNAL`.
#[derive(Debug)]
pub struct EmbeddedDefaultError {
    cause: String,
    backtrace: Backtrace,
}

impl EmbeddedDefaultError {
    pub(crate) fn internal(cause: impl Into<String>) -> Self {
        Self {
            cause: cause.into(),
            backtrace: Backtrace::capture(),
        }
    }

    /// Always `DOCLI.INTERNAL`.
    pub fn code(&self) -> &'static str {
        "DOCLI.INTERNAL"
    }

    /// Why the embedded pack could not render.
    pub fn cause(&self) -> &str {
        &self.cause
    }

    /// Captured backtrace from the point of failure.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl Display for EmbeddedDefaultError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.cause)
    }
}

impl std::error::Error for EmbeddedDefaultError {}

/// Load a bundled pack or a filesystem pack directory.
///
/// # Errors
///
/// Returns [`PackResolveError`] when the id is not bundled, a required file
/// cannot be read, or `template.toml` is not the expected manifest.
pub fn resolve_pack(template: &TemplateRef) -> Result<Pack, PackResolveError> {
    match template {
        TemplateRef::Bundled(id) => {
            if id != DEFAULT_PACK_ID {
                return Err(PackResolveError::not_bundled(id));
            }
            load_embedded()
        }
        TemplateRef::Dir(path) => load_dir(path),
    }
}

/// Render `pack` for `model` with `theme` overrides on top of schema defaults.
///
/// The context includes `model`, `model_json`, `search_json`, `theme`,
/// `title`, `generator`, `version`, the rendered stylesheet, the pack script,
/// and the static command tree.
///
/// # Errors
///
/// Returns [`RenderError`] when a template fails to parse or a required theme
/// key is missing after defaults are applied.
pub fn render_pack(pack: &Pack, model: &CliModel, theme: &ThemeMap) -> Result<String, RenderError> {
    let theme = merge_theme(pack, theme);
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .map_err(|err| RenderError::new(err.to_string()))?,
    );
    env.add_filter("docli_escape", page::docli_escape);
    env.add_template_owned("style.css.j2", pack.style_template.clone())
        .map_err(|err| RenderError::new(err.to_string()))?;
    env.add_template_owned("page.html.j2", pack.page_template.clone())
        .map_err(|err| RenderError::new(err.to_string()))?;

    let style = env
        .get_template("style.css.j2")
        .map_err(|err| RenderError::new(err.to_string()))?
        .render(context! { theme => Serde(&theme) })
        .map_err(|err| RenderError::new(err.to_string()))?;

    let version = model.version.clone().unwrap_or_default();
    let generator = format!("docli {}", env!("CARGO_PKG_VERSION"));
    // page.html.j2 is HTML-escaped. Prebuilt markup, CSS, JS, and JSON are
    // already in their final form.
    env.get_template("page.html.j2")
        .map_err(|err| RenderError::new(err.to_string()))?
        .render(context! {
            model => Serde(model),
            model_json => Value::from_safe_string(page::embed_json(model)),
            search_json => Value::from_safe_string(page::embed_json(&search_index(model))),
            theme => Serde(&theme),
            title => model.name.as_str(),
            generator => generator,
            version => version,
            tree => Value::from_safe_string(page::render_tree(model)),
            style => Value::from_safe_string(style),
            script => Value::from_safe_string(pack.script.clone()),
        })
        .map_err(|err| RenderError::new(err.to_string()))
}

/// Render the embedded `default` pack with its default theme.
///
/// # Errors
///
/// Returns [`EmbeddedDefaultError`] (`DOCLI.INTERNAL`) when the compiled-in
/// pack cannot be resolved or rendered.
pub(crate) fn render_embedded_default(model: &CliModel) -> Result<String, EmbeddedDefaultError> {
    let pack = resolve_pack(&TemplateRef::bundled(DEFAULT_PACK_ID))
        .map_err(|err| EmbeddedDefaultError::internal(err.to_string()))?;
    let theme = ThemeMap::defaults(&pack);
    render_pack(&pack, model, &theme).map_err(|err| EmbeddedDefaultError::internal(err.to_string()))
}

fn merge_theme(pack: &Pack, theme: &ThemeMap) -> ThemeMap {
    let mut merged = ThemeMap::defaults(pack);
    for (key, value) in &theme.0 {
        merged.0.insert(key.clone(), value.clone());
    }
    merged
}

fn load_embedded() -> Result<Pack, PackResolveError> {
    let manifest_text = embedded_file(MANIFEST_FILE)?;
    let manifest = parse_manifest(&manifest_text, Path::new(MANIFEST_FILE), true)?;
    if manifest.id != DEFAULT_PACK_ID {
        return Err(PackResolveError::embedded(format!(
            "embedded pack id is {}, expected {DEFAULT_PACK_ID}",
            manifest.id
        )));
    }
    Ok(Pack {
        manifest,
        page_template: embedded_file(PAGE_FILE)?,
        style_template: embedded_file(STYLE_FILE)?,
        script: embedded_file(SCRIPT_FILE)?,
    })
}

fn load_dir(dir: &Path) -> Result<Pack, PackResolveError> {
    let manifest_path = dir.join(MANIFEST_FILE);
    let manifest_text = read_required(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text, &manifest_path, false)?;
    Ok(Pack {
        manifest,
        page_template: read_required(&dir.join(PAGE_FILE))?,
        style_template: read_required(&dir.join(STYLE_FILE))?,
        script: read_required(&dir.join(SCRIPT_FILE))?,
    })
}

fn embedded_file(name: &str) -> Result<String, PackResolveError> {
    let file = DEFAULT_PACK.get_file(name).ok_or_else(|| {
        PackResolveError::embedded(format!("embedded default pack is missing {name}"))
    })?;
    file.contents_utf8()
        .map(str::to_owned)
        .ok_or_else(|| PackResolveError::embedded(format!("{name} is not valid UTF-8")))
}

fn read_required(path: &Path) -> Result<String, PackResolveError> {
    std::fs::read_to_string(path).map_err(|source| PackResolveError::io(path.to_path_buf(), source))
}

fn parse_manifest(
    text: &str,
    path: &Path,
    embedded: bool,
) -> Result<TemplateManifest, PackResolveError> {
    toml::from_str(text).map_err(|err| {
        let cause = format!("template.toml: {err}");
        if embedded {
            PackResolveError::embedded(cause)
        } else {
            PackResolveError::invalid(path.to_path_buf(), cause)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_model() -> CliModel {
        serde_json::from_str(r#"{"name":"a&b<c>d\"e'f/g","version":"1.0"}"#).expect("model")
    }

    #[test]
    fn filesystem_pack_matches_embedded_default() {
        let bundled = resolve_pack(&TemplateRef::bundled("default")).expect("embedded");
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/html/default");
        let from_disk = resolve_pack(&TemplateRef::dir(&dir)).expect("disk");
        let model = demo_model();
        let theme = ThemeMap::defaults(&bundled);
        let bundled_html = render_pack(&bundled, &model, &theme).expect("render embedded");
        let disk_html = render_pack(&from_disk, &model, &theme).expect("render disk");
        assert_eq!(bundled_html, disk_html);
        assert!(bundled_html.contains("id=\"docli-default-pack\""));
        assert!(bundled_html.contains("a&amp;b&lt;c&gt;d&quot;e&#39;f/g"));
        assert!(!bundled_html.contains("&#x27;"));
        assert!(!bundled_html.contains("&#x2f;"));
    }

    #[test]
    fn theme_override_replaces_default_accent() {
        let pack = resolve_pack(&TemplateRef::bundled("default")).expect("embedded");
        let mut theme = ThemeMap::new();
        theme.insert("accent", "#ff00aa");
        let model = demo_model();
        let html = render_pack(&pack, &model, &theme).expect("render");
        assert!(html.contains("--accent:#ff00aa"));
        assert!(!html.contains("--accent:#007acc"));
        assert!(html.contains("--accent:#4da3ff"));
    }

    #[test]
    fn missing_directory_is_io_not_template_code() {
        let missing = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/html/missing-pack");
        let err = resolve_pack(&TemplateRef::dir(missing)).expect_err("missing");
        assert!(err.is_io());
        assert_eq!(err.machine_code(), Some("DOCLI.IO"));
        assert!(!err.to_string().contains("TEMPLATE"));
    }

    #[test]
    fn invalid_manifest_has_no_template_code_yet() {
        let dir = std::env::temp_dir().join(format!(
            "docli-b7-invalid-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("template.toml"), "id = [\n").expect("write");
        let err = resolve_pack(&TemplateRef::dir(&dir)).expect_err("invalid");
        assert_eq!(err.machine_code(), None);
        assert!(!err.to_string().contains("DOCLI.TEMPLATE"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_bundled_id_is_not_a_template_code() {
        let err = resolve_pack(&TemplateRef::bundled("cli-doc")).expect_err("missing id");
        assert!(err.is_not_bundled());
        assert_eq!(err.machine_code(), None);
        assert!(!err.to_string().contains("DOCLI.TEMPLATE"));
    }

    #[test]
    fn embedded_default_error_is_internal() {
        let err = EmbeddedDefaultError::internal("broken pack");
        assert_eq!(err.code(), "DOCLI.INTERNAL");
        assert!(!err.code().contains("TEMPLATE"));
        assert!(err.cause().contains("broken pack"));
    }
}
