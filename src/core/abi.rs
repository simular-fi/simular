use alloy_dyn_abi::{DynSolType, DynSolValue, ResolveSolType};
use alloy_json_abi::{ContractObject, Function, JsonAbi, StateMutability};
use alloy_primitives::Bytes;
use anyhow::{anyhow, bail, Result};

pub struct ContractAbi {
    pub abi: JsonAbi,
    pub bytecode: Option<Bytes>,
}

impl ContractAbi {
    pub fn load_from_full_json(raw: &str) -> Self {
        let co = serde_json::from_str::<ContractObject>(raw).expect("parsing abi json");
        if co.abi.is_none() {
            panic!("ABI not found in given file")
        }
        Self {
            abi: co.abi.unwrap(),
            bytecode: co.bytecode,
        }
    }

    pub fn load_from_parts(raw: &str, bytecode: Vec<u8>) -> Self {
        let abi = serde_json::from_str::<JsonAbi>(raw).expect("parsing abi input");
        Self {
            abi,
            bytecode: Some(bytecode.into()),
        }
    }

    pub fn load_from_only_abi(raw: &str) -> Self {
        let abi = serde_json::from_str::<JsonAbi>(raw).expect("parsing abi input");
        Self {
            abi,
            bytecode: None,
        }
    }

    pub fn load_human_readable(input: Vec<&str>) -> Self {
        let abi = JsonAbi::parse(input).expect("valid solidity functions information");
        Self {
            abi,
            bytecode: None,
        }
    }

    /// Is there a function with the given name?
    /// Called from the Python Contract's __getattr__
    pub fn has_function(&self, name: &str) -> bool {
        self.abi.functions.contains_key(name)
    }

    pub fn has_fallback(&self) -> bool {
        self.abi.fallback.is_some()
    }

    pub fn has_receive(&self) -> bool {
        self.abi.receive.is_some()
    }

    /// Return the contract bytecode as a Vec
    pub fn bytecode(&self) -> Option<Vec<u8>> {
        self.bytecode.as_ref().map(|b| b.to_vec())
    }

    pub fn encode_constructor(&self, args: &str) -> Result<(Vec<u8>, bool)> {
        let bytecode = match self.bytecode() {
            Some(b) => b,
            _ => bail!("Missing contract bytecode!"),
        };

        let constructor = match &self.abi.constructor {
            Some(c) => c,
            _ => return Ok((bytecode, false)),
        };

        let types = constructor
            .inputs
            .iter()
            .map(|i| i.resolve().unwrap())
            .collect::<Vec<_>>();

        let ty = DynSolType::Tuple(types);
        let dynavalues = ty.coerce_str(args).map_err(|_| anyhow!("HERE error..."))?;
        let encoded_args = dynavalues.abi_encode_params();
        let is_payable = match constructor.state_mutability {
            StateMutability::Payable => true,
            _ => false,
        };

        Ok(([bytecode, encoded_args].concat(), is_payable))
    }

    fn extract(funcs: &Function, args: &str) -> Result<DynSolValue> {
        let types = funcs
            .inputs
            .iter()
            .map(|i| i.resolve().unwrap())
            .collect::<Vec<_>>();
        let ty = DynSolType::Tuple(types);
        ty.coerce_str(args)
            .map_err(|_| anyhow!("Error coercing the arguments for the function call"))
    }

