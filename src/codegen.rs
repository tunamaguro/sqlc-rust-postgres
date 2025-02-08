use prost::Message as _;
use quote::{quote, ToTokens};

use crate::{plugin, query::PostgresQuery, user_type::PostgresEnum};

pub fn deserialize_codegen_request(
    buf: &[u8],
) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(buf)
}

pub fn serialize_codegen_response(resp: &plugin::GenerateResponse) -> Vec<u8> {
    resp.encode_to_vec()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PostgresGenerator {
    enums: Vec<PostgresEnum>,
    queries: Vec<PostgresQuery>,
}

impl PostgresGenerator {
    fn new(req: &plugin::GenerateRequest) -> Self {
        let catalog = req.catalog.as_ref().expect("catalog not found");
        let pg_enums = catalog
            .schemas
            .iter()
            .flat_map(|s| s.enums.iter().map(PostgresEnum::new))
            .collect::<Vec<_>>();
        let pg_queries = req
            .queries
            .iter()
            .map(PostgresQuery::new)
            .collect::<Vec<_>>();

        Self {
            enums: pg_enums,
            queries: pg_queries,
        }
    }
}

impl ToTokens for PostgresGenerator {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PostgresGenerator { enums, queries } = self;

        tokens.extend(quote! {
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

    {
        let f = plugin::File {
            name: "debug.txt".into(),
            contents: format!("{:?}", req).into(),
        };
        resp.files.push(f);
    }

    resp
}
