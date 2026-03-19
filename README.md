# Ogham

**A modern schema language for building data-oriented applications.**

Ogham lets you define your data types, services, and contracts in a single source of truth, then generate type-safe code for multiple languages and wire formats. Think of it as Protocol Buffers redesigned for today's workflows — with generics, composable shapes, built-in validation, a package manager, and breaking-change detection out of the box.

> **Status:** Ogham is under active development. The language and tooling are evolving rapidly; breaking changes are expected before 1.0.

## Why Ogham?

- **Schema-first** — define once, generate everywhere. No hand-written DTOs or manual sync across services.
- **Generics & Shapes** — compile-time generics with monomorphization and composable shapes (mixins) keep schemas DRY without runtime cost.
- **Pick / Omit** — derive subsets of types inline, just like TypeScript utility types.
- **Built-in validation** — annotate fields with `@validate::Required`, `@validate::Range(min=1)`, etc. Validators travel with the schema.
- **Breaking-change detection** — `ogham breaking --against git:main` catches wire-incompatible changes before they ship.
- **Package manager** — Go-style module system with git dependencies, vendoring, and optional proxy registry.
- **Multi-language codegen** — plugin architecture with SDKs in Rust, Go, and TypeScript. Ship a new language target without touching the compiler.
- **LSP included** — first-class editor support via `ogham-lsp`.

## Quick Example

```ogham
package myapp;

import github.com/oghamlang/std/uuid;
import github.com/oghamlang/std/time;
import github.com/oghamlang/std/validate;

// Composable shapes — reusable field sets
shape Timestamps {
    time.Timestamp created_at;
    time.Timestamp updated_at;
}

// Type with shapes, optionals, and validation
type User {
    Timestamps;
    uuid.UUID id = 1;
    @validate::Length(min=1, max=100)
    string name = 2;
    string? email = 3;
    Role role = 4;
}

enum Role {
    ADMIN = 1;
    MEMBER = 2;
    GUEST = 3;
}

// Generics — one definition, many instantiations
type ListResponse<T> {
    []T items = 1;
    string? next_cursor = 2;
}

// Services with streaming, inline types, and Pick
service UserService {
    rpc GetUser(uuid.UUID) -> User;
    rpc ListUsers(void) -> ListResponse<User>;
    rpc CreateUser({
        string name = 1;
        string? email = 2;
        Role role = 3;
    }) -> User;
    rpc UpdateUser(Pick<User, id, name, email>) -> User;
    rpc WatchUsers(void) -> stream User;
}
```

## Installation

### Via Cargo

```bash
cargo install --git https://github.com/oghamlang/ogham.git ogham-cli
cargo install --git https://github.com/oghamlang/ogham.git ogham-gen-proto
```

This installs the `ogham` CLI and the protobuf generator. For the Go code generator, build it separately:

```bash
go install github.com/oghamlang/ogham/go/ogham-gen-go@latest
```

### From Source

**Prerequisites:** Rust 1.75+, Go 1.21+ (for the Go code generator).

```bash
git clone https://github.com/oghamlang/ogham.git
cd ogham

# Build all binaries (ogham, ogham-lsp, ogham-gen-proto, ogham-gen-go)
make build

# Install to ~/.ogham/bin (or $OGHAM_BIN)
make install
```

Add `~/.ogham/bin` to your `PATH`:

```bash
export PATH="$HOME/.ogham/bin:$PATH"
```

### Verify

```bash
ogham --help
```

## Getting Started

### 1. Initialize a Project

Create a directory with two config files:

**`ogham.mod.yaml`** — module manifest:

```yaml
module: github.com/yourorg/yourproject
version: 0.1.0
ogham: ">= 0.1.0"

require:
  github.com/oghamlang/std: ^0.1.0

breaking:
  against: git:main
  policy: warn
```

**`ogham.gen.yaml`** — code generation config:

```yaml
generate:
  plugins:
    - name: github.com/oghamlang/ogham-gen-proto
      out: gen/proto/
      opts:
        go_package_prefix: github.com/yourorg/yourproject/gen/proto
    - name: github.com/oghamlang/ogham-gen-go
      out: gen/go/
```

### 2. Write Schemas

