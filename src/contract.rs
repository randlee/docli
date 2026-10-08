//! Version `"1"` JSON envelope, error bodies, and process exit codes.
//!
//! Success and failure share one shape. `data` and `error` always serialize,
//! including as JSON `null`. Callers branch on `ok` and `error.code`.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// Process exit code for a successful command.
pub const EXIT_SUCCESS: i32 = 0;
/// Process exit code for validation failures (`DOCLI.USAGE`, `DOCLI.INPUT_INVALID`).
pub const EXIT_VALIDATION: i32 = 2;
/// Process exit code for missing paths (`DOCLI.INPUT_NOT_FOUND`, `DOCLI.OUTPUT_NOT_FOUND`).
pub const EXIT_NOT_FOUND: i32 = 3;
/// Process exit code for I/O dependency failures (`DOCLI.IO`).
pub const EXIT_DEPENDENCY: i32 = 4;
/// Process exit code for unexpected failures (`DOCLI.INTERNAL`).
pub const EXIT_INTERNAL: i32 = 1;

/// Version `"1"` success or failure wrapper.
#[derive(Debug, Serialize)]
pub struct Envelope<T> {
    /// Envelope schema version. Always `"1"`.
    pub version: &'static str,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Response body on success; JSON `null` on failure.
    pub data: Option<T>,
    /// Error body on failure; JSON `null` on success.
    pub error: Option<ErrorBody>,
}

impl<T> Envelope<T> {
    /// Build a success envelope with `error` serialized as `null`.
    pub fn success(data: T) -> Self {
        Self {
            version: "1",
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    /// Build a failure envelope with `data` serialized as `null`.
    pub fn failure(error: ErrorBody) -> Self {
        Self {
            version: "1",
            ok: false,
            data: None,
            error: Some(error),
        }
    }

    /// Process exit code that matches this envelope.
    pub fn exit_code(&self) -> i32 {
        match &self.error {
            None => EXIT_SUCCESS,
            Some(error) => error.code.exit_code(),
        }
    }
}

/// Stable error category used by automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Caller supplied an invalid command, flag, or model.
    Validation,
    /// A requested path does not exist.
    NotFound,
    /// An external I/O dependency failed.
    Dependency,
    /// An unexpected failure.
    Internal,
}

/// Stable machine-readable error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Unknown command or invalid flags.
    #[serde(rename = "DOCLI.USAGE")]
    Usage,
    /// Model JSON is missing, empty, or does not match known field types.
    #[serde(rename = "DOCLI.INPUT_INVALID")]
    InputInvalid,
    /// `--input` path does not exist.
    #[serde(rename = "DOCLI.INPUT_NOT_FOUND")]
    InputNotFound,
    /// `show` asked for an artifact that is not on disk.
    #[serde(rename = "DOCLI.OUTPUT_NOT_FOUND")]
    OutputNotFound,
    /// Reading or writing a file failed.
    #[serde(rename = "DOCLI.IO")]
    Io,
    /// An unexpected failure.
    #[serde(rename = "DOCLI.INTERNAL")]
    Internal,
}

impl ErrorCode {
    /// Process exit code for this error code.
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Usage | Self::InputInvalid => EXIT_VALIDATION,
            Self::InputNotFound | Self::OutputNotFound => EXIT_NOT_FOUND,
            Self::Io => EXIT_DEPENDENCY,
            Self::Internal => EXIT_INTERNAL,
        }
    }
}

/// Failure object carried in [`Envelope::error`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error category.
    pub kind: ErrorKind,
    /// Stable `DOCLI.*` code.
    pub code: ErrorCode,
    /// One-sentence description of what happened.
    pub message: String,
    /// Code-specific object. Shape is fixed per code.
    pub details: serde_json::Value,
    /// One recovery sentence.
    pub suggested_action: String,
    /// Documentation URL, or JSON `null` when none applies.
    pub docs: Option<String>,
}

impl Display for ErrorBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code_name(), self.message)?;
        write!(f, "\n{}", self.suggested_action)?;
        if let Some(cause) = self
            .details
            .get("cause")
            .and_then(serde_json::Value::as_str)
        {
            write!(f, "\n{cause}")?;
        }
        if let Some(docs) = &self.docs {
            write!(f, "\n{docs}")?;
        }
        Ok(())
    }
}

