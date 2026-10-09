//! In-process `generate`, `show`, and `templates` operations shared by the CLI.
//!
//! These functions return the same [`Envelope`] the CLI serializes. They write
//! or read files, hash artifacts, and never print.

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::contract::{Envelope, ErrorBody};
use crate::render;
use crate::schema::CliModel;
use crate::templates::{
    resolve_pack, Pack, PackResolveError, TemplateManifest, TemplateRef, ThemeKeySpec,
    EMBEDDED_PATH_PREFIX,
};

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

/// One pack returned by [`templates_list`].
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSummary {
    /// Pack id from `template.toml`.
    pub id: String,
    /// Display name from `template.toml`.
    pub name: String,
    /// Pack version string.
    pub version: String,
    /// `embedded:<id>` for a bundled pack, or the pack directory.
    pub path: String,
}

/// Request for [`templates_list`].
#[derive(Debug, Clone)]
pub struct TemplatesListRequest {
    /// Directory of optional extra packs (`{prefix}/share/docli/templates`).
    pub install_root: PathBuf,
}

/// Successful `templates list` response.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesListResponse {
    /// Always `"templates_list"`.
    pub operation: &'static str,
    /// Optional-pack directory from the request.
    pub install_root: PathBuf,
    /// Bundled packs, then install-root packs sorted by id.
    pub templates: Vec<TemplateSummary>,
}

/// Request for [`templates_show`].
#[derive(Debug, Clone)]
pub struct TemplatesShowRequest {
    /// Bundled id, `embedded:<id>`, or a pack directory.
    pub id_or_path: String,
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
    /// Theme keys from the manifest, in the same object form.
    pub theme_schema: std::collections::BTreeMap<String, ThemeKeySpec>,
    /// Argv that renders this pack once `generate` flags exist.
    ///
    /// Sprint b.8 omits `--template`, `--theme`, and `--preview`. Sprint b.9
    /// extends this argv when those flags exist.
    pub example_generate_argv: &'static [&'static str],
}

/// Request for [`templates_validate`].
#[derive(Debug, Clone)]
pub struct TemplatesValidateRequest {
    /// Pack directory or `embedded:<id>`.
    pub path: String,
}

/// Successful `templates validate` response.
#[derive(Debug, Clone, Serialize)]
pub struct TemplatesValidateResponse {
    /// Always `"templates_validate"`.
    pub operation: &'static str,
    /// The path argument, unchanged.
    pub path: String,
    /// Pack id from the manifest.
    pub id: String,
    /// Pack version from the manifest.
    pub version: String,
    /// Always `true` on success. Invalid packs are a failure envelope.
    pub valid: bool,
}

/// Argv returned by [`templates_show`] until b.9 adds template flags.
pub const EXAMPLE_GENERATE_ARGV: &[&str] = &[
    "docli",
    "generate",
    "--input",
    "model.json",
    "--html",
    "site/cli",
];

/// `{prefix}/share/docli/templates`, where `{prefix}` is the parent of the
/// executable's directory.
///
/// `/usr/local/bin/docli` resolves to `/usr/local/share/docli/templates`. When
/// the executable path is unavailable, `{prefix}` is `/usr/local`.
pub fn default_template_install_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().and_then(Path::parent).map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("/usr/local"))
        .join("share/docli/templates")
}

/// List the embedded `default` pack and optional packs under `install_root`.
///
/// A missing install root is an empty extra list, not an error. A directory
/// without `template.toml` is skipped. A pack whose id matches a bundled pack
/// is omitted. An invalid extra pack fails the list with
/// `DOCLI.TEMPLATE_INVALID`.
///
/// # Errors
///
/// Returns a failure envelope when the embedded pack cannot load
/// (`DOCLI.INTERNAL`), the install root cannot be read (`DOCLI.IO`), or an
/// extra pack is invalid (`DOCLI.TEMPLATE_INVALID`).
pub fn templates_list(req: TemplatesListRequest) -> Envelope<TemplatesListResponse> {
    let mut templates = match bundled_summaries() {
        Ok(templates) => templates,
        Err(err) => return Envelope::failure(pack_failure(err)),
    };
    match scan_install_root(&req.install_root, &templates) {
        Ok(extras) => templates.extend(extras),
        Err(error) => return Envelope::failure(error),
    }
    Envelope::success(TemplatesListResponse {
        operation: "templates_list",
        install_root: req.install_root,
        templates,
    })
}

