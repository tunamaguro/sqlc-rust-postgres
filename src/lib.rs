pub mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

mod codegen;
pub(crate) mod const_query;

pub use codegen::*;
