use anyhow::Result;
use ast_grep_core::Node;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::{Language, SupportLang};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

// phase 4 types...
#[derive(Debug, Clone)]
pub struct BoutiqueRecipe {
    pub name: String,
    pub description: String,
    // For now, recipes will be ast-grep patterns or rules.
    // In the future, these can be complex Rust functions.
}

pub struct RecipeRegistry {
    recipes: HashMap<String, BoutiqueRecipe>,
}

impl RecipeRegistry {
    pub fn new() -> Self {
        let mut recipes = HashMap::new();
        recipes.insert("cjs-to-esm".to_string(), BoutiqueRecipe {
            name: "cjs-to-esm".to_string(),
            description: "Structurally converts CommonJS require() statements to ES6 top-level imports.".to_string(),
        });
        recipes.insert("react-class-to-hooks".to_string(), BoutiqueRecipe {
            name: "react-class-to-hooks".to_string(),
            description: "Refactors class components into functional components utilizing useState/useEffect.".to_string(),
        });
        Self { recipes }
    }

    pub fn get(&self, name: &str) -> Option<&BoutiqueRecipe> {
        self.recipes.get(name)
    }

    pub fn list(&self) -> Vec<BoutiqueRecipe> {
        self.recipes.values().cloned().collect()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompilerStatus {
    Success,
    PartialFail,
    Fatal,
    RollbackInitiated,
    DryRunComplete,
    NoMutations,
    ReadComplete,
}

#[derive(Debug, Serialize)]
pub struct CompilerMeta {
    pub status: CompilerStatus,
    pub mutations: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    #[serde(rename = "readBuffer", skip_serializing_if = "Option::is_none")]
    pub read_buffer: Option<Map<String, Value>>,
}

impl CompilerMeta {
    pub fn new() -> Self {
        Self {
            status: CompilerStatus::Success,
            mutations: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            read_buffer: Some(Map::new()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ActionTarget {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "functionName", default)]
    pub function_name: Option<String>,
    #[serde(rename = "hardcodedDependency", default)]
    pub hardcoded_dependency: Option<String>,
    #[serde(rename = "namedImport", default)]
    pub named_import: Option<String>,
    #[serde(rename = "nodeName", default)]
    pub node_name: Option<String>,
    #[serde(rename = "recipeName", default)]
    pub recipe_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionMutations {
    pub rename: Option<String>,
    #[serde(rename = "injectArgs")]
    pub inject_args: Option<Map<String, Value>>,
    #[serde(rename = "targetArgIndex")]
    pub target_arg_index: Option<usize>,

    #[serde(rename = "enforceExplicitType")]
    pub enforce_explicit_type: Option<String>,
    #[serde(rename = "generateInterface")]
    pub generate_interface: Option<Vec<String>>,
    #[serde(rename = "targetParamIndex")]
    pub target_param_index: Option<usize>,

    #[serde(rename = "extractToParameter")]
    pub extract_to_parameter: Option<String>,

    #[serde(rename = "replaceWith")]
    pub replace_with: Option<String>,
    #[serde(rename = "moduleSpecifier")]
    pub module_specifier: Option<String>,
    #[serde(rename = "ensureImport")]
    pub ensure_import: Option<String>,
    #[serde(rename = "isTypeOnly")]
    pub is_type_only: Option<bool>,

    pub extract: Option<String>,

    pub options: Option<Map<String, Value>>,
    #[serde(rename = "targetFiles")]
    pub target_files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum DialecticAction {
    #[serde(rename = "MUTATE_CALL")]
    MutateCall { target: ActionTarget, mutations: ActionMutations },
    #[serde(rename = "TRANSLATE_DIALECT")]
    TranslateDialect { target: ActionTarget, mutations: ActionMutations },
    #[serde(rename = "RESTRUCTURE_TOPOLOGY")]
    RestructureTopology { target: ActionTarget, mutations: ActionMutations },
    #[serde(rename = "MANAGE_IMPORT")]
    ManageImport { target: ActionTarget, mutations: ActionMutations },
    #[serde(rename = "READ_TOPOLOGY")]
    ReadTopology { target: ActionTarget, mutations: ActionMutations },
    #[serde(rename = "EXECUTE_RECIPE")]
    ExecuteRecipe { target: ActionTarget, mutations: ActionMutations },
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Deserialize)]
pub struct CompilerPayload {
    pub file_path: Option<String>,
    pub source: Option<String>,
    #[serde(default)]
    pub intent: Vec<DialecticAction>,
    #[serde(rename = "dryRun", default)]
    pub dry_run: bool,
}

use ast_grep_language::LanguageExt;

// ----------------------------------------------------------------------------
// HELPER FUNCTIONS
// ----------------------------------------------------------------------------

fn resolve_callable_topology<'a>(root: Node<'a, StrDoc<SupportLang>>, target: &str) -> Option<Node<'a, StrDoc<SupportLang>>> {
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

fn apply_edits(source: &mut String, mut edits: Vec<(usize, usize, String)>) {
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, text) in edits {
        if start <= end && end <= source.len() {
            source.replace_range(start..end, &text);
        }
    }
}

fn render_value(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
        _ => v.to_string(),
    }
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

// ----------------------------------------------------------------------------
// MAIN ENGINE
// ----------------------------------------------------------------------------

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

    let mut meta = CompilerMeta::new();
    let registry = RecipeRegistry::new();

    if input_data.trim().is_empty() {
        meta.status = CompilerStatus::Fatal;
        meta.errors.push("Empty input provided.".to_string());
        println!("{}", serde_json::to_string(&meta).unwrap());
        return;
    }

    let payload_result: Result<CompilerPayload, _> = serde_json::from_str(&input_data);
    let payload = match payload_result {
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
                meta.status = CompilerStatus::Fatal;
                meta.errors.push("Invalid JSON payload sequence.".to_string());
                println!("{}", serde_json::to_string(&meta).unwrap());
                return;
            }
        }
    };

