mod pyabi;
mod pyevm;

use alloy_primitives::Address;
use pyo3::{exceptions::PyRuntimeError, prelude::PyErr, prelude::*};
use std::fmt::Debug;

/// Used to map an Error to PyErr
pub fn pyerr<T: Debug>(err: T) -> PyErr {
    PyRuntimeError::new_err(format!("{:?}", err))
}

/// Convert strings to addresses.  String addresses are passed through from Python.
pub fn str_to_address(caller: &str) -> Result<Address, PyErr> {
    let c = caller
        .parse::<Address>()
        .map_err(|_| pyerr("failed to parse caller address from string"))?;
    Ok(c)
}

#[pymodule]
fn simular(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<pyabi::PyAbi>()?;
    m.add_class::<pyevm::PyEvm>()?;
    Ok(())
}
