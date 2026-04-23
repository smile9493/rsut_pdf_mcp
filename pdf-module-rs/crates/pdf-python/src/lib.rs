//! Python bindings for PDF extraction (optional)

use pyo3::prelude::*;

/// PDF extraction module for Python
#[pymodule]
fn pdf_python(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // TODO: Add Python bindings
    Ok(())
}
