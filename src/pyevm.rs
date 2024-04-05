use alloy_dyn_abi::DynSolValue;
use alloy_primitives::U256;
use anyhow::{anyhow, Result};
use core::ffi::c_uchar;
use pyo3::{ffi, prelude::*};
use simular_core::{BaseEvm, CreateFork, SnapShot};

use crate::{pyabi::PyAbi, str_to_address};

#[pyclass]
pub struct PyEvm(BaseEvm);

#[pymethods]
impl PyEvm {
    /// Create an in-memory EVM
    #[new]
    pub fn new() -> Self {
        Self(BaseEvm::default())
    }

    /// Create a fork EVM
    #[staticmethod]
    #[pyo3(signature = (url, blocknumber=None))]
    pub fn from_fork(url: &str, blocknumber: Option<u64>) -> Self {
        let forkinfo = CreateFork {
            url: url.into(),
            blocknumber,
        };
        Self(BaseEvm::new(Some(forkinfo)))
    }

    /// Create an in-memory EVM from a `SnapShot`
    #[staticmethod]
    pub fn from_snapshot(raw: &str) -> Self {
        let snap: SnapShot = serde_json::from_str(raw).expect("unable to parse raw snapshot");
        Self(BaseEvm::new_from_snapshot(snap))
    }

    /// Create a `SnapShot` of the current EVM state
    pub fn create_snapshot(&self) -> Result<String> {
        let snapshot = self.0.create_snapshot()?;
        serde_json::to_string_pretty(&snapshot).map_err(|e| anyhow!("{:?}", e))
    }

    /// Create account with an initial balance
    pub fn create_account(&mut self, address: &str, balance: Option<u128>) -> Result<()> {
        let caller = str_to_address(address)?;
        let value = balance.map(U256::from);
        self.0.create_account(caller, value)
    }

    /// Get the balance of the given user
    pub fn get_balance(&mut self, user: &str) -> Result<u128> {
        let user = str_to_address(user)?;
        let v = self.0.get_balance(user)?;
        Ok(v.to::<u128>())
    }

    /// Transfer the amount of value from `caller` to the given recipient `to`.
    pub fn transfer(&mut self, caller: &str, to: &str, amount: u128) -> Result<()> {
        let a = str_to_address(caller)?;
        let b = str_to_address(to)?;
        let value = U256::try_from(amount)?;
        self.0.transfer(a, b, value)
    }

    /// Deploy a contract
    pub fn deploy(&mut self, args: &str, caller: &str, value: u128, abi: &PyAbi) -> Result<String> {
        let a = str_to_address(caller)?;
        let v = U256::try_from(value)?;
        let (bits, _is_payable) = abi.encode_constructor(args)?;
        let addy = self.0.deploy(a, bits, v)?;
        Ok(addy.to_string())
    }

    /// Transaction (write) operation to a contract at the given address `to`. This
    /// will change state in the EVM.
    pub fn transact(
        &mut self,
        fn_name: &str,
        args: &str,
        caller: &str,
        to: &str,
        value: u128,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<PyObject> {
        let a = str_to_address(caller)?;
        let b = str_to_address(to)?;
        let v = U256::try_from(value)?;
        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.transact_commit(a, b, calldata, v)?;
        let dynvalues = decoder.0.abi_decode_params(&output.result)?;
        let dsm = DynSolMap(dynvalues.clone());
        Ok(dsm.into_py(py))
    }

    /// Transaction (read) operation to a contract at the given address `to`. This
    /// will NOT change state in the EVM.
    pub fn call(
        &mut self,
        fn_name: &str,
        args: &str,
        to: &str,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<PyObject> {
        let to_address = str_to_address(to)?;
        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.transact_call(to_address, calldata, U256::from(0))?;
        let dynvalues = decoder.0.abi_decode_params(&output.result)?;
        let dsm = DynSolMap(dynvalues.clone());
        Ok(dsm.into_py(py))
    }

    /// Transaction operation to a contract at the given address `to`. This
    /// can simulate a transact/call operation, but will NOT change state in the EVM.
    pub fn simulate(
        &mut self,
        fn_name: &str,
        args: &str,
        caller: &str,
        to: &str,
        value: u128,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<PyObject> {
        let caller_address = str_to_address(caller)?;
        let to_address = str_to_address(to)?;
        let v = U256::try_from(value)?;

        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.simulate(caller_address, to_address, calldata, v)?;
        let dynvalues = decoder.0.abi_decode_params(&output.result)?;
        let dsm = DynSolMap(dynvalues.clone());
        Ok(dsm.into_py(py))
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
