use prost::Message as _;
use quote::quote;
use serde::Deserialize;

use crate::{
    plugin,
    query::PostgresQuery,
    user_type::{PgTypeMap, PostgresEnum},
};

pub fn deserialize_codegen_request(
    buf: &[u8],
) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(buf)
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
    use_async: bool,
    overrides: Vec<CustomType>,
    enum_derives: Vec<String>,
    row_derives: Vec<String>,
}

#[derive(Debug, Clone)]
struct PostgresGenerator {
    config: PgGeneratorConfig,
    enum_derive: proc_macro2::TokenStream,
    row_derive: proc_macro2::TokenStream,
    sqlc_version: String,
}

impl PostgresGenerator {
    fn new(req: &plugin::GenerateRequest) -> Self {
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

        Self {
            config,
            enum_derive: quote! {#[derive(#(#enum_derive),*)]},
            row_derive: quote! {#[derive(#(#row_derive),*)]},
            sqlc_version: req.sqlc_version.clone(),
        }
    }

    fn gen_comment(&self) -> proc_macro2::TokenStream {
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
        .unwrap()
    }

    fn generate_type_map(&self, catalog: &plugin::Catalog) -> PgTypeMap {
        let mut pg_type_map = PgTypeMap::new(catalog);

        for m in &self.config.overrides {
            pg_type_map.add(&m.db_type, &m.rs_type);
        }

        pg_type_map
    }
    fn generate_tokens(&self, req: &plugin::GenerateRequest) -> proc_macro2::TokenStream {
        let catalog = req.catalog.as_ref().expect("catalog not found");

        let pg_type_map = self.generate_type_map(catalog);

        let pg_enums = catalog
            .schemas
            .iter()
            .flat_map(|s| s.enums.iter().map(PostgresEnum::new))
            .collect::<Vec<_>>();

        let pg_queries = req
            .queries
            .iter()
            .map(|query| PostgresQuery::new(query, &pg_type_map, self.config.use_async))
            .collect::<Vec<_>>();

        let pg_queries = pg_queries
            .iter()
            .map(|v| v.with_derive(&self.row_derive))
            .collect::<Vec<_>>();
        let pg_enums = pg_enums
            .iter()
            .map(|v| v.with_derive(&self.enum_derive))
            .collect::<Vec<_>>();

        let comment = self.gen_comment();
        quote! {
            #comment
            #(#pg_enums)*
            #(#pg_queries)*
        }
    }
}

pub fn create_codegen_response(req: &plugin::GenerateRequest) -> plugin::GenerateResponse {
    let mut resp = plugin::GenerateResponse::default();
    {
        let generator = PostgresGenerator::new(req);
        let tt = generator.generate_tokens(req);
        let ast = syn::parse2(tt).unwrap();
        let f = plugin::File {
            name: "queries.rs".into(),
            contents: prettyplease::unparse(&ast).into(),
        };
        resp.files.push(f);
    }

    resp
}
