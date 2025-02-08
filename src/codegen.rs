use prost::Message as _;
use quote::ToTokens;

use crate::{query::PostgresQuery, plugin};

pub fn deserialize_codegen_request(
    buf: &[u8],
) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(buf)
}

pub fn serialize_codegen_response(resp: &plugin::GenerateResponse) -> Vec<u8> {
    resp.encode_to_vec()
}

pub fn create_codegen_response(req: &plugin::GenerateRequest) -> plugin::GenerateResponse {
    let mut f = plugin::File {
        name: "queries.rs".into(),
        ..Default::default()
    };
    for query in &req.queries {
        let pq = PostgresQuery::new(query);
        let mut tt = pq.to_token_stream().to_string().into_bytes();
        f.contents.append(&mut tt);
    }

    let mut resp = plugin::GenerateResponse::default();

    resp.files.push(f);

    resp
}
