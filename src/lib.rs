extern crate core;

pub mod auth;
#[cfg(windows)]
#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::all)]
mod bindings;
pub mod endpoint;
pub mod error;
pub mod net;
pub mod rpc;
