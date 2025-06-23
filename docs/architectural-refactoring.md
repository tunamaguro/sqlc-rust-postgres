# Architectural Refactoring Plan

## Overview

This document outlines the architectural refactoring plan to address the growing complexity of `src/query.rs` (818 lines, 53% of total codebase) and improve maintainability through better separation of concerns.

## Current State Analysis

### File Size Distribution
- `src/query.rs`: 818 lines (53% of codebase)
- `src/user_type.rs`: 264 lines
- `src/codegen.rs`: 159 lines
- Other files: < 110 lines each

### Problems Identified

1. **Single Responsibility Principle Violation**: `query.rs` handles multiple responsibilities:
   - SQL query processing
   - Rust struct generation
   - Parameter processing
   - Field name generation logic
   - Type conversion processing

2. **Low Cohesion**: Unrelated functionality mixed in single file

3. **Maintainability Issues**: Large file size makes navigation and modification difficult

## Proposed Refactoring Options

### Option 1: Domain-Driven Separation
```
src/
├── query/
│   ├── mod.rs          # Public API
│   ├── annotation.rs   # QueryAnnotation related
│   ├── parser.rs       # SQL query parsing
│   └── builder.rs      # PostgresQuery construction
├── codegen/
│   ├── mod.rs
│   ├── struct_gen.rs   # PgStruct generation
│   ├── param_gen.rs    # PgParams generation
│   ├── func_gen.rs     # PostgresFunc generation
│   └── naming.rs       # Field name generation logic
└── db_crates/
    ├── mod.rs
    ├── traits.rs       # Common traits
    └── variants.rs     # DbCrate implementation
```

### Option 2: Layer Separation
```
src/
├── domain/            # Domain logic
│   ├── query.rs      # Query domain objects
│   ├── column.rs     # Column information
│   └── naming.rs     # Name generation rules
├── generator/         # Code generation responsibility
│   ├── struct_gen.rs
│   ├── func_gen.rs
│   └── const_gen.rs
└── infrastructure/    # Technical details
    ├── db_crates.rs
    └── type_mapping.rs
```

### Option 3: Feature-Based Separation (Recommended)
```
src/
├── query/
│   ├── mod.rs
│   ├── annotation.rs      # :exec, :one, :many processing
│   └── postgres_query.rs  # Main query processing
├── rust_gen/             # Rust element generation
│   ├── mod.rs
│   ├── struct_gen.rs     # PgStruct + field name logic
│   ├── param_gen.rs      # PgParams processing
│   ├── func_gen.rs       # PostgresFunc generation
│   └── const_gen.rs      # PostgresConstQuery
└── db_support/
    ├── mod.rs
    ├── crate_types.rs    # DbCrate + client/error/async
    └── column_types.rs   # PgColumn + PgColumnRef
```

## Implementation Strategy

### Phase 1: Extract Field Name Generation Logic
- Move field name generation functions (~200 lines) to `rust_gen/naming.rs`
- Functions to move:
  - `generate_unique_field_names()`
  - `generate_unique_param_names()`
  - `simulate_field_names()`
  - `has_field_name_conflicts()`
  - Related helper functions

### Phase 2: Separate DB Crate Handling
- Move `DbCrate` enum and related methods to `db_support/crate_types.rs`
- Extract client/error/async handling logic

### Phase 3: Extract Code Generation Logic
- Move struct generation logic to `rust_gen/struct_gen.rs`
- Move parameter generation to `rust_gen/param_gen.rs`
- Move function generation to `rust_gen/func_gen.rs`

### Phase 4: Refactor Main Query Logic
- Keep core `PostgresQuery` in `query/postgres_query.rs`
- Maintain clean public API in `query/mod.rs`

## Expected Benefits

1. **Reduced File Size**: `query.rs` from 818 lines to ~400 lines
2. **Better Maintainability**: Related functionality grouped together
3. **Improved Testability**: Individual components can be tested in isolation
4. **Enhanced Readability**: Clearer separation of concerns
5. **Easier Extension**: New features can be added to appropriate modules

## Risk Mitigation

1. **Maintain Public API**: Ensure no breaking changes to existing API
2. **Comprehensive Testing**: Run full test suite after each phase
3. **Incremental Approach**: Implement in small, reviewable chunks
4. **Documentation Updates**: Update module documentation as refactoring progresses

## Success Metrics

- [ ] `query.rs` reduced to < 500 lines
- [ ] No breaking changes to public API
- [ ] All tests continue to pass
- [ ] Improved code navigation and understanding
- [ ] Easier to add new DB crate support

## Implementation Timeline

1. **Week 1**: Phase 1 - Extract field name generation
2. **Week 2**: Phase 2 - Separate DB crate handling  
3. **Week 3**: Phase 3 - Extract code generation logic
4. **Week 4**: Phase 4 - Refactor main query logic
5. **Week 5**: Testing, documentation, and cleanup