# Patchiest

**Patchiest** is the Rust-native mutation engine for the Alchememe stack.

It is now a standalone CLI that consumes **Patchiest Protocol** JSON and executes structural mutations via **ast-grep**, replacing the older Python `ast` package implementation.

## Features

* **Rust Standalone Runtime:** No Python package dependency for code mutation.
* **ast-grep Structural Search:** Fast, polyglot-ready matching and deterministic call-site targeting.
* **Protocol-Driven Mutations:** JSON action payloads (`DialecticAction`) control mutation behavior.
* **Transactional Status Contract:** Returns explicit `SUCCESS`, `NO_MUTATIONS`, or `ROLLBACK` responses.

## Current Runtime Slice

Patchiest currently supports:

* `MUTATE_CALL`
  * match by `target.name`
  * optional `mutations.rename`
  * optional `mutations.injectArgs`

Additional protocol actions remain on the roadmap.

## Usage

```bash
# Build
cargo build --manifest-path patchiest/Cargo.toml

# Run with JSON file
cargo run --manifest-path patchiest/Cargo.toml -- action.json

# Run from stdin
cat action.json | cargo run --manifest-path patchiest/Cargo.toml --
```

## Testing & Coverage

### Run Test Suite

To run the Rust test suite:

```bash
cargo test
```

### Coverage Report

To generate an HTML coverage report using `tarpaulin`:

```bash
cargo tarpaulin --ignore-tests --out html
```

The report will be generated at `tarpaulin-report.html` in the project root.

Sample payload:

```json
{
  "action": "MUTATE_CALL",
  "file_path": "src/example.py",
  "target": { "name": "old_fn" },
  "mutations": {
    "rename": "new_fn",
    "injectArgs": { "timeout": 30 }
  }
}
```

## Architecture

Patchiest is designed as the mutation blade while Athanor orchestrates intent. Athanor now invokes Patchiest as a subprocess CLI bridge, preserving a strict protocol boundary while aligning performance-critical mutation logic with Rust.

## Polyglot Support (Planned / Early)

Patchiest is moving from Python-only AST mutation toward a broader `ast-grep`-based surgical layer. This is a **roadmap matrix**, not a claim of fully shipped runtime behavior.

> **Maturity disclaimer:** Current end-to-end implementation maturity is approximately **5%** and under active development.

| Language | Patchiest (ast-grep manipulation) | Athanor (Wasmtime execution target) | Note |
| :--- | :---: | :---: | :--- |
| Rust | ✅ | ✅ (Tier 1) | Strong long-term synergy.
| C / C++ | ✅ | ✅ (Tier 1) | WASI-SDK path.
| Go | ✅ | ✅ (via TinyGo) | Standard Go Wasm still maturing.
| JavaScript / TypeScript | ✅ | ✅ (via Javy/QuickJS) | TS requires transpilation for Wasm paths.
| Python | ✅ | ✅ (componentize-py) | Currently central to both projects.
| Ruby | ✅ | ✅ (ruby.wasm) | WASI-VFS model.
| C# (.NET) | ✅ | ✅ (NativeAOT) | Growing Bytecode Alliance momentum.
| Java | ✅ | ✅ (TeaVM/Graal) | JVM/Wasm constraints apply.
| Swift | ✅ | ✅ | Official Wasm support emerging.
| Kotlin | ✅ | ✅ (K/Wasm) | Strong roadmap via WasmGC.
| Zig | ❌ (planned via custom grammar) | ✅ | Requires custom Tree-sitter plumbing.
| PHP | ✅ | ✅ | Available precompiled runtimes.
| Elixir / Erlang | ✅ | ⚠️ Experimental | Execution side still experimental.
| Bash | ✅ | ❌ | Patchiest can modify scripts; Athanor will not execute Bash.
| Solidity | ✅ | ❌ | Important for contract editing workflows.
| Haskell | ✅ | ✅ (GHC Wasm) | |
| Lua | ✅ | ✅ | |
| Scala | ✅ | ✅ (via Native) | |
| Mojo | 🧩 Custom | ⚠️ WIP | Requires custom grammar and runtime maturation.

### Polyglot Gap (Structure vs. Execution)

Patchiest's surgical value extends beyond runtime languages into config and scripting domains (YAML/JSON/TOML/HCL/Bash), while Athanor's Wasmtime layer focuses on safe WebAssembly execution. The strategic end-state is a closed loop: mutate structure safely, then validate behavior immediately.
