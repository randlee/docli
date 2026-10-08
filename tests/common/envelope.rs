//! Assert version `"1"` envelopes match `REQ-DOCLI-CLI-002` / `REQ-DOCLI-CLI-003`.

use serde_json::Value;

pub fn parse_envelope(stdout: &[u8]) -> Value {
    let text = String::from_utf8_lossy(stdout);
    let trimmed = text.trim();
    assert!(
        !trimmed.is_empty(),
        "stdout must contain the JSON envelope, was empty"
    );
    assert!(
        !trimmed.contains("\n\n"),
        "stdout must be a single JSON envelope, got multiple blocks: {trimmed}"
    );
    serde_json::from_str(trimmed).unwrap_or_else(|err| {
        panic!("stdout is not valid JSON envelope ({err}): {trimmed}")
    })
}

pub fn assert_success(envelope: &Value) {
    assert_eq!(envelope["version"], "1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["error"], Value::Null);
    assert!(envelope["data"].is_object(), "data must be an object");
}

pub fn assert_failure(envelope: &Value, exit_code: i32, kind: &str, code: &str) -> Value {
    assert_eq!(envelope["version"], "1");
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["data"], Value::Null);
    let error = envelope["error"].clone();
    assert_eq!(error["kind"], kind);
    assert_eq!(error["code"], code);
    let message = error["message"]
        .as_str()
        .unwrap_or_else(|| panic!("message must be a non-empty string, got {}", error["message"]));
    assert!(
        !message.is_empty(),
        "message must describe what failed for agent callers"
    );
    assert!(
        error["details"].is_object(),
        "details must be an object, got {}",
        error["details"]
    );
    let suggested_action = error["suggested_action"]
        .as_str()
        .expect("suggested_action must be a string");
    assert!(
        !suggested_action.is_empty(),
        "suggested_action must name a recovery step (creating-ai-clis error contract)"
    );
    let docs = &error["docs"];
    assert!(
        docs.is_string() || docs.is_null(),
        "docs must be string or null, got {docs}"
    );
    let _ = exit_code;
    error
}

pub fn assert_human_actionable(stderr: &[u8], code: &str, needles: &[&str]) {
    let text = String::from_utf8_lossy(stderr);
    assert!(
        text.contains(code),
        "human stderr must include stable code {code} for agents; stderr={text}"
    );
    for needle in needles {
        assert!(
            text.contains(needle),
            "human stderr must mention {needle:?} in suggested_action or message; stderr={text}"
        );
    }
}

pub fn assert_json_mode_stdout_only_envelope(stdout: &[u8], stderr: &[u8]) {
    let _ = parse_envelope(stdout);
    assert!(
        stderr.is_empty() || String::from_utf8_lossy(stderr).trim().is_empty(),
        "--json failures must not rely on stderr for the contract; stderr={}",
        String::from_utf8_lossy(stderr)
    );
}
