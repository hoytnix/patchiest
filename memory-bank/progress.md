# progress.md

**Current Status**
* **What Works:** 
    * **SPA PWA:** Modern dark-mode SPA in `www/` featuring glitch animations, PWA support, MIT License, Athanor integration, and a native Python `server.py` using `starlette`.
    * **TUI Focus:** Auto-focusing prompt on TUI start; native gitignore respect during agent file traversal via `pathspec` library; Real-time tool action logging within the left column; Core package renamed to `athanor` with full CLI and TUI alignment; standard nested package structure implemented; global CLI command `athanor` available via `~/.local/bin`; Full-screen adaptive TUI layout completely rebuilt using `textual` for proper input handling; Interactive prompt visibility fixed and moved to the Prompt panel; Native `patchiest.py` AST engine; Token and Financial cost calculation logic; Protocol-enforced agentic loop; `patchiest` package executable with `python -m patchiest`.
* **What's Left:**
    * Expand `patchiest.py` to handle complex Python type-hint injections.
    * Implement `git_commit` tool for agentic work saving.
* **Known Issues:** Pydantic models for Unions require explicit `TypeAdapter` logic to prevent schema generation errors.