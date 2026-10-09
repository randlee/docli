//! In-process `generate`, `show`, and `templates` operations shared by the CLI.
//!
//! These functions return the same [`Envelope`] the CLI serializes. They write
//! or read files, hash artifacts, and never print.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::contract::{Envelope, ErrorBody};
use crate::render;
use crate::schema::CliModel;
use crate::templates::{
    install_root, render_pack, resolve_pack, BundledPackId, EmbeddedDefaultError, PackId,
    PackResolveError, TemplateManifest, TemplateRef, ThemeKeySpec, ThemeMap,
    EMBEDDED_PACK_RECOVERY,
};

/// Example `generate` argv included in `templates show`.
pub const EXAMPLE_GENERATE_ARGV: &[&str] = &[
    "docli",
    "generate",
    "--input",
    "model.json",
    "--preview",
    "--template",
    "default",
    "--theme",
    r##"{"accent":"#007acc"}"##,
];

/// Where `generate` reads the model JSON.
#[derive(Debug, Clone)]
pub enum InputSource {
    /// Read the model from stdin.
    Stdin,
    /// Read the model from this path.
    File(PathBuf),
}

/// One written or inspected artifact.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactReport {
    /// `"html"` or `"markdown"`.
    pub kind: &'static str,
    /// Path that was written or inspected.
    pub path: PathBuf,
    /// Byte length of the file contents.
    pub bytes: u64,
    /// Lowercase hex SHA-256 of the file contents.
    pub sha256: String,
}

/// Request for [`generate`].
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    /// Model JSON source.
    pub input: InputSource,
    /// HTML output directory. `None` resolves to `site/cli` unless [`Self::preview`].
    pub html_dir: Option<PathBuf>,
    /// Write HTML under a temporary `docli-preview-{pid}-{nanos}-{seq}` directory.
    ///
    /// Mutually exclusive with [`Self::html_dir`].
    pub preview: bool,
    /// Bundled pack or pack directory. `None` uses the embedded `default` pack.
    pub template: Option<TemplateRef>,
    /// Theme overrides. `None` uses pack defaults.
    pub theme: Option<ThemeMap>,
    /// Optional Markdown output path. Omitted means no Markdown file.
    pub markdown: Option<PathBuf>,
}

/// Successful `generate` response.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateResponse {
    /// Always `"generate"`.
    pub operation: &'static str,
    /// `"stdin"` or the input path.
    pub input: String,
    /// `CliModel.name` of the rendered model.
    pub model_name: String,
    /// Resolved HTML directory. In preview mode this is the temp directory.
    pub html_dir: PathBuf,
    /// Preview directory when [`GenerateRequest::preview`] is set; JSON `null` otherwise.
    pub preview_dir: Option<PathBuf>,
    /// Artifacts that were written.
    pub outputs: Vec<ArtifactReport>,
}

/// Request for [`show`].
#[derive(Debug, Clone)]
pub struct ShowRequest {
    /// HTML directory to inspect. Looks for `index.html` inside.
    pub html_dir: Option<PathBuf>,
    /// Markdown file to inspect.
    pub markdown: Option<PathBuf>,
}

/// Successful `show` response.
#[derive(Debug, Clone, Serialize)]
pub struct ShowResponse {
    /// Always `"show"`.
    pub operation: &'static str,
    /// Artifacts that were inspected.
    pub artifacts: Vec<ArtifactReport>,
}

/// One pack reported by [`templates_list`].
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSummary {
    /// Pack id from `template.toml`.
    pub id: PackId,
    /// Display name.
    pub name: String,
    /// Pack version string.
    pub version: String,
    /// `embedded:<id>` for a compiled-in pack, or the directory of an extra pack.
    pub path: String,
}

/// Successful `templates list` response.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesListResponse {
    /// Always `"templates_list"`.
    pub operation: &'static str,
    /// Directory of optional extra packs (`<prefix>/share/docli/templates`).
    pub install_root: PathBuf,
    /// Embedded packs, then extra packs sorted by id.
    pub templates: Vec<TemplateSummary>,
}

