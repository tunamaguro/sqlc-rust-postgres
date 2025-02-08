_default:
  just --list 

set dotenv-load

alias f:= format
alias l:= lint
alias lf:= lint-fix

setup-tools:
    rustup target add wasm32-wasip1

# format
format:
    cargo fmt

# Show lint error
lint:
    cargo clippy --all-targets --all-features --fix

# Fix clippy error
lint-fix:
    cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged

# build wasm plugin
build-wasm:
    cargo build --release --target wasm32-wasip1

# rust sqlc
generate: build-wasm
    #!/usr/bin/env bash
    set -euxo pipefail

    WASM_SHA256=$(sha256sum target/wasm32-wasip1/release/sqlc-rust-postgres.wasm | awk '{print $1}');
    sed "s/\$WASM_SHA256/${WASM_SHA256}/g" _sqlc.json > sqlc.json
    sqlc generate -f sqlc.json 
