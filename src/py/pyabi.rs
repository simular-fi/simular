///! Wraps [`core::ContractAbi`]
use alloy_dyn_abi::DynSolValue;
use alloy_primitives::{Address, I256, U256};
use pyo3::{
    prelude::*,
    types::{PyAny, PyTuple},
};

use super::pyerr;
use crate::core::abi::ContractAbi;

/// Provides the ability to load and parse ABI information.
#[pyclass]
pub struct PyAbi(ContractAbi);

#[pymethods]
impl PyAbi {
    /// Load a complete ABI file from compiling a Solidity contract.  
    /// This is a raw unparsed json file that includes both `abi` and `bytecode`.  
    #[staticmethod]
    pub fn load_from_json(abi: &str) -> Self {
        Self(ContractAbi::load_from_full_json(abi))
    }

    /// Load an ABI from the unparsed json `abi` and `bytecode`
    #[staticmethod]
    pub fn load_from_parts(abi: &str, bytes: Vec<u8>) -> Self {
        Self(ContractAbi::load_from_parts(abi, bytes))
    }

    /// Load an ABI by providing shortened definitions of the functions
    /// of interest.  For example:
    /// `["function hello() (uint256)"]` creates the function `hello` that
    /// can be encoded/decoded for calls to the Evm.
    #[staticmethod]
    pub fn load_from_human_readable(values: Vec<&str>) -> Self {
        Self(ContractAbi::load_human_readable(values))
    }

    /// Does the ABI contain the function `name`
    pub fn has_function(&self, name: &str) -> bool {
        self.0.has_function(name)
    }

    /// Does the Contract have a fallback function?
    pub fn has_fallback(&self) -> bool {
        self.0.has_fallback()
    }

    /// Does the contract have a receive function?
    pub fn has_receive(&self) -> bool {
        self.0.has_receive()
    }

    /// Return the contract bytecode
    pub fn bytecode(&self) -> Option<Vec<u8>> {
        self.0.bytecode()
    }

    /// Return the list of solidity types expected as contract constructor parameters (if any)
    pub fn constructor_input_types(&self) -> Option<Vec<String>> {
        self.0.constructor_input_types()
    }

    /// Encode a function call with any arguments into the format expected by the Evm.
    /// Where:
    /// - `name` is the function name
    /// - `args` are any expected argument to the function
    pub fn encode_function_input(
        &self,
        name: &str,
        args: &PyTuple,
    ) -> PyResult<(Vec<u8>, Vec<String>)> {
        let iargs = args_to_value(&args);
        self.0
            .encode_function_input(name, iargs)
            .map_err(|e| pyerr(e))
    }
}

// ***  Conversions stuff below *** //

/// Convert python function input arguments into DynSolValues for encoding
/// and validation

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct DynSolValueWrap<T>(pub T);

impl<T> Clone for DynSolValueWrap<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        DynSolValueWrap(self.0.clone())
    }
}
impl<T> From<T> for DynSolValueWrap<T> {
    fn from(t: T) -> Self {
        DynSolValueWrap(t)
    }
}

impl FromPyObject<'_> for DynSolValueWrap<I256> {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let result = ob
            .extract::<i128>()
            .and_then(|v| I256::try_from(v).map_err(pyerr))?;
        Ok(DynSolValueWrap(result))
    }
}

impl FromPyObject<'_> for DynSolValueWrap<U256> {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let v = ob.extract::<u128>()?;
        Ok(DynSolValueWrap(U256::from(v)))
    }
}

impl FromPyObject<'_> for DynSolValueWrap<Address> {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let value = ob.extract::<String>()?;
        value
            .parse::<Address>()
            .map(|inner| DynSolValueWrap(inner))
            .map_err(|_| pyerr("failed to parse address from str"))
    }
}

fn extract_from_sequence(ob: &PyAny) -> PyResult<Vec<DynSolValue>> {
    let mut results = Vec::new();
    for v in ob.iter()? {
        let item = v?;
        let value = item.extract::<DynSolValueWrap<DynSolValue>>()?;
        results.push(value.0);
    }
    Ok(results.clone())
}

impl FromPyObject<'_> for DynSolValueWrap<DynSolValue> {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let type_name = ob.get_type().name()?;
        match type_name {
            "int" => {
                if let Ok(n) = ob.extract::<DynSolValueWrap<U256>>() {
                    Ok(DynSolValueWrap(DynSolValue::Uint(n.0, 256)))
                } else if let Ok(n) = ob.extract::<DynSolValueWrap<I256>>() {
                    Ok(DynSolValueWrap(DynSolValue::Int(n.0, 256)))
                } else {
                    Err(pyerr(anyhow::anyhow!("cant handle int")))
                }
            }
            "str" => {
                if let Ok(a) = ob.extract::<DynSolValueWrap<Address>>() {
                    Ok(DynSolValueWrap(DynSolValue::Address(a.0)))
                } else {
                    let v = ob.extract::<String>()?;
                    Ok(DynSolValueWrap(DynSolValue::String(v)))
                }
            }
            "tuple" => {
                let r = extract_from_sequence(ob)?;
                Ok(DynSolValueWrap(DynSolValue::Tuple(r)))
            }
            "list" => {
                let r = extract_from_sequence(ob)?;
                Ok(DynSolValueWrap(DynSolValue::FixedArray(r)))
            }
            "bool" => {
                let result = ob.extract::<bool>()?;
                Ok(DynSolValueWrap(DynSolValue::Bool(result)))
            }
            "bytes" => {
                let result = ob.extract::<Vec<u8>>()?;
                Ok(DynSolValueWrap(DynSolValue::Bytes(result)))
            }

            _ => Err(pyerr(anyhow::anyhow!(
                "unrecognized type to convert to DynSolValue"
            ))),
        }
    }
}

/// Convert a tuple of Python objects to DynSolValues
pub fn args_to_value(args: &PyTuple) -> Option<DynSolValue> {
    if args.is_empty() {
        return None;
    }
    // Why this structure?  All args are wrapped in a tuple on the Python side
    // so we check here to deal with Tuples with 1 element
    let pyany = match args.len() {
        1 => args.get_item(0).and_then(|i| {
            i.extract::<DynSolValueWrap<DynSolValue>>()
                .map(|inner| inner.0)
        }),
        _ => args
            .extract::<DynSolValueWrap<DynSolValue>>()
            .map(|inner| inner.0),
    };

    pyany.ok()
}
