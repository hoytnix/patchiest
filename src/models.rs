use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compiler_meta_new() {
        let meta = CompilerMeta::new();
        assert_eq!(meta.status, CompilerStatus::Success);
        assert_eq!(meta.mutations, 0);
        assert!(meta.read_buffer.is_some());
    }

    #[test]
    fn test_deserialize_mutate_call() {
        let data = json!({
            "action": "MUTATE_CALL",
            "target": { "name": "foo" },
            "mutations": { "rename": "bar" }
        });
        let action: DialecticAction = serde_json::from_value(data).unwrap();
        if let DialecticAction::MutateCall { target, mutations } = action {
            assert_eq!(target.name, Some("foo".to_string()));
            assert_eq!(mutations.rename, Some("bar".to_string()));
        } else {
            panic!("Expected MutateCall variant");
        }
    }

    #[test]
    fn test_deserialize_compiler_payload() {
        let data = json!({
            "file_path": "test.ts",
            "intent": [
                {
                    "action": "READ_TOPOLOGY",
                    "target": { "nodeName": "SYSTEM" },
                    "mutations": { "extract": "AVAILABLE_RECIPES" }
                }
            ],
            "dryRun": true
        });
        let payload: CompilerPayload = serde_json::from_value(data).unwrap();
        assert_eq!(payload.file_path, Some("test.ts".to_string()));
        assert_eq!(payload.intent.len(), 1);
        assert!(payload.dry_run);
    }
}
