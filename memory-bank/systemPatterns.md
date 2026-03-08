# systemPatterns.md

**Architecture & Design**
* **Athanor (The Orchestrator):** The `athanor/athanor/athanor.py` CLI which manages the TUI, the Gemini session, and the background file watcher.
* **Quintessence (The Runtime):** A native Python environment utilizing the `patchiest` Rust engine for AST-deterministic mutations.
    * `src/models.rs`: Core data structures and serializable actions.
    * `src/registry.rs`: Boutique recipe management.
    * `src/helpers.rs`: AST manipulation and snippet transformation utilities.
    * `src/engine.rs`: Core payload execution and AST surgery.
    * `src/cli.rs`: Command line parsing and orchestration.
    * `src/main.rs`: CLI entry point.
* **Patchiest Protocol:** A strict, discriminated union of actions (`TRANSLATE_DIALECT`, `MUTATE_CALL`, etc.) that Gemini must fulfill to interact with the codebase.
* **Agentic Phases:**
    1.  **Discover:** `list_files` identifies project topology.
    2.  **Analyze:** `read_file` performs semantic inspection.
    3.  **Transmute:** `apply_patchiest_action` performs AST surgery.
    4.  **Verify:** `run_command` validates the build/tests.