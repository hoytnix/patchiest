mod models;
mod registry;
mod helpers;

use anyhow::Result;
use ast_grep_language::{Language, SupportLang, LanguageExt};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::models::*;
use crate::registry::*;
use crate::helpers::*;

pub fn execute_payload(payload: CompilerPayload, registry: &RecipeRegistry) -> CompilerMeta {
    let mut meta = CompilerMeta::new();
    let is_dry_run = payload.dry_run;
    let path_str = payload.file_path.clone().unwrap_or_else(|| "temp.rs".to_string());
    let path = Path::new(&path_str);
    
    let pristine_state = if let Some(src) = payload.source {
        src
    } else if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        "".to_string()
    };

    if pristine_state.is_empty() {
        meta.status = CompilerStatus::Fatal;
        meta.errors.push(format!("File '{}' not found or empty.", path_str));
        return meta;
    }

    let mut current_source = pristine_state.clone();
    let lang = SupportLang::from_path(path).unwrap_or(SupportLang::Rust);

    let mut has_read_action = false;

    for intent in payload.intent {
        let grep = lang.ast_grep(&current_source);
        let root = grep.root();
        let mut edits = Vec::new();

        match intent {
            DialecticAction::Unsupported => {
                meta.warnings.push("Unsupported action encountered.".to_string());
            }

            DialecticAction::MutateCall { target, mutations } => {
                let target_name = target.name.as_deref().unwrap_or("");
                if target_name.is_empty() {
                    meta.errors.push("MUTATE_CALL missing target.name".to_string());
                    continue;
                }

                let target_chars = target_name.to_string();
                for node in root.dfs() {
                    let k = node.kind().to_string().to_lowercase();
                    if k.contains("call") || k.contains("expression") {
                        if node.text().starts_with(&target_chars) && node.text().contains("(") {
                            let range = node.range();
                            let snippet = node.text().to_string();
                            let replaced = transform_call_snippet(
                                &snippet,
                                target_name,
                                mutations.rename.as_deref(),
                                mutations.inject_args.as_ref(),
                            );
                            if replaced != snippet {
                                edits.push((range.start, range.end, replaced));
                                meta.mutations += 1;
                            }
                        }
                    }
                }
            }

            DialecticAction::TranslateDialect { target, mutations } => {
                let func_name = target.function_name.as_deref().unwrap_or("");
                if let Some(_func_node) = resolve_callable_topology(root.clone(), func_name) {
                    if let Some(props) = mutations.generate_interface {
                        if current_source.contains(&mutations.enforce_explicit_type.clone().unwrap_or_default()) {
                        } else {
                            let interface_name = mutations.enforce_explicit_type.as_deref().unwrap_or("DynamicInterface");
                            let mut iface_str = format!("\ninterface {} {{\n", interface_name);
                            for prop in props {
                                let parts: Vec<&str> = prop.split(':').collect();
                                if parts.len() == 2 {
                                    iface_str.push_str(&format!("  {}: {};\n", parts[0].trim(), parts[1].trim()));
                                } else {
                                    iface_str.push_str(&format!("  {}: any;\n", prop.trim()));
                                }
                            }
                            iface_str.push_str("}\n");
                            
                            edits.push((0, 0, iface_str));
                            meta.mutations += 1;
                        }
                    }

                    if let Some(_explicit_type) = mutations.enforce_explicit_type {
                        meta.warnings.push("Type injection (enforce_explicit_type) uses naive matching. AST surgery on parameters is complex.".to_string());
                    }
                } else {
                    meta.warnings.push(format!("Target '{}' missing. Epistemological translation aborted.", func_name));
                }
            }

            DialecticAction::RestructureTopology { target, mutations } => {
                let func_name = target.function_name.as_deref().unwrap_or("");
                let dep_name = target.hardcoded_dependency.as_deref().unwrap_or("");
                let param_name = mutations.extract_to_parameter.as_deref().unwrap_or("");

                if let Some(func_node) = resolve_callable_topology(root.clone(), func_name) {
                    for node in func_node.dfs() {
                        let k = node.kind().to_string().to_lowercase();
                        if (k.contains("new") || k.contains("instantiation") || k.contains("call") || k.contains("identifier")) && node.text().contains(dep_name) {
                            let range = node.range();
                            edits.push((range.start, range.end, param_name.to_string()));
                            meta.mutations += 1;
                        }
                    }
                }
            }

            DialecticAction::ManageImport { target, mutations } => {
                let named_import = target.named_import.as_deref().unwrap_or("");
                
                if let Some(replace_w) = &mutations.replace_with {
                    for node in root.dfs() {
                        let k = node.kind().to_string().to_lowercase();
                        if k.contains("import") || k.contains("declaration") || k.contains("use") {
                            let text = node.text();
                            if text.contains(named_import) {
                                let replaced = text.replace(named_import, replace_w);
                                let range = node.range();
                                edits.push((range.start, range.end, replaced));
                                meta.mutations += 1;
                            }
                        }
                    }
                }

                if let Some(ensure_imp) = &mutations.ensure_import {
                    if let Some(mod_spec) = &mutations.module_specifier {
                        let mut found_spec = false;
                        for node in root.dfs() {
                            let k = node.kind().to_string().to_lowercase();
                            if (k.contains("import") || k.contains("declaration")) && node.text().contains(mod_spec) {
                                found_spec = true;
                                if !node.text().contains(ensure_imp) {
                                    if let Some(bracket_idx) = node.text().rfind('}') {
                                        let mut new_text = node.text().to_string();
                                        new_text.insert_str(bracket_idx, &format!(", {}", ensure_imp));
                                        edits.push((node.range().start, node.range().end, new_text));
                                        meta.mutations += 1;
                                    }
                                }
                                break;
                            }
                        }
                        if !found_spec {
                            let is_type = if mutations.is_type_only.unwrap_or(false) { "type " } else { "" };
                            let imp_str = format!("import {} {{ {} }} from '{}';\n", is_type, ensure_imp, mod_spec);
                            edits.push((0, 0, imp_str));
                            meta.mutations += 1;
                        }
                    }
                }
            }

            DialecticAction::ReadTopology { target, mutations } => {
                has_read_action = true;
                let target_node_name = target.node_name.as_deref().unwrap_or("");
                
                if let Some(cmd) = mutations.extract {
                    if target_node_name == "SYSTEM" && cmd == "AVAILABLE_RECIPES" {
                        let recipes = registry.list().into_iter().map(|r| {
                            serde_json::json!({
                                "name": r.name,
                                "description": r.description
                            })
                        }).collect::<Vec<_>>();

                        meta.read_buffer.as_mut().unwrap().insert(
                            "AVAILABLE_RECIPES".to_string(), 
                            Value::Array(recipes)
                        );
                        continue;
                    }

                    if let Some(node) = resolve_callable_topology(root.clone(), target_node_name) {
                        match cmd.as_str() {
                            "FULL_NODE" => {
                                meta.read_buffer.as_mut().unwrap().insert(
                                    target_node_name.to_string(),
                                    Value::String(node.text().to_string())
                                );
                            }
                            "SIGNATURE" => {
                                let mut sig = Map::new();
                                let signature_line = node.text().lines().next().unwrap_or("").to_string();
                                sig.insert("signature".to_string(), Value::String(signature_line));
                                meta.read_buffer.as_mut().unwrap().insert(
                                    target_node_name.to_string(),
                                    Value::Object(sig)
                                );
                            }
                            "DEPENDENCIES" => {
                                let mut deps = Vec::new();
                                for child in node.dfs() {
                                    if child.is_leaf() && child.kind().to_string().contains("identifier") {
                                        let t = child.text().to_string();
                                        if !deps.contains(&t) {
                                            deps.push(t);
                                        }
                                    }
                                }
                                meta.read_buffer.as_mut().unwrap().insert(
                                    target_node_name.to_string(),
                                    Value::Array(deps.into_iter().map(Value::String).collect())
                                );
                            }
                            _ => {
                                meta.warnings.push(format!("Unknown extraction command: {}", cmd));
                            }
                        }
                    } else {
                        meta.warnings.push(format!("Diagnostic failed: Node '{}' not found in spatial topology.", target_node_name));
                    }
                }
            }
            
            DialecticAction::ExecuteRecipe { target, .. } => {
                let recipe_name = target.recipe_name.as_deref().unwrap_or("");
                if let Some(_recipe) = registry.get(recipe_name) {
                    meta.warnings.push(format!("Boutique recipe '{}' invoked but no-op in this engine version.", recipe_name));
                } else {
                    meta.warnings.push(format!("Boutique recipe '{}' not found in registry.", recipe_name));
                }
            }
        }

        if !edits.is_empty() {
            apply_edits(&mut current_source, edits);
        }
    }

    if !meta.errors.is_empty() {
        meta.status = CompilerStatus::RollbackInitiated;
        meta.mutations = 0;
    } else if is_dry_run {
        meta.status = CompilerStatus::DryRunComplete;
    } else if meta.mutations > 0 && current_source != pristine_state {
        if path_str == "temp.rs" || path_str.is_empty() {
             meta.status = CompilerStatus::Success;
        } else if let Err(e) = fs::write(path, &current_source) {
            meta.status = CompilerStatus::RollbackInitiated;
            meta.errors.push(format!("Failed to write to file: {}", e));
            meta.mutations = 0;
        } else {
            meta.status = CompilerStatus::Success;
        }
    } else {
        meta.status = if has_read_action { CompilerStatus::ReadComplete } else { CompilerStatus::NoMutations };
    }

    meta
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut dry_run_cli = false;
    let mut input_file = None;

    while let Some(arg) = args.next() {
        if arg == "--dry-run" {
            dry_run_cli = true;
        } else if input_file.is_none() {
            input_file = Some(arg);
        }
    }

    let input_data = if let Some(file) = input_file {
        fs::read_to_string(&file).unwrap_or_else(|_| {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        })
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();
        buf
    };

    let registry = RecipeRegistry::new();

    if input_data.trim().is_empty() {
        let mut meta = CompilerMeta::new();
        meta.status = CompilerStatus::Fatal;
        meta.errors.push("Empty input provided.".to_string());
        println!("{}", serde_json::to_string(&meta).unwrap());
        return;
    }

    let payload_result: Result<CompilerPayload, _> = serde_json::from_str(&input_data);
    let mut payload = match payload_result {
        Ok(p) => p,
        Err(_) => {
            if let Ok(action) = serde_json::from_str::<DialecticAction>(&input_data) {
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
                println!("{}", serde_json::to_string(&meta).unwrap());
                return;
            }
        }
    };

    if dry_run_cli {
        payload.dry_run = true;
    }

    let meta = execute_payload(payload, &registry);
    println!("{}", serde_json::to_string(&meta).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
