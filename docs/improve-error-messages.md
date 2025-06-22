# Error Message Improvement Task

## Overview

This document records the work done to improve error messages for unsupported database types in the sqlc-rust-postgres plugin.

## Problem

When users encountered unsupported PostgreSQL types, the error messages were not helpful:

```
error generating code: InvalidRustType(
    "pg_catalog.interval",
)
```

This Debug format output didn't provide guidance on how to resolve the issue.

## Solution

### Changes Made

1. **Fixed Error Output Format** (`src/main.rs`)
   - Changed `eprintln!("{e:#?}");` to `eprintln!("{e}");` 
   - This switched from Debug format to Display format for better user experience

2. **Improved Error Message** (`src/error.rs`)
   - Updated `InvalidRustType` error message to be more descriptive and actionable
   - Old: `"{} is not valid rust type"`
   - New: `"Cannot find rust type that matches column type of \`{}\`. Add an entry to the 'overrides' section in your sqlc.json configuration."`

### Testing Process

1. **Created Test Case**
   - Created `test_error/schema.sql` with unsupported `interval` type
   - Created `test_error/query.sql` to trigger the error
   - Created `test_error_sqlc.json` configuration file

2. **Reproduced Error**
   - Built WASM plugin: `just build-wasm-dev`
   - Generated config with dynamic hash: `WASM_SHA256=$(sha256sum target/wasm32-wasip1/debug/sqlc-rust-postgres.wasm | awk '{print $1}'); sed "s/\$WASM_SHA256/${WASM_SHA256}/g" test_error_sqlc.json > _test_error_sqlc.json`
   - Ran sqlc: `sqlc generate -f _test_error_sqlc.json`

3. **Verified Improvement**
   - Before: Debug format with unclear message
   - After: Clear, actionable error message

## Code Quality Process

### Formatting Issues
- Initial commit failed CI due to rustfmt formatting requirements
- Used `just format` to automatically fix formatting
- Committed formatting fix separately: `fix: format error message code according to rustfmt`

### Quality Checks
- Used `just lint-fix` to check and fix clippy issues
- All tests passed after changes

## GitHub Integration

### Issue Creation
- Created Issue #39: "Improve error message for unsupported database types"
- Documented problem, expected behavior, and impact

### Pull Request
- Created PR #40: "feat: improve error message for unsupported database types"
- Included before/after examples
- Referenced the issue with "Closes #39"
- CI passed after formatting fix

## Results

### Before
```
error generating code: InvalidRustType(
    "pg_catalog.interval",
)
```

### After
```
error generating code: Cannot find rust type that matches column type of `pg_catalog.interval`. Add an entry to the 'overrides' section in your sqlc.json configuration.
```

## Lessons Learned

### Development Workflow
1. Always use `just` commands for consistency:
   - `just format` for code formatting
   - `just lint-fix` for clippy fixes
   - `just build-wasm-dev` for development builds

2. Test error scenarios with actual reproduction cases
3. Verify both Debug and Display format outputs
4. Use CI feedback to catch formatting issues early

### Error Message Design
1. Prefer Display format over Debug for user-facing errors
2. Make error messages actionable with specific guidance
3. Reference configuration options and documentation

### CI/CD Best Practices
1. Run `just format` and `just lint-fix` before committing
2. Use `gh pr checks` and `gh run view --log-failed` for CI debugging
3. Commit formatting fixes separately from functional changes

## Files Modified

- `src/main.rs`: Changed error output format
- `src/error.rs`: Improved InvalidRustType error message
- `docs/improve-error-messages.md`: This documentation

## Related Links

- Issue: https://github.com/tunamaguro/sqlc-rust-postgres/issues/39
- Pull Request: https://github.com/tunamaguro/sqlc-rust-postgres/pull/40