# Struct-Based Query API Design

## Overview

This document outlines the design for migrating from function-based query generation to a struct-based API inspired by `typed-builder` and `cornucopia` patterns. This approach provides better type safety, composability, and modern Rust API ergonomics.

## Current vs Proposed API

### Current Function-Based Approach
```rust
// Generated code
pub async fn get_author(
    client: &impl tokio_postgres::GenericClient, 
    author_id: i32
) -> Result<GetAuthorRow, tokio_postgres::Error> {
    client.query_opt(GET_AUTHOR, &[&author_id]).await?
        .map(|row| GetAuthorRow {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            // ...
        })
        .ok_or_else(|| /* error */)
}

// Usage
let author = get_author(&client, 123).await?;
```

### Proposed Struct-Based Approach
```rust
// Generated code - Custom builder implementation without external dependencies
use std::borrow::Cow;

// Type-level markers for builder state
pub struct Set;
pub struct Unset;

pub struct GetAuthor<AuthorIdState = Unset> {
    author_id: Option<Cow<'static, i32>>,
    name_filter: Option<Cow<'static, str>>,
    _phantom_author_id: std::marker::PhantomData<AuthorIdState>,
}

impl GetAuthor<Unset> {
    pub fn builder() -> Self {
        Self {
            author_id: None,
            name_filter: None,
            _phantom_author_id: std::marker::PhantomData,
        }
    }
}

impl<AuthorIdState> GetAuthor<AuthorIdState> {
    pub fn author_id(self, value: impl Into<Cow<'static, i32>>) -> GetAuthor<Set> {
        GetAuthor {
            author_id: Some(value.into()),
            name_filter: self.name_filter,
            _phantom_author_id: std::marker::PhantomData,
        }
    }
    
    pub fn name_filter(self, value: impl Into<Cow<'static, str>>) -> GetAuthor<AuthorIdState> {
        GetAuthor {
            author_id: self.author_id,
            name_filter: Some(value.into()),
            _phantom_author_id: self._phantom_author_id,
        }
    }
}

// Only allow execution when all required fields are set
impl GetAuthor<Set> {
    pub async fn fetch_one(
        &self, 
        client: &impl tokio_postgres::GenericClient
    ) -> Result<GetAuthorRow, tokio_postgres::Error> {
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &*self.author_id.as_ref().unwrap(),
        ];
        let row = client.query_one(GET_AUTHOR, params).await?;
        Ok(GetAuthorRow::from_row(&row)?)
    }
    
    pub async fn fetch_optional(
        &self, 
        client: &impl tokio_postgres::GenericClient  
    ) -> Result<Option<GetAuthorRow>, tokio_postgres::Error> {
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &*self.author_id.as_ref().unwrap(),
        ];
        if let Some(row) = client.query_opt(GET_AUTHOR, params).await? {
            Ok(Some(GetAuthorRow::from_row(&row)?))
        } else {
            Ok(None)
        }
    }
}

// Usage - Compile-time safety
let author = GetAuthor::builder()
    .author_id(123)  // Required field
    .name_filter("Agatha")  // Optional field
    .fetch_one(&client)  // Only available when all required fields are set
    .await?;

// This won't compile - missing required field
// GetAuthor::builder().fetch_one(&client).await?;  // Compile error!
```

## Design Principles

### 1. Type Safety at Compile Time
- Required parameters enforced by custom generic type system
- Uses phantom types (`Set`/`Unset`) to track field completion state
- Compile-time checks prevent execution with missing required fields
- No external dependencies - only uses `std::borrow::Cow` and `std::marker::PhantomData`

### 2. Method-Based Execution
- `fetch_one()`: Returns single row, errors if not found
- `fetch_optional()`: Returns `Option<Row>`, None if not found
- `fetch_all()`: Returns `Vec<Row>` for multiple results
- `execute()`: For INSERT/UPDATE/DELETE, returns affected row count

### 3. Cow-Based Parameter Handling
- Parameters accept both owned values and references via `impl Into<Cow<'static, T>>`
- Efficient memory usage - no unnecessary allocations
- Supports string literals, owned strings, and string references seamlessly

