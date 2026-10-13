//! In-process `generate`, `show`, and `templates` operations shared by the CLI.
//!
//! These functions return the same [`Envelope`] the CLI serializes. They write
//! or read files, hash artifacts, and never print.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::contract::{Envelope, ErrorBody};
use crate::render;
use crate::schema::CliModel;
use crate::templates::{
    install_root, render_pack, resolve_pack, BundledPackId, EmbeddedDefaultError, PackId,
    PackResolveError, RenderError, TemplateManifest, TemplateRef, ThemeKeySpec, ThemeMap,
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

/// Flag name for `docli generate --dotnet-project`.
const DOTNET_PROJECT_FLAG: &str = "--dotnet-project";

/// Deadline for one `docli-dotnet project` invocation (its build and run).
const DOTNET_PROJECT_TIMEOUT: Duration = Duration::from_secs(180);

/// Positive seconds, at most one day, override [`DOTNET_PROJECT_TIMEOUT`].
const DOTNET_PROJECT_TIMEOUT_ENV: &str = "DOCLI_DOTNET_PROJECT_TIMEOUT_SECS";

/// Characters kept from the end of a collapsed child diagnostic.
///
/// `dotnet build` writes the banner first and the compiler error last. A
/// 240-character head keeps the banner. 1024 from the tail still fits an
/// error line whose project path is long, plus the build summary.
const DIAGNOSTIC_CHARS: usize = 1024;

/// Run `docli-dotnet project` for [`DOTNET_PROJECT_FLAG`] and return a temp JSON path.
///
/// Resolution uses an executable `docli-dotnet` on `PATH` when that file
/// exists. A non-executable file of that name does not block the repo DLL.
/// Otherwise it runs `dotnet` on
/// `dotnet/src/Docli.Tool/bin/Release/<tfm>/Docli.Tool.dll`, where `<tfm>` is
/// the `TargetFramework` in that tool project (the ADR-005 pin). The repo root
/// is the nearest ancestor of the running binary, then of the current
/// directory, that contains both `Cargo.toml` and the tool project. The child
/// is killed if it exceeds [`DOTNET_PROJECT_TIMEOUT`] (`DOCLI_DOTNET_PROJECT_TIMEOUT_SECS`
/// overrides that deadline). The returned file is the child stdout.
/// [`generate`] reads it as [`InputSource::File`].
///
/// # Errors
///
/// Returns `DOCLI.IO` when `docli-dotnet` or `dotnet` cannot be started, the
/// repo or Release DLL is missing, the tool `TargetFramework` contains a path
/// separator or `..`, the child exceeds the deadline, or the child exits
/// non-zero for a reason other than bad export JSON. Returns
/// `DOCLI.INPUT_INVALID` when the child stdout is empty, is not UTF-8, or is
/// not `CliModel` JSON, and when the child exits 2 and stderr reports an empty
/// or invalid JSON export. Those codes are the section 9 codes; the messages
/// name the export host or the subprocess.
pub fn materialize_dotnet_project(csproj: &Path) -> Result<PathBuf, ErrorBody> {
    let output = run_dotnet_project(csproj)?;
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        return Err(project_exit_error(
            csproj,
            code,
            &String::from_utf8_lossy(&output.stderr),
        ));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|err| {
        export_stdout_error(
            csproj,
            "docli-dotnet project stdout was not UTF-8",
            err.to_string(),
        )
    })?;
    reject_export_stdout(csproj, &stdout)?;

    let path = dotnet_project_temp_json();
    std::fs::write(&path, stdout.as_bytes())
        .map_err(|err| ErrorBody::io(err.to_string(), &path, None))?;
    Ok(path)
}

fn reject_export_stdout(csproj: &Path, stdout: &str) -> Result<(), ErrorBody> {
    if stdout.trim().is_empty() {
        return Err(export_stdout_error(
            csproj,
            "docli-dotnet project returned empty stdout",
            "export host stdout was empty",
        ));
    }
    if let Err(err) = serde_json::from_str::<CliModel>(stdout) {
        return Err(export_stdout_error(
            csproj,
            "docli-dotnet project stdout is not CliModel JSON",
            err.to_string(),
        ));
    }
    Ok(())
}

fn export_stdout_error(csproj: &Path, message: &str, cause: impl Into<String>) -> ErrorBody {
    ErrorBody::input_invalid_with_action(
        message,
        cause,
        format!(
            "Make {} write one UTF-8 CliModel JSON document to stdout and rerun docli generate {DOTNET_PROJECT_FLAG}",
            csproj.display()
        ),
    )
}

