# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `sqlc-rust-postgres`, a sqlc plugin that generates Rust code for PostgreSQL databases using tokio_postgres, postgres, and deadpool_postgres crates. The plugin processes SQL schemas and queries to generate type-safe Rust database access code.

## Architecture

- **Core Plugin**: `src/main.rs` - WASM plugin entry point that reads protobuf input and outputs generated code
- **Code Generation**: `src/codegen.rs` - Main code generation logic with `PostgresGenerator` struct
- **Query Processing**: `src/query.rs` - Handles SQL query parsing and parameter extraction
- **Type System**: `src/user_type.rs` - PostgreSQL to Rust type mapping with custom type overrides
- **Database Support**: `src/db_support/` - Database-specific type handling and optimizations
  - `column_types.rs` - Column type definitions and copy-cheap type optimizations
  - `crate_types.rs` - Database crate abstractions (tokio_postgres, postgres, deadpool_postgres)
- **Rust Code Generation**: `src/rust_gen/` - Rust-specific code generation modules
  - `const_gen.rs` - SQL query constant generation
  - `func_gen.rs` - Database function generation
  - `param_gen.rs` - Function parameter generation with copy-cheap optimizations
  - `struct_gen.rs` - Row struct generation
  - `naming.rs` - Rust naming conventions and utilities
- **Error Handling**: `src/error.rs` - Plugin-specific error types
- **Protobuf Integration**: `src/protos/plugin/codegen.proto` - sqlc plugin protocol definitions

The plugin generates:
- Const strings for SQL queries
- Row structs for query results with configurable derives
- Async functions for database operations with optimized parameter passing
- Support for custom type overrides via configuration
- Copy-cheap type optimizations for better performance (primitive types, database enums)

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
- `copy_types`: Additional types that should be passed by value for better performance

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

### Branch Management and Pre-Commit Workflow
**ALWAYS** create a feature branch before starting work:
```bash
git checkout -b feature/your-feature-name
```

**MANDATORY** checks before every commit:
1. **Format**: `just format` - Ensure code is properly formatted
2. **Lint**: `just lint-fix` - Fix all clippy warnings and errors
3. **Compile**: Verify no compilation errors exist
4. **Test**: `just test` - Ensure all tests pass

**NEVER** commit code that fails any of the above checks. The codebase must always remain in a working state.

### New Feature Development Process

#### Requirements Analysis and Planning
**Before implementing, always execute the following:**

1. **Root Cause Analysis**
   - Identify not just the immediate problem (e.g., `id: &i64`) but all related issues
   - Consider similar cases (enums, user-defined types, etc.) simultaneously
   - Map out the complete problem domain

2. **Define Expected Completion State**
   ```rust
   // Before: get_author(client, &123, &Status::Open)
   // After:  get_author(client, 123, Status::Open)
   ```

3. **Identify Technical Constraints**
   - Future extensibility requirements
   - Performance requirements (static vs dynamic dispatch)
   - Backward compatibility with existing APIs

4. **Design Comprehensive Solution**
   - Avoid piecemeal fixes; design to solve the entire problem domain
   - Balance configurability with automation
   - Consider the impact on the entire system

#### Implementation Process

1. **Use Plan Mode for Complex Features**
   - Always present overall design in plan mode for complex features
   - Define specific phases with clear completion criteria
   - Identify dependencies and potential roadblocks upfront

2. **Staged but Consistent Implementation**
   - Each stage should maintain a working state
   - Design comprehensively upfront to avoid rework
   - Commit at meaningful milestones (not arbitrary stopping points)

3. **Validation and Feedback**
   - Verify functionality at each stage
   - Test with real use cases
   - Incorporate feedback early and often

#### Anti-Patterns to Avoid

- **Reactive Implementation**: Solving only the immediate symptom
- **Late Requirements**: Discovering requirements during implementation
- **Technical Debt**: Leaving inefficient implementations (e.g., dynamic dispatch)
- **Scope Creep**: Adding unrelated features during implementation

#### Success Patterns

- **Proactive Design**: Identify all related problems before coding
- **Comprehensive Solutions**: Design to solve the entire problem domain
- **Staged Implementation**: Comprehensive design, incremental execution

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

### Pull Request Workflow
Before creating a PR:
1. **Branch**: Work on a feature branch (never directly on main)
2. **Quality**: Run the mandatory pre-commit checks (format, lint, compile, test)
3. **Generate**: Run `just generate` to ensure WASM builds and code generation works
4. **Push**: Push the feature branch to remote
5. **PR**: Create PR with descriptive title and comprehensive description

### CI/CD Integration
- Always run `just format` and `just lint-fix` before pushing
- Check CI status with `gh pr checks <pr-number>`
- Use `gh run view <run-id> --log-failed` to diagnose CI failures
- Commit formatting fixes separately from functional changes