use prost::Message as _;
use quote::quote;
use serde::Deserialize;

use crate::{
    Error,
    db_support::DbCrate,
    plugin,
    query::PostgresQuery,
    user_type::{PgTypeMap, PostgresEnum, TypeMap as _},
};

pub fn deserialize_codegen_request(buf: &[u8]) -> crate::Result<plugin::GenerateRequest> {
    let req = plugin::GenerateRequest::decode(buf)?;
    Ok(req)
}

pub fn serialize_codegen_response(resp: &plugin::GenerateResponse) -> Vec<u8> {
    resp.encode_to_vec()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
struct CustomType {
    db_type: String,
    rs_type: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(default)]
struct PgGeneratorConfig {
    db_crate: DbCrate,
    overrides: Vec<CustomType>,
    enum_derives: Vec<String>,
    row_derives: Vec<String>,
    copy_types: Vec<String>,
}

struct PostgresGenerator {
    db_crate: DbCrate,
    catalog: plugin::Catalog,
    queries: Vec<plugin::Query>,
    type_map: PgTypeMap,
    enum_derive: proc_macro2::TokenStream,
    row_derive: proc_macro2::TokenStream,
    sqlc_version: String,
}

impl PostgresGenerator {
    fn new(req: plugin::GenerateRequest) -> crate::Result<Self> {
        let config =
            serde_json::from_slice::<PgGeneratorConfig>(&req.plugin_options).unwrap_or_default();

        const DEFAULT_ENUM_DERIVES: &[&str] = &[
            "Debug",
            "Clone",
            "postgres_types::ToSql",
            "postgres_types::FromSql",
        ];
        let enum_derive = config
            .enum_derives
            .iter()
            .map(|s| s.as_str())
            .chain(DEFAULT_ENUM_DERIVES.iter().cloned())
            .map(|s| s.parse::<proc_macro2::TokenStream>().unwrap())
            .collect::<Vec<_>>();

        const DEFAULT_ROW_DERIVES: &[&str] = &["Debug", "Clone"];
        let row_derive = config
            .row_derives
            .iter()
            .map(|s| s.as_str())
            .chain(DEFAULT_ROW_DERIVES.iter().cloned())
            .map(|s| s.parse::<proc_macro2::TokenStream>().unwrap())
            .collect::<Vec<_>>();

        let catalog = req
            .catalog
            .ok_or_else(|| Error::any_error("catalog not found"))?;
        let mut pg_type_map = PgTypeMap::new(&catalog)?;

        for m in config.overrides {
            pg_type_map.add(&m.db_type, &m.rs_type)?;
        }

        for copy_type in config.copy_types {
            pg_type_map.add_copy_type(&copy_type);
        }

        Ok(Self {
            db_crate: config.db_crate,
            type_map: pg_type_map,
            catalog,
            queries: req.queries,
            enum_derive: quote! {#[derive(#(#enum_derive),*)]},
            row_derive: quote! {#[derive(#(#row_derive),*)]},
            sqlc_version: req.sqlc_version.clone(),
        })
    }

    fn gen_comment(&self) -> crate::Result<proc_macro2::TokenStream> {
        format!(
            r#"
        //! Code generated by sqlc. SHOULD NOT EDIT.
        //! sqlc version: {}
        //! {} version: v{}
        "#,
            self.sqlc_version,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        )
        .parse()
        .map_err(Error::any_error)
    }

    fn generate_tokens(&self) -> crate::Result<proc_macro2::TokenStream> {
        let pg_enums = self
            .catalog
            .schemas
            .iter()
            .flat_map(|s| s.enums.iter().map(PostgresEnum::new))
            .collect::<Vec<_>>();

        let pg_queries = self
            .queries
            .iter()
            .map(|query| PostgresQuery::new(query, &self.type_map, self.db_crate))
            .collect::<crate::Result<Vec<_>>>()?;

        let pg_queries = pg_queries
            .iter()
            .map(|v| v.with_derive(&self.row_derive, &self.type_map))
            .collect::<crate::Result<Vec<_>>>()?;
        let pg_enums = pg_enums
            .iter()
            .map(|v| v.with_derive(&self.enum_derive))
            .collect::<Vec<_>>();

        let comment = self.gen_comment()?;

        let tt = quote! {
            #comment
            #(#pg_enums)*
            #(#pg_queries)*
        };
        Ok(tt)
    }
}

pub fn create_codegen_response(
    req: plugin::GenerateRequest,
) -> crate::Result<plugin::GenerateResponse> {
    let mut resp = plugin::GenerateResponse::default();

    {
        let generator = PostgresGenerator::new(req)?;
        let tt = generator.generate_tokens()?;
        let ast = syn::parse2(tt).map_err(Error::any_error)?;
        let f = plugin::File {
            name: "queries.rs".into(),
            contents: prettyplease::unparse(&ast).into(),
        };
        resp.files.push(f);
    }

    Ok(resp)
}
