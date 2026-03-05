# activeContext.md

**What Works:** 
    * **Patchiest Rust Engine:** Full programmatic AST mutation engine supporting `ast-grep` programmatic API, polyglot structural surgery, and atomic transactions.
    * **SPA PWA:** Modern dark-mode SPA in `www/` featuring glitch animations, PWA support, MIT License, Athanor integration, and a native Python `server.py` using `starlette`.
    * **TUI Focus:** Auto-focusing prompt on TUI start; native gitignore respect during agent file traversal via `pathspec` library; Real-time tool action logging within the left column; Core package renamed to `athanor` with full CLI and TUI alignment; standard nested package structure implemented; global CLI command `athanor` available via `~/.local/bin`; Full-screen adaptive TUI layout completely rebuilt using `textual` for proper input handling.
* **What's Left:**
    * Implement `git_commit` tool for agentic work saving.
    * Expand `RecipeRegistry` with ready-to-use boutique codemods.
* **Learnings:** Standard nested package structures are essential for reliable `setuptools` editable installations and entry point discovery in Python 3.14.