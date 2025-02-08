_default:
  just --list 

set dotenv-load

alias f:= format
alias l:= lint
alias lf:= lint-fix

format:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features --fix


# Fix clippy error
lint-fix:
    cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged