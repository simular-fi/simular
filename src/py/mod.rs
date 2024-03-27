pub(crate) mod pyabi;
///! The [`py`] module exposes the [`core`] code to the Python environment.
///! It provides wrappers to the Evm and Abi parser.
pub(crate) mod pyevm;

use pyo3::prelude::PyErr;
use revm::primitives::Address;

/// Used to map an Error to PyErr
//pub fn pyerr<T: Debug>(err: T) -> PyErr {
//    PyRuntimeError::new_err(format!("{:?}", err))
//}

/// Convert strings to addresses.  String addresses are passed through from Python.
pub fn str_to_address(caller: &str) -> anyhow::Result<Address, PyErr> {
    let c = caller
        .parse::<Address>()
        .map_err(|_| anyhow::anyhow!("failed to parse caller address from string"))?;
    Ok(c)
}
