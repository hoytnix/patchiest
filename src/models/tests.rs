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
