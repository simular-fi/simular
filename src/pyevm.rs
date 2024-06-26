use alloy_dyn_abi::DynSolValue;
use alloy_primitives::U256;
use anyhow::{anyhow, Result};
use core::ffi::c_uchar;
use pyo3::{ffi, prelude::*};
use simular_core::{evm::CallResult, BaseEvm, CreateFork, SnapShot};
use std::collections::HashMap;

use crate::{
    pyabi::{DynSolTypeWrapper, PyAbi},
    str_to_address,
};

/// default block interval for advancing block time (12s)
const DEFAULT_BLOCK_INTERVAL: u64 = 12;

/// Container to hold the results of calling `transact` or `simulate`
#[derive(Debug)]
#[pyclass]
pub struct TxResult {
    /// contract function call return value, if any
    #[pyo3(get)]
    pub output: Option<PyObject>,
    /// emitted event information, if any
    #[pyo3(get)]
    pub event: Option<HashMap<String, PyObject>>,
    #[pyo3(get)]
    pub gas_used: u64,
}

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
    ///
    /// Returns any results of the call and a map of any emitted events.
    /// Where the event map is:
    /// `key`   is the name of the event
    /// `value` is the decoded log
    pub fn transact(
        &mut self,
        fn_name: &str,
        args: &str,
        caller: &str,
        to: &str,
        value: u128,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<TxResult> {
        let a = str_to_address(caller)?;
        let b = str_to_address(to)?;
        let v = U256::try_from(value)?;
        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.transact_commit(a, b, calldata, v)?;
        process_results_and_events(abi, output, decoder, py)
    }

    /// Transaction (read) operation to a contract at the given address `to`. This
    /// will NOT change state in the EVM.
    ///
    /// Returns any results of the call
    pub fn call(
        &mut self,
        fn_name: &str,
        args: &str,
        to: &str,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<Option<PyObject>> {
        let to_address = str_to_address(to)?;
        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.transact_call(to_address, calldata, U256::from(0))?;
        let res = process_results(output, decoder, py);
        Ok(res)
    }

    /// Transaction operation to a contract at the given address `to`. This
    /// can simulate a transact operation, but will NOT change state in the EVM.
    ///
    /// Returns any results of the call and a map of any emitted events.
    /// Where the event map is:
    /// `key`   is the name of the event
    /// `value` is the decoded log
    pub fn simulate(
        &mut self,
        fn_name: &str,
        args: &str,
        caller: &str,
        to: &str,
        value: u128,
        abi: &PyAbi,
        py: Python<'_>,
    ) -> Result<TxResult> {
        let caller_address = str_to_address(caller)?;
        let to_address = str_to_address(to)?;
        let v = U256::try_from(value)?;
        let (calldata, _is_payable, decoder) = abi.encode_function(fn_name, args)?;
        let output = self.0.simulate(caller_address, to_address, calldata, v)?;
        process_results_and_events(abi, output, decoder, py)
    }

    /// Advance block.number and block.timestamp. Set interval to the amount of
    /// time in seconds you want to advance the timestamp (default: 12s). Block
    /// number will automatically increment.
    ///
    /// When using a fork the initial block.number/timestamp will come from the snapshot.
    pub fn advance_block(&mut self, interval: Option<u64>) {
        let it = interval.unwrap_or(DEFAULT_BLOCK_INTERVAL);
        self.0.update_block(it);
    }
}

// *** lil' Helpers *** //

fn process_results(
    output: CallResult,
    decoder: DynSolTypeWrapper,
    py: Python<'_>,
) -> Option<PyObject> {
    if let Some(de) = decoder.0 {
        let dynvalues = de.abi_decode(&output.result).unwrap();
        let d = DynSolMap(dynvalues.clone());
        Some(d.into_py(py))
    } else {
        None
    }
}

// convert results and events to Python
fn process_results_and_events(
    abi: &PyAbi,
    output_result: CallResult,
    decoder: DynSolTypeWrapper,
    py: Python<'_>,
) -> Result<TxResult> {
    let logs = output_result.logs.clone();
    let gas_used = output_result.gas_used.clone();

    // process return value
    let output = process_results(output_result, decoder, py);

    // process logs
    let event = if logs.len() > 0 {
        let raw_events = abi.0.extract_logs(logs);
        let mut map = HashMap::<String, PyObject>::new();
        for (k, v) in raw_events {
            let d = DynSolMap(v);
            map.insert(k, d.into_py(py));
        }
        Some(map)
    } else {
        None
    };
    Ok(TxResult {
        output,
        event,
        gas_used,
    })
}

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
