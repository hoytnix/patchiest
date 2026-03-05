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
mod tests {
    use super::*;
    use serde_json::json;
    use ast_grep_language::LanguageExt;

    #[test]
    fn test_apply_edits() {
        let mut source = "Hello World".to_string();
        let edits = vec![
            (0, 5, "Hi".to_string()),
            (6, 11, "Universe".to_string()),
        ];
        apply_edits(&mut source, edits);
        assert_eq!(source, "Hi Universe");
    }

    #[test]
    fn test_render_value() {
        assert_eq!(render_value(&json!("foo")), "\"foo\"");
        assert_eq!(render_value(&json!(123)), "123");
        assert_eq!(render_value(&json!(true)), "true");
    }

    #[test]
    fn test_transform_call_snippet_rename() {
        let snippet = "foo()";
        let out = transform_call_snippet(snippet, "foo", Some("bar"), None);
        assert_eq!(out, "bar()");
    }

    #[test]
    fn test_transform_call_snippet_inject() {
        let snippet = "foo()";
        let mut args = Map::new();
        args.insert("a".to_string(), json!(1));
        let out = transform_call_snippet(snippet, "foo", None, Some(&args));
        assert_eq!(out, "foo(a=1)");
    }

    #[test]
    fn test_resolve_callable_topology() {
        let source = "fn foo() { bar(); }";
        let lang = SupportLang::Rust;
        let grep = lang.ast_grep(source);
        let root = grep.root();
        let node = resolve_callable_topology(root, "bar");
        assert!(node.is_some());
        assert!(node.unwrap().text().contains("fn foo"));
    }

    #[test]
    fn test_resolve_callable_topology_none() {
        let source = "fn foo() { bar(); }";
        let lang = SupportLang::Rust;
        let grep = lang.ast_grep(source);
        let root = grep.root();
        let node = resolve_callable_topology(root, "baz");
        assert!(node.is_none());
    }

    #[test]
    fn test_apply_edits_overlapping() {
        let mut source = "Hello World".to_string();
        let edits = vec![
            (0, 5, "Hi".to_string()),
            (0, 2, "Hey".to_string()),
        ];
        apply_edits(&mut source, edits);
        // Sorting by start index descending: (0, 5) then (0, 2)
        // Actually the code sorts by b.0.cmp(&a.0) which is correct for non-overlapping.
        // For overlapping it's undefined but let's see what happens.
        // It's mostly to ensure it doesn't crash and handles the range correctly.
    }

    #[test]
    fn test_apply_edits_out_of_bounds() {
        let mut source = "Hello".to_string();
        let edits = vec![(0, 10, "World".to_string())];
        apply_edits(&mut source, edits.clone());
        assert_eq!(source, "Hello"); // Should be no-op
        
        let mut source = "Hello".to_string();
        let edits = vec![(10, 5, "World".to_string())];
        apply_edits(&mut source, edits);
        assert_eq!(source, "Hello"); // Should be no-op
    }

    #[test]
    fn test_transform_call_snippet_no_parentheses() {
        let snippet = "foo";
        let out = transform_call_snippet(snippet, "foo", Some("bar"), None);
        assert_eq!(out, "bar");
        
        let mut args = Map::new();
        args.insert("a".to_string(), json!(1));
        let out = transform_call_snippet(snippet, "foo", None, Some(&args));
        assert_eq!(out, "foo"); // No parentheses, so no injection
    }

    #[test]
    fn test_transform_call_snippet_rename_mismatch() {
        let snippet = "foo()";
        let out = transform_call_snippet(snippet, "bar", Some("baz"), None);
        assert_eq!(out, "foo()");
    }

    #[test]
    fn test_transform_call_snippet_empty_args() {
        let snippet = "foo()";
        let args = Map::new();
        let out = transform_call_snippet(snippet, "foo", None, Some(&args));
        assert_eq!(out, "foo()");
    }

    #[test]
    fn test_resolve_callable_topology_no_parent_match() {
        let source = "baz;"; // Expression statement, no declaration
        let lang = SupportLang::Rust;
        let grep = lang.ast_grep(source);
        let root = grep.root();
        let node = resolve_callable_topology(root, "baz");
        assert!(node.is_none());
    }
}
