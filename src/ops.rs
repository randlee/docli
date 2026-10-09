//! In-process `generate`, `show`, and `templates` operations shared by the CLI.
//!
//! These functions return the same [`Envelope`] the CLI serializes. They write
//! or read files, hash artifacts, and never print.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::contract::{Envelope, ErrorBody};
use crate::render;
use crate::schema::CliModel;
use crate::templates::{
    install_root, resolve_pack, Pack, PackResolveError, TemplateManifest, TemplateRef, ThemeKeySpec,
};

/// Example `generate` argv included in `templates show` until b.9 adds preview flags.
pub const EXAMPLE_GENERATE_ARGV: &[&str] = &[
    "docli",
    "generate",
    "--input",
    "model.json",
    "--html",
    "site/cli",
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
    /// HTML output directory. `None` resolves to `site/cli`.
    pub html_dir: Option<PathBuf>,
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
    /// Resolved HTML directory.
    pub html_dir: PathBuf,
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
    pub id: String,
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
    pub id: String,
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
    pub id: String,
    /// Directory that was validated.
    pub path: PathBuf,
    /// Always `true` on success. Failures use the error envelope.
    pub valid: bool,
}

/// Render the model and write HTML (and optional Markdown).
///
/// `html_dir: None` resolves to `site/cli` before writing. A write failure
/// after another file landed returns `DOCLI.IO` with `details.outputs_written`.
///
/// # Errors
///
/// Returns a failure envelope for missing input, invalid JSON, I/O errors,
/// or an embedded default-pack failure (`DOCLI.INTERNAL`).
pub fn generate(req: GenerateRequest) -> Envelope<GenerateResponse> {
    let html_dir = req.html_dir.unwrap_or_else(|| PathBuf::from("site/cli"));
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

    let html = match render::html::try_render(&model) {
        Ok(html) => html,
        Err(error) => {
            debug_assert_eq!(error.code(), "DOCLI.INTERNAL");
            return Envelope::failure(ErrorBody::internal(error.cause()));
        }
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
            let written = serde_json::to_value(&outputs).unwrap_or_else(|_| serde_json::json!([]));
            return Envelope::failure(ErrorBody::io(error, &markdown_path, Some(written)));
        }
        outputs.push(artifact("markdown", markdown_path, markdown.as_bytes()));
    }

    Envelope::success(GenerateResponse {
        operation: "generate",
        input: input_label,
        model_name: model.name,
        html_dir,
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

/// List the embedded `default` pack and any valid extra packs under [`install_root`].
///
/// A missing install root is an empty extra list, not an error. Unreadable
/// install roots are `DOCLI.IO`. Extra directories that do not load are omitted
/// so one broken pack does not hide the others; [`templates_validate`] reports
/// that failure.
///
/// # Errors
///
/// Returns a failure envelope when the embedded pack cannot load
/// (`DOCLI.INTERNAL`) or the install root exists but cannot be read (`DOCLI.IO`).
pub fn templates_list() -> Envelope<TemplatesListResponse> {
    let install_root = install_root();
    let pack = match resolve_pack(&TemplateRef::bundled_default()) {
        Ok(pack) => pack,
        Err(err) => return Envelope::failure(pack_failure(err)),
    };
    let mut templates = vec![TemplateSummary {
        path: format!("embedded:{}", pack.manifest.id),
        id: pack.manifest.id.clone(),
        name: pack.manifest.name,
        version: pack.manifest.version,
    }];
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
/// `DOCLI.IO` when a directory cannot be read, and `DOCLI.INTERNAL` for a
/// damaged embedded pack or an unknown id (unknown ids become
/// `DOCLI.TEMPLATE_NOT_FOUND` in b.9).
pub fn templates_show(selector: &str) -> Envelope<TemplatesShowResponse> {
    let pack = match resolve_show_target(selector) {
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
        if pack.manifest.id == "default" {
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

fn resolve_show_target(selector: &str) -> Result<Pack, PackResolveError> {
    if selector_is_path(selector) {
        return resolve_pack(&TemplateRef::dir(Path::new(selector)));
    }
    match TemplateRef::try_bundled(selector) {
        Ok(bundled) => resolve_pack(&bundled),
        Err(not_bundled) => {
            let installed = install_root().join(selector);
            if installed.is_dir() {
                resolve_pack(&TemplateRef::dir(installed))
            } else {
                Err(not_bundled)
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