/// Successful `templates show` response.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesShowResponse {
    /// Always `"templates_show"`.
    pub operation: &'static str,
    /// Pack id.
    pub id: PackId,
    /// Parsed `template.toml`.
    pub manifest: TemplateManifest,
    /// Theme keys from the manifest, in AUTHOR.md object form.
    pub theme_schema: BTreeMap<String, ThemeKeySpec>,
    /// Example `docli generate` argv for this pack.
    pub example_generate_argv: &'static [&'static str],
}

/// Successful `templates validate` response.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesValidateResponse {
    /// Always `"templates_validate"`.
    pub operation: &'static str,
    /// Pack id from the manifest.
    pub id: PackId,
    /// Directory that was validated.
    pub path: PathBuf,
    /// Always `true` on success. Failures use the error envelope.
    pub valid: bool,
}

/// Render the model and write HTML (and optional Markdown).
///
/// `--preview` and `html_dir` together are `DOCLI.USAGE`. Preview writes
/// `docli-preview-{pid}-{nanos}-{seq}/index.html` under the system temp
/// directory and sets both `html_dir` and `preview_dir` to that directory.
/// `{seq}` is a per-process counter so parallel calls do not share a path.
/// Otherwise `html_dir: None` resolves to `site/cli` and `preview_dir` is null.
///
/// Omitting `template` and `theme` renders the embedded `default` pack with its
/// default theme, the same bytes as [`crate::render::html::render`]. A write failure after
/// another file landed returns `DOCLI.IO` with `details.outputs_written`.
///
/// # Errors
///
/// Returns a failure envelope for usage, missing input, invalid JSON, I/O
/// errors, an unknown template, a filesystem pack that fails to render
/// (`DOCLI.TEMPLATE_INVALID`), or an embedded pack failure (`DOCLI.INTERNAL`).
pub fn generate(req: GenerateRequest) -> Envelope<GenerateResponse> {
    if req.preview && req.html_dir.is_some() {
        return Envelope::failure(ErrorBody::usage(
            "Pass either --preview or --html DIR, not both",
        ));
    }
    let (html_dir, preview_dir) = if req.preview {
        let dir = preview_output_dir();
        (dir.clone(), Some(dir))
    } else {
        (
            req.html_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from("site/cli")),
            None,
        )
    };

    let (input_label, json) = match read_input(&req.input) {
        Ok(value) => value,
        Err(error) => return Envelope::failure(error),
    };
    if json.trim().is_empty() {
        return Envelope::failure(ErrorBody::input_invalid(&input_label, None));
    }
    let model: CliModel = match serde_json::from_str(&json) {
        Ok(model) => model,
        Err(err) => {
            return Envelope::failure(ErrorBody::input_invalid(
                &input_label,
                Some(&err.to_string()),
            ));
        }
    };

    let html = match render_generate_html(&model, req.template.as_ref(), req.theme.as_ref()) {
        Ok(html) => html,
        Err(error) => return Envelope::failure(error),
    };
    let html_path = html_dir.join("index.html");
    let mut outputs = Vec::new();

    if let Err(error) = write_file(&html_path, html.as_bytes()) {
        return Envelope::failure(ErrorBody::io(error, &html_path, None));
    }
    outputs.push(artifact("html", html_path, html.as_bytes()));

    if let Some(markdown_path) = req.markdown {
        let markdown = render::markdown::render(&model);
        if let Err(error) = write_file(&markdown_path, markdown.as_bytes()) {
            let written = serde_json::to_value(&outputs).expect("ArtifactReport vector serializes");
            return Envelope::failure(ErrorBody::io(error, &markdown_path, Some(written)));
        }
        outputs.push(artifact("markdown", markdown_path, markdown.as_bytes()));
    }

    Envelope::success(GenerateResponse {
        operation: "generate",
        input: input_label,
        model_name: model.name,
        html_dir,
        preview_dir,
        outputs,
    })
}

