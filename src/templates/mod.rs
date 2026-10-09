//! HTML template packs.
//!
//! The bundled `default` pack is compiled into the binary with [`include_dir`].
//! [`resolve_pack`] also loads a pack directory from disk. [`render_pack`]
//! fills `page.html.j2` and `style.css.j2` with MiniJinja.
//!
//! Embedded-default failures are `DOCLI.INTERNAL` (the pack is not read from
//! disk). A filesystem read failure is `DOCLI.IO`. An unknown bundled id is
//! `DOCLI.TEMPLATE_NOT_FOUND`. An on-disk manifest or template that does not
//! compile is `DOCLI.TEMPLATE_INVALID`. The embedded-default path stores
//! [`PackResolveError::cause`] and does not copy [`Display`] into that cause.
//!
//! # Examples
//!
//! ```
//! use docli::schema::CliModel;
//! use docli::templates::{render_pack, resolve_pack, TemplateRef, ThemeMap};
//!
//! let model: CliModel = serde_json::from_str(r#"{"name":"demo"}"#)?;
//! let pack = resolve_pack(&TemplateRef::bundled_default())?;
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

/// A bundled pack id compiled into this binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundledPackId {
    /// The embedded `default` pack.
    Default,
}

impl BundledPackId {
    /// Stable id string for this pack.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => DEFAULT_PACK_ID,
        }
    }

    /// Accept only ids that are compiled into this binary.
    ///
    /// # Errors
    ///
    /// Returns [`PackResolveError`] when `id` is not embedded.
    pub fn try_from_str(id: &str) -> Result<Self, PackResolveError> {
        if id == DEFAULT_PACK_ID {
            Ok(Self::Default)
        } else {
            Err(PackResolveError::not_bundled(id))
        }
    }
}

/// Bundled pack id or a filesystem pack directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRef {
    /// Pack compiled into this binary. Phase B embeds `default` only.
    Bundled(BundledPackId),
    /// Directory that contains `template.toml`, `page.html.j2`, `style.css.j2`,
    /// and `script.js`.
    Dir(PathBuf),
}

impl TemplateRef {
    /// The embedded `default` pack.
    pub fn bundled_default() -> Self {
        Self::Bundled(BundledPackId::Default)
    }

    /// A bundled pack id such as `default`.
    ///
    /// # Errors
    ///
    /// Returns [`PackResolveError`] when `id` is not embedded.
    pub fn try_bundled(id: &str) -> Result<Self, PackResolveError> {
        Ok(Self::Bundled(BundledPackId::try_from_str(id)?))
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
///
/// `page.html.j2` and `style.css.j2` are compiled into a MiniJinja environment
/// once, when the pack is loaded. [`render_pack`] reuses that environment.
#[derive(Clone)]
pub struct Pack {
    /// Parsed manifest.
    pub manifest: TemplateManifest,
    /// `page.html.j2` source.
    pub page_template: String,
    /// `style.css.j2` source.
    pub style_template: String,
    /// `script.js` source, inserted verbatim.
    pub script: String,
    env: Environment<'static>,
}

impl std::fmt::Debug for Pack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pack")
            .field("manifest", &self.manifest)
            .field("page_template", &self.page_template)
            .field("style_template", &self.style_template)
            .field("script", &self.script)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Pack {
    fn eq(&self, other: &Self) -> bool {
        self.manifest == other.manifest
            && self.page_template == other.page_template
            && self.style_template == other.style_template
            && self.script == other.script
    }
}

impl Eq for Pack {}

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
/// `DOCLI.INTERNAL`. Unknown bundled ids and invalid manifests map to
/// `DOCLI.TEMPLATE_NOT_FOUND` and `DOCLI.TEMPLATE_INVALID`.
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

    /// Stable `DOCLI.*` code for envelope mapping.
    pub fn machine_code(&self) -> &'static str {
        match self.kind {
            PackResolveKind::Io { .. } => "DOCLI.IO",
            PackResolveKind::Embedded { .. } => "DOCLI.INTERNAL",
            PackResolveKind::NotBundled { .. } => "DOCLI.TEMPLATE_NOT_FOUND",
            PackResolveKind::Invalid { .. } => "DOCLI.TEMPLATE_INVALID",
        }
    }

