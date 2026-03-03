# systemPatterns.md

**Architecture & Design**
* **Athanor (The Orchestrator):** The `athanor/athanor/athanor.py` CLI which manages the TUI, the Gemini session, and the background file watcher.
* **Quintessence (The Runtime):** A native Python environment utilizing the `patchiest/patchiest/patchiest.py` engine for AST-deterministic mutations.
* **Patchiest Protocol:** A strict, discriminated union of actions (`TRANSLATE_DIALECT`, `MUTATE_CALL`, etc.) that Gemini must fulfill to interact with the codebase.
* **Agentic Phases:**
    1.  **Discover:** `list_files` identifies project topology.
    2.  **Analyze:** `read_file` performs semantic inspection.
    3.  **Transmute:** `apply_patchiest_action` performs AST surgery.
    4.  **Verify:** `run_command` validates the build/tests.