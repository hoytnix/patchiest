# techContext.md

**The Stack**
* **Runtime:** Python 3.14 (leveraging the new tail-call interpreter).
* **AI Engine:** Gemini 3.1 Pro via `google-genai` (utilizing Advanced Tool Use / Custom Tools).
* **TUI Framework:** `rich` for layout/live-updates and `questionary` for the interactive "Control Deck" input.
* **Schema Enforcement:** `pydantic` and `TypeAdapter` for strict protocol validation and schema generation.
* **Bundling/Transpilation:** `esbuild-py` for node-less, in-memory transmutation of TSX/TS to browser-ready JS.