fn project_exit_error(csproj: &Path, code: i32, stderr: &str) -> ErrorBody {
    let detail = one_line(stderr);
    let detail = if detail.is_empty() {
        "no stderr".to_owned()
    } else {
        detail
    };
    // Classify on the full stderr. The stored cause is the tail, which can
    // drop a leading `json parse failed` once the compiler text is long.
    if code == 2 && stderr_reports_invalid_export(stderr) {
        let message = if stderr.to_ascii_lowercase().contains("empty") {
            "docli-dotnet project returned empty stdout"
        } else {
            "docli-dotnet project stdout is not CliModel JSON"
        };
        return export_stdout_error(csproj, message, detail);
    }
    ErrorBody::io_with_action(
        format!("docli-dotnet project exited {code}"),
        detail,
        format!(
            "Fix the export project {} and rerun docli generate {DOTNET_PROJECT_FLAG}",
            csproj.display()
        ),
    )
}

/// Section 9: exit 2 plus an empty or invalid JSON export is `DOCLI.INPUT_INVALID`.
///
/// Other non-zero exits stay `DOCLI.IO`. The phrases are the tool's stderr
/// (`empty export stdout`, `json parse failed`) and the same conditions in prose.
fn stderr_reports_invalid_export(detail: &str) -> bool {
    let lower = detail.to_ascii_lowercase();
    lower.contains("empty export stdout")
        || lower.contains("json parse failed")
        || lower.contains("empty stdout")
        || lower.contains("not climodel")
        || lower.contains("invalid json")
}

fn run_dotnet_project(csproj: &Path) -> Result<std::process::Output, ErrorBody> {
    let mut command = command_for_dotnet_project(csproj)?;
    command
        .arg("--project")
        .arg(csproj)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|err| start_failure(&command, err, csproj))?;
    let stdout_thread = std::thread::spawn({
        let pipe = child.stdout.take();
        move || read_pipe(pipe)
    });
    let stderr_thread = std::thread::spawn({
        let pipe = child.stderr.take();
        move || read_pipe(pipe)
    });
    let timeout = project_timeout();
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                kill_child_tree(&mut child);
                let _ = child.wait();
                let stderr = join_bytes(stderr_thread);
                let _stdout = join_bytes(stdout_thread);
                return Err(project_timeout_error(csproj, timeout, &stderr));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(err) => {
                kill_child_tree(&mut child);
                let _ = child.wait();
                let _ = join_bytes(stdout_thread);
                let _ = join_bytes(stderr_thread);
                return Err(ErrorBody::io_with_action(
                    "failed to wait for docli-dotnet",
                    err.to_string(),
                    format!(
                        "Rerun docli generate {DOTNET_PROJECT_FLAG} for {}",
                        csproj.display()
                    ),
                ));
            }
        }
    };
    Ok(std::process::Output {
        status,
        stdout: join_bytes(stdout_thread),
        stderr: join_bytes(stderr_thread),
    })
}

fn read_pipe(pipe: Option<impl Read>) -> Vec<u8> {
    let mut buf = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut buf);
    }
    buf
}

fn join_bytes(handle: std::thread::JoinHandle<Vec<u8>>) -> Vec<u8> {
    handle.join().unwrap_or_default()
}

fn project_timeout() -> Duration {
    timeout_from_env(std::env::var(DOTNET_PROJECT_TIMEOUT_ENV).ok().as_deref())
}

fn timeout_from_env(raw: Option<&str>) -> Duration {
    let Some(seconds) = raw.map(str::trim).and_then(|text| text.parse::<u64>().ok()) else {
        return DOTNET_PROJECT_TIMEOUT;
    };
    if seconds == 0 || seconds > 24 * 60 * 60 {
        return DOTNET_PROJECT_TIMEOUT;
    }
    Duration::from_secs(seconds)
}

fn project_timeout_error(csproj: &Path, timeout: Duration, stderr: &[u8]) -> ErrorBody {
    let detail = one_line(&String::from_utf8_lossy(stderr));
    let detail = if detail.is_empty() {
        format!("no stderr after {}s", timeout.as_secs())
    } else {
        detail
    };
    ErrorBody::io_with_action(
        format!("docli-dotnet project timed out after {}s", timeout.as_secs()),
        detail,
        format!(
            "The export project {} did not finish within {}s. Fix the project or raise {DOTNET_PROJECT_TIMEOUT_ENV}, then rerun docli generate {DOTNET_PROJECT_FLAG}",
            csproj.display(),
            timeout.as_secs()
        ),
    )
}

