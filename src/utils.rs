use convert_case::{Case, Casing};
use regex_lite::Regex;
use std::sync::LazyLock;

fn normalize_str(value: &str) -> String {
    static IDENT_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[^a-zA-Z0-9_]"#).unwrap());

    let value = value.replace("-", "_");
    let value = value.replace(":", "_");
    let value = value.replace("/", "_");
    let value = IDENT_PATTERN.replace_all(&value, "");
    value.to_string()
}

/// convert str to valid rust const ident
pub(crate) fn rust_const_ident(value: &str) -> String {
    normalize_str(value).to_case(Case::UpperSnake)
}

/// convert str to valid rust ident like struct, enum, enum value
pub(crate) fn rust_value_ident(value: &str) -> String {
    normalize_str(value).to_case(Case::Pascal)
}

/// convert str to valid rust struct field
pub(crate) fn rust_struct_field(value: &str) -> String {
    normalize_str(value).to_case(Case::Snake)
}

/// convert str to valid fn ident
pub(crate) fn rust_fn_ident(value: &str) -> String {
    normalize_str(value).to_case(Case::Snake)
}
