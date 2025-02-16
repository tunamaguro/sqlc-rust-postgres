use std::io;
use std::io::prelude::*;

use sqlc_rust_postgres::{
    create_codegen_response, deserialize_codegen_request, serialize_codegen_response,
};

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let buffer = stdin.fill_buf().unwrap();

    let req = deserialize_codegen_request(buffer).expect("Cannot deserialize request");

    let resp = create_codegen_response(&req).unwrap();
    let out = serialize_codegen_response(&resp);

    io::stdout().write_all(&out).expect("Cannot write stdout");
}
