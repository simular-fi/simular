//!
//! Python wrapper for `simular-core::ContractAbi`
//!
use alloy_dyn_abi::DynSolType;
use pyo3::prelude::*;

use simular_core::ContractAbi;

/// Can load and parse ABI information.  Used in `Contract.py` to
/// process function calls.
#[pyclass]
pub struct PyAbi(ContractAbi);

#[pymethods]
impl PyAbi {
    /// Load a complete ABI file from a compiled  Solidity contract.  
    /// This is a raw un-parsed json file that includes both `abi` and `bytecode`.  
    #[staticmethod]
    pub fn from_full_json(abi: &str) -> Self {
        Self(ContractAbi::from_full_json(abi))
    }

    /// Load from the un-parsed json `abi` and optionally `bytecode`
    #[staticmethod]
    pub fn from_abi_bytecode(abi: &str, bytes: Option<Vec<u8>>) -> Self {
        Self(ContractAbi::from_abi_bytecode(abi, bytes))
    }

    /// Create an ABI by providing shortened definitions of the functions
    /// of interest.  
    ///
    /// ## Example:
    ///
    /// `["function hello() (uint256)"]` creates the function `hello` that
    /// can be encoded/decoded for calls to the Evm.
    #[staticmethod]
    pub fn from_human_readable(values: Vec<&str>) -> Self {
        Self(ContractAbi::from_human_readable(values))
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

    /// Encode constructor arguments.
    /// Returns the encoded args, and whether the constructor is payable
    pub fn encode_constructor(&self, args: &str) -> anyhow::Result<(Vec<u8>, bool)> {
        self.0.encode_constructor(args)
    }

    /// Encode the arguments for a specific function.
    /// Returns:
    /// - `encoded args`
    /// - `is the function payable?`
    /// - `DynSolType` to decode output from function
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
