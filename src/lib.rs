#![allow(unsafe_op_in_unsafe_fn)]

pub mod internal;

#[pyo3::pymodule(gil_used = false)]
mod _core {}
