#![allow(unsafe_op_in_unsafe_fn)]

#[pyo3::pymodule(gil_used = false)]
mod _native {}
