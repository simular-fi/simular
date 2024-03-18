mod abi;
mod baseevm;
pub mod pyabi;
pub mod pyevm;
mod snapshot;
//mod types;

use pyo3::prelude::*;

#[pymodule]
fn simular(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<pyevm::PyEvm>()?;
    m.add_class::<pyevm::PyEvmFork>()?;
    m.add_class::<pyabi::PyAbi>()?;
    Ok(())
}
