use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ActionBase {
    file_path: String,
    target: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct MutateCallMutations {
    rename: Option<String>,
    #[serde(rename = "injectArgs")]
    inject_args: Option<Map<String, Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum DialecticAction {
    #[serde(rename = "MUTATE_CALL")]
    MutateCall {
        #[serde(flatten)]
        base: ActionBase,
        mutations: MutateCallMutations,
    },

    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone, Deserialize)]
struct Position {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct Range {
    start: Position,
    end: Position,
}

#[derive(Debug, Clone, Deserialize)]
struct MatchItem {
    range: Range,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input_data = if let Some(file) = args.next() {
        fs::read_to_string(&file).with_context(|| format!("failed to read input file '{file}'"))?
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("failed to read stdin")?;
        buf
    };

    if input_data.trim().is_empty() {
        return Ok(());
    }

    let action: DialecticAction = serde_json::from_str(&input_data).context("invalid JSON input")?;

    let result = apply(action);
    println!("{result}");
    Ok(())
}

fn apply(action: DialecticAction) -> String {
    match action {
        DialecticAction::Unsupported => {
            "NO_MUTATIONS: Unsupported action in Rust Patchiest runtime.".to_string()
        }
        DialecticAction::MutateCall { base, mutations } => {
            let file_path = base.file_path;
            let path = Path::new(&file_path);
            if !path.exists() {
                return format!("Error: File '{file_path}' not found.");
            }

            let target_name = match base.target.get("name") {
                Some(v) if !v.trim().is_empty() => v,
                _ => {
                    return format!(
                        "ROLLBACK: Critical failure in MUTATE_CALL: missing target.name for {file_path}."
                    )
                }
            };

            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    return format!(
                        "ROLLBACK: Critical failure in MUTATE_CALL: cannot read file: {e}."
                    )
                }
            };

            let matches = match scan_call_matches(path, target_name) {
                Ok(m) => m,
                Err(e) => {
                    return format!("ROLLBACK: Critical failure in MUTATE_CALL via ast-grep: {e}.");
                }
            };

            if matches.is_empty() {
                return format!("NO_MUTATIONS: MUTATE_CALL target not found in {file_path}.");
            }

            let mut edits: Vec<(usize, usize, String)> = Vec::new();
            for m in &matches {
                let Some(start) = line_col_to_offset(&source, m.range.start.line, m.range.start.column)
                else {
                    continue;
                };
                let Some(end) = line_col_to_offset(&source, m.range.end.line, m.range.end.column) else {
                    continue;
                };
                if start >= end || end > source.len() {
                    continue;
                }
                let snippet = &source[start..end];
                let replacement = transform_call_snippet(
                    snippet,
                    target_name,
                    mutations.rename.as_deref(),
                    mutations.inject_args.as_ref(),
                );

                if replacement != snippet {
                    edits.push((start, end, replacement));
                }
            }

            if edits.is_empty() {
                return format!("NO_MUTATIONS: MUTATE_CALL target not found in {file_path}.");
            }

            edits.sort_by(|a, b| b.0.cmp(&a.0));
            let mut updated = source;
            for (start, end, replacement) in edits {
                updated.replace_range(start..end, &replacement);
            }

            match fs::write(path, updated) {
                Ok(_) => format!("SUCCESS: Applied MUTATE_CALL to {file_path}."),
                Err(e) => {
                    format!("ROLLBACK: Critical failure in MUTATE_CALL: failed to write file: {e}.")
                }
            }
        }
    }
}

fn scan_call_matches(path: &Path, target_name: &str) -> Result<Vec<MatchItem>> {
    let pattern = format!("{target_name}($$$ARGS)");
    let output = Command::new("ast-grep")
        .args(["scan", "--pattern", &pattern, "--json"])
        .arg(path)
        .output()
        .context("failed to execute ast-grep. Is it installed in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!(
            "ast-grep scan failed: {}",
            if stderr.is_empty() { "unknown error" } else { &stderr }
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    if let Ok(items) = serde_json::from_str::<Vec<MatchItem>>(&stdout) {
        return Ok(items);
    }

    let mut items = Vec::new();
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        if let Ok(item) = serde_json::from_str::<MatchItem>(line) {
            items.push(item);
        }
    }
    Ok(items)
}

fn transform_call_snippet(
    snippet: &str,
    target_name: &str,
    rename: Option<&str>,
    inject_args: Option<&Map<String, Value>>,
) -> String {
    let mut out = snippet.to_string();

    if let Some(new_name) = rename {
        let trimmed_start = out.trim_start();
        let leading_ws_len = out.len() - trimmed_start.len();
        if trimmed_start.starts_with(target_name) {
            let start = leading_ws_len;
            let end = start + target_name.len();
            out.replace_range(start..end, new_name);
        }
    }

    if let Some(args) = inject_args {
        if !args.is_empty() {
            let insertion = args
                .iter()
                .map(|(k, v)| format!("{k}={}", render_value(v)))
                .collect::<Vec<_>>()
                .join(", ");

            if let Some(close_idx) = out.rfind(')') {
                let open_idx = out.find('(').unwrap_or(0);
                let has_existing = out[open_idx + 1..close_idx].trim().len() > 0;
                let prefix = if has_existing { ", " } else { "" };
                out.insert_str(close_idx, &format!("{prefix}{insertion}"));
            }
        }
    }

    out
}

fn render_value(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        _ => v.to_string(),
    }
}

fn line_col_to_offset(text: &str, line: usize, col: usize) -> Option<usize> {
    let mut offset = 0usize;
    let mut current_line = 0usize;

    for segment in text.split_inclusive('\n') {
        if current_line == line {
            return Some(offset + col);
        }
        offset += segment.len();
        current_line += 1;
    }

    if current_line == line {
        Some(offset + col)
    } else {
        None
    }
}
