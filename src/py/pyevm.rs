use pyo3::prelude::*;
use revm::{db::InMemoryDB, primitives::U256};

use super::{pyerr, str_to_address};
use crate::core::baseevm::{BaseEvm, ForkDb};
use crate::core::snapshot::SerializableState;

/// Macro that implements common function across implementations
macro_rules! implement_common_functions {
    ($name: ident) => {
        #[pymethods]
        impl $name {
            /// Create an account in the Evm DB for the given address and optional amount.
            pub fn create_account(&mut self, address: &str, amount: Option<u128>) -> PyResult<()> {
                let caller = str_to_address(address)?;
                let value = amount.and_then(|v| Some(U256::from(v)));
                self.0.create_account(caller, value).map_err(|e| pyerr(e))
            }

            /// Return the balance for the given address
            pub fn get_balance(&mut self, caller: &str) -> PyResult<u128> {
                let caller = str_to_address(caller)?;
                let v = self.0.get_balance(caller).map_err(|e| pyerr(e))?;
                Ok(v.to::<u128>())
            }

            /// Transfer the amount of value from `caller` to the given recipient `to`.
            pub fn transfer(&mut self, caller: &str, to: &str, amount: u128) -> PyResult<u64> {
                let a = str_to_address(caller)?;
                let b = str_to_address(to)?;
                let value = U256::try_from(amount).map_err(|e| pyerr(e))?;
                self.0.transfer(a, b, value).map_err(|e| pyerr(e))
            }

            /// Deploy a contract
            pub fn deploy(
                &mut self,
                caller: &str,
                bincode: Vec<u8>,
                value: u128,
            ) -> PyResult<String> {
                let a = str_to_address(caller)?;
                let v = U256::try_from(value).map_err(|e| pyerr(e))?;
                let addy = self.0.deploy(a, bincode, v).map_err(|e| pyerr(e))?;

                Ok(format!("{:?}", addy))
            }

            /// Write operation to a contract at the given address `to`.
            pub fn transact(
                &mut self,
                caller: &str,
                to: &str,
                data: Vec<u8>,
                value: u128,
            ) -> PyResult<(Vec<u8>, u64)> {
                let a = str_to_address(caller)?;
                let b = str_to_address(to)?;
                let v = U256::try_from(value).map_err(|e| pyerr(e))?;
                self.0.transact(a, b, data, v).map_err(|e| pyerr(e))
            }

            /// Read operation to a contract at the given address `to`.
            pub fn call(&mut self, to: &str, data: Vec<u8>) -> PyResult<(Vec<u8>, u64)> {
                let a = str_to_address(to)?;
                self.0.call(a, data).map_err(|e| pyerr(e))
            }

            /// Dump the current state of the Evm DB to a json encoded String.
            pub fn dump_state(&mut self) -> PyResult<String> {
                let r = self.0.dump_state().map_err(|e| pyerr(e))?;
                serde_json::to_string_pretty(&r).map_err(|e| pyerr(e))
            }

            /// View the value of a specific storage slot for a given contract.
            pub fn view_storage_slot(&mut self, address: &str, index: u128) -> PyResult<Vec<u8>> {
                let location = str_to_address(address)?;
                let idx = U256::try_from(index).map_err(|e| pyerr(e))?;
                let r = self
                    .0
                    .view_storage_slot(location, idx)
                    .map_err(|e| pyerr(e))?;
                Ok(r.to_le_bytes_vec())
            }
        }
    };
}

/// Creates an EVM that can pulls state from a remote json-rpc endpoint.
/// Calls to this EVM will first look in the local cache for data.  If not
/// found in the cache, it will attempt to pull data from the remote node and
/// then cache locally.
#[pyclass]
pub struct PyEvmFork(BaseEvm<ForkDb>);

implement_common_functions!(PyEvmFork);

#[pymethods]
impl PyEvmFork {
    /// Create an instance. `url` should be a valid URL to
    /// an Evm json-rpc endpoint.
    #[new]
    #[pyo3(signature = (url))]
    pub fn new(url: &str) -> Self {
        Self(BaseEvm::<ForkDb>::create(url))
    }
}

/// An Evm with in-memory only support
#[pyclass]
pub struct PyEvmLocal(BaseEvm<InMemoryDB>);

implement_common_functions!(PyEvmLocal);

#[pymethods]
impl PyEvmLocal {
    #[new]
    pub fn new() -> Self {
        Self(BaseEvm::<InMemoryDB>::create())
    }

    /// Load state from dumped state. `raw` is a String, the unparsed json file.
    pub fn load_state(&mut self, raw: &str) -> PyResult<()> {
        let state: SerializableState = serde_json::from_str(raw).map_err(|e| pyerr(e))?;
        self.0.load_state(state);
        Ok(())
    }
}
