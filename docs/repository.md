# Repository Structure

`oghamlang/ogham` — the Ogham language: compiler, LSP, proto definitions, and plugin SDKs for all supported languages. Everything lives in one repository so CI validates the full stack on every change.

## Layout

```
ogham/
├── crates/                  # Rust workspace
│   ├── ogham-cli/           # CLI binary (`ogham`)
│   ├── ogham-compiler/      # Parser, type checker, linter, AST → IR lowering
│   ├── ogham-core/          # Shared types and utilities
│   ├── ogham-lsp/           # Language Server Protocol implementation
│   ├── ogham-plugin-sdk/    # Rust Plugin SDK (ogham-plugin-sdk crate)
│   └── ogham-proto/         # Generated Rust code from proto/ (prost/tonic)
│
├── proto/                   # Protobuf definitions — source of truth for IR/AST
│   ├── ogham/               # .proto files
│   ├── assets/              # easyp templates (Cargo.toml.tmpl, etc.)
│   └── easyp.yaml           # easyp generation config
│
├── go/
│   └── oghamgen/            # Go Plugin SDK (github.com/oghamlang/ogham/go/oghamgen)
│
├── ts/
│   └── oghamgen/            # TypeScript Plugin SDK (@ogham/oghamgen)
│
├── docs/
│   ├── adr/                 # Architecture Decision Records
│   │   ├── syntax/          # Ogham syntax examples and EBNF grammar
│   │   ├── language.md      # Language specification
│   │   ├── package.md       # Package management & module system
│   │   ├── plugin_sdk.md    # Plugin SDK architecture (AST vs IR, pipeline)
│   │   ├── cmd.md           # CLI commands reference
│   │   ├── compatibility.md # Breaking change detection
│   │   ├── validation.md    # Annotation-based validation
│   │   └── wire.md          # Wire formats and serialization
│   └── repository.md        # ← this file
│
├── Cargo.toml               # Workspace manifest
├── Cargo.lock
├── Makefile                  # Build commands (proto, check, fmt, clippy, test, ci)
├── AGENTS.md                # Agent workflow and code style guidelines
├── LICENSE
└── .gitignore
```

## Components

### Compiler (`crates/ogham-compiler`)

Lexer, parser, type checker, linter, and the AST → IR lowering pass. Takes `.ogham` source files, produces a fully resolved IR that plugins consume. See [adr/plugin_sdk.md](adr/plugin_sdk.md) for the compilation pipeline.

### CLI (`crates/ogham-cli`)

The `ogham` binary. Package management (`get`, `install`, `update`, `vendor`), code generation (`generate`), proto export, breaking change detection, plugin scaffolding. See [adr/cmd.md](adr/cmd.md).

### LSP (`crates/ogham-lsp`)

Language server for editor integration: diagnostics, hover, completion, go-to-definition. Works against the AST (not IR).

### Core (`crates/ogham-core`)

Shared types and utilities used across compiler, CLI, and LSP.

### Proto definitions (`proto/`)

The `.proto` files are the single source of truth for AST, IR, and compiler request/response messages. Generated code flows into `crates/ogham-proto` (Rust), `go/oghamgen/` (Go), and `ts/oghamgen/` (TypeScript).

Regenerate after changing `.proto` files:

```bash
make proto
```

### Rust Plugin SDK (`crates/ogham-plugin-sdk`)

Part of the Cargo workspace. Depends on `ogham-proto` for IR types, adds the plugin runner and code generation utilities. Published as `ogham-plugin-sdk` on crates.io.

### Go Plugin SDK (`go/oghamgen`)

Go module with its own `go.mod`. Import path: `github.com/oghamlang/ogham/go/oghamgen`. Contains generated IR types from proto and hand-written plugin runner / codegen helpers.

### TypeScript Plugin SDK (`ts/oghamgen`)

npm package with its own `package.json`. Published as `@ogham/oghamgen`. Contains generated IR types from proto and hand-written plugin runner / codegen helpers.

### SDK summary

| Directory | Published as | Language |
|-----------|-------------|----------|
| `crates/ogham-plugin-sdk` | `ogham-plugin-sdk` | Rust |
| `go/oghamgen` | `github.com/oghamlang/ogham/go/oghamgen` | Go |
| `ts/oghamgen` | `@ogham/oghamgen` | TypeScript |

All SDKs are tested in CI alongside the compiler — a proto change that breaks an SDK is caught immediately.

## Build

`Makefile` is the single entry point for all build, lint, test, and code generation commands. Run `make help` to see available targets.

## Why monorepo

- **Proto changes are validated end-to-end.** Changing an IR message updates the Rust proto crate and all SDKs in one PR. CI catches breakage across languages before merge.
- **Compiler and SDK versions stay in sync.** No cross-repo version matrix. One release tag covers the compiler and all SDKs.
- **Single CI pipeline.** Lint, test, and publish everything from one place.
