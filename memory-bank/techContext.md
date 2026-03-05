# techContext.md

**Technology Stack**
* **Runtime:** Rust 2021 (Patchiest), Python 3.14 (Athanor)
* **AST Engine:** `ast-grep` (programmatic), `tree-sitter`
* **Frontend:** Vanilla JS/CSS/HTML (PWA)
* **Web Server:** `starlette`, `uvicorn`
* **TUI Framework:** `textual`

**Development Environment**
* **Venv:** `.venv/`
* **Dependencies:** `ast-grep-core`, `ast_grep_language`, `serde`, `serde_json`, `anyhow` (Rust); `textual`, `starlette`, `pathspec` (Python)
* **Binary Path:** `~/.local/bin/patchiest` (pointing to Rust target)

**Technical Constraints**
* **Sovereign Architecture:** Local-first, node-less, high-performance.
* **Transactional Reliability:** Atomic file writes with automatic rollbacks.