    use super::*;

    #[test]
    fn test_execute_payload_empty_source_fatal() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("nonexistent.rs".to_string()),
            source: None,
            intent: vec![],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::Fatal);
        assert!(meta.errors[0].contains("not found or empty"));
    }

    #[test]
    fn test_execute_payload_mutate_call() {
        let registry = RecipeRegistry::new();
        let source = "fn main() { foo(); }";
        let mutations = ActionMutations {
            rename: Some("bar".to_string()),
            inject_args: None,
            target_arg_index: None,
            enforce_explicit_type: None,
            generate_interface: None,
            target_param_index: None,
            extract_to_parameter: None,
            replace_with: None,
            module_specifier: None,
            ensure_import: None,
            is_type_only: None,
            extract: None,
            options: None,
            target_files: None,
        };
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::MutateCall {
                target: ActionTarget {
                    name: Some("foo".to_string()),
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations,
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert!(meta.mutations >= 1);
    }

    #[test]
    fn test_execute_payload_translate_dialect() {
        let registry = RecipeRegistry::new();
        let source = "function foo(a: any) { return a; }";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::TranslateDialect {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: Some("IFoo".to_string()),
                    generate_interface: Some(vec!["a: string".to_string()]),
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert_eq!(meta.mutations, 1);
    }

    #[test]
    fn test_execute_payload_restructure_topology() {
        let registry = RecipeRegistry::new();
        let source = "fn foo() { bar; }";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::RestructureTopology {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: Some("bar".to_string()),
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: Some("baz".to_string()),
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert_eq!(meta.mutations, 1);
    }

    #[test]
    fn test_execute_payload_manage_import() {
        let registry = RecipeRegistry::new();
        let source = "use mod::A;";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ManageImport {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: Some("A".to_string()),
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: Some("B".to_string()),
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert_eq!(meta.mutations, 1);
    }

    #[test]
    fn test_execute_payload_read_topology() {
        let registry = RecipeRegistry::new();
        let source = "fn foo() {}";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ReadTopology {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: Some("foo".to_string()),
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: Some("FULL_NODE".to_string()),
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: false, // ensures status is ReadComplete
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::ReadComplete);
        assert!(meta.read_buffer.unwrap().contains_key("foo"));
    }

    #[test]
    fn test_execute_payload_execute_recipe() {
        let registry = RecipeRegistry::new();
        let source = "fn main() {}";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ExecuteRecipe {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: Some("cjs-to-esm".to_string()),
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert!(meta.warnings[0].contains("no-op"));
    }

    #[test]
    fn test_execute_payload_unsupported() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some("fn main() {}".to_string()),
            intent: vec![DialecticAction::Unsupported],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert!(meta.warnings[0].contains("Unsupported action encountered"));
    }

    #[test]
    fn test_execute_payload_mutate_call_missing_name() {
        let registry = RecipeRegistry::new();
        let source = "fn main() { foo(); }";
        let mutations = ActionMutations {
            rename: Some("bar".to_string()),
            inject_args: None,
            target_arg_index: None,
            enforce_explicit_type: None,
            generate_interface: None,
            target_param_index: None,
            extract_to_parameter: None,
            replace_with: None,
            module_specifier: None,
            ensure_import: None,
            is_type_only: None,
            extract: None,
            options: None,
            target_files: None,
        };
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::MutateCall {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations,
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::RollbackInitiated);
        assert!(meta.errors[0].contains("missing target.name"));
    }

    #[test]
    fn test_execute_payload_translate_dialect_missing_target() {
        let registry = RecipeRegistry::new();
        let source = "function bar() { }";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::TranslateDialect {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert!(meta.warnings[0].contains("Target 'foo' missing"));
    }

    #[test]
    fn test_execute_payload_translate_dialect_interface_exists() {
        let registry = RecipeRegistry::new();
        let source = "interface IFoo {}\nfunction foo(a: any) { return a; }";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::TranslateDialect {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: Some("IFoo".to_string()),
                    generate_interface: Some(vec!["b".to_string(), "a: string".to_string()]),
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
    }
    
    #[test]
    fn test_execute_payload_manage_import_ensure_missing_inside_existing() {
        let registry = RecipeRegistry::new();
        let source = "import { A } from 'mod';";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ManageImport {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: Some("mod".to_string()),
                    ensure_import: Some("B".to_string()),
                    is_type_only: Some(false),
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.mutations, 1);
    }
    
    #[test]
    fn test_execute_payload_manage_import_type_only_new() {
        let registry = RecipeRegistry::new();
        let source = "fn main() { }";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ManageImport {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: Some("mod".to_string()),
                    ensure_import: Some("B".to_string()),
                    is_type_only: Some(true),
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.mutations, 1);
    }
    
    #[test]
    fn test_execute_payload_read_topology_system() {
        let registry = RecipeRegistry::new();
        let source = "fn main() {}";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::ReadTopology {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: Some("SYSTEM".to_string()),
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: Some("AVAILABLE_RECIPES".to_string()),
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert!(meta.read_buffer.unwrap().contains_key("AVAILABLE_RECIPES"));
    }

    #[test]
    fn test_execute_payload_read_topology_signature_deps() {
        let registry = RecipeRegistry::new();
        let source = "fn foo() { bar(); }";
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some(source.to_string()),
            intent: vec![
                DialecticAction::ReadTopology {
                    target: ActionTarget {
                        name: None,
                        function_name: None,
                        hardcoded_dependency: None,
                        named_import: None,
                        node_name: Some("foo".to_string()),
                        recipe_name: None,
                    },
                    mutations: ActionMutations {
                        rename: None,
                        inject_args: None,
                        target_arg_index: None,
                        enforce_explicit_type: None,
                        generate_interface: None,
                        target_param_index: None,
                        extract_to_parameter: None,
                        replace_with: None,
                        module_specifier: None,
                        ensure_import: None,
                        is_type_only: None,
                        extract: Some("SIGNATURE".to_string()),
                        options: None,
                        target_files: None,
                    },
                },
                DialecticAction::ReadTopology {
                    target: ActionTarget {
                        name: None,
                        function_name: None,
                        hardcoded_dependency: None,
                        named_import: None,
                        node_name: Some("foo".to_string()),
                        recipe_name: None,
                    },
                    mutations: ActionMutations {
                        rename: None,
                        inject_args: None,
                        target_arg_index: None,
                        enforce_explicit_type: None,
                        generate_interface: None,
                        target_param_index: None,
                        extract_to_parameter: None,
                        replace_with: None,
                        module_specifier: None,
                        ensure_import: None,
                        is_type_only: None,
                        extract: Some("DEPENDENCIES".to_string()),
                        options: None,
                        target_files: None,
                    },
                },
                DialecticAction::ReadTopology {
                    target: ActionTarget {
                        name: None,
                        function_name: None,
                        hardcoded_dependency: None,
                        named_import: None,
                        node_name: Some("foo".to_string()),
                        recipe_name: None,
                    },
                    mutations: ActionMutations {
                        rename: None,
                        inject_args: None,
                        target_arg_index: None,
                        enforce_explicit_type: None,
                        generate_interface: None,
                        target_param_index: None,
                        extract_to_parameter: None,
                        replace_with: None,
                        module_specifier: None,
                        ensure_import: None,
                        is_type_only: None,
                        extract: Some("INVALID_CMD".to_string()),
                        options: None,
                        target_files: None,
                    },
                },
                DialecticAction::ReadTopology {
                    target: ActionTarget {
                        name: None,
                        function_name: None,
                        hardcoded_dependency: None,
                        named_import: None,
                        node_name: Some("unknown".to_string()),
                        recipe_name: None,
                    },
                    mutations: ActionMutations {
                        rename: None,
                        inject_args: None,
                        target_arg_index: None,
                        enforce_explicit_type: None,
                        generate_interface: None,
                        target_param_index: None,
                        extract_to_parameter: None,
                        replace_with: None,
                        module_specifier: None,
                        ensure_import: None,
                        is_type_only: None,
                        extract: Some("SIGNATURE".to_string()),
                        options: None,
                        target_files: None,
                    },
                }
            ],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::ReadComplete);
        let rb = meta.read_buffer.unwrap();
        assert!(rb.contains_key("foo"));
        assert!(meta.warnings.iter().any(|w| w.contains("Unknown extraction command: INVALID_CMD")));
        assert!(meta.warnings.iter().any(|w| w.contains("not found in spatial topology")));
    }

    #[test]
    fn test_execute_payload_execute_recipe_not_found() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("test.rs".to_string()),
            source: Some("fn main() {}".to_string()),
            intent: vec![DialecticAction::ExecuteRecipe {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: Some("nonexistent_recipe".to_string()),
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::DryRunComplete);
        assert!(meta.warnings[0].contains("not found in registry"));
    }

    #[test]
    fn test_execute_payload_success_write() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("not_temp.rs".to_string()),
            source: Some("fn main() { foo(); }".to_string()),
            intent: vec![DialecticAction::MutateCall {
                target: ActionTarget {
                    name: Some("foo".to_string()),
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: Some("bar".to_string()),
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::Success);
        let _ = std::fs::remove_file("not_temp.rs");
    }

    #[test]
    fn test_execute_payload_write_fail() {
        let registry = RecipeRegistry::new();
        
        let path = if cfg!(windows) { "Z:\\invalid_path_for_write.rs" } else { "/invalid_path_for_write.rs" };
        let payload = CompilerPayload {
            file_path: Some(path.to_string()),
            source: Some("fn main() { foo(); }".to_string()),
            intent: vec![DialecticAction::MutateCall {
                target: ActionTarget {
                    name: Some("foo".to_string()),
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: Some("bar".to_string()),
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::RollbackInitiated);
        assert!(meta.errors[0].contains("Failed to write"));
    }
    #[test]
    fn test_execute_payload_no_intent_no_mutations() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: None,
            source: Some("fn main() {}".to_string()),
            intent: vec![],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::NoMutations);
    }

    #[test]
    fn test_execute_payload_empty_source_explicit_fatal() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("empty_file.rs".to_string()),
            source: Some("".to_string()),
            intent: vec![],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::Fatal);
    }
    
    #[test]
    fn test_execute_payload_path_exists_but_no_source() {
        let registry = RecipeRegistry::new();
        let path = "exists_test.rs";
        fs::write(path, "fn main() {}").unwrap();
        let payload = CompilerPayload {
            file_path: Some(path.to_string()),
            source: None,
            intent: vec![],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::NoMutations);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_execute_payload_translate_dialect_type_injection_warning() {
        let registry = RecipeRegistry::new();
        let source = "function foo() {}";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::TranslateDialect {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: Some("IFoo".to_string()),
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert!(meta.warnings.iter().any(|w| w.contains("Type injection")));
    }

    #[test]
    fn test_execute_payload_manage_import_ensure_no_mod_spec() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some("".to_string()),
            intent: vec![DialecticAction::ManageImport {
                target: ActionTarget {
                    name: None,
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: Some("A".to_string()),
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.mutations, 0);
    }

    #[test]
    fn test_execute_payload_translate_dialect_any_type() {
        let registry = RecipeRegistry::new();
        let source = "function foo(a: any) { return a; }";
        let payload = CompilerPayload {
            file_path: Some("test.ts".to_string()),
            source: Some(source.to_string()),
            intent: vec![DialecticAction::TranslateDialect {
                target: ActionTarget {
                    name: None,
                    function_name: Some("foo".to_string()),
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: None,
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: Some("IFoo".to_string()),
                    generate_interface: Some(vec!["a".to_string()]), // No colon
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: true,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.mutations, 1);
    }

    #[test]
    fn test_execute_payload_temp_rs_success() {
        let registry = RecipeRegistry::new();
        let payload = CompilerPayload {
            file_path: None,
            source: Some("fn main() { foo(); }".to_string()),
            intent: vec![DialecticAction::MutateCall {
                target: ActionTarget {
                    name: Some("foo".to_string()),
                    function_name: None,
                    hardcoded_dependency: None,
                    named_import: None,
                    node_name: None,
                    recipe_name: None,
                },
                mutations: ActionMutations {
                    rename: Some("bar".to_string()),
                    inject_args: None,
                    target_arg_index: None,
                    enforce_explicit_type: None,
                    generate_interface: None,
                    target_param_index: None,
                    extract_to_parameter: None,
                    replace_with: None,
                    module_specifier: None,
                    ensure_import: None,
                    is_type_only: None,
                    extract: None,
                    options: None,
                    target_files: None,
                },
            }],
            dry_run: false,
        };
        let meta = execute_payload(payload, &registry);
        assert_eq!(meta.status, CompilerStatus::Success);
    }
