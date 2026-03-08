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