    pub fn encode_function(
        &self,
        name: &str,
        args: &str,
    ) -> anyhow::Result<(Vec<u8>, bool, DynSolType)> {
        let funcs = match self.abi.function(name) {
            Some(funcs) => funcs,
            _ => bail!("Function {} not found in the ABI!", name),
        };

        for f in funcs {
            let result = Self::extract(f, args);
            let is_payable = match f.state_mutability {
                StateMutability::Payable => true,
                _ => false,
            };
            // find the first function that matches the input args
            if result.is_ok() {
                let types = f
                    .outputs
                    .iter()
                    .map(|i| i.resolve().unwrap())
                    .collect::<Vec<_>>();
                let ty = DynSolType::Tuple(types);
                let selector = f.selector().to_vec();
                let encoded_args = result.unwrap().abi_encode_params();
                let all = [selector, encoded_args].concat();

                return Ok((all, is_payable, ty));
            }
        }

        // if we get here, it means we didn't find a function that
        // matched the input arguments
        Err(anyhow::anyhow!(
            "Arguments to the function do not match what is expected"
        ))
    }
}

#[cfg(test)]
mod tests {
    use alloy_dyn_abi::DynSolType;
    use alloy_primitives::Address;

    use super::ContractAbi;

    #[test]
    fn encode_function_no_param_single_output() {
        let contract = ContractAbi::load_human_readable(vec!["function hello() (address)"]);
        assert!(
            contract
                .abi
                .function("hello")
                .unwrap()
                .get(0)
                .unwrap()
                .inputs
                .len()
                == 0
        );

        let (enc, _, out) = contract.encode_function("hello", "()").unwrap();
        assert!(enc.len() > 0);
        assert_eq!(out, DynSolType::Tuple(vec![DynSolType::Address]));
    }

    #[test]
    fn encode_function_single_param() {
        let contract = ContractAbi::load_human_readable(vec!["function hello(address name)"]);
        let args = Address::repeat_byte(1).to_string();
        let (enc, _, out) = contract
            .encode_function("hello", &format!("({:})", args))
            .unwrap();
        assert!(enc.len() > 0);
        assert_eq!(out, DynSolType::Tuple(vec![]));
    }

    #[test]
    fn encode_simple_function() {
        let contract = ContractAbi::load_human_readable(vec![
            "function hello(address name, uint256 value) (uint256)",
        ]);

        assert!(contract.abi.functions.contains_key("hello"));
        assert!(
            contract
                .abi
                .function("hello")
                .unwrap()
                .get(0)
                .unwrap()
                .inputs
                .len()
                == 2
        );

        let (enc, _, out) = contract
            .encode_function("hello", "(0x0101010101010101010101010101010101010101, 1)")
            .unwrap();
        assert!(enc.len() > 0);
        assert_eq!(out, DynSolType::Tuple(vec![DynSolType::Uint(256)]));
    }

    #[test]
    fn parse_contract_object_from_full_json() {
        let raw = include_str!("../../tests/fixtures/KitchenSink.json");
        let contract = ContractAbi::load_from_full_json(raw);
        assert!(contract.bytecode.is_some());
        assert!(contract.abi.functions.contains_key("increment"));
        assert!(contract.abi.functions.contains_key("setInput"));
    }

    #[test]
    fn parse_contract_object_from_human_readable() {
        let contract = ContractAbi::load_human_readable(vec![
            "function number() (uint256)",
            "function addAndSet(uint256 value)",
        ]);
        assert!(contract.bytecode.is_none());
        assert!(contract.abi.functions.len() == 2);
        assert!(contract.abi.functions.contains_key("addAndSet"));
        assert!(contract.abi.functions.contains_key("number"));
    }

    #[test]
    fn parse_complex_input() {
        let contract = ContractAbi::load_human_readable(vec![
            "function exactInputSingle(tuple(address,address,uint24,address,uint256,uint256,uint256,uint160)) (uint256)"
        ]);
        assert!(contract.abi.functions.contains_key("exactInputSingle"));

        let (enc, _, out) = contract
            .encode_function(
                "exactInputSingle",
                "((0x0101010101010101010101010101010101010101,0x0101010101010101010101010101010101010101,3,0x0101010101010101010101010101010101010101,1,2,3,4))"
            ).unwrap();

        assert!(enc.len() > 0);
        assert_eq!(out, DynSolType::Tuple(vec![DynSolType::Uint(256)]));
    }
}