/// Hash requested artifacts that already exist on disk.
///
/// Both paths `None` is `DOCLI.USAGE`. A missing artifact is
/// `DOCLI.OUTPUT_NOT_FOUND` and lists every requested artifact.
///
/// # Errors
///
/// Returns a failure envelope for usage, missing artifacts, or read errors.
pub fn show(req: ShowRequest) -> Envelope<ShowResponse> {
    if req.html_dir.is_none() && req.markdown.is_none() {
        return Envelope::failure(ErrorBody::usage("Pass --html DIR and/or --markdown FILE"));
    }

    let mut requested = Vec::new();
    if let Some(html_dir) = &req.html_dir {
        requested.push(("html", html_dir.join("index.html")));
    }
    if let Some(markdown) = req.markdown {
        requested.push(("markdown", markdown));
    }

    let mut artifact_details = Vec::new();
    let mut missing = Vec::new();
    for (_, path) in &requested {
        let exists = path.exists();
        artifact_details.push(serde_json::json!({
            "path": path.display().to_string(),
            "exists": exists,
        }));
        if !exists {
            missing.push(path.display().to_string());
        }
    }
    if !missing.is_empty() {
        return Envelope::failure(ErrorBody::output_not_found(artifact_details, &missing));
    }

    let mut artifacts = Vec::new();
    for (kind, path) in requested {
        match std::fs::read(&path) {
            Ok(bytes) => artifacts.push(artifact(kind, path, &bytes)),
            Err(err) => return Envelope::failure(ErrorBody::io(err.to_string(), &path, None)),
        }
    }

    Envelope::success(ShowResponse {
        operation: "show",
        artifacts,
    })
}

/// List embedded packs and any valid extra packs under [`install_root`].
///
/// Embedded order is [`BundledPackId::all`]: `default`, then `cli-doc`.
/// A missing install root is an empty extra list, not an error. Unreadable
/// install roots are `DOCLI.IO`. Extra directories that do not load are omitted
/// so one broken pack does not hide the others; [`templates_validate`] reports
/// that failure. An extra whose id is already bundled, or whose id
/// [`PackId::excluded_from_list`] reports (the `_skeleton` starter), is omitted.
///
/// # Errors
///
/// Returns a failure envelope when an embedded pack cannot load
/// (`DOCLI.INTERNAL`) or the install root exists but cannot be read (`DOCLI.IO`).
pub fn templates_list() -> Envelope<TemplatesListResponse> {
    let install_root = install_root();
    let mut templates = Vec::new();
    for id in BundledPackId::all() {
        let pack = match resolve_pack(&TemplateRef::bundled(id)) {
            Ok(pack) => pack,
            Err(err) => return Envelope::failure(pack_failure(err)),
        };
        templates.push(TemplateSummary {
            path: format!("embedded:{}", pack.manifest.id),
            id: pack.manifest.id,
            name: pack.manifest.name,
            version: pack.manifest.version,
        });
    }
    match installed_summaries(&install_root) {
        Ok(extras) => templates.extend(extras),
        Err(error) => return Envelope::failure(error),
    }
    Envelope::success(TemplatesListResponse {
        operation: "templates_list",
        install_root,
        templates,
    })
}

/// Show a bundled id, an installed extra id, or a pack directory.
///
/// A selector that contains a path separator, is absolute, or names an existing
/// path is loaded as a directory. Any other selector is a pack id: embedded
/// first, then `<install_root>/<id>`.
///
/// # Errors
///
/// Returns `DOCLI.TEMPLATE_INVALID` when the pack content does not compile,
/// `DOCLI.IO` when a directory cannot be read, `DOCLI.TEMPLATE_NOT_FOUND` for
/// an unknown pack id, and `DOCLI.INTERNAL` for a damaged embedded pack.
pub fn templates_show(template: TemplateRef) -> Envelope<TemplatesShowResponse> {
    let pack = match resolve_pack(&template) {
        Ok(pack) => pack,
        Err(err) => return Envelope::failure(pack_failure(err)),
    };
    let theme_schema = pack.manifest.theme_schema.clone();
    Envelope::success(TemplatesShowResponse {
        operation: "templates_show",
        id: pack.manifest.id.clone(),
        manifest: pack.manifest,
        theme_schema,
        example_generate_argv: EXAMPLE_GENERATE_ARGV,
    })
}

