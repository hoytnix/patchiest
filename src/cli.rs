use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::io::Read;

use crate::models::*;
use crate::registry::*;
use crate::engine::*;

pub fn run_app<R: Read>(args: Vec<String>, mut reader: R, registry: &RecipeRegistry) -> String {
    let mut dry_run_cli = false;
    let mut input_file = None;

    let mut args_iter = args.into_iter().skip(1);
    while let Some(arg) = args_iter.next() {
        if arg == "--dry-run" {
            dry_run_cli = true;
        } else if input_file.is_none() {
            input_file = Some(arg);
        }
    }

    let input_data = if let Some(file) = input_file {
        fs::read_to_string(&file).unwrap_or_else(|_| {
            let mut buf = String::new();
            let _ = reader.read_to_string(&mut buf);
            buf
        })
    } else {
        let mut buf = String::new();
        let _ = reader.read_to_string(&mut buf);
        buf
    };

    if input_data.trim().is_empty() {
        let mut meta = CompilerMeta::new();
        meta.status = CompilerStatus::Fatal;
        meta.errors.push("Empty input provided.".to_string());
        return serde_json::to_string(&meta).unwrap();
    }

    let payload_result: Result<CompilerPayload, _> = serde_json::from_str(&input_data);
    let mut payload = if let Ok(p) = payload_result {
        p
    } else if let Ok(action) = serde_json::from_str::<DialecticAction>(&input_data) {
        let v = serde_json::from_str::<Value>(&input_data).unwrap();
        let file_path = v.get("file_path").and_then(|f| f.as_str()).map(|s| s.to_string());
        CompilerPayload {
            file_path,
            source: None,
            intent: vec![action],
            dry_run: dry_run_cli,
        }
    } else {
        let mut meta = CompilerMeta::new();
        meta.status = CompilerStatus::Fatal;
        meta.errors.push("Invalid JSON payload sequence.".to_string());
        return serde_json::to_string(&meta).unwrap();
    };

    if dry_run_cli {
        payload.dry_run = true;
    }

    let meta = execute_payload(payload, registry);
    serde_json::to_string(&meta).unwrap()
}

#[cfg(test)]
mod tests;