    /// One recovery sentence for operators and automation.
    pub fn suggested_action(&self) -> String {
        match &self.kind {
            PackResolveKind::NotBundled { id } => format!(
                "Run `docli templates list --json` and pass a bundled id such as default instead of {id}"
            ),
            PackResolveKind::Invalid { path, .. } => format!(
                "Run `docli templates validate {} --json` and fix template.toml and the pack templates",
                path.display()
            ),
            PackResolveKind::Io { path, .. } => {
                format!("Check that {} exists and is readable", path.display())
            }
            PackResolveKind::Embedded { .. } => {
                "Retry or report a bug — the embedded default pack failed to load".to_owned()
            }
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
        // Recovery stays on [`Self::suggested_action`]. [`Self::cause`] is the
        // envelope `details.cause` and must stay free of that sentence.
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

    /// Keeps [`PackResolveError::cause`], not the [`Display`] text.
    fn from_resolve(err: PackResolveError) -> Self {
        Self::internal(err.cause())
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
/// Returns [`PackResolveError`] when a required file cannot be read,
/// `template.toml` is not the expected manifest, or a template fails to compile.
///
/// Unknown bundled ids fail in [`TemplateRef::try_bundled`] with
/// `DOCLI.TEMPLATE_NOT_FOUND`.
pub fn resolve_pack(template: &TemplateRef) -> Result<Pack, PackResolveError> {
    match template {
        TemplateRef::Bundled(BundledPackId::Default) => load_embedded(),
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
/// Returns [`RenderError`] when rendering fails, for example a theme key that
/// is still missing after defaults are applied. Parse failures are reported by
/// [`resolve_pack`].
pub fn render_pack(pack: &Pack, model: &CliModel, theme: &ThemeMap) -> Result<String, RenderError> {
    let theme = merge_theme(pack, theme);
    let env = &pack.env;

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
    let pack = resolve_pack(&TemplateRef::bundled_default())
        .map_err(EmbeddedDefaultError::from_resolve)?;
    let theme = ThemeMap::defaults(&pack);
    render_pack(&pack, model, &theme).map_err(|err| EmbeddedDefaultError::internal(err.cause()))
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
    pack_from_sources(
        manifest,
        embedded_file(PAGE_FILE)?,
        embedded_file(STYLE_FILE)?,
        embedded_file(SCRIPT_FILE)?,
        CompileSite::Embedded,
    )
}

fn load_dir(dir: &Path) -> Result<Pack, PackResolveError> {
    let manifest_path = dir.join(MANIFEST_FILE);
    let manifest_text = read_required(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text, &manifest_path, false)?;
    pack_from_sources(
        manifest,
        read_required(&dir.join(PAGE_FILE))?,
        read_required(&dir.join(STYLE_FILE))?,
        read_required(&dir.join(SCRIPT_FILE))?,
        CompileSite::Dir(dir),
    )
}

/// Where a pack's templates were loaded, so compile failures keep the right code.
enum CompileSite<'a> {
    Embedded,
    Dir(&'a Path),
}

fn pack_from_sources(
    manifest: TemplateManifest,
    page_template: String,
    style_template: String,
    script: String,
    site: CompileSite<'_>,
) -> Result<Pack, PackResolveError> {
    let env = compile_environment(&page_template, &style_template).map_err(|err| {
        let cause = err.cause().to_owned();
        match site {
            CompileSite::Embedded => PackResolveError::embedded(cause),
            CompileSite::Dir(dir) => PackResolveError::invalid(dir.to_path_buf(), cause),
        }
    })?;
    Ok(Pack {
        manifest,
        page_template,
        style_template,
        script,
        env,
    })
}

fn compile_environment(
    page_template: &str,
    style_template: &str,
) -> Result<Environment<'static>, RenderError> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .map_err(|err| RenderError::new(err.to_string()))?,
    );
    env.add_filter("docli_escape", page::docli_escape);
    // Owned copies live in the environment for its `'static` lifetime. This
    // happens once per load, not on each [`render_pack`] call.
    env.add_template_owned("style.css.j2", style_template.to_owned())
        .map_err(|err| RenderError::new(err.to_string()))?;
    env.add_template_owned("page.html.j2", page_template.to_owned())
        .map_err(|err| RenderError::new(err.to_string()))?;
    Ok(env)
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
        let bundled = resolve_pack(&TemplateRef::bundled_default()).expect("embedded");
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
        let pack = resolve_pack(&TemplateRef::bundled_default()).expect("embedded");
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
        assert_eq!(err.machine_code(), "DOCLI.IO");
        assert!(!err.to_string().contains("TEMPLATE"));
    }

    #[test]
    fn invalid_manifest_uses_template_invalid_code() {
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
        assert_eq!(err.machine_code(), "DOCLI.TEMPLATE_INVALID");
        assert!(err.suggested_action().contains("templates validate"));
        assert!(err.cause().contains("template.toml"));
        assert!(!err.cause().contains("templates validate"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn uncompilable_directory_template_is_template_invalid() {
        let dir = std::env::temp_dir().join(format!(
            "docli-b7-compile-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(
            dir.join("template.toml"),
            "id = \"broken\"\nname = \"broken\"\nversion = \"1\"\ndescription = \"broken\"\ntheme_schema = {}\n",
        )
        .expect("manifest");
        std::fs::write(dir.join("page.html.j2"), "{{ unclosed").expect("page");
        std::fs::write(dir.join("style.css.j2"), "body{}\n").expect("style");
        std::fs::write(dir.join("script.js"), "").expect("script");
        let err = resolve_pack(&TemplateRef::dir(&dir)).expect_err("compile");
        assert_eq!(err.machine_code(), "DOCLI.TEMPLATE_INVALID");
        assert!(err.suggested_action().contains("templates validate"));
        assert_ne!(err.machine_code(), "DOCLI.INTERNAL");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_bundled_id_uses_template_not_found_code() {
        let err = TemplateRef::try_bundled("cli-doc").expect_err("missing id");
        assert!(err.is_not_bundled());
        assert_eq!(err.machine_code(), "DOCLI.TEMPLATE_NOT_FOUND");
        assert!(err.suggested_action().contains("templates list"));
        assert!(err.suggested_action().contains("cli-doc"));
        let cause = err.cause();
        let wrapped = EmbeddedDefaultError::from_resolve(err);
        assert_eq!(wrapped.cause(), cause);
        assert!(!wrapped.cause().contains("templates list"));
        assert!(matches!(
            TemplateRef::try_bundled("default"),
            Ok(TemplateRef::Bundled(BundledPackId::Default))
        ));
    }

    #[test]
    fn embedded_default_error_is_internal() {
        let err = EmbeddedDefaultError::internal("broken pack");
        assert_eq!(err.code(), "DOCLI.INTERNAL");
        assert!(!err.code().contains("TEMPLATE"));
        assert!(err.cause().contains("broken pack"));
    }
}