/// Load `path` as a pack directory and compile its templates.
///
/// # Errors
///
/// Returns `DOCLI.TEMPLATE_INVALID` when `template.toml` or a template does
/// not compile, and `DOCLI.IO` when a required file cannot be read.
pub fn templates_validate(path: &Path) -> Envelope<TemplatesValidateResponse> {
    match resolve_pack(&TemplateRef::dir(path)) {
        Ok(pack) => Envelope::success(TemplatesValidateResponse {
            operation: "templates_validate",
            id: pack.manifest.id,
            path: path.to_path_buf(),
            valid: true,
        }),
        Err(err) => Envelope::failure(pack_failure(err)),
    }
}

fn installed_summaries(root: &Path) -> Result<Vec<TemplateSummary>, ErrorBody> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let entries =
        std::fs::read_dir(root).map_err(|err| ErrorBody::io(err.to_string(), root, None))?;
    let mut extras = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|err| ErrorBody::io(err.to_string(), root, None))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(pack) = resolve_pack(&TemplateRef::dir(&path)) else {
            continue;
        };
        if !listed_extra(&pack.manifest.id) {
            continue;
        }
        extras
            .entry(pack.manifest.id.clone())
            .or_insert(TemplateSummary {
                id: pack.manifest.id,
                name: pack.manifest.name,
                version: pack.manifest.version,
                path: path.display().to_string(),
            });
    }
    Ok(extras.into_values().collect())
}

/// Resolve a `--template` selector to a bundled id, an installed extra id, or a directory.
///
/// A selector that contains a path separator, is absolute, or names an existing
/// path is a directory. Any other selector is a pack id: embedded first, then
/// `<install_root>/<id>`. An id that matches neither is `DOCLI.TEMPLATE_NOT_FOUND`.
///
/// # Errors
///
/// Returns [`ErrorBody`] with `DOCLI.TEMPLATE_NOT_FOUND` when `selector` is an
/// unknown pack id.
pub(crate) fn parse_template_selector(selector: &str) -> Result<TemplateRef, ErrorBody> {
    if selector_is_path(selector) {
        return Ok(TemplateRef::dir(selector));
    }
    let id = match PackId::new(selector) {
        Ok(id) => id,
        Err(_) => return Err(pack_failure(PackResolveError::unknown_id(selector))),
    };
    match TemplateRef::from_pack_id(&id) {
        Ok(bundled) => Ok(bundled),
        Err(not_bundled) => {
            let installed = install_root().join(id.as_str());
            if installed.is_dir() {
                Ok(TemplateRef::dir(installed))
            } else {
                Err(pack_failure(not_bundled))
            }
        }
    }
}

fn selector_is_path(selector: &str) -> bool {
    let path = Path::new(selector);
    path.is_absolute() || selector.contains('/') || selector.contains('\\') || path.exists()
}

fn pack_failure(err: PackResolveError) -> ErrorBody {
    let code = err.machine_code();
    match code {
        "DOCLI.TEMPLATE_INVALID" => {
            ErrorBody::template_invalid(err.cause(), err.suggested_action())
        }
        "DOCLI.TEMPLATE_NOT_FOUND" => {
            let template = err.not_bundled_id().unwrap_or("unknown");
            ErrorBody::template_not_found(template, err.suggested_action())
        }
        "DOCLI.IO" => {
            let path = err
                .failed_path()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."));
            let cause = err.io_message().unwrap_or_else(|| err.cause());
            ErrorBody::io(cause, path, None)
        }
        _ => ErrorBody::internal_with_action(err.cause(), err.suggested_action()),
    }
}

fn render_generate_html(
    model: &CliModel,
    template: Option<&TemplateRef>,
    theme: Option<&ThemeMap>,
) -> Result<String, ErrorBody> {
    if template.is_none() && theme.is_none() {
        return render::html::try_render(model).map_err(embedded_default_failure);
    }
    let template_ref = template
        .cloned()
        .unwrap_or_else(TemplateRef::bundled_default);
    let pack = resolve_pack(&template_ref).map_err(pack_failure)?;
    let empty_theme = ThemeMap::new();
    let theme = theme.unwrap_or(&empty_theme);
    render_pack(&pack, model, theme)
        .map_err(|err| map_render_failure(&template_ref, &pack.manifest.id, err.cause()))
}

