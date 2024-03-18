pub(crate) mod pyabi;
pub(crate) mod pyevm;

use pyo3::{exceptions::PyRuntimeError, prelude::PyErr};
use revm::primitives::Address;
use std::fmt::Debug;

pub fn pyerr<T: Debug>(err: T) -> PyErr {
    PyRuntimeError::new_err(format!("{:?}", err))
}

/// Helper to convert strings to addresses.  String addresses are passed through
/// from Python.
///
pub fn str_to_address(caller: &str) -> Result<Address, PyErr> {
    let c = caller
        .parse::<Address>()
        .map_err(|_| pyerr("failed to parse caller address from string"))?;
    Ok(c)
}
