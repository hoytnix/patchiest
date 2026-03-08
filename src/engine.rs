use ast_grep_language::{Language, SupportLang, LanguageExt};
use serde_json::{Map, Value};
use std::fs;
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

#[cfg(test)]
mod tests;
