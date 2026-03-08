    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = RecipeRegistry::new();
        let recipes = registry.list();
        assert!(recipes.len() >= 2);
    }

    #[test]
    fn test_registry_get() {
        let registry = RecipeRegistry::new();
        let recipe = registry.get("cjs-to-esm");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().name, "cjs-to-esm");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = RecipeRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }
