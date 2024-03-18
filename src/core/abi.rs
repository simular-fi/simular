use alloy_dyn_abi::{DynSolType, DynSolValue, ResolveSolType};
use alloy_json_abi::{ContractObject, Function, JsonAbi};
use alloy_primitives::Bytes;

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

    /// Return the constructor input types (if any) to be used for
    /// encoding when deploying a contract.
    pub fn constructor_input_types(&self) -> Option<Vec<String>> {
        self.abi.constructor.as_ref().map(|c| {
            c.inputs
                .iter()
                .map(|entry| entry.ty.to_string())
                .collect::<Vec<String>>()
        })
    }

    pub fn encode_function_input(
        &self,
        name: &str,
        args: Option<DynSolValue>,
    ) -> anyhow::Result<(Vec<u8>, Vec<String>)> {
        let func = self
            .abi
            .function(name)
            .and_then(|f| {
                f.iter()
                    .find(|func| find_function_by_input(func, args.clone()))
            })
            .ok_or(anyhow::anyhow!("Function not found"))?;

        // abi encode the input arguments
        let encoded_input_args = match args {
            Some(v) => v.abi_encode_params(),
            None => vec![],
        };

        let encoded_call = func
            .selector()
            .iter()
            .copied()
            .chain(encoded_input_args)
            .collect::<Vec<u8>>();

        let output_params = func
            .outputs
            .iter()
            .map(|p| p.ty.clone())
            .collect::<Vec<_>>();

        Ok((encoded_call, output_params))
    }
}

/// Find the first function that matches 'args' (input parameters)
fn find_function_by_input(func: &&Function, args: Option<DynSolValue>) -> bool {
    // no args, verify the function expects no inputs
    if args.is_none() {
        return func.inputs.is_empty();
    }

    // we have args... check the function is expecting inputs
    if func.inputs.is_empty() {
        return false;
    }

    // resolve the function's input params to DynSolTypes
    let resolved_input_params = func
        .inputs
        .iter()
        .map(|i| i.resolve().expect("failed to resolve to DynSolType"))
        .collect::<Vec<_>>();

    let param_type = match resolved_input_params.len() {
        1 => resolved_input_params[0].clone(),
        _ => DynSolType::Tuple(resolved_input_params),
    };

    // validate the input args match the required parameter types
    param_type.matches(&args.unwrap())
}

#[cfg(test)]
mod tests {
    use alloy_dyn_abi::DynSolValue;
    use alloy_primitives::{Address, U256};

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

        let (enc, out) = contract.encode_function_input("hello", None).unwrap();
        assert!(enc.len() > 0);
        assert!(out.len() == 1);
    }

    #[test]
    fn encode_function_single_param() {
        let contract = ContractAbi::load_human_readable(vec!["function hello(address name)"]);
        let args = DynSolValue::Address(Address::repeat_byte(1));

        let (enc, out) = contract.encode_function_input("hello", Some(args)).unwrap();
        assert!(enc.len() > 0);
        assert!(out.len() == 0);
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

        // address: 0x0101010101010101010101010101010101010101
        let args = DynSolValue::Tuple(vec![
            DynSolValue::Address(Address::repeat_byte(1)),
            DynSolValue::Uint(U256::from(1u8), 256),
        ]);

        let (enc, out) = contract.encode_function_input("hello", Some(args)).unwrap();
        assert!(enc.len() > 0);
        assert!(out.len() == 1);
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
}
