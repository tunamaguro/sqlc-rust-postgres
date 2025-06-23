# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `sqlc-rust-postgres`, a sqlc plugin that generates Rust code for PostgreSQL databases using tokio_postgres, postgres, and deadpool_postgres crates. The plugin processes SQL schemas and queries to generate type-safe Rust database access code.

## Architecture

- **Core Plugin**: `src/main.rs` - WASM plugin entry point that reads protobuf input and outputs generated code
- **Code Generation**: `src/codegen.rs` - Main code generation logic with `PostgresGenerator` struct
- **Query Processing**: `src/query.rs` - Handles SQL query parsing and parameter extraction
- **Type System**: `src/user_type.rs` - PostgreSQL to Rust type mapping with custom type overrides
- **Error Handling**: `src/error.rs` - Plugin-specific error types
- **Protobuf Integration**: `src/protos/plugin/codegen.proto` - sqlc plugin protocol definitions

The plugin generates:
- Const strings for SQL queries
- Row structs for query results
- Async functions for database operations
- Support for custom type overrides via configuration

## Development Commands

All development commands use `just` (justfile runner). Available commands:

### Setup
```bash
just setup-tools           # Install required tools (wasm32-wasip1 target)
```

### Code Quality
```bash
just format                # Format code (alias: just f)
just lint                  # Show clippy errors (alias: just l)
just lint-fix              # Fix clippy errors (alias: just lf)
```

### Testing
```bash
just test                  # Run all tests
just test-d                # Run tests with stdout output
```

### Build
```bash
just build-wasm-dev        # Build WASM plugin for development
just build-wasm-release    # Build WASM plugin for release
```

### Code Generation
```bash
just generate              # Build dev WASM and run sqlc generate
```

### Complete Workflow
```bash
just ready                 # Full pipeline: generate, format, lint-fix, test, build-wasm-release
```

## Configuration

- `sqlc.json` - Production sqlc configuration
- `sqlc_dev.json` - Development sqlc configuration template (uses $WASM_SHA256 placeholder)
- Examples in `examples/` directory show different database crate configurations:
  - `examples/authors/` - tokio_postgres example
  - `examples/jets/` - postgres example  
  - `examples/ondeck/` - deadpool_postgres example
  - `examples/custom_type/` - custom type overrides example

## Plugin Options

The plugin supports these configuration options in sqlc.json:
- `db_crate`: "tokio_postgres" (default), "postgres", or "deadpool_postgres"
- `overrides`: Custom type mappings for unsupported PostgreSQL types
- `enum_derives`: Additional derive attributes for generated enums
- `row_derives`: Additional derive attributes for generated row structs

## Testing Individual Examples

```bash
cargo test -p authors
cargo test -p jets  
cargo test -p ondeck
cargo test -p custom_type
```

## Prerequisites

The plugin requires `protoc` (Protocol Buffers compiler) to be installed for building protobuf definitions.

## Development Best Practices

### Code Quality Workflow
Always use `just` commands for code quality checks:
- Use `just format` to format code with rustfmt before committing
- Use `just lint-fix` to automatically fix clippy issues (allows dirty workspace)
- Use `just test` to run all tests
- Use `just ready` for complete pipeline before submitting PRs

### Error Message Development
When improving error messages:
1. Test actual error scenarios by creating minimal reproduction cases
2. Check both Debug (`{:#?}`) and Display (`{}`) formatting - prefer Display for user-facing messages
3. Ensure error messages are actionable and guide users to solutions
4. Use `just format` and `just lint-fix` to maintain code standards

### Refactoring and Code Changes
When performing refactoring or making structural changes:
1. **ALWAYS** ensure all code compiles before committing
2. **ALWAYS** run `just lint-fix` to fix clippy warnings and errors
3. **ALWAYS** run `just test` to ensure all tests pass
4. **ALWAYS** run `just generate` to ensure WASM builds successfully
5. Fix ALL compilation errors and lint warnings before considering the work complete
6. Never leave the codebase in a broken state - compilation and tests must pass

### Testing Error Scenarios
- Create temporary test configurations in `test_error/` directory
- Use `_test_error_sqlc.json` with dynamic WASM SHA256 replacement
- Build WASM plugin with `just build-wasm-dev` before testing
- Generate dynamic config: `WASM_SHA256=$(sha256sum target/wasm32-wasip1/debug/sqlc-rust-postgres.wasm | awk '{print $1}'); sed "s/\$WASM_SHA256/${WASM_SHA256}/g" config_template.json > actual_config.json`

### CI/CD Integration
- Always run `just format` and `just lint-fix` before pushing
- Check CI status with `gh pr checks <pr-number>`
- Use `gh run view <run-id> --log-failed` to diagnose CI failures
- Commit formatting fixes separately from functional changes