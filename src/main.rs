use sqlc_rust_postgres::{
    Error, create_codegen_response, deserialize_codegen_request, serialize_codegen_response,
};
use std::io;
use std::io::prelude::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer)?;

    let req = deserialize_codegen_request(&buffer)?;

    let resp = create_codegen_response(req)?;

    let out = serialize_codegen_response(&resp);

    io::stdout().write_all(&out)?;
    Ok(())
}