Create `.ogham` files in your project directory (see the [Quick Example](#quick-example) above).

### 3. Check & Generate

```bash
# Validate schemas
ogham check

# Generate code
ogham generate

# Dump the compiled IR for debugging
ogham dump -o ir.json
```

### 4. Manage Dependencies

```bash
ogham get github.com/org/schemas           # Add a dependency
ogham get github.com/org/schemas@v1.2.0    # Pin a version
ogham install                              # Fetch all deps
ogham update                               # Update to latest
ogham vendor                               # Copy deps to vendor/
```

### 5. Detect Breaking Changes

```bash
ogham breaking --against git:main          # Compare with main branch
ogham breaking --against git:v1.0.0        # Compare with a tag
```

## Language Highlights

| Feature | Syntax |
|---|---|
| Type aliases | `type UserId = uuid.UUID;` |
| Optional fields | `string? email = 3;` |
| Collections | `[]T`, `map<string, T>` |
| Shapes (mixins) | `shape Timestamps { ... }` |
| Generics | `type Page<T> { []T items = 1; }` |
| Pick / Omit | `Pick<User, id, name>`, `Omit<User, password>` |
| Enums | `enum Status { ACTIVE = 1; }` |
| Oneof | `oneof payment { Card card = 1; Cash cash = 2; }` |
| Annotations | `@validate::Range(min=0, max=100)` |
| Services | `service Foo { rpc Bar(Req) -> Res; }` |
| Streaming | `rpc Watch(void) -> stream Event;` |
| Inline types | `rpc Create({ string name = 1; }) -> User;` |

## Standard Library

Ogham ships with a standard library (`std/`) providing common types:

| Package | Description |
|---|---|
| `std/uuid` | UUID types with validation |
| `std/time` | Timestamp, Date, DateTime, TimeOfDay, TimeZone |
| `std/duration` | Duration type |
| `std/money` | Money with currency |
| `std/geo` | Geographic coordinates (LatLng) |
| `std/decimal` | Arbitrary-precision decimals |
| `std/ulid` | ULID identifiers |
| `std/validate` | Validation annotations (Required, Length, Range, Pattern, etc.) |
| `std/default` | Default value annotations |
| `std/rpc` | Pagination, sorting, and other RPC utilities |
| `std/proto` | Proto Well-Known Types (Timestamp, Duration, Any, Struct, etc.) |

## Code Generators

| Generator | Language | Description |
|---|---|---|
| `ogham-gen-proto` | Protocol Buffers | Emits `.proto` files with options for Go, Java, C#, Swift, PHP, Ruby, Obj-C |
| `ogham-gen-go` | Go | Native Go structs, enums, getters; optional protowire and JSON support |
| `ogham-gen-ts` | TypeScript | TypeScript types and serialization (WIP) |

### Writing Your Own Generator

Generators are standalone binaries that read an `OghamCompileRequest` from stdin (protobuf) and write an `OghamCompileResponse` to stdout. SDKs are available in:

- **Rust** — `oghamgen` crate
- **Go** — `go/oghamgen` package
- **TypeScript** — `@ogham/sdk`

See [`docs/adr/plugin_sdk.md`](docs/adr/plugin_sdk.md) for the plugin protocol.

## Compiler Pipeline

```
*.ogham files
    │
    ▼
  Lexer (logos)  →  Token stream
    │
    ▼
  Parser (recursive descent)  →  Lossless CST (rowan)
    │
    ▼
  Type checker / Validator
    │  shape expansion, monomorphization,
    │  alias resolution, Pick/Omit, annotations
    ▼
  IR (protobuf)  →  OghamCompileRequest
    │
    ▼
  Plugin (stdin/stdout)  →  Generated files
```

The LSP operates on the CST layer for real-time editor feedback.

## Repository Structure

```
crates/
  ogham-cli/           CLI binary
  ogham-compiler/      Lexer, parser, type checker, IR lowering
  ogham-core/          Shared utilities
  ogham-lsp/           Language Server Protocol
  ogham-proto/         Compiled protobuf definitions
  oghamgen/            Rust plugin SDK
  ogham-gen-proto/     Proto file generator

go/
  ogham-gen-go/        Go code generator
  oghamgen/            Go plugin SDK

ts/                    TypeScript plugin SDK

std/                   Standard library packages
proto/                 Protobuf sources for the IR protocol
docs/adr/              Architecture Decision Records
examples/golden/       Full example project (logistics system)
```

## Architecture Decision Records

Design decisions are documented in [`docs/adr/`](docs/adr/):

| Document | Topic |
|---|---|
| [`language.md`](docs/adr/language.md) | Language design, type system, and semantics |
| [`package.md`](docs/adr/package.md) | Module system, dependency resolution, and vendoring |
| [`cmd.md`](docs/adr/cmd.md) | CLI commands and flags |
| [`plugin_sdk.md`](docs/adr/plugin_sdk.md) | Plugin protocol and SDK architecture |
| [`compatibility.md`](docs/adr/compatibility.md) | Breaking-change detection rules |
| [`validation.md`](docs/adr/validation.md) | Annotation-based validation framework |
| [`wire.md`](docs/adr/wire.md) | Wire formats (protobuf binary, JSON, Arrow) |
| [`syntax/grammar.ebnf`](docs/adr/syntax/grammar.ebnf) | Formal EBNF grammar |

## Development

```bash
make help      # Show all targets
make check     # cargo check
make test      # Run all tests (Rust + Go + TS)
make clippy    # Lint
make fmt       # Format
make ci        # fmt + clippy + test
make example   # Build and generate the golden example
```

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.