### 4. Query Reusability
- Query structs can be stored and reused once all required fields are set
- Immutable builder pattern ensures thread safety
- Supports batching and streaming operations

## Implementation Architecture

### 1. Code Generation Pipeline

```rust
// query/postgres_query.rs
impl PostgresQuery {
    pub(crate) fn generate_struct_based(&self) -> crate::Result<TokenStream> {
        quote! {
            #(self.generate_type_markers())?
            #(self.generate_query_struct())?
            #(self.generate_builder_impl())?
            #(self.generate_setter_impls())?
            #(self.generate_execution_impl())?
        }
    }
    
    fn generate_type_markers(&self) -> TokenStream {
        quote! {
            pub struct Set;
            pub struct Unset;
        }
    }
    
    fn generate_query_struct(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let params = self.query_params.to_phantom_fields();
        let fields = self.query_params.to_cow_fields();
        
        quote! {
            pub struct #struct_name<#(#params = Unset),*> {
                #(#fields),*
                #(#self.query_params.to_phantom_markers()),*
            }
        }
    }
    
    fn generate_builder_impl(&self) -> TokenStream {
        let struct_name = self.struct_name();
        
        quote! {
            impl #struct_name<#(Unset),*> {
                pub fn builder() -> Self {
                    Self {
                        #(#self.query_params.to_none_initializers()),*
                        #(#self.query_params.to_phantom_initializers()),*
                    }
                }
            }
        }
    }
    
    fn generate_setter_impls(&self) -> TokenStream {
        // Generate setter methods that transition type states
        self.query_params.to_setter_methods(&self.struct_name())
    }
    
    fn generate_execution_impl(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let required_bounds = self.query_params.to_required_type_bounds();
        let methods = match self.query_type {
            QueryAnnotation::One => self.generate_fetch_methods(),
            QueryAnnotation::Many => self.generate_fetch_all_method(),
            QueryAnnotation::Exec => self.generate_execute_method(),
        };
        
        quote! {
            impl #struct_name<#(#required_bounds),*> {
                #methods
            }
        }
    }
}
```

### 2. Parameter Builder Generation

```rust
// rust_gen/param_builder.rs
impl PgParams {
    fn to_cow_fields(&self) -> Vec<TokenStream> {
        self.params.iter().map(|p| {
            let field_name = Ident::new(&p.inner.name, Span::call_site());
            let field_type = &p.wrap_type();
            
            quote! {
                #field_name: Option<std::borrow::Cow<'static, #field_type>>
            }
        }).collect()
    }
    
    fn to_phantom_fields(&self) -> Vec<TokenStream> {
        self.params.iter()
            .filter(|p| !p.inner.is_nullable) // Only required fields get phantom types
            .map(|p| {
                let type_name = format!("{}State", pascalize(&p.inner.name));
                let ident = Ident::new(&type_name, Span::call_site());
                quote! { #ident }
            })
            .collect()
    }
    
    fn to_phantom_markers(&self) -> Vec<TokenStream> {
        self.params.iter()
            .filter(|p| !p.inner.is_nullable)
            .map(|p| {
                let field_name = format!("_phantom_{}", &p.inner.name);
                let phantom_field = Ident::new(&field_name, Span::call_site());
                let type_name = format!("{}State", pascalize(&p.inner.name));
                let type_ident = Ident::new(&type_name, Span::call_site());
                
                quote! {
                    #phantom_field: std::marker::PhantomData<#type_ident>
                }
            })
            .collect()
    }
    
    fn to_setter_methods(&self, struct_name: &Ident) -> TokenStream {
        let setters: Vec<_> = self.params.iter().map(|p| {
            let field_name = Ident::new(&p.inner.name, Span::call_site());
            let field_type = &p.wrap_type();
            
            if p.inner.is_nullable {
                // Optional field - doesn't change type state
                quote! {
                    pub fn #field_name<T>(self, value: T) -> Self 
                    where 
                        T: Into<std::borrow::Cow<'static, #field_type>>
                    {
                        Self {
                            #field_name: Some(value.into()),
                            ..self
                        }
                    }
                }
            } else {
                // Required field - transitions from Unset to Set
                let type_name = format!("{}State", pascalize(&p.inner.name));
                let state_type = Ident::new(&type_name, Span::call_site());
                
                quote! {
                    pub fn #field_name<T>(self, value: T) -> #struct_name<Set, /* other states */> 
                    where 
                        T: Into<std::borrow::Cow<'static, #field_type>>
                    {
                        // Complex type state transition logic here
                        // This would need to be generated based on parameter combinations
                    }
                }
            }
        }).collect();
        
        quote! {
            #(#setters)*
        }
    }
    
    fn to_required_type_bounds(&self) -> Vec<TokenStream> {
        self.params.iter()
            .filter(|p| !p.inner.is_nullable)
            .map(|_| quote! { Set })
            .collect()
    }
    
    fn to_query_params(&self) -> TokenStream {
        let param_refs: Vec<_> = self.params.iter()
            .map(|p| {
                let field = Ident::new(&p.inner.name, Span::call_site());
                quote! { 
                    &*self.#field.as_ref().unwrap() 
                }
            })
            .collect();
            
        quote! { 
            &[#(#param_refs as &(dyn tokio_postgres::types::ToSql + Sync)),*] 
        }
    }
}
```

