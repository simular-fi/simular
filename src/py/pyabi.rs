///! Wraps [`core::ContractAbi`]
use alloy_dyn_abi::DynSolType;
use alloy_primitives::{Address, I256, U256};
use pyo3::{
    prelude::*,
    types::{PyAny, PyTuple},
};

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

    pub fn encode_constructor(&self, args: &str) -> anyhow::Result<(Vec<u8>, bool)> {
        self.0.encode_constructor(args)
    }

    pub fn encode_function(
        &self,
        name: &str,
        args: &str,
    ) -> anyhow::Result<(Vec<u8>, bool, DynSolTypeWrapper)> {
        let (enc, is_payable, dt) = self.0.encode_function(name, args).unwrap();
        Ok((enc, is_payable, DynSolTypeWrapper(dt)))
    }
}

#[pyclass]
pub struct DynSolTypeWrapper(pub DynSolType);
