use crate::rust_gen::param_gen::PgParams;
use crate::user_type::TypeMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Type-state builder pattern generator for zero-cost query construction
/// This is a foundation for future typed-builder pattern implementation
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct PostgresBuilderGen {
    query_name: String,
}

#[allow(dead_code)]
impl PostgresBuilderGen {
    pub(crate) fn new(query_name: String) -> Self {
        Self { query_name }
    }

    /// Generate the complete builder pattern (type-state or Option-based)
    pub(crate) fn generate_builder(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        if query_params.params.is_empty() {
            return quote! {};
        }

        // Decide between type-state and Option-based builder
        if self.should_use_type_state_builder(query_params, type_map) {
            self.generate_type_state_builder(query_params, type_map)
        } else {
            self.generate_simple_builder(query_params, type_map)
        }
    }

    /// Determine if type-state builder should be used based on constraints
    fn should_use_type_state_builder(
        &self,
        query_params: &PgParams,
        _type_map: &impl TypeMap,
    ) -> bool {
        // Type-state builder constraints
        let param_count = query_params.params.len();

        // Phase 6: Enable type-state for up to 16 copy parameters, including nullable  
        // (Rust tuple limit is much higher, but we set practical limit for readability)
        param_count <= 16
            && query_params
                .params
                .iter()
                .all(|p| p.is_copy_cheap_type(_type_map))
    }

