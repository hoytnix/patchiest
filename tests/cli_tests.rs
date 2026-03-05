use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;

const BIN_PATH: &str = env!("CARGO_BIN_EXE_patchiest");

#[test]
fn test_cli_help() {
    // Just verify the binary runs
    let _ = Command::new(BIN_PATH)
        .arg("--help") // Not implemented but should still exit or wait
        .stdin(Stdio::null())
        .output();
}

#[test]
fn test_cli_dry_run_stdin() {
    let mut child = Command::new(BIN_PATH)
        .arg("--dry-run")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let payload = serde_json::json!({
        "file_path": "test.rs",
        "source": "fn main() { foo(); }",
        "intent": [
            {
                "action": "MUTATE_CALL",
                "target": { "name": "foo" },
                "mutations": { "rename": "bar" }
            }
        ],
        "dryRun": true
    });

    stdin.write_all(payload.to_string().as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("\"status\":\"DRY_RUN_COMPLETE\""));
}

#[test]
fn test_cli_read_from_file() {
    let temp_file = "temp_test_payload.json";
    let payload = serde_json::json!({
        "file_path": "test.rs",
        "source": "fn main() { foo(); }",
        "intent": [],
        "dryRun": true
    });
    fs::write(temp_file, payload.to_string()).unwrap();
    
    let output = Command::new(BIN_PATH)
        .arg(temp_file)
        .output()
        .expect("Failed to execute command");
    
    let _ = fs::remove_file(temp_file);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"status\":\"DRY_RUN_COMPLETE\""));
}

#[test]
fn test_cli_invalid_json() {
    let mut child = Command::new(BIN_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(b"not json").expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("\"status\":\"FATAL\""));
    assert!(stdout.contains("Invalid JSON payload sequence"));
}

#[test]
fn test_cli_empty_input() {
    // This is tricky because stdin might hang if not closed
    let mut child = Command::new(BIN_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("\"status\":\"FATAL\""));
    assert!(stdout.contains("Empty input provided"));
}

#[test]
fn test_cli_direct_action_payload() {
    let mut child = Command::new(BIN_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    // Just a DialecticAction without the wrap
    let action = serde_json::json!({
        "action": "MUTATE_CALL",
        "target": { "name": "foo" },
        "mutations": { "rename": "bar" }
    });

    stdin.write_all(action.to_string().as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Status should be Fatal because source is empty
    assert!(stdout.contains("\"status\":\"FATAL\""));
}

#[test]
fn test_cli_direct_action_payload_with_file_path() {
    let mut child = Command::new(BIN_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let action_with_file_path = serde_json::json!({
        "file_path": "test.rs",
        "action": "MUTATE_CALL",
        "target": { "name": "foo" },
        "mutations": { "rename": "bar" }
    });

    stdin.write_all(action_with_file_path.to_string().as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("\"status\":\"FATAL\""));
}

#[test]
fn test_cli_dry_run_args() {
    let mut child = Command::new(BIN_PATH)
        .arg("--dry-run")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let payload = serde_json::json!({
        "file_path": "test.rs",
        "source": "fn main() { foo(); }",
        "intent": [],
        "dryRun": false
    });

    stdin.write_all(payload.to_string().as_bytes()).expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // It should dry run anyway
    assert!(stdout.contains("\"status\":\"DRY_RUN_COMPLETE\""));
}
