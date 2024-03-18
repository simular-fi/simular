mod core;
mod py;

use pyo3::prelude::*;

#[pymodule]
fn simular(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<py::pyevm::PyEvmLocal>()?;
    m.add_class::<py::pyevm::PyEvmFork>()?;
    m.add_class::<py::pyabi::PyAbi>()?;
    Ok(())
}
