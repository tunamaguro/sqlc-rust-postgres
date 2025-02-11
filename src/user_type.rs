use std::{borrow::Cow, collections::BTreeMap};

use crate::plugin;
use convert_case::{Case, Casing};
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use syn::{Ident, TypePath};

pub(crate) trait GenericEnum {
    fn ident_str(&self) -> String;
    fn ident(&self) -> Ident {
        Ident::new(&self.ident_str(), Span::call_site())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PostgresEnum {
    name: String,
    values: Vec<proc_macro2::TokenStream>,
}

impl PostgresEnum {
    pub(crate) fn new(catalog_enum: &plugin::Enum) -> Self {
        let name = catalog_enum.name.clone();
        let values = catalog_enum
            .vals
            .iter()
            .map(|v| {
                let original_literal = Literal::string(v);
                let ident_str = v.to_case(Case::Pascal);
                let rs_ident = syn::parse_str::<Ident>(&ident_str)
                    .unwrap_or_else(|_| panic!("`{}` is not a valid enum ident", &v));
                quote! {
                    #[postgres(name = #original_literal)]
                    #rs_ident
                }
            })
            .collect();
        Self { name, values }
    }

    pub(crate) fn with_derive(
        &self,
        derive: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        quote! {
            #derive
            #self
        }
    }
}

impl GenericEnum for PostgresEnum {
    fn ident_str(&self) -> String {
        self.name.to_case(Case::Pascal)
    }
}

impl ToTokens for PostgresEnum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let original_literal = Literal::string(&self.name);
        let ident = self.ident();
        let variants = &self.values;

        let tt = quote! {
            #[postgres(name = #original_literal)]
            pub enum #ident {
                #(#variants),*
            }
        };

        tokens.extend(tt);
    }
}

struct PostgresType {
    schema: Option<Cow<'static, str>>,
    name: Cow<'static, str>,
}

impl PostgresType {
    pub(crate) fn pg_catalog(name: impl Into<Cow<'static, str>>) -> Self {
        const PG_CATALOG: &str = "pg_catalog";
        Self {
            schema: Some(PG_CATALOG.into()),
            name: name.into(),
        }
    }

    pub(crate) fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            schema: None,
            name: name.into(),
        }
    }
}

impl std::fmt::Display for PostgresType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(schema) = &self.schema {
            write!(f, "{}.{}", schema, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub(crate) fn col_type(col_t: Option<&plugin::Identifier>) -> String {
    let ident = col_t.expect("column type is not found");
    if ident.schema.is_empty() {
        ident.name.clone()
    } else {
        format!("{}.{}", ident.schema, ident.name)
    }
}

pub(crate) trait TypeMap {
    fn get(&self, column_type: &str) -> Option<&proc_macro2::TokenStream>;
}

pub(crate) struct PgTypeMap {
    m: BTreeMap<String, proc_macro2::TokenStream>,
}

impl TypeMap for PgTypeMap {
    fn get(&self, column_type: &str) -> Option<&proc_macro2::TokenStream> {
        self.m.get(&column_type.to_lowercase())
    }
}

impl PgTypeMap {
    pub(crate) fn new(catalog: &plugin::Catalog) -> Self {
        let mut type_map = Self::default();
        for pg_enum in catalog
            .schemas
            .iter()
            .flat_map(|s| s.enums.as_slice())
            .map(PostgresEnum::new)
        {
            let ident = pg_enum.ident();
            type_map.m.insert(pg_enum.name, ident.to_token_stream());
        }
        type_map
    }
    pub(crate) fn add(&mut self, db_type: &str, rs_type: &str) {
        let rs_type = syn::parse_str::<TypePath>(rs_type)
            .unwrap_or_else(|_| panic!("`{}` is not rust type", rs_type));
        self.m
            .insert(db_type.to_string(), rs_type.to_token_stream());
    }
    fn initialize() -> Self {
        // Map sqlc type and Rust type
        // - https://github.com/sqlc-dev/sqlc/blob/v1.28.0/internal/codegen/golang/postgresql_type.go#L37
        // - https://docs.rs/postgres-types/latest/postgres_types/trait.ToSql.html#types
        // - https://www.postgresql.jp/docs/9.4/datatype.html
        let default_types = [
            (
                vec![
                    PostgresType::new("boolean"),
                    PostgresType::new("bool"),
                    PostgresType::pg_catalog("bool"),
                ],
                "bool",
            ),
            (vec![PostgresType::new(r#""char""#)], "i8"),
            (
                vec![
                    PostgresType::new("smallserial"),
                    PostgresType::new("serial2"),
                    PostgresType::pg_catalog("serial2"),
                    PostgresType::new("smallint"),
                    PostgresType::new("int2"),
                    PostgresType::pg_catalog("int2"),
                ],
                "i16",
            ),
            (
                vec![
                    PostgresType::new("serial"),
                    PostgresType::new("serial4"),
                    PostgresType::pg_catalog("serial4"),
                    PostgresType::new("integer"),
                    PostgresType::new("int"),
                    PostgresType::new("int4"),
                    PostgresType::pg_catalog("int4"),
                ],
                "i32",
            ),
            (
                vec![
                    PostgresType::new("bigserial"),
                    PostgresType::new("serial8"),
                    PostgresType::pg_catalog("serial8"),
                    PostgresType::new("bigint"),
                    PostgresType::new("int8"),
                    PostgresType::pg_catalog("int8"),
                ],
                "i64",
            ),
            (vec![PostgresType::new("oid")], "u32"),
            (
                vec![
                    PostgresType::new("real"),
                    PostgresType::new("float4"),
                    PostgresType::pg_catalog("float4"),
                ],
                "f32",
            ),
            (
                vec![
                    PostgresType::new("float"),
                    PostgresType::new("double precision"),
                    PostgresType::new("float8"),
                    PostgresType::pg_catalog("float8"),
                ],
                "f64",
            ),
            (
                vec![
                    PostgresType::new("text"),
                    PostgresType::pg_catalog("varchar"),
                    PostgresType::pg_catalog("bpchar"),
                    PostgresType::new("string"),
                    PostgresType::new("citext"),
                    PostgresType::new("name"),
                ],
                "String",
            ),
            (
                vec![
                    PostgresType::new("bytea"),
                    PostgresType::new("blob"),
                    PostgresType::pg_catalog("bytea"),
                ],
                "Vec<u8>",
            ),
            (
                vec![PostgresType::new("hstore")],
                "HashMap<String, Option<String>>",
            ),
            (
                vec![
                    PostgresType::pg_catalog("timestamp"),
                    PostgresType::pg_catalog("timestamptz"),
                    PostgresType::new("timestamptz"),
                ],
                "::std::time::SystemTime",
            ),
            (vec![PostgresType::new("inet")], "::std::net::IpAddr"),
        ];

        let mut map = BTreeMap::new();
        for (pg_types, rs_type) in default_types {
            let rs_type: proc_macro2::TokenStream =
                rs_type.parse().expect("Cannot parse as TokenStream");
            for pg in pg_types {
                map.insert(pg.to_string(), rs_type.clone());
            }
        }

        Self { m: map }
    }
}

impl Default for PgTypeMap {
    fn default() -> Self {
        Self::initialize()
    }
}