fn embedded_default_failure(error: EmbeddedDefaultError) -> ErrorBody {
    debug_assert_eq!(error.code(), "DOCLI.INTERNAL");
    ErrorBody::internal_with_action(error.cause(), error.suggested_action())
}

/// Embedded packs stay `DOCLI.INTERNAL`. Directory packs use `templates validate`.
fn map_render_failure(template: &TemplateRef, pack_id: &PackId, cause: &str) -> ErrorBody {
    match template {
        TemplateRef::Bundled(_) => ErrorBody::internal_with_action(cause, EMBEDDED_PACK_RECOVERY),
        TemplateRef::Dir(_) => ErrorBody::template_invalid(
            cause,
            format!(
                "Run `docli templates validate` on the `{pack_id}` template pack and fix the reported issue"
            ),
        ),
    }
}

/// Installed extras omit bundled ids and `_`-prefixed starters.
fn listed_extra(id: &PackId) -> bool {
    !id.excluded_from_list() && id.to_bundled().is_none()
}

/// Parse `--theme` JSON into a [`ThemeMap`].
///
/// # Errors
///
/// Returns [`ErrorBody`] with `DOCLI.INPUT_INVALID` when JSON is invalid, not an
/// object, or contains non-string values.
pub fn parse_theme_json(theme_json: &str) -> Result<ThemeMap, ErrorBody> {
    let value: serde_json::Value = match serde_json::from_str(theme_json) {
        Ok(value) => value,
        Err(err) => return Err(ErrorBody::theme_invalid(err.to_string())),
    };
    let Some(object) = value.as_object() else {
        return Err(ErrorBody::theme_invalid(
            "theme JSON must be an object of strings",
        ));
    };
    let mut theme = ThemeMap::new();
    for (key, value) in object {
        let Some(text) = value.as_str() else {
            return Err(ErrorBody::theme_invalid(format!(
                "theme key {key} must be a string"
            )));
        };
        theme.insert(key, text);
    }
    Ok(theme)
}

fn preview_output_dir() -> PathBuf {
    static NEXT_PREVIEW: AtomicU64 = AtomicU64::new(0);
    let seq = NEXT_PREVIEW.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "docli-preview-{}-{nanos}-{seq}",
        std::process::id()
    ))
}

fn read_input(input: &InputSource) -> Result<(String, String), ErrorBody> {
    match input {
        InputSource::Stdin => {
            let mut json = String::new();
            std::io::stdin()
                .read_to_string(&mut json)
                .map_err(|err| ErrorBody::io(err.to_string(), Path::new("stdin"), None))?;
            Ok(("stdin".to_owned(), json))
        }
        InputSource::File(path) => {
            if !path.exists() {
                return Err(ErrorBody::input_not_found(path));
            }
            let json = std::fs::read_to_string(path)
                .map_err(|err| ErrorBody::io(err.to_string(), path, None))?;
            Ok((path.display().to_string(), json))
        }
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
    }
    std::fs::write(path, bytes).map_err(|err| err.to_string())
}