    let is_dry_run = dry_run_cli || payload.dry_run;
    let path_str = payload.file_path.unwrap_or_else(|| "temp.rs".to_string());
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
        println!("{}", serde_json::to_string(&meta).unwrap());
        return;
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

            // ========================================================================
            // PRIMITIVE C: THE CLASSIC MUTATOR
            // ========================================================================
            DialecticAction::MutateCall { target, mutations } => {
                let target_name = target.name.as_deref().unwrap_or("");
                if target_name.is_empty() {
                    meta.errors.push("MUTATE_CALL missing target.name".to_string());
                    continue;
                }

                // In ASTs like tree-sitter, a call_expression usually has a child identifier matching target_name
                let target_chars = target_name.to_string();
                for node in root.dfs() {
                    let k = node.kind().to_string().to_lowercase();
                    if k.contains("call") || k.contains("expression") {
                        // Check if node starts with the target name (naive heuristic for polyglot support)
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

            // ========================================================================
            // PRIMITIVE A: IDIOMATIC TRANSLATION
            // ========================================================================
            DialecticAction::TranslateDialect { target, mutations } => {
                let func_name = target.function_name.as_deref().unwrap_or("");
                if let Some(_func_node) = resolve_callable_topology(root.clone(), func_name) {
                    if let Some(props) = mutations.generate_interface {
                        if current_source.contains(&mutations.enforce_explicit_type.clone().unwrap_or_default()) {
                            // interface exists theoretically
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
                            
                            // Insert at the beginning of the file (or after imports)
                            edits.push((0, 0, iface_str));
                            meta.mutations += 1;
                        }
                    }

                    // Extract and mutate param type (naive append)
                    if let Some(_explicit_type) = mutations.enforce_explicit_type {
                        // Very naive parameter type injection for TS/Python.
                        // Wait, a better way is simply replacing the function signature line if it doesn't have the explicit type.
                        // Or we can just log a warning that full explicit type injection requires TS language server.
                        meta.warnings.push("Type injection (enforce_explicit_type) uses naive matching. AST surgery on parameters is complex.".to_string());
                    }
                } else {
                    meta.warnings.push(format!("Target '{}' missing. Epistemological translation aborted.", func_name));
                }
            }

            // ========================================================================
            // PRIMITIVE B: TOPOLOGICAL RESTRUCTURING
            // ========================================================================
            DialecticAction::RestructureTopology { target, mutations } => {
                let func_name = target.function_name.as_deref().unwrap_or("");
                let dep_name = target.hardcoded_dependency.as_deref().unwrap_or("");
                let param_name = mutations.extract_to_parameter.as_deref().unwrap_or("");

                if let Some(func_node) = resolve_callable_topology(root.clone(), func_name) {
                    for node in func_node.dfs() {
                        let k = node.kind().to_string().to_lowercase();
                        if (k.contains("new") || k.contains("instantiation") || k.contains("call")) && node.text().contains(dep_name) {
                            let range = node.range();
                            // Substitute the instantiation with the parameter name
                            edits.push((range.start, range.end, param_name.to_string()));
                            meta.mutations += 1;
                        }
                    }
                }
            }

            // ========================================================================
            // PRIMITIVE D: EPISTEMOLOGICAL IMPORT MANAGEMENT
            // ========================================================================
            DialecticAction::ManageImport { target, mutations } => {
                let named_import = target.named_import.as_deref().unwrap_or("");
                
                if let Some(replace_w) = &mutations.replace_with {
                    for node in root.dfs() {
                        let k = node.kind().to_string().to_lowercase();
                        if k.contains("import") {
                            let text = node.text();
                            if text.contains(named_import) {
                                // A very naive text replacement for the import block
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
                            if k.contains("import") && node.text().contains(mod_spec) {
                                found_spec = true;
                                if !node.text().contains(ensure_imp) {
                                    // Inject into existing import
                                    // E.g. `import { foo } from 'bar'`
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
                            // Prepend an import
                            let is_type = if mutations.is_type_only.unwrap_or(false) { "type " } else { "" };
                            // Handle polyglot naive
                            let imp_str = format!("import {} {{ {} }} from '{}';\n", is_type, ensure_imp, mod_spec);
                            edits.push((0, 0, imp_str));
                            meta.mutations += 1;
                        }
                    }
                }
            }

            // ========================================================================
            // PRIMITIVE E: DIAGNOSTIC PROBE
            // ========================================================================
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
            
            // ========================================================================
            // PRIMITIVE F: BOUTIQUE RECIPE
            // ========================================================================
            DialecticAction::ExecuteRecipe { target, .. } => {
                let recipe_name = target.recipe_name.as_deref().unwrap_or("");
                if let Some(_recipe) = registry.get(recipe_name) {
                    // Future: actually execute a complex migration here.
                    // For now, we'll just flag it as a mutation in metadata to simulate the hook.
                    meta.warnings.push(format!("Boutique recipe '{}' invoked but no-op in this engine version.", recipe_name));
                } else {
                    meta.warnings.push(format!("Boutique recipe '{}' not found in registry.", recipe_name));
                }
            }
        }

        // Apply edits collected in this intent to the current_source buffer.
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

    println!("{}", serde_json::to_string(&meta).unwrap());
}
