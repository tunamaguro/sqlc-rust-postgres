use crate::plugin;
use std::collections::{HashMap, HashSet};

/// Extracts table identifier (alias or name) from a column
pub(crate) fn get_table_identifier(column: &plugin::Column) -> Option<String> {
    if let Some(table) = &column.table {
        // Use table alias if available, otherwise use table name
        let identifier = if !column.table_alias.is_empty() {
            column.table_alias.clone()
        } else {
            table.name.clone()
        };
        Some(identifier)
    } else {
        None
    }
}

/// Gets prefix for field names based on table information
pub(crate) fn get_field_prefix(column: &plugin::Column) -> Option<String> {
    if let Some(table) = &column.table {
        if !column.table_alias.is_empty() {
            // Use table alias if available (e.g., "e", "m" for self-joins)
            Some(column.table_alias.clone())
        } else {
            // Use table name if no alias (e.g., "authors", "books")
            Some(table.name.clone())
        }
    } else {
        None
    }
}

/// Simulates field name generation with option for simple vs prefixed names
pub(crate) fn simulate_field_names(query: &plugin::Query, use_simple_names: bool) -> Vec<String> {
    query
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let name = if let Some(prefix) = get_field_prefix(col) {
                if use_simple_names {
                    col.name.clone()
                } else {
                    format!("{}_{}", prefix, col.name)
                }
            } else if !col.name.is_empty() {
                col.name.clone()
            } else {
                format!("column_{}", idx)
            };
            crate::utils::rust_struct_field(&name)
        })
        .collect()
}

/// Checks if field names have conflicts using HashSet
pub(crate) fn has_field_name_conflicts(field_names: &[String]) -> bool {
    let unique_names: HashSet<&String> = field_names.iter().collect();
    field_names.len() != unique_names.len()
}

/// Checks if query involves only a single table identifier
pub(crate) fn has_single_table_identifier_basic(query: &plugin::Query) -> bool {
    let unique_identifiers: HashSet<String> = query
        .columns
        .iter()
        .filter_map(get_table_identifier)
        .collect();
    unique_identifiers.len() <= 1
}

/// Determines if simple field names can be used without conflicts
pub(crate) fn should_use_simple_names(query: &plugin::Query) -> bool {
    // Phase 1: Check if we have a single table identifier
    let single_table_identifier = has_single_table_identifier_basic(query);

    if !single_table_identifier {
        return false;
    }

    // Phase 2: Check for field name conflicts
    let simple_field_names = simulate_field_names(query, true);
    !has_field_name_conflicts(&simple_field_names)
}

/// Wrapper that uses should_use_simple_names for single table detection
pub(crate) fn has_single_table_identifier(query: &plugin::Query) -> bool {
    should_use_simple_names(query)
}

/// Main algorithm for generating unique field names with conflict resolution
pub(crate) fn generate_unique_field_names(query: &plugin::Query) -> Vec<String> {
    // Step 1: Check for column name conflicts (ignoring table prefixes)
    let column_names: Vec<String> = query
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            if !col.name.is_empty() {
                col.name.clone()
            } else {
                format!("column_{}", idx)
            }
        })
        .collect();

    let mut column_name_counts: HashMap<String, usize> = HashMap::new();
    for name in &column_names {
        *column_name_counts.entry(name.clone()).or_insert(0) += 1;
    }

    // Step 2: Generate names based on conflict resolution rules
    let tentative_names: Vec<String> = query
        .columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let col_name = if !col.name.is_empty() {
                col.name.clone()
            } else {
                format!("column_{}", idx)
            };

            let col_count = column_name_counts.get(&col_name).unwrap_or(&1);

            if *col_count <= 1 {
                // Rule 1: No column name conflicts - use column name only
                col_name
            } else {
                // Rule 2: Column name conflicts - use table_column format
                if let Some(prefix) = get_field_prefix(col) {
                    format!("{}_{}", prefix, col_name)
                } else {
                    col_name
                }
            }
        })
        .collect();

    // Step 3: Check for conflicts in tentative names and apply Rule 3
    let mut tentative_name_counts: HashMap<String, usize> = HashMap::new();
    for name in &tentative_names {
        *tentative_name_counts.entry(name.clone()).or_insert(0) += 1;
    }

    // Step 4: Add sequential numbers for remaining conflicts
    let mut used_names: HashMap<String, usize> = HashMap::new();
    let final_names: Vec<String> = tentative_names
        .into_iter()
        .map(|name| {
            let count = tentative_name_counts.get(&name).unwrap_or(&1);
            if *count <= 1 {
                // No conflict in tentative names
                name
            } else {
                // Rule 3: Table+column conflicts - add sequential numbers
                let counter = used_names.entry(name.clone()).or_insert(0);
                *counter += 1;
                format!("{}_{}", name, counter)
            }
        })
        .map(|name| crate::utils::rust_struct_field(&name))
        .collect();

    final_names
}

/// Generates unique parameter names for SQL query parameters
pub(crate) fn generate_unique_param_names(params: &[(i32, &plugin::Column)]) -> Vec<String> {
    // First pass: generate initial names and count conflicts
    let initial_names: Vec<String> = params
        .iter()
        .map(|(_, col)| {
            if !col.name.is_empty() {
                col.name.clone()
            } else {
                "param".to_string()
            }
        })
        .collect();

    // Count occurrences of each name
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for name in &initial_names {
        *name_counts.entry(name.clone()).or_insert(0) += 1;
    }

    // Second pass: generate unique names for parameters
    let mut name_counters: HashMap<String, usize> = HashMap::new();
    let final_names: Vec<String> = initial_names
        .iter()
        .map(|name| {
            let count = name_counts.get(name).unwrap_or(&1);
            if *count <= 1 {
                // No conflict, use original name
                crate::utils::rust_struct_field(name)
            } else {
                // Conflict detected, append numerical suffix
                let counter = name_counters.entry(name.clone()).or_insert(0);
                *counter += 1;
                let unique_name = format!("{}_{}", name, counter);
                crate::utils::rust_struct_field(&unique_name)
            }
        })
        .collect();

    final_names
}

/// Helper to get column name from generated list with fallback
pub(crate) fn column_name_from_list(field_names: &[String], idx: usize) -> String {
    field_names
        .get(idx)
        .cloned()
        .unwrap_or_else(|| format!("unknown_field_{}", idx))
}