fn artifact(kind: &'static str, path: PathBuf, bytes: &[u8]) -> ArtifactReport {
    ArtifactReport {
        kind,
        path,
        bytes: bytes.len() as u64,
        sha256: sha256_hex(bytes),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::ErrorCode;
    use crate::schema::CliModel;

    fn demo_model() -> CliModel {
        serde_json::from_str(r#"{"name":"demo"}"#).expect("model")
    }

    #[test]
    fn omitted_template_and_theme_renders_embedded_default() {
        let html = render_generate_html(&demo_model(), None, None).expect("render");
        assert!(html.contains("id=\"docli-default-pack\""));
    }

    #[test]
    fn omitted_template_failure_uses_embedded_recovery() {
        let body = embedded_default_failure(EmbeddedDefaultError::internal("broken pack"));
        assert_eq!(body.code, ErrorCode::Internal);
        assert_eq!(body.details["cause"], "broken pack");
        assert_eq!(body.suggested_action, EMBEDDED_PACK_RECOVERY);
        assert!(!body.suggested_action.contains("Report this cause"));
    }

    #[test]
    fn bundled_render_failure_is_internal() {
        let body = map_render_failure(
            &TemplateRef::bundled_default(),
            &PackId::new("default").expect("id"),
            "missing value",
        );
        assert_eq!(body.code, ErrorCode::Internal);
        assert_eq!(body.suggested_action, EMBEDDED_PACK_RECOVERY);
        assert!(!body.suggested_action.contains("templates validate"));
        let cli_doc = map_render_failure(
            &TemplateRef::bundled(BundledPackId::CliDoc),
            &PackId::new("cli-doc").expect("id"),
            "missing value",
        );
        assert_eq!(cli_doc.code, ErrorCode::Internal);
        assert_eq!(cli_doc.suggested_action, EMBEDDED_PACK_RECOVERY);
    }

    #[test]
    fn directory_render_failure_is_template_invalid() {
        let scratch = tempfile::tempdir().expect("temp dir");
        let dir = scratch.path();
        std::fs::write(
            dir.join("template.toml"),
            "id = \"custom\"\nname = \"custom\"\nversion = \"1\"\ndescription = \"custom\"\ntheme_schema = {}\n",
        )
        .expect("manifest");
        std::fs::write(dir.join("page.html.j2"), "{{ missing_render_value }}").expect("page");
        std::fs::write(dir.join("style.css.j2"), "body{}\n").expect("style");
        std::fs::write(dir.join("script.js"), "").expect("script");
        let template = TemplateRef::dir(dir);
        let err = render_generate_html(&demo_model(), Some(&template), None).expect_err("render");
        assert_eq!(err.code, ErrorCode::TemplateInvalid);
        assert!(err.suggested_action.contains("templates validate"));
        assert!(err.suggested_action.contains("custom"));
        assert!(!err.suggested_action.contains("embedded template pack"));
    }

    #[test]
    fn listed_extra_skips_starters_and_bundled_ids() {
        assert!(!listed_extra(&PackId::new("_skeleton").expect("starter")));
        assert!(!listed_extra(&PackId::new("default").expect("default")));
        assert!(!listed_extra(&PackId::new("cli-doc").expect("cli-doc")));
        assert!(listed_extra(&PackId::new("brand").expect("brand")));
    }

    #[test]
    fn preview_output_dirs_are_unique_per_call() {
        let mut handles = Vec::new();
        for _ in 0..8 {
            handles.push(std::thread::spawn(|| {
                (0..32).map(|_| preview_output_dir()).collect::<Vec<_>>()
            }));
        }
        let mut dirs = Vec::new();
        for handle in handles {
            dirs.extend(handle.join().expect("preview thread"));
        }
        let unique: std::collections::HashSet<_> = dirs.iter().cloned().collect();
        assert_eq!(unique.len(), dirs.len());
        let prefix = format!("docli-preview-{}-", std::process::id());
        for dir in &dirs {
            assert_eq!(dir.parent(), Some(std::env::temp_dir().as_path()));
            let name = dir
                .file_name()
                .and_then(|name| name.to_str())
                .expect("name");
            let rest = name
                .strip_prefix(&prefix)
                .unwrap_or_else(|| panic!("preview dir {name}"));
            let (nanos, seq) = rest
                .split_once('-')
                .unwrap_or_else(|| panic!("preview dir {name}"));
            assert!(nanos.chars().all(|ch| ch.is_ascii_digit()) && !nanos.is_empty());
            assert!(seq.chars().all(|ch| ch.is_ascii_digit()) && !seq.is_empty());
        }
    }

    #[test]
    fn selector_parses_pack_id_once() {
        let bundled = parse_template_selector("cli-doc").expect("cli-doc");
        assert_eq!(bundled, TemplateRef::bundled(BundledPackId::CliDoc));
        let missing = parse_template_selector("not-a-bundled-pack").expect_err("missing");
        assert_eq!(missing.code, ErrorCode::TemplateNotFound);
        let empty = parse_template_selector("").expect_err("empty");
        assert_eq!(empty.code, ErrorCode::TemplateNotFound);
    }
}
