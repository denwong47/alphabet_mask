use thiserror::Error;

use pyo3::{PyErr, PyTypeInfo};

/// Various Error types that can arise from Graphaurus operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum LocalError {
    #[error("unknown error occurred: {context}")]
    Unknown { context: String },
}

impl LocalError {
    /// Consume this [`LocalError`], wrap it in a [`PyErr`], then return it.
    pub fn into_pyerr<E>(self) -> PyErr
    where
        E: PyTypeInfo,
    {
        PyErr::new::<E, _>(self.to_string())
    }
}