    /// Generate type-state builder pattern with compile-time safety
    fn generate_type_state_builder(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let _struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate initial type state (all (), meaning unset)
        let initial_elements: Vec<TokenStream> = (0..param_count)
            .map(|_| quote! { () })
            .collect();
        let initial_tuple_type = self.generate_dynamic_tuple_type(&initial_elements);

        let lifetime_param = if has_lifetime {
            quote! { <'a, Fields = #initial_tuple_type> }
        } else {
            quote! { <Fields = #initial_tuple_type> }
        };

        let phantom_type = if has_lifetime {
            quote! { std::marker::PhantomData<&'a ()> }
        } else {
            quote! { std::marker::PhantomData<()> }
        };

        // Generate builder struct
        let builder_struct = quote! {
            #[derive(Debug)]
            pub struct #builder_ident #lifetime_param {
                fields: Fields,
                phantom: #phantom_type,
            }
        };

        // Generate constructor method
        let constructor_method = self.generate_type_state_constructor(query_params, type_map);

        // Generate setter methods for each parameter
        let setter_methods = self.generate_type_state_setters(query_params, type_map);

        // Generate build method (only when all fields are set)
        let build_method = self.generate_type_state_build(query_params, type_map);

        quote! {
            #builder_struct
            #constructor_method
            #setter_methods
            #build_method
        }
    }

    /// Generate a simpler builder pattern without complex type-state
    fn generate_simple_builder(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate fields for the builder struct
        let builder_fields: Vec<TokenStream> = query_params
            .params
            .iter()
            .map(|param| {
                let field_name = Ident::new(&param.inner.name, proc_macro2::Span::call_site());
                if param.is_copy_cheap_type(type_map) {
                    let rs_type = &param.inner.rs_type;
                    if param.inner.is_nullable {
                        quote! { #field_name: Option<Option<#rs_type>> }
                    } else {
                        quote! { #field_name: Option<#rs_type> }
                    }
                } else {
                    let base_type = param.wrap_type();
                    if param.inner.is_nullable {
                        if has_lifetime {
                            quote! { #field_name: Option<Option<std::borrow::Cow<'a, #base_type>>> }
                        } else {
                            quote! { #field_name: Option<Option<#base_type>> }
                        }
                    } else if has_lifetime {
                        quote! { #field_name: Option<std::borrow::Cow<'a, #base_type>> }
                    } else {
                        quote! { #field_name: Option<#base_type> }
                    }
                }
            })
            .collect();

        let lifetime_param = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        // Generate setter methods
        let setter_methods: Vec<TokenStream> = query_params
            .params
            .iter()
            .map(|param| {
                let method_name = Ident::new(&param.inner.name, proc_macro2::Span::call_site());
                let field_name = Ident::new(&param.inner.name, proc_macro2::Span::call_site());

                if param.is_copy_cheap_type(type_map) {
                    let param_type = if param.inner.is_nullable {
                        let rs_type = &param.inner.rs_type;
                        quote! { Option<#rs_type> }
                    } else {
                        let rs_type = &param.inner.rs_type;
                        quote! { #rs_type }
                    };

                    quote! {
                        pub fn #method_name(mut self, #method_name: #param_type) -> Self {
                            self.#field_name = Some(#method_name);
                            self
                        }
                    }
                } else {
                    let param_type = if param.inner.is_nullable {
                        if has_lifetime {
                            let base_type = param.wrap_type();
                            quote! { Option<std::borrow::Cow<'a, #base_type>> }
                        } else {
                            let rs_type = &param.inner.rs_type;
                            quote! { Option<#rs_type> }
                        }
                    } else if has_lifetime {
                        let base_type = param.wrap_type();
                        quote! { std::borrow::Cow<'a, #base_type> }
                    } else {
                        let rs_type = &param.inner.rs_type;
                        quote! { #rs_type }
                    };

                    quote! {
                        pub fn #method_name<T>(mut self, #method_name: T) -> Self
                        where T: Into<#param_type>
                        {
                            self.#field_name = Some(#method_name.into());
                            self
                        }
                    }
                }
            })
            .collect();

        // Generate build method
        let build_fields: Vec<TokenStream> = query_params
            .params
            .iter()
            .map(|param| {
                let field_name = Ident::new(&param.inner.name, proc_macro2::Span::call_site());
                quote! {
                    #field_name: self.#field_name.expect("Missing required field")
                }
            })
            .collect();

        let return_type = if has_lifetime {
            quote! { #struct_ident<'a> }
        } else {
            quote! { #struct_ident }
        };

        quote! {
            #[derive(Debug, Default)]
            pub struct #builder_ident #lifetime_param {
                #(#builder_fields,)*
            }

            impl #lifetime_param #struct_ident #lifetime_param {
                pub fn builder() -> #builder_ident #lifetime_param {
                    #builder_ident::default()
                }
            }

            impl #lifetime_param #builder_ident #lifetime_param {
                #(#setter_methods)*

                pub fn build(self) -> #return_type {
                    #struct_ident {
                        #(#build_fields,)*
                    }
                }
            }
        }
    }

    /// Generate the builder struct with type-state parameters
    fn generate_builder_struct(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate initial type state (all (), meaning unset)
        let initial_fields_type = self.generate_tuple_type(&vec![quote! { () }; param_count]);

        let lifetime_param = if has_lifetime {
            quote! { <'a, Fields = #initial_fields_type> }
        } else {
            quote! { <Fields = #initial_fields_type> }
        };

        let phantom_type = if has_lifetime {
            quote! { std::marker::PhantomData<&'a ()> }
        } else {
            quote! { std::marker::PhantomData<()> }
        };

        quote! {
            #[derive(Debug)]
            pub struct #builder_ident #lifetime_param {
                fields: Fields,
                phantom: #phantom_type,
            }
        }
    }

    /// Generate builder constructor method on the main struct
    fn generate_constructor_method(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate initial tuple of () values
        let initial_tuple = self.generate_tuple_value(&vec![quote! { () }; param_count]);
        let initial_fields_type = self.generate_tuple_type(&vec![quote! { () }; param_count]);

        let lifetime_param = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let return_type = if has_lifetime {
            quote! { #builder_ident<'a, #initial_fields_type> }
        } else {
            quote! { #builder_ident<#initial_fields_type> }
        };

        quote! {
            impl #lifetime_param #struct_ident #lifetime_param {
                pub fn builder() -> #return_type {
                    #builder_ident {
                        fields: #initial_tuple,
                        phantom: std::marker::PhantomData,
                    }
                }
            }
        }
    }

    /// Generate setter methods for each parameter
    fn generate_builder_methods(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        let mut methods = quote! {};

        for (param_index, param) in query_params.params.iter().enumerate() {
            let method_ident = Ident::new(&param.inner.name, proc_macro2::Span::call_site());

            // Generate type state for this specific parameter position
            let before_types = self.generate_generic_types_for_position(
                param_count,
                param_index,
                false,
                param,
                type_map,
            );
            let after_types = self.generate_generic_types_for_position(
                param_count,
                param_index,
                true,
                param,
                type_map,
            );

            // Determine parameter type based on copy_types optimization
            let param_type = if param.is_copy_cheap_type(type_map) {
                let rs_type = &param.inner.rs_type;
                if param.inner.is_nullable {
                    quote! { Option<#rs_type> }
                } else {
                    quote! { #rs_type }
                }
            } else {
                let _base_type = param.wrap_type();
                if param.inner.is_nullable {
                    quote! { Option<T> }
                } else {
                    quote! { T }
                }
            };

            // Generate where clause for Into<Cow> for non-copy types (currently unused)
            let (_where_clause, _value_conversion) = if param.is_copy_cheap_type(type_map) {
                (quote! {}, quote! { #method_ident })
            } else if has_lifetime {
                let base_type = param.wrap_type();
                if param.inner.is_nullable {
                    (
                        quote! { where T: Into<Option<std::borrow::Cow<'a, #base_type>>> },
                        quote! { #method_ident.into() },
                    )
                } else {
                    (
                        quote! { where T: Into<std::borrow::Cow<'a, #base_type>> },
                        quote! { #method_ident.into() },
                    )
                }
            } else {
                // For non-lifetime cases, use direct type reference
                let rs_type = &param.inner.rs_type;
                if param.inner.is_nullable {
                    (
                        quote! { where T: Into<Option<#rs_type>> },
                        quote! { #method_ident.into() },
                    )
                } else {
                    (
                        quote! { where T: Into<#rs_type> },
                        quote! { #method_ident.into() },
                    )
                }
            };

            // Generate destructuring pattern for current state
            let destructure_pattern = self.generate_destructure_pattern(param_count, param_index);
            let param_value = if param.is_copy_cheap_type(type_map) {
                let ident = Ident::new(&param.inner.name, proc_macro2::Span::call_site());
                quote! { #ident }
            } else {
                quote! { converted_value }
            };
            let reconstruct_pattern =
                self.generate_reconstruct_pattern_with_value(param_count, param_index, param_value);

            let before_fields_type = self.generate_tuple_type(&before_types);
            let after_fields_type = self.generate_tuple_type(&after_types);

            let lifetime_bounds = if has_lifetime {
                quote! { <'a, #before_fields_type> }
            } else {
                quote! { <#before_fields_type> }
            };

            let return_type = if has_lifetime {
                quote! { #builder_ident<'a, #after_fields_type> }
            } else {
                quote! { #builder_ident<#after_fields_type> }
            };

            let method = if param.is_copy_cheap_type(type_map) {
                quote! {
                    impl #lifetime_bounds #builder_ident #lifetime_bounds {
                        pub fn #method_ident(self, #method_ident: #param_type) -> #return_type {
                            let #destructure_pattern = self.fields;
                            #builder_ident {
                                fields: #reconstruct_pattern,
                                phantom: std::marker::PhantomData,
                            }
                        }
                    }
                }
            } else {
                // For non-copy types, use specific type constraints instead of generics
                let actual_param_type = if param.inner.is_nullable {
                    if has_lifetime {
                        let base_type = param.wrap_type();
                        quote! { Option<std::borrow::Cow<'a, #base_type>> }
                    } else {
                        let rs_type = &param.inner.rs_type;
                        quote! { Option<#rs_type> }
                    }
                } else if has_lifetime {
                    let base_type = param.wrap_type();
                    quote! { std::borrow::Cow<'a, #base_type> }
                } else {
                    let rs_type = &param.inner.rs_type;
                    quote! { #rs_type }
                };

                quote! {
                    impl #lifetime_bounds #builder_ident #lifetime_bounds {
                        pub fn #method_ident<T>(self, #method_ident: T) -> #return_type
                        where T: Into<#actual_param_type>
                        {
                            let #destructure_pattern = self.fields;
                            let converted_value = #method_ident.into();
                            #builder_ident {
                                fields: #reconstruct_pattern,
                                phantom: std::marker::PhantomData,
                            }
                        }
                    }
                }
            };

            methods.extend(method);
        }

        methods
    }

    /// Generate the build method (only available when all fields are set)
    fn generate_build_method(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate complete type state (all fields set)
        let complete_types: Vec<TokenStream> = query_params
            .params
            .iter()
            .map(|param| {
                if param.is_copy_cheap_type(type_map) {
                    let rs_type = &param.inner.rs_type;
                    if param.inner.is_nullable {
                        quote! { Option<#rs_type> }
                    } else {
                        quote! { #rs_type }
                    }
                } else {
                    let base_type = param.wrap_type();
                    if param.inner.is_nullable {
                        quote! { Option<std::borrow::Cow<'a, #base_type>> }
                    } else {
                        quote! { std::borrow::Cow<'a, #base_type> }
                    }
                }
            })
            .collect();

        let complete_fields_type = self.generate_tuple_type(&complete_types);

        // Generate destructuring for all fields
        let field_names: Vec<Ident> = query_params
            .params
            .iter()
            .map(|param| Ident::new(&param.inner.name, proc_macro2::Span::call_site()))
            .collect();

        let destructure_all = if param_count == 1 {
            let field = &field_names[0];
            quote! { #field }
        } else {
            quote! { (#(#field_names),*) }
        };

        let lifetime_bounds = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let builder_type = if has_lifetime {
            quote! { #builder_ident<'a, #complete_fields_type> }
        } else {
            quote! { #builder_ident<#complete_fields_type> }
        };

        let return_type = if has_lifetime {
            quote! { #struct_ident<'a> }
        } else {
            quote! { #struct_ident }
        };

        quote! {
            impl #lifetime_bounds #builder_type {
                pub fn build(self) -> #return_type {
                    let #destructure_all = self.fields;
                    #struct_ident {
                        #(#field_names),*
                    }
                }
            }
        }
    }

    fn query_struct_ident(&self) -> Ident {
        Ident::new(&self.query_name, proc_macro2::Span::call_site())
    }

    fn builder_struct_ident(&self) -> Ident {
        let builder_name = format!("{}Builder", self.query_name);
        Ident::new(&builder_name, proc_macro2::Span::call_site())
    }

    fn needs_lifetime(&self, query_params: &PgParams, type_map: &impl TypeMap) -> bool {
        query_params
            .params
            .iter()
            .any(|param| !param.is_copy_cheap_type(type_map))
    }

    fn generate_tuple_type(&self, types: &[TokenStream]) -> TokenStream {
        match types.len() {
            0 => quote! { () },
            1 => types[0].clone(),
            _ => quote! { (#(#types),*) },
        }
    }

    fn generate_tuple_value(&self, values: &[TokenStream]) -> TokenStream {
        match values.len() {
            0 => quote! { () },
            1 => values[0].clone(),
            _ => quote! { (#(#values),*) },
        }
    }

    /// Generate a tuple type with dynamic number of elements
    fn generate_dynamic_tuple_type(&self, elements: &[TokenStream]) -> TokenStream {
        match elements.len() {
            0 => quote! { () },
            1 => elements[0].clone(),
            _ => quote! { (#(#elements),*) },
        }
    }

    /// Generate a tuple value with dynamic number of elements
    fn generate_dynamic_tuple_value(&self, elements: &[TokenStream]) -> TokenStream {
        match elements.len() {
            0 => quote! { () },
            1 => elements[0].clone(),
            _ => quote! { (#(#elements),*) },
        }
    }

    /// Generate a destructuring pattern with dynamic number of elements
    fn generate_dynamic_destructure_pattern(&self, patterns: &[TokenStream]) -> TokenStream {
        match patterns.len() {
            0 => quote! { () },
            1 => patterns[0].clone(),
            _ => quote! { (#(#patterns),*) },
        }
    }

    fn generate_generic_types_for_position(
        &self,
        total_count: usize,
        position: usize,
        is_set: bool,
        param: &crate::db_support::PgColumnRef,
        type_map: &impl TypeMap,
    ) -> Vec<TokenStream> {
        (0..total_count)
            .map(|i| {
                if i == position && is_set {
                    // This position is being set - use the actual parameter type
                    if param.is_copy_cheap_type(type_map) {
                        let rs_type = &param.inner.rs_type;
                        if param.inner.is_nullable {
                            quote! { Option<#rs_type> }
                        } else {
                            quote! { #rs_type }
                        }
                    } else {
                        let base_type = param.wrap_type();
                        if param.inner.is_nullable {
                            quote! { Option<std::borrow::Cow<'a, #base_type>> }
                        } else {
                            quote! { std::borrow::Cow<'a, #base_type> }
                        }
                    }
                } else if i == position {
                    // This position is unset
                    quote! { () }
                } else {
                    // Other positions use generic type variables
                    let var_name = format!("V{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect()
    }

    fn generate_destructure_pattern(
        &self,
        total_count: usize,
        setting_position: usize,
    ) -> TokenStream {
        let vars: Vec<TokenStream> = (0..total_count)
            .map(|i| {
                if i == setting_position {
                    quote! { () }
                } else {
                    let var_name = format!("v{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        match vars.len() {
            1 => vars[0].clone(),
            _ => quote! { (#(#vars),*) },
        }
    }

    fn generate_reconstruct_pattern(
        &self,
        total_count: usize,
        setting_position: usize,
        param_name: &str,
    ) -> TokenStream {
        let ident = Ident::new(param_name, proc_macro2::Span::call_site());
        let param_value = quote! { #ident };
        self.generate_reconstruct_pattern_with_value(total_count, setting_position, param_value)
    }

    fn generate_reconstruct_pattern_with_value(
        &self,
        total_count: usize,
        setting_position: usize,
        param_value: TokenStream,
    ) -> TokenStream {
        let vars: Vec<TokenStream> = (0..total_count)
            .map(|i| {
                if i == setting_position {
                    param_value.clone()
                } else {
                    let var_name = format!("v{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        match vars.len() {
            1 => vars[0].clone(),
            _ => quote! { (#(#vars),*) },
        }
    }

    /// Generate constructor method for type-state builder
    fn generate_type_state_constructor(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate initial tuple value (all (), meaning unset)
        let initial_tuple_value = match param_count {
            1 => quote! { () },
            2 => quote! { ((), ()) },
            3 => quote! { ((), (), ()) },
            4 => quote! { ((), (), (), ()) },
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        };

        let initial_tuple_type = initial_tuple_value.clone();

        let lifetime_param = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let return_type = if has_lifetime {
            quote! { #builder_ident<'a, #initial_tuple_type> }
        } else {
            quote! { #builder_ident<#initial_tuple_type> }
        };

        quote! {
            impl #lifetime_param #struct_ident #lifetime_param {
                pub fn builder() -> #return_type {
                    #builder_ident {
                        fields: #initial_tuple_value,
                        phantom: std::marker::PhantomData,
                    }
                }
            }
        }
    }

    /// Generate setter methods for type-state builder
    fn generate_type_state_setters(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        let mut methods = quote! {};

        for (param_index, param) in query_params.params.iter().enumerate() {
            let method_ident = Ident::new(&param.inner.name, proc_macro2::Span::call_site());

            // Generate parameter type
            let param_type = if param.is_copy_cheap_type(type_map) {
                let rs_type = &param.inner.rs_type;
                if param.inner.is_nullable {
                    quote! { Option<#rs_type> }
                } else {
                    quote! { #rs_type }
                }
            } else if has_lifetime {
                let base_type = param.wrap_type();
                if param.inner.is_nullable {
                    quote! { Option<std::borrow::Cow<'a, #base_type>> }
                } else {
                    quote! { std::borrow::Cow<'a, #base_type> }
                }
            } else {
                let rs_type = &param.inner.rs_type;
                quote! { #rs_type }
            };

            // Generate type states: before and after setting this parameter
            let before_state = self.generate_type_state_signature_before(param_count, param_index);
            let after_state =
                self.generate_type_state_signature_after(param_count, param_index, &param_type);

            // Generate type parameters for generic variables (V0, V1, etc.)
            let type_params: Vec<TokenStream> = (0..param_count)
                .filter(|&i| i != param_index) // Exclude the parameter we're setting
                .map(|i| {
                    let var_name = format!("V{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();

            let lifetime_and_type_bounds = if has_lifetime {
                if type_params.is_empty() {
                    quote! { <'a> }
                } else {
                    quote! { <'a, #(#type_params),*> }
                }
            } else if type_params.is_empty() {
                quote! {}
            } else {
                quote! { <#(#type_params),*> }
            };

            let return_type = if has_lifetime {
                quote! { #builder_ident<'a, #after_state> }
            } else {
                quote! { #builder_ident<#after_state> }
            };

            // Generate destructuring and reconstruction for tuple
            let (destructure_pattern, reconstruct_pattern) =
                self.generate_type_state_patterns(param_count, param_index, &param.inner.name);

            let builder_type_bounds = if has_lifetime {
                quote! { <'a, #before_state> }
            } else {
                quote! { <#before_state> }
            };

            let method = quote! {
                impl #lifetime_and_type_bounds #builder_ident #builder_type_bounds {
                    pub fn #method_ident(self, #method_ident: #param_type) -> #return_type {
                        let #destructure_pattern = self.fields;
                        #builder_ident {
                            fields: #reconstruct_pattern,
                            phantom: std::marker::PhantomData,
                        }
                    }
                }
            };

            methods.extend(method);
        }

        methods
    }

    /// Generate build method for type-state builder (only when all fields are set)
    fn generate_type_state_build(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        let struct_ident = self.query_struct_ident();
        let builder_ident = self.builder_struct_ident();
        let param_count = query_params.params.len();
        let has_lifetime = self.needs_lifetime(query_params, type_map);

        // Generate complete type state (all fields set with actual types)
        let complete_types: Vec<TokenStream> = query_params
            .params
            .iter()
            .map(|param| {
                if param.is_copy_cheap_type(type_map) {
                    let rs_type = &param.inner.rs_type;
                    if param.inner.is_nullable {
                        quote! { Option<#rs_type> }
                    } else {
                        quote! { #rs_type }
                    }
                } else if has_lifetime {
                    let base_type = param.wrap_type();
                    if param.inner.is_nullable {
                        quote! { Option<std::borrow::Cow<'a, #base_type>> }
                    } else {
                        quote! { std::borrow::Cow<'a, #base_type> }
                    }
                } else {
                    let rs_type = &param.inner.rs_type;
                    quote! { #rs_type }
                }
            })
            .collect();

        let complete_state = match param_count {
            1 => complete_types[0].clone(),
            2 => {
                let t0 = &complete_types[0];
                let t1 = &complete_types[1];
                quote! { (#t0, #t1) }
            }
            3 => {
                let t0 = &complete_types[0];
                let t1 = &complete_types[1];
                let t2 = &complete_types[2];
                quote! { (#t0, #t1, #t2) }
            }
            4 => {
                let t0 = &complete_types[0];
                let t1 = &complete_types[1];
                let t2 = &complete_types[2];
                let t3 = &complete_types[3];
                quote! { (#t0, #t1, #t2, #t3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        };

        // Generate field extraction from complete tuple
        let field_names: Vec<Ident> = query_params
            .params
            .iter()
            .map(|param| Ident::new(&param.inner.name, proc_macro2::Span::call_site()))
            .collect();

        let destructure_all = match param_count {
            1 => {
                let field = &field_names[0];
                quote! { #field }
            }
            2 => {
                let f0 = &field_names[0];
                let f1 = &field_names[1];
                quote! { (#f0, #f1) }
            }
            3 => {
                let f0 = &field_names[0];
                let f1 = &field_names[1];
                let f2 = &field_names[2];
                quote! { (#f0, #f1, #f2) }
            }
            4 => {
                let f0 = &field_names[0];
                let f1 = &field_names[1];
                let f2 = &field_names[2];
                let f3 = &field_names[3];
                quote! { (#f0, #f1, #f2, #f3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        };

        let lifetime_bounds = if has_lifetime {
            quote! { <'a> }
        } else {
            quote! {}
        };

        let return_type = if has_lifetime {
            quote! { #struct_ident<'a> }
        } else {
            quote! { #struct_ident }
        };

        quote! {
            impl #lifetime_bounds #builder_ident<#lifetime_bounds #complete_state> {
                pub fn build(self) -> #return_type {
                    let #destructure_all = self.fields;
                    #struct_ident {
                        #(#field_names),*
                    }
                }
            }
        }
    }

    /// Generate type signature before setting a parameter (with () at setting_index)
    fn generate_type_state_signature_before(
        &self,
        param_count: usize,
        setting_index: usize,
    ) -> TokenStream {
        let types: Vec<TokenStream> = (0..param_count)
            .map(|i| {
                if i == setting_index {
                    quote! { () } // Unset parameter
                } else {
                    // Use generic type variables for other positions
                    let var_name = format!("V{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        match param_count {
            1 => types[0].clone(),
            2 => {
                let t0 = &types[0];
                let t1 = &types[1];
                quote! { (#t0, #t1) }
            }
            3 => {
                let t0 = &types[0];
                let t1 = &types[1];
                let t2 = &types[2];
                quote! { (#t0, #t1, #t2) }
            }
            4 => {
                let t0 = &types[0];
                let t1 = &types[1];
                let t2 = &types[2];
                let t3 = &types[3];
                quote! { (#t0, #t1, #t2, #t3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        }
    }

    /// Generate type signature after setting a parameter (with actual type at setting_index)
    fn generate_type_state_signature_after(
        &self,
        param_count: usize,
        setting_index: usize,
        param_type: &TokenStream,
    ) -> TokenStream {
        let types: Vec<TokenStream> = (0..param_count)
            .map(|i| {
                if i == setting_index {
                    param_type.clone() // Set parameter with actual type
                } else {
                    // Use generic type variables for other positions
                    let var_name = format!("V{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        match param_count {
            1 => types[0].clone(),
            2 => {
                let t0 = &types[0];
                let t1 = &types[1];
                quote! { (#t0, #t1) }
            }
            3 => {
                let t0 = &types[0];
                let t1 = &types[1];
                let t2 = &types[2];
                quote! { (#t0, #t1, #t2) }
            }
            4 => {
                let t0 = &types[0];
                let t1 = &types[1];
                let t2 = &types[2];
                let t3 = &types[3];
                quote! { (#t0, #t1, #t2, #t3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        }
    }

    /// Generate destructure and reconstruct patterns for type-state transitions
    fn generate_type_state_patterns(
        &self,
        param_count: usize,
        setting_index: usize,
        param_name: &str,
    ) -> (TokenStream, TokenStream) {
        let field_vars: Vec<TokenStream> = (0..param_count)
            .map(|i| {
                if i == setting_index {
                    quote! { () } // This position is being replaced
                } else {
                    let var_name = format!("v{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        let reconstruct_vars: Vec<TokenStream> = (0..param_count)
            .map(|i| {
                if i == setting_index {
                    // Use the actual method parameter name
                    let method_ident = Ident::new(param_name, proc_macro2::Span::call_site());
                    quote! { #method_ident }
                } else {
                    let var_name = format!("v{}", i);
                    let ident = Ident::new(&var_name, proc_macro2::Span::call_site());
                    quote! { #ident }
                }
            })
            .collect();

        let destructure = match param_count {
            1 => field_vars[0].clone(),
            2 => {
                let v0 = &field_vars[0];
                let v1 = &field_vars[1];
                quote! { (#v0, #v1) }
            }
            3 => {
                let v0 = &field_vars[0];
                let v1 = &field_vars[1];
                let v2 = &field_vars[2];
                quote! { (#v0, #v1, #v2) }
            }
            4 => {
                let v0 = &field_vars[0];
                let v1 = &field_vars[1];
                let v2 = &field_vars[2];
                let v3 = &field_vars[3];
                quote! { (#v0, #v1, #v2, #v3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        };

        let reconstruct = match param_count {
            1 => reconstruct_vars[0].clone(),
            2 => {
                let r0 = &reconstruct_vars[0];
                let r1 = &reconstruct_vars[1];
                quote! { (#r0, #r1) }
            }
            3 => {
                let r0 = &reconstruct_vars[0];
                let r1 = &reconstruct_vars[1];
                let r2 = &reconstruct_vars[2];
                quote! { (#r0, #r1, #r2) }
            }
            4 => {
                let r0 = &reconstruct_vars[0];
                let r1 = &reconstruct_vars[1];
                let r2 = &reconstruct_vars[2];
                let r3 = &reconstruct_vars[3];
                quote! { (#r0, #r1, #r2, #r3) }
            }
            _ => unreachable!("Type-state builder limited to ≤4 parameters"),
        };

        (destructure, reconstruct)
    }
}
