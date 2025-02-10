use prost::Message as _;
use quote::{quote, ToTokens};

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

#[derive(Debug, Clone)]
struct PostgresGenerator {
    enums: Vec<PostgresEnum>,
    queries: Vec<PostgresQuery>,
    enum_derive: proc_macro2::TokenStream,
    row_derive: proc_macro2::TokenStream,
    sqlc_version: String,
}

impl PostgresGenerator {
    fn new(req: &plugin::GenerateRequest) -> Self {
        let catalog = req.catalog.as_ref().expect("catalog not found");

        let pg_enums = catalog
            .schemas
            .iter()
            .flat_map(|s| s.enums.iter().map(PostgresEnum::new))
            .collect::<Vec<_>>();

        let pg_type_map = PgTypeMap::new(catalog);

        let pg_queries = req
            .queries
            .iter()
            .map(|query| PostgresQuery::new(query, &pg_type_map))
            .collect::<Vec<_>>();

        let enum_derive = [
            "Debug",
            "Clone",
            "PartialEq",
            "Eq",
            "PartialOrd",
            "Ord",
            "postgres_types::ToSql",
            "postgres_types::FromSql",
        ]
        .map(|s| s.parse::<proc_macro2::TokenStream>().unwrap());
        let row_derive = ["Debug", "Clone"].map(|s| s.parse::<proc_macro2::TokenStream>().unwrap());
        Self {
            enums: pg_enums,
            queries: pg_queries,
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
        //! {} version: {}
        "#,
            self.sqlc_version,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )
        .parse()
        .unwrap()
    }
}

impl ToTokens for PostgresGenerator {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PostgresGenerator { enums, queries, .. } = self;

        let queries = queries
            .iter()
            .map(|v| v.with_derive(&self.row_derive))
            .collect::<Vec<_>>();
        let enums = enums
            .iter()
            .map(|v| v.with_derive(&self.enum_derive))
            .collect::<Vec<_>>();

        let comment = self.gen_comment();
        tokens.extend(quote! {
            #comment
            #(#enums)*
            #(#queries)*
        });
    }
}

pub fn create_codegen_response(req: &plugin::GenerateRequest) -> plugin::GenerateResponse {
    let mut resp = plugin::GenerateResponse::default();
    {
        let generator = PostgresGenerator::new(req);
        let tt = generator.to_token_stream();
        let ast = syn::parse2(tt).unwrap();
        let f = plugin::File {
            name: "queries.rs".into(),
            contents: prettyplease::unparse(&ast).into(),
        };
        resp.files.push(f);
    }

    resp
}
