//! Raw bindings to Carla's standalone plugin-host backend.
//!
//! Every function in this crate exposes Carla's C API directly and is unsafe to call.
//! Pointers returned by Carla remain owned by Carla unless the upstream API explicitly
//! states otherwise; callers must not free them.

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;
