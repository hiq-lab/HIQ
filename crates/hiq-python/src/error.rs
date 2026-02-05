//! Error handling and exception mapping for Python bindings.

use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;

/// Convert an IR error to a Python exception.
pub fn ir_to_py_err(e: hiq_ir::IrError) -> PyErr {
    PyRuntimeError::new_err(format!("IR Error: {}", e))
}

/// Convert a parse error to a Python exception.
pub fn parse_to_py_err(e: hiq_qasm3::ParseError) -> PyErr {
    PyRuntimeError::new_err(format!("Parse Error: {}", e))
}
