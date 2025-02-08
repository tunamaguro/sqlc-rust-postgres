# sqlc-rust-postgres
sqlc plugin for `rust-postgres`


## Setup develop environment

Install `protoc`. 

```bash
sudo apt-get install protobuf-compiler
```

Ref: https://docs.rs/prost-build/latest/prost_build/#sourcing-protoc

Install just

```bash
cargo install just
```


## Update sqlc proto

Copy from https://github.com/sqlc-dev/sqlc/blob/main/protos/plugin/codegen.proto