fn kill_child_tree(child: &mut Child) {
    #[cfg(unix)]
    {
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        if let Ok(pid) = i32::try_from(child.id()) {
            // SAFETY: `pid` is the child spawned as a new process-group leader.
            // A negative pid delivers SIGKILL (9) to that group, including
            // processes the child started in the same group.
            unsafe {
                let _ = kill(-pid, 9);
            }
        }
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

fn start_failure(command: &Command, err: std::io::Error, csproj: &Path) -> ErrorBody {
    let program = command.get_program().to_string_lossy();
    ErrorBody::io_with_action(
        format!("failed to start {program}"),
        err.to_string(),
        format!(
            "Install docli-dotnet on PATH, or install the .NET SDK and build with `dotnet build dotnet/Docli.sln -c Release`, then rerun docli generate {DOTNET_PROJECT_FLAG} for {}",
            csproj.display()
        ),
    )
}

fn command_for_dotnet_project(csproj: &Path) -> Result<Command, ErrorBody> {
    if let Some(tool) = docli_dotnet_on_path() {
        let mut command = Command::new(tool);
        command.arg("project");
        return Ok(command);
    }

    let Some(repo) = find_repo_root() else {
        return Err(missing_dotnet_tool(csproj, None));
    };
    let dll = release_tool_dll(&repo)?;
    if !dll.is_file() {
        return Err(missing_dotnet_tool(csproj, Some(dll)));
    }
    let mut command = Command::new("dotnet");
    command.arg(&dll).arg("project");
    Ok(command)
}

fn missing_dotnet_tool(csproj: &Path, expected_dll: Option<PathBuf>) -> ErrorBody {
    let (location, action) = match &expected_dll {
        Some(dll) => (
            format!(
                "docli-dotnet was not on PATH and {} was not found",
                dll.display()
            ),
            format!(
                "Build with `dotnet build dotnet/Docli.sln -c Release` from the docli repo, or install docli-dotnet on PATH, then rerun docli generate {DOTNET_PROJECT_FLAG} for {}",
                csproj.display()
            ),
        ),
        None => (
            "docli-dotnet was not on PATH and the repo root could not be resolved".to_owned(),
            format!(
                "Install docli-dotnet on PATH, then rerun docli generate {DOTNET_PROJECT_FLAG} for {}",
                csproj.display()
            ),
        ),
    };
    ErrorBody::io_with_action("docli-dotnet was not found", location, action)
}

fn docli_dotnet_on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| executable_docli_dotnet_on_path(&path_var))
}

