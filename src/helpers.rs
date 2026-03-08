use ast_grep_core::Node;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::SupportLang;
use serde_json::{Map, Value};

pub fn resolve_callable_topology<'a>(root: Node<'a, StrDoc<SupportLang>>, target: &str) -> Option<Node<'a, StrDoc<SupportLang>>> {
    for node in root.dfs() {
        if node.is_leaf() && node.text() == target {
            let mut curr = Some(node.clone());
            while let Some(parent) = curr.as_ref().unwrap().parent() {
                let kind = parent.kind();
                let kind = kind.to_string().to_lowercase();
                if kind.contains("function") || kind.contains("method") || kind.contains("class") || kind.contains("declaration") || kind.contains("definition") {
                    return Some(parent);
                }
                curr = Some(parent);
            }
        }
    }
    None
}

pub fn apply_edits(source: &mut String, mut edits: Vec<(usize, usize, String)>) {
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, text) in edits {
        if start <= end && end <= source.len() {
            source.replace_range(start..end, &text);
        }
    }
}

pub fn render_value(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
        _ => v.to_string(),
    }
}

pub fn transform_call_snippet(
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
                .map(|(k, v)| format!("{}={}", k, render_value(v)))
                .collect::<Vec<_>>()
                .join(", ");

            if let Some(close_idx) = out.rfind(')') {
                let open_idx = out.find('(').unwrap_or(0);
                let has_existing = out[open_idx + 1..close_idx].trim().len() > 0;
                let prefix = if has_existing { ", " } else { "" };
                out.insert_str(close_idx, &format!("{}{}", prefix, insertion));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests;
