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

    /// Generate the complete type-state builder pattern
    pub(crate) fn generate_builder(
        &self,
        query_params: &PgParams,
        type_map: &impl TypeMap,
    ) -> TokenStream {
        if query_params.params.is_empty() {
            return quote! {};
        }

        // For now, generate a simpler non-type-state builder to avoid syntax errors
        self.generate_simple_builder(query_params, type_map)
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
}