impl ErrorBody {
    fn code_name(&self) -> &'static str {
        match self.code {
            ErrorCode::Usage => "DOCLI.USAGE",
            ErrorCode::InputInvalid => "DOCLI.INPUT_INVALID",
            ErrorCode::InputNotFound => "DOCLI.INPUT_NOT_FOUND",
            ErrorCode::OutputNotFound => "DOCLI.OUTPUT_NOT_FOUND",
            ErrorCode::Io => "DOCLI.IO",
            ErrorCode::Internal => "DOCLI.INTERNAL",
        }
    }

    /// Usage failure with empty `details`.
    pub fn usage(suggested_action: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Validation,
            code: ErrorCode::Usage,
            message: "unknown command or invalid flags".to_owned(),
            details: serde_json::json!({}),
            suggested_action: suggested_action.into(),
            docs: None,
        }
    }

    /// Invalid model JSON. `cause` becomes `details.cause` when present.
    pub fn input_invalid(input: &str, cause: Option<&str>) -> Self {
        let details = match cause {
            Some(cause) => serde_json::json!({ "cause": cause }),
            None => serde_json::json!({}),
        };
        let suggested_action = match cause {
            Some(cause) => format!("Fix the model JSON at {input}: {cause}"),
            None => format!("Fix the model JSON at {input}"),
        };
        Self {
            kind: ErrorKind::Validation,
            code: ErrorCode::InputInvalid,
            message: "model JSON is missing, empty, or does not match known field types".to_owned(),
            details,
            suggested_action,
            docs: None,
        }
    }

    /// Missing `--input` path.
    pub fn input_not_found(path: impl AsRef<std::path::Path>) -> Self {
        let path = path.as_ref().display().to_string();
        Self {
            kind: ErrorKind::NotFound,
            code: ErrorCode::InputNotFound,
            message: format!("input file not found: {path}"),
            details: serde_json::json!({ "path": path }),
            suggested_action: format!("Create or correct the missing input file {path}"),
            docs: None,
        }
    }

    /// `show` asked for artifacts that are not on disk.
    pub fn output_not_found(artifacts: Vec<serde_json::Value>, missing: &[String]) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            code: ErrorCode::OutputNotFound,
            message: "one or more requested artifacts were not found".to_owned(),
            details: serde_json::json!({ "artifacts": artifacts }),
            suggested_action: format!("Generate the missing artifacts: {}", missing.join(", ")),
            docs: None,
        }
    }

    /// Read or write failure. `outputs_written` is set only after a partial write.
    pub fn io(
        cause: impl Into<String>,
        failed_path: impl AsRef<std::path::Path>,
        outputs_written: Option<serde_json::Value>,
    ) -> Self {
        let cause = cause.into();
        let failed_path = failed_path.as_ref().display().to_string();
        let details = match outputs_written {
            Some(outputs_written) => {
                serde_json::json!({ "cause": cause, "outputs_written": outputs_written })
            }
            None => serde_json::json!({ "cause": cause }),
        };
        Self {
            kind: ErrorKind::Dependency,
            code: ErrorCode::Io,
            message: format!("failed to read or write {failed_path}"),
            details,
            suggested_action: format!("Retry after fixing access to {failed_path}: {cause}"),
            docs: None,
        }
    }

    /// Unexpected failure.
    pub fn internal(cause: impl Into<String>) -> Self {
        let cause = cause.into();
        Self {
            kind: ErrorKind::Internal,
            code: ErrorCode::Internal,
            message: "an unexpected failure occurred".to_owned(),
            details: serde_json::json!({ "cause": cause }),
            suggested_action: format!("Report this cause: {cause}"),
            docs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_suggested_action_cause_and_docs() {
        let error = ErrorBody {
            kind: ErrorKind::Validation,
            code: ErrorCode::InputInvalid,
            message: "bad model".to_owned(),
            details: serde_json::json!({ "cause": "expected string" }),
            suggested_action: "Fix the model JSON".to_owned(),
            docs: Some("https://example.test/errors".to_owned()),
        };
        let text = error.to_string();
        assert!(text.contains("DOCLI.INPUT_INVALID"));
        assert!(text.contains("bad model"));
        assert!(text.contains("Fix the model JSON"));
        assert!(text.contains("expected string"));
        assert!(text.contains("https://example.test/errors"));
    }

    #[test]
    fn display_omits_missing_cause_and_docs() {
        let error = ErrorBody::usage("Pass --html DIR and/or --markdown FILE");
        let text = error.to_string();
        assert!(text.contains("DOCLI.USAGE"));
        assert!(text.contains("unknown command or invalid flags"));
        assert!(text.contains("Pass --html DIR and/or --markdown FILE"));
        assert!(!text.contains("cause"));
    }
}
