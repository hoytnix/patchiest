# activeContext.md

**What Works:** 
    * **Patchiest Rust Engine:** Full programmatic AST mutation engine supporting `ast-grep` programmatic API, polyglot structural surgery, and atomic transactions. Achieved 98.02% overall test coverage with `cargo tarpaulin --ignore-tests` and **100% coverage for cli.rs**. Codebase neatly modularized (engine, cli, models, etc.).
    * **SPA PWA:** Modern dark-mode SPA in `www/` featuring glitch animations, PWA support, MIT License, Athanor integration, up-to-date progress reflecting high engine coverage, and a native Python `server.py` using `starlette`.
    * **TUI Focus:** Auto-focusing prompt on TUI start; native gitignore respect during agent file traversal via `pathspec` library; Real-time tool action logging within the left column; Core package renamed to `athanor` with full CLI and TUI alignment; standard nested package structure implemented; global CLI command `athanor` available via `~/.local/bin`; Full-screen adaptive TUI layout completely rebuilt using `textual` for proper input handling.
* **What's Left:**
    * Implement `git_commit` tool for agentic work saving.
    * Expand `RecipeRegistry` with ready-to-use boutique codemods.
* **Learnings:** Standard nested package structures are essential for reliable `setuptools` editable installations and entry point discovery in Python 3.14. Achieving 100% coverage requires carefully asserting error outputs, handling AST traversal edge-cases, and sometimes refactoring match statements for better instrumentation.