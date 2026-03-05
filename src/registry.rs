use std::collections::HashMap;

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

#[cfg(test)]
mod tests {
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
}