fn executable_docli_dotnet_on_path(path_var: &std::ffi::OsStr) -> Option<PathBuf> {
    let file_name = if cfg!(windows) {
        "docli-dotnet.exe"
    } else {
        "docli-dotnet"
    };
    std::env::split_paths(path_var)
        .map(|dir| dir.join(file_name))
        .find(|candidate| is_executable_file(candidate))
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn find_repo_root() -> Option<PathBuf> {
    let mut starts = Vec::new();
    // The binary's checkout wins. cwd is the fallback for a `docli` that is
    // not sitting inside a checkout (for example `cargo install`).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            starts.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    repo_root_from(&starts)
}

fn repo_root_from(starts: &[PathBuf]) -> Option<PathBuf> {
    for start in starts {
        for ancestor in start.ancestors() {
            if ancestor.join("Cargo.toml").is_file()
                && ancestor
                    .join("dotnet/src/Docli.Tool/Docli.Tool.csproj")
                    .is_file()
            {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}

fn release_tool_dll(repo: &Path) -> Result<PathBuf, ErrorBody> {
    let csproj = repo.join("dotnet/src/Docli.Tool/Docli.Tool.csproj");
    let tfm = tool_target_framework(&csproj)?;
    Ok(repo
        .join("dotnet/src/Docli.Tool/bin/Release")
        .join(tfm)
        .join("Docli.Tool.dll"))
}

fn tool_target_framework(csproj: &Path) -> Result<String, ErrorBody> {
    let text = std::fs::read_to_string(csproj).map_err(|err| {
        ErrorBody::io(
            format!(
                "docli generate {DOTNET_PROJECT_FLAG} could not read {}: {err}",
                csproj.display()
            ),
            csproj,
            None,
        )
    })?;
    let Some(rest) = text.split_once("<TargetFramework>") else {
        return Err(ErrorBody::io(
            format!(
                "docli generate {DOTNET_PROJECT_FLAG} found no TargetFramework in {}",
                csproj.display()
            ),
            csproj,
            None,
        ));
    };
    let Some(tfm) = rest.1.split_once("</TargetFramework>") else {
        return Err(ErrorBody::io(
            format!(
                "docli generate {DOTNET_PROJECT_FLAG} found an unclosed TargetFramework in {}",
                csproj.display()
            ),
            csproj,
            None,
        ));
    };
    let tfm = tfm.0.trim();
    if tfm.is_empty() {
        return Err(ErrorBody::io(
            format!(
                "docli generate {DOTNET_PROJECT_FLAG} found an empty TargetFramework in {}",
                csproj.display()
            ),
            csproj,
            None,
        ));
    }
    // The TFM is one path segment under bin/Release. A separator or `..`
    // would leave that directory when the Release DLL is probed.
    if tfm.contains(['/', '\\']) || tfm.contains("..") {
        return Err(ErrorBody::io(
            format!(
                "docli generate {DOTNET_PROJECT_FLAG} rejected TargetFramework {tfm:?} in {}: \
                 a framework moniker cannot contain a path separator or '..'",
                csproj.display()
            ),
            csproj,
            None,
        ));
    }
    Ok(tfm.to_owned())
}

fn dotnet_project_temp_json() -> PathBuf {
    static NEXT_MODEL: AtomicU64 = AtomicU64::new(0);
    let seq = NEXT_MODEL.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "docli-dotnet-project-{}-{nanos}-{seq}.json",
        std::process::id()
    ))
}

fn one_line(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let count = collapsed.chars().count();
    if count <= DIAGNOSTIC_CHARS {
        collapsed
    } else {
        collapsed.chars().skip(count - DIAGNOSTIC_CHARS).collect()
    }
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
    render_pack(&pack, model, theme).map_err(map_render_failure)
}

fn embedded_default_failure(error: EmbeddedDefaultError) -> ErrorBody {
    debug_assert_eq!(error.code(), "DOCLI.INTERNAL");
    ErrorBody::internal_with_action(error.cause(), error.suggested_action())
}

/// Uses the pack origin recorded on [`RenderError`].
///
/// Directory packs are `DOCLI.TEMPLATE_INVALID`. Embedded packs stay
/// `DOCLI.INTERNAL`. Recovery text comes from [`RenderError::suggested_action`].
fn map_render_failure(err: RenderError) -> ErrorBody {
    if err.machine_code() == "DOCLI.TEMPLATE_INVALID" {
        ErrorBody::template_invalid(err.cause(), err.suggested_action())
    } else {
        ErrorBody::internal_with_action(err.cause(), err.suggested_action())
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
    use crate::templates::EMBEDDED_PACK_RECOVERY;

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
        let pack = resolve_pack(&TemplateRef::bundled_default()).expect("default");
        let body = map_render_failure(RenderError::from_pack(&pack, "missing value"));
        assert_eq!(body.code, ErrorCode::Internal);
        assert_eq!(body.suggested_action, EMBEDDED_PACK_RECOVERY);
        assert!(!body.suggested_action.contains("templates validate"));
        let cli_doc = resolve_pack(&TemplateRef::bundled(BundledPackId::CliDoc)).expect("cli-doc");
        let body = map_render_failure(RenderError::from_pack(&cli_doc, "missing value"));
        assert_eq!(body.code, ErrorCode::Internal);
        assert_eq!(body.suggested_action, EMBEDDED_PACK_RECOVERY);
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
    fn export_stdout_errors_name_the_host_output() {
        let csproj = Path::new("sample.csproj");
        let empty = reject_export_stdout(csproj, " \n").expect_err("empty");
        assert_eq!(empty.code, ErrorCode::InputInvalid);
        assert_eq!(empty.message, "docli-dotnet project returned empty stdout");
        assert_eq!(empty.details["cause"], "export host stdout was empty");
        assert!(empty.suggested_action.contains("sample.csproj"));
        assert!(!empty.suggested_action.contains("Fix the model JSON"));
        assert!(!empty.message.contains("failed to read or write"));

        let bad = reject_export_stdout(csproj, "not-json").expect_err("bad json");
        assert_eq!(bad.code, ErrorCode::InputInvalid);
        assert_eq!(
            bad.message,
            "docli-dotnet project stdout is not CliModel JSON"
        );
        assert!(bad.details["cause"].as_str().unwrap().contains("expected"));

        let utf8 = export_stdout_error(
            csproj,
            "docli-dotnet project stdout was not UTF-8",
            "bad bytes".to_owned(),
        );
        assert_eq!(utf8.code, ErrorCode::InputInvalid);
        assert_eq!(utf8.message, "docli-dotnet project stdout was not UTF-8");

        reject_export_stdout(csproj, r#"{"name":"sample"}"#).expect("model json");
    }

    #[test]
    fn subprocess_failures_name_docli_dotnet() {
        let csproj = Path::new("sample.csproj");
        let missing = missing_dotnet_tool(csproj, None);
        assert_eq!(missing.code, ErrorCode::Io);
        assert_eq!(missing.message, "docli-dotnet was not found");
        assert!(missing
            .suggested_action
            .contains("Install docli-dotnet on PATH"));
        assert!(!missing.suggested_action.contains("dotnet/Docli.sln"));
        assert!(missing.suggested_action.contains("sample.csproj"));
        assert!(!missing.message.contains("failed to read or write"));
        assert!(!missing.suggested_action.contains("fixing access"));

        let dll = Path::new("dotnet/src/Docli.Tool/bin/Release/net10.0/Docli.Tool.dll");
        let missing_dll = missing_dotnet_tool(csproj, Some(dll.to_path_buf()));
        assert!(missing_dll.details["cause"]
            .as_str()
            .unwrap()
            .contains("Docli.Tool.dll"));
        assert!(missing_dll.suggested_action.contains("dotnet build"));
        assert!(missing_dll.suggested_action.contains("dotnet/Docli.sln"));

        let exited = project_exit_error(csproj, 2, "build failed");
        assert_eq!(exited.code, ErrorCode::Io);
        assert_eq!(exited.message, "docli-dotnet project exited 2");
        assert_eq!(exited.details["cause"], "build failed");
        assert!(exited.suggested_action.contains("sample.csproj"));
        assert!(exited.details.get("outputs_written").is_none());

        let empty = project_exit_error(
            csproj,
            2,
            "docli-dotnet: empty export stdout: sample.csproj",
        );
        assert_eq!(empty.code, ErrorCode::InputInvalid);
        assert_eq!(empty.message, "docli-dotnet project returned empty stdout");
        let parsed = project_exit_error(
            csproj,
            2,
            "docli-dotnet: json parse failed: unexpected token",
        );
        assert_eq!(parsed.code, ErrorCode::InputInvalid);
        assert_eq!(
            parsed.message,
            "docli-dotnet project stdout is not CliModel JSON"
        );
        let other = project_exit_error(
            csproj,
            1,
            "docli-dotnet: json parse failed: unexpected token",
        );
        assert_eq!(other.code, ErrorCode::Io);

        let long = format!(
            "docli-dotnet: json parse failed: {} unexpected trailing",
            "x".repeat(DIAGNOSTIC_CHARS + 20)
        );
        let long_parse = project_exit_error(csproj, 2, &long);
        assert_eq!(long_parse.code, ErrorCode::InputInvalid);
        let cause = long_parse.details["cause"].as_str().unwrap();
        assert!(cause.contains("unexpected trailing"), "{cause}");
        assert!(!cause.contains("json parse failed"), "{cause}");
        assert!(cause.chars().count() <= DIAGNOSTIC_CHARS);

        let mut command = Command::new("dotnet");
        let started = start_failure(&command, std::io::Error::other("no such file"), csproj);
        assert_eq!(started.message, "failed to start dotnet");
        assert_eq!(started.code, ErrorCode::Io);
        command = Command::new("docli-dotnet");
        let started = start_failure(&command, std::io::Error::other("denied"), csproj);
        assert_eq!(started.message, "failed to start docli-dotnet");
    }

    #[test]
    fn diagnostic_tail_keeps_the_compiler_error() {
        let text = format!(
            "HEADLINE {} error CS1002: ; expected [/tmp/host.csproj]",
            "x".repeat(DIAGNOSTIC_CHARS + 50)
        );
        let tail = one_line(&text);
        assert!(tail.contains("error CS1002"), "{tail}");
        assert!(!tail.contains("HEADLINE"), "{tail}");
        assert!(tail.chars().count() <= DIAGNOSTIC_CHARS);
    }

    #[test]
    fn target_framework_rejects_path_separators_and_parent_segments() {
        let scratch = tempfile::tempdir().expect("temp dir");
        let csproj = scratch.path().join("Docli.Tool.csproj");
        std::fs::write(
            &csproj,
            "<Project><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>",
        )
        .expect("csproj");
        assert_eq!(tool_target_framework(&csproj).expect("tfm"), "net10.0");

        for tfm in [
            "../net10.0",
            "net10.0/extra",
            r"net10.0\extra",
            "net..10.0",
            "..",
        ] {
            std::fs::write(
                &csproj,
                format!(
                    "<Project><PropertyGroup><TargetFramework>{tfm}</TargetFramework></PropertyGroup></Project>"
                ),
            )
            .expect("csproj");
            let err = tool_target_framework(&csproj).expect_err(tfm);
            assert_eq!(err.code, ErrorCode::Io, "{tfm}");
            let cause = err.details["cause"].as_str().expect("cause");
            assert!(cause.contains("rejected TargetFramework"), "{tfm}: {cause}");
            assert!(
                cause.contains("path separator") && cause.contains("'..'"),
                "{tfm}: {cause}"
            );
            assert!(!cause.contains("Docli.Tool.dll"), "{tfm}: {cause}");
        }
    }

    #[test]
    fn timeout_from_env_rejects_missing_and_out_of_range() {
        assert_eq!(timeout_from_env(None), DOTNET_PROJECT_TIMEOUT);
        assert_eq!(timeout_from_env(Some("0")), DOTNET_PROJECT_TIMEOUT);
        assert_eq!(timeout_from_env(Some("nope")), DOTNET_PROJECT_TIMEOUT);
        assert_eq!(timeout_from_env(Some("999999")), DOTNET_PROJECT_TIMEOUT);
        assert_eq!(timeout_from_env(Some(" 12 ")), Duration::from_secs(12));
    }

    #[test]
    fn repo_root_prefers_the_running_binary_checkout() {
        let root = std::env::temp_dir().join(format!(
            "docli-repo-root-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let exe_repo = root.join("exe-checkout");
        let cwd_repo = root.join("cwd-checkout");
        for repo in [&exe_repo, &cwd_repo] {
            std::fs::create_dir_all(repo.join("dotnet/src/Docli.Tool")).expect("dirs");
            std::fs::write(repo.join("Cargo.toml"), "[package]\nname = \"x\"\n").expect("toml");
            std::fs::write(
                repo.join("dotnet/src/Docli.Tool/Docli.Tool.csproj"),
                "<Project></Project>",
            )
            .expect("csproj");
        }
        let exe_bin = exe_repo.join("target/debug");
        std::fs::create_dir_all(&exe_bin).expect("bin");
        let found = repo_root_from(&[exe_bin, cwd_repo.clone()]).expect("repo");
        assert_eq!(found, exe_repo);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn path_probe_skips_non_executable_file() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "docli-path-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let stale = root.join("stale");
        let real = root.join("real");
        std::fs::create_dir_all(&stale).expect("stale");
        std::fs::create_dir_all(&real).expect("real");
        let stale_bin = stale.join("docli-dotnet");
        let real_bin = real.join("docli-dotnet");
        std::fs::write(&stale_bin, "#!/bin/sh\nexit 9\n").expect("stale file");
        std::fs::write(&real_bin, "#!/bin/sh\nexit 0\n").expect("real file");
        let mut stale_mode = std::fs::metadata(&stale_bin).expect("meta").permissions();
        stale_mode.set_mode(0o644);
        std::fs::set_permissions(&stale_bin, stale_mode).expect("chmod stale");
        let mut real_mode = std::fs::metadata(&real_bin).expect("meta").permissions();
        real_mode.set_mode(0o755);
        std::fs::set_permissions(&real_bin, real_mode).expect("chmod real");

        let both = std::env::join_paths([&stale, &real]).expect("path");
        assert_eq!(
            executable_docli_dotnet_on_path(&both).expect("executable"),
            real_bin
        );
        let only_stale = std::env::join_paths([&stale]).expect("path");
        assert!(executable_docli_dotnet_on_path(&only_stale).is_none());
        let _ = std::fs::remove_dir_all(&root);
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
