# Patchiest

**Patchiest** is the core "Engine" that powers the **Athanor** orchestrator. It is a robust, native Python environment engineered for AST-safe, deterministic code mutations.

By utilizing the **Patchiest Protocol**, it acts as the "Legislator of Intent" – translating the AI's abstract Dialectic Actions into surgical codebase modifications.

## Features

* **Surgical AST Mutations:** Instead of relying on full-file regeneration, Patchiest performs precise AST (Abstract Syntax Tree) insertions, modifications, and deletions.
* **The Patchiest Protocol:** Implements a strict, discriminated union of actionable events (e.g., `TRANSLATE_DIALECT`, `MUTATE_CALL`, `MANAGE_IMPORT`). The AI must fulfill these strict schemas to interact with the codebase, drastically lowering hallucination risks and import errors.
* **High Performance:** Optimized for native execution within Python 3.14, leveraging features like the new tail-call interpreter to ensure maximum runtime efficiency. 
* **AST-Safe Precision:** Focused strictly on transactional reliability, ensuring that automatic code mutations are safe, deterministic, and structurally sound before they are committed to the disk.

## Architecture

While Athanor handles the user cockpit and orchestration (TUI, agentic loop), Patchiest handles the literal transformation of code at the syntax tree level. It ensures the prompt commands translate into effective, error-free software engineering.
