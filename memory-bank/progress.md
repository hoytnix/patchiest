# progress.md

**Current Status**
* **What Works:** 
    * **Patchiest Rust Engine:** Full programmatic AST mutation engine. Modularized into `models`, `registry`, `helpers`, `engine`, and `cli` for improved maintainability.
    * **Test Suite:** Comprehensive unit and integration test suite with coverage reporting. 98.02% overall test coverage achieved with 100% for cli.rs using `cargo tarpaulin --ignore-tests`.
    * **SPA PWA:** Modern dark-mode SPA in `www/` featuring glitch animations, PWA support, MIT License, Athanor integration, up-to-date progress reflecting high engine coverage, and a native Python `server.py` using `starlette`.
    * **TUI Focus:** Auto-focusing prompt on TUI start; native gitignore respect during agent file traversal via `pathspec` library; Real-time tool action logging within the left column; Core package renamed to `athanor` with full CLI and TUI alignment; standard nested package structure implemented; global CLI command `athanor` available via `~/.local/bin`; Full-screen adaptive TUI layout completely rebuilt using `textual` for proper input handling.
* **What's Left:**
    * Implement `git_commit` tool for agentic work saving.
    * Expand `RecipeRegistry` with ready-to-use boutique codemods.
* **Known Issues:** Pydantic models for Unions in Athanor require explicit `TypeAdapter` logic to prevent schema generation errors.