/// Show one pack's manifest, theme schema, and example generate argv.
///
/// # Errors
///
/// Returns `DOCLI.TEMPLATE_INVALID` when the pack does not compile or the id
/// is not bundled. Returns `DOCLI.IO` when a pack directory cannot be read.
/// Returns `DOCLI.INTERNAL` when the embedded pack itself is damaged.
pub fn templates_show(req: TemplatesShowRequest) -> Envelope<TemplatesShowResponse> {
    match resolve_selector(&req.id_or_path) {
        Ok(pack) => Envelope::success(show_response(pack)),
        Err(err) => Envelope::failure(pack_failure(err)),
    }
}

/// Validate a pack directory or `embedded:<id>` by resolving it.
///
/// Success means `template.toml` parsed and the pack templates compiled.
///
/// # Errors
///
/// Returns `DOCLI.TEMPLATE_INVALID` for an invalid manifest, a template that
/// does not compile, or an unknown `embedded:<id>`. Returns `DOCLI.IO` when
/// a required file cannot be read. Returns `DOCLI.INTERNAL` for a damaged
/// embedded pack.
pub fn templates_validate(req: TemplatesValidateRequest) -> Envelope<TemplatesValidateResponse> {
    match resolve_selector(&req.path) {
        Ok(pack) => Envelope::success(TemplatesValidateResponse {
            operation: "templates_validate",
            path: req.path,
            id: pack.manifest.id,
            version: pack.manifest.version,
            valid: true,
        }),
        Err(err) => Envelope::failure(pack_failure(err)),
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

fn bundled_summaries() -> Result<Vec<TemplateSummary>, PackResolveError> {
    let pack = resolve_pack(&TemplateRef::bundled_default())?;
    Ok(vec![summary_from_pack(
        &pack,
        format!("{EMBEDDED_PATH_PREFIX}{}", pack.manifest.id),
    )])
}

fn scan_install_root(
    root: &Path,
    bundled: &[TemplateSummary],
) -> Result<Vec<TemplateSummary>, ErrorBody> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let entries =
        std::fs::read_dir(root).map_err(|err| ErrorBody::io(err.to_string(), root, None))?;
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| ErrorBody::io(err.to_string(), root, None))?;
        let path = entry.path();
        if path.is_dir() && path.join("template.toml").is_file() {
            dirs.push(path);
        }
    }
    dirs.sort();

    let mut extras: Vec<TemplateSummary> = Vec::new();
    for dir in dirs {
        let pack = match resolve_pack(&TemplateRef::dir(&dir)) {
            Ok(pack) => pack,
            Err(err) => return Err(pack_failure(err)),
        };
        if bundled.iter().any(|item| item.id == pack.manifest.id) {
            continue;
        }
        if extras.iter().any(|item| item.id == pack.manifest.id) {
            return Err(ErrorBody::template_invalid(
                format!(
                    "install root {} contains more than one pack with id {}",
                    root.display(),
                    pack.manifest.id
                ),
                format!(
                    "Keep one directory for id {} under {} and run `docli templates list --json`",
                    pack.manifest.id,
                    root.display()
                ),
            ));
        }
        extras.push(summary_from_pack(&pack, dir.display().to_string()));
    }
    extras.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(extras)
}

fn summary_from_pack(pack: &Pack, path: String) -> TemplateSummary {
    TemplateSummary {
        id: pack.manifest.id.clone(),
        name: pack.manifest.name.clone(),
        version: pack.manifest.version.clone(),
        path,
    }
}

fn show_response(pack: Pack) -> TemplatesShowResponse {
    TemplatesShowResponse {
        operation: "templates_show",
        id: pack.manifest.id.clone(),
        theme_schema: pack.manifest.theme_schema.clone(),
        example_generate_argv: EXAMPLE_GENERATE_ARGV,
        manifest: pack.manifest,
    }
}

fn resolve_selector(selector: &str) -> Result<Pack, PackResolveError> {
    if let Some(id) = selector.strip_prefix(EMBEDDED_PATH_PREFIX) {
        if !id.is_empty() && !id.contains(['/', '\\']) {
            return resolve_pack(&TemplateRef::try_bundled(id)?);
        }
    }
    let path = Path::new(selector);
    if path.exists() || path.is_absolute() || selector.contains(['/', '\\']) {
        return resolve_pack(&TemplateRef::dir(path));
    }
    resolve_pack(&TemplateRef::try_bundled(selector)?)
}

fn pack_failure(err: PackResolveError) -> ErrorBody {
    if err.is_invalid() || err.is_not_bundled() {
        let suggested_action = err.suggested_action();
        ErrorBody::template_invalid(err.cause(), suggested_action)
    } else if err.is_io() {
        let path = err
            .path()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("template"));
        ErrorBody::io(err.cause(), path, None)
    } else {
        debug_assert_eq!(err.machine_code(), "DOCLI.INTERNAL");
        ErrorBody::internal(err.cause())
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
