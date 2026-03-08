pub mod models;
pub mod registry;
pub mod helpers;
pub mod engine;
pub mod cli;

use crate::registry::RecipeRegistry;
use crate::cli::run_app;
use std::io;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let registry = RecipeRegistry::new();
    let result = run_app(args, io::stdin(), &registry);
    println!("{}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_logic() {
        // We can't easily test main() because of std::env::args and io::stdin
        // But we can test that the entry points it uses are reachable.
        let registry = RecipeRegistry::new();
        let args = vec!["patchiest".to_string(), "--dry-run".to_string()];
        let payload = serde_json::json!({
            "dryRun": true,
            "source": "fn main() {}",
            "intent": []
        });
        let result = run_app(args, payload.to_string().as_bytes(), &registry);
        assert!(result.contains("DRY_RUN_COMPLETE") || result.contains("NO_MUTATIONS"));
    }
}
