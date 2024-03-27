use alloy_dyn_abi::DynSolValue;
use core::ffi::c_uchar;
use pyo3::{ffi, prelude::*};
use revm::{db::InMemoryDB, primitives::U256};

use crate::{
    core::{baseevm::BaseEvm, baseevm::ForkDb, snapshot::SerializableState},
    py::{pyabi::PyAbi, str_to_address},
};

macro_rules! implement_common_functions {
    ($name: ident) => {
        #[pymethods]
        impl $name {
            pub fn create_account(
                &mut self,
                address: &str,
                amount: Option<u128>,
            ) -> anyhow::Result<()> {
                let caller = str_to_address(address)?;
                let value = amount.and_then(|v| Some(U256::from(v)));
                self.0.create_account(caller, value)
            }

            /// Return the balance for the given address
            pub fn get_balance(&mut self, caller: &str) -> anyhow::Result<u128> {
                let caller = str_to_address(caller)?;
                let v = self.0.get_balance(caller)?;
                Ok(v.to::<u128>())
            }

            pub fn deploy(
                &mut self,
                args: &str,
                caller: &str,
                value: u128,
                abi: &PyAbi,
            ) -> anyhow::Result<String> {
                let a = str_to_address(caller)?;
                let v = U256::try_from(value)?;
                let (bits, _is_payable) = abi.encode_constructor(args)?;
                let addy = self.0.deploy(a, bits, v)?;
                Ok(addy.to_string())
            }

            pub fn transact(
                &mut self,
                fn_name: &str,
                args: &str,
                caller: &str,
                to: &str,
                value: u128,
                abi: &PyAbi,
                py: Python<'_>,
            ) -> anyhow::Result<PyObject> {
                let a = str_to_address(caller)?;
                let b = str_to_address(to)?;
                let v = U256::try_from(value)?;

                let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
                let (output, _) = self.0.transact(a, b, calldata, v)?;
                let dynvalues = decoder.0.abi_decode_params(&output)?;
                let dsm = DynSolMap(dynvalues.clone());
                Ok(dsm.into_py(py))
            }

            pub fn call(
                &mut self,
                fn_name: &str,
                args: &str,
                to: &str,
                abi: &PyAbi,
                py: Python<'_>,
            ) -> anyhow::Result<PyObject> {
                let a = str_to_address(to)?;
                let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
                let (output, _) = self.0.call(a, calldata)?;
                let dynvalues = decoder.0.abi_decode_params(&output)?;
                let dsm = DynSolMap(dynvalues.clone());
                Ok(dsm.into_py(py))
            }

            pub fn simulate(
                &mut self,
                fn_name: &str,
                args: &str,
                caller: &str,
                to: &str,
                abi: &PyAbi,
                py: Python<'_>,
            ) -> anyhow::Result<PyObject> {
                let a = str_to_address(to)?;
                let b = str_to_address(caller)?;
                let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
                let (output, _) = self.0.simulate(b, a, calldata)?;
                let dynvalues = decoder.0.abi_decode_params(&output)?;
                let dsm = DynSolMap(dynvalues.clone());
                Ok(dsm.into_py(py))
            }

            pub fn transfer(
                &mut self,
                caller: &str,
                to: &str,
                amount: u128,
            ) -> anyhow::Result<u64> {
                let a = str_to_address(caller)?;
                let b = str_to_address(to)?;
                let value = U256::try_from(amount)?;
                self.0.transfer(a, b, value)
            }

            pub fn dump_state(&mut self) -> anyhow::Result<String> {
                let r = self.0.dump_state()?;
                let x = serde_json::to_string_pretty(&r)?;
                Ok(x)
            }

            /// View the value of a specific storage slot for a given contract.
            pub fn view_storage_slot(
                &mut self,
                address: &str,
                index: u128,
            ) -> anyhow::Result<Vec<u8>> {
                let location = str_to_address(address)?;
                let idx = U256::try_from(index)?;
                let r = self.0.view_storage_slot(location, idx)?;
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
    pub fn load_state(&mut self, raw: &str) -> anyhow::Result<()> {
        let state: SerializableState = serde_json::from_str(raw)?;
        self.0.load_state(state);
        Ok(())
    }
}

// *** Helpers to convert DynSolValues to PyObject *** //

fn walk_list(values: Vec<DynSolValue>, py: Python<'_>) -> PyObject {
    values
        .into_iter()
        .map(|dv| base_exctract(dv, py))
        .collect::<Vec<_>>()
        .into_py(py)
}

fn base_exctract(dv: DynSolValue, py: Python<'_>) -> PyObject {
    match dv {
        DynSolValue::Address(a) => format!("{a:?}").into_py(py),
        DynSolValue::Bool(a) => a.into_py(py),
        DynSolValue::String(a) => a.into_py(py),
        DynSolValue::Tuple(a) => walk_list(a, py),
        DynSolValue::Int(a, _) => a.as_i64().into_py(py),
        DynSolValue::Uint(a, _) => {
            let bytes = a.as_le_bytes();
            // put on your life jacket we're entering 'unsafe' waters
            // ...adapted from ruint pyo3 extension
            unsafe {
                let obj =
                    ffi::_PyLong_FromByteArray(bytes.as_ptr().cast::<c_uchar>(), bytes.len(), 1, 0);
                PyObject::from_owned_ptr(py, obj)
            }
        }
        DynSolValue::Bytes(a) => a.into_py(py),
        DynSolValue::FixedBytes(a, _) => a.to_vec().into_py(py),
        DynSolValue::Array(a) => walk_list(a, py),
        DynSolValue::FixedArray(a) => walk_list(a, py),
        _ => unimplemented!(),
    }
}

pub struct DynSolMap(DynSolValue);

impl IntoPy<PyObject> for DynSolMap {
    fn into_py(self, py: Python<'_>) -> PyObject {
        base_exctract(self.0, py)
    }
}
