# sqlc-rust-postgres
sqlc plugin for [tokio_postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/index.html) and [postgres](https://docs.rs/postgres/latest/postgres/)


## Usage

```json
{
  "version": "2",
  "plugins": [
    {
      "name": "rust-postgres",
      "wasm": {
        "url": "https://github.com/tunamaguro/sqlc-rust-postgres/releases/download/0.1.0/sqlc-rust-postgres_0.1.0.wasm",
        "sha256": "7348d21fb14f1939631ad9e20b6beeeedf7da6ea1ae169d3a50cd2b901b7ce58"
      }
    }
  ],
  "sql": [
    {
      "schema": "examples/custom_type/src/schema.sql",
      "queries": "examples/custom_type/src/query.sql",
      "engine": "postgresql",
      "codegen": [
        {
          "out": "examples/custom_type/src",
          "plugin": "rust-postgres",
          "options": {
            "use_async": true,
            "enum_derives": [
              "PartialEq"
            ],
            "row_derives": [
              "PartialEq"
            ],
            "overrides": [
              {
                "db_type": "voiceactor",
                "rs_type": "crate::VoiceActor"
              },
              {
                "db_type": "money",
                "rs_type": "postgres_money::Money"
              },
              {
                "db_type": "pg_catalog.numeric",
                "rs_type": "rust_decimal::Decimal"
              }
            ]
          }
        }
      ]
    }
  ]
}
```

## Options

> NOTE: This plugin supports json only.

### `use_async`

If true, the generated code will use tokio_postgres. If false, it will use postgres.

- Example of `use_async = true`: https://github.com/tunamaguro/sqlc-rust-postgres/blob/main/examples/authors/src/queries.rs
- Example of `use_async = false`: https://github.com/tunamaguro/sqlc-rust-postgres/blob/main/examples/jets/src/queries.rs

### `enum_derives`

Strings added here will be included in the generated `enum`'s derive attributes.

### `row_derives`

Strings added here will be included in the generated `XXXRow` struct's derive attributes.

### `overrides`

By default, this plugin does not support [third-party crate types]((https://docs.rs/postgres-types/0.2.9/postgres_types/trait.FromSql.html#types)). If you wish to use them, add an entry here.

When an unsupported DB type is encountered, you might see an error like:

```bash
$ sqlc generate
# package rust-postgres
error generating code: thread 'main' panicked at src/query.rs:308:17:
Cannot find rs_type that matches column type of `pg_catalog.numeric`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

In this case, add the following entry to overrides:
```diff
            "overrides": [
              {
                "db_type": "voiceactor",
                "rs_type": "crate::VoiceActor"
              },
              {
                "db_type": "money",
                "rs_type": "postgres_money::Money"
              },
+              {
+                "db_type": "pg_catalog.numeric",
+                "rs_type": "rust_decimal::Decimal"
+              }
            ]
```

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