### 3. Database Client Abstraction

```rust
// No external traits needed - direct implementation per DB crate
impl GetAuthor<Set> {  // Only when all required fields are set
    pub async fn fetch_one(
        &self,
        client: &impl tokio_postgres::GenericClient
    ) -> Result<GetAuthorRow, tokio_postgres::Error> {
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &*self.author_id.as_ref().unwrap(),
        ];
        let row = client.query_one(GET_AUTHOR, params).await?;
        Ok(GetAuthorRow {
            id: row.try_get(0)?,
            name: row.try_get(1)?,
            // ... other fields
        })
    }
    
    pub async fn fetch_optional(
        &self,
        client: &impl tokio_postgres::GenericClient
    ) -> Result<Option<GetAuthorRow>, tokio_postgres::Error> {
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &*self.author_id.as_ref().unwrap(),
        ];
        if let Some(row) = client.query_opt(GET_AUTHOR, params).await? {
            Ok(Some(GetAuthorRow {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                // ... other fields
            }))
        } else {
            Ok(None)
        }
    }
}

// For multi-parameter queries
impl SearchBooks<TitleState, AuthorState> {
    pub fn title<T>(self, value: T) -> SearchBooks<Set, AuthorState>
    where
        T: Into<std::borrow::Cow<'static, str>>
    {
        SearchBooks {
            title: Some(value.into()),
            author_name: self.author_name,
            _phantom_title: std::marker::PhantomData,
            _phantom_author: self._phantom_author,
        }
    }
    
    pub fn author_name<T>(self, value: T) -> SearchBooks<TitleState, Set>
    where
        T: Into<std::borrow::Cow<'static, str>>
    {
        SearchBooks {
            title: self.title,
            author_name: Some(value.into()),
            _phantom_title: self._phantom_title,
            _phantom_author: std::marker::PhantomData,
        }
    }
}

// Only allow execution when all required parameters are set
impl SearchBooks<Set, Set> {
    pub async fn fetch_all(
        &self,
        client: &impl tokio_postgres::GenericClient
    ) -> Result<Vec<SearchBooksRow>, tokio_postgres::Error> {
        let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
            &*self.title.as_ref().unwrap(),
            &*self.author_name.as_ref().unwrap(),
        ];
        let rows = client.query(SEARCH_BOOKS, params).await?;
        rows.into_iter()
            .map(|row| Ok(SearchBooksRow {
                id: row.try_get(0)?,
                title: row.try_get(1)?,
                author: row.try_get(2)?,
            }))
            .collect()
    }
}
```

## Advanced Features

### 1. Query Composition
```rust
// Complex queries with multiple optional filters
// Only required fields need to be set for compilation
let query = SearchAuthors::builder()
    .name_pattern("%Agatha%")  // Required parameter
    .country("England")        // Optional parameter
    .birth_year_after(1800);   // Optional parameter

let authors = query.fetch_all(&client).await?;

// Memory efficient with Cow
let pattern = String::from("%Agatha%");
let query = SearchAuthors::builder()
    .name_pattern(&pattern)    // Borrows from pattern
    .country("England");       // Uses string literal

let authors = query.fetch_all(&client).await?;
```

### 2. Batch Operations
```rust
// Reuse query structure for batch processing
for author_data in author_list {
    let query = InsertAuthor::builder()
        .name(&author_data.name)     // Borrows from author_data
        .country(&author_data.country);
    
    query.execute(&client).await?;
}

// Or with owned values
let queries: Vec<_> = author_list.into_iter()
    .map(|data| InsertAuthor::builder()
        .name(data.name)         // Takes ownership
        .country(data.country))
    .collect();

for query in queries {
    query.execute(&client).await?;
}
```

### 3. Type Safety Examples
```rust
// This compiles - all required fields set
let query = SearchBooks::builder()
    .title("1984")
    .author_name("George Orwell");
let books = query.fetch_all(&client).await?;

// This doesn't compile - missing required field
// let query = SearchBooks::builder().title("1984");
// let books = query.fetch_all(&client).await?;  // Compile error!

// This compiles - optional fields can be omitted
let query = GetAuthor::builder()
    .author_id(123);  // Required field
    // name_filter is optional
let author = query.fetch_one(&client).await?;
```

## Migration Strategy

### Phase 1: Dual API Generation
- Generate both function-based and struct-based APIs
- Existing code continues to work unchanged
- New code can opt into struct-based API

### Phase 2: Deprecation Period
- Mark function-based API as deprecated
- Provide migration guide and tooling
- Update examples and documentation

### Phase 3: Full Migration
- Remove function-based API generation
- Clean up deprecated code paths
- Optimize generated code size

### Backward Compatibility Bridge
```rust
// Keep existing function API, implement using struct API
pub async fn get_author(
    client: &impl tokio_postgres::GenericClient,
    author_id: i32
) -> Result<GetAuthorRow, tokio_postgres::Error> {
    GetAuthor::builder()
        .author_id(author_id)
        .build()
        .fetch_one(client)
        .await
}
```

## Configuration Options

### Plugin Configuration
```json
{
  "db_crate": "tokio_postgres",
  "api_style": "struct", // "function" or "struct" or "both"
  "struct_derives": ["Debug", "Clone", "PartialEq"],
  "enable_streaming": true,
  "enable_batch_operations": true,
  "typed_builder_options": {
    "into": true,
    "strip_option": true
  }
}
```

### Generated Code Customization
```rust
// Optional features can be configured
#[derive(TypedBuilder, Debug, Clone)]
#[builder(build_method(into = Result<Self, ValidationError>))]
pub struct ComplexQuery {
    #[builder(setter(validate = validate_positive))]
    pub id: i32,
    
    #[builder(default, setter(strip_option, into))]
    pub name: Option<String>,
}
```

## Benefits

### Developer Experience
- **Type Safety**: Compile-time parameter validation
- **IDE Support**: Better autocomplete and error detection
- **Composability**: Query objects can be stored, reused, and modified
- **Testing**: Easier to mock and test query structures

### Performance
- **Zero-Cost Abstractions**: No runtime overhead from builder pattern
- **Query Reuse**: Prepared statements can be cached more effectively
- **Memory Efficiency**: No intermediate string allocations

### Maintainability
- **Consistent API**: All queries follow same pattern
- **Evolution Path**: Easy to add new execution methods
- **Documentation**: Self-documenting through type system

## Implementation Roadmap

1. **Week 1-2**: Core struct generation and typed-builder integration
2. **Week 3-4**: Execution method generation for all DB crates
3. **Week 5-6**: Advanced features (streaming, batch operations)
4. **Week 7-8**: Migration tooling and documentation
5. **Week 9-10**: Testing, optimization, and refinement

## Success Criteria

- [ ] Generate type-safe query structs with compile-time validation
- [ ] Support all existing query patterns (:one, :many, :exec)
- [ ] Maintain performance parity with function-based approach
- [ ] Provide smooth migration path from existing API
- [ ] Support all currently supported DB crates
- [ ] Comprehensive test coverage and documentation