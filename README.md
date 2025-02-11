# sqlc-rust-postgres
sqlc plugin for [tokio_postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/index.html) and [postgres](https://docs.rs/postgres/latest/postgres/)


## Setup develop environment

Install `protoc`. 

```bash
sudo apt-get install protobuf-compiler
```

Ref: https://docs.rs/prost-build/latest/prost_build/#sourcing-protoc

Install just and run setup

```bash
cargo install just
just setup-tools
```

Run sqlc

```bash
just generate
```

## Update sqlc proto

Copy from https://github.com/sqlc-dev/sqlc/blob/main/protos/plugin/codegen.proto