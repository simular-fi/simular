//!
//! Parse contract ABIs to encode, decode contract calls
//!
use alloy_dyn_abi::{DynSolEvent, DynSolType, DynSolValue, Specifier};
use alloy_json_abi::{ContractObject, Function, JsonAbi, StateMutability};
use alloy_primitives::{Bytes, Log, LogData};
use anyhow::{anyhow, bail, Result};
use std::collections::BTreeMap;

type EventMap = BTreeMap<std::string::String, Vec<alloy_json_abi::Event>>;

///
/// Wrapper around pre-processed Events to help extract log information.
/// We flatten the structure of `events` in JsonAbi to make it easier to
/// automatically decode Logs from a `transact/simulate`.
///
/// EventLog contains `DynSolEvent` to be used to decode log information
#[derive(Debug)]
pub struct EventLog {
    /// the event name
    pub name: String,
    /// the decoder resolved from the original event
    pub decoder: DynSolEvent,
}

impl EventLog {
    /// Attempt to decode the log and return the event name and extracted values as
    /// DynSolValues.
    pub fn decode(&self, log: &LogData) -> Option<(String, DynSolValue)> {
        if let Ok(r) = self.decoder.decode_log_data(log, true) {
            let v = DynSolValue::Tuple([r.indexed, r.body].concat());
            return Some((self.name.clone(), v));
        }
        None
    }
}

pub struct ContractAbi {
    /// alloy's json abi object
    pub abi: JsonAbi,
    /// optional contract bytecode
    pub bytecode: Option<Bytes>,
    /// Contract event information with a log decoder
    pub events_logs: Vec<EventLog>,
}

// walk through the events in JsonAbi to flatten the
// structure and convert to `EventLog`.
fn convert_events(ev: &EventMap) -> Vec<EventLog> {
    ev.iter()
        .flat_map(|(k, v)| {
            v.iter()
                .map(|e| EventLog {
                    name: k.clone(),
                    decoder: e.resolve().unwrap(),
                })
                .collect::<Vec<EventLog>>()
        })
        .collect::<Vec<EventLog>>()
}

impl ContractAbi {
    /// Parse the `abi` and `bytecode` from a compiled contract's json file.
    /// Note: `raw` is un-parsed json.
    pub fn from_full_json(raw: &str) -> Self {
        let co =
            serde_json::from_str::<ContractObject>(raw).expect("Abi: failed to parse abi to json");
        if co.abi.is_none() {
            panic!("Abi: ABI not found in file")
        }
        if co.bytecode.is_none() {
            panic!("Abi: Bytecode not found in file")
        }
        let abi = co.abi.unwrap();
        let evts = convert_events(&abi.events);
        Self {
            abi,
            bytecode: co.bytecode,
            events_logs: evts,
        }
    }

    /// Parse the `abi` and `bytecode`
    /// Note: `raw` is un-parsed json.
    pub fn from_abi_bytecode(raw: &str, bytecode: Option<Vec<u8>>) -> Self {
        let abi = serde_json::from_str::<JsonAbi>(raw).expect("Abi: failed to parse abi");
        let evts = convert_events(&abi.events);
        Self {
            abi,
            bytecode: bytecode.map(Bytes::from),
            events_logs: evts,
        }
    }

    /// Parse an ABI (without bytecode) from a `Vec` of contract function definitions.
    /// See [human readable abi](https://docs.ethers.org/v5/api/utils/abi/formats/#abi-formats--human-readable-abi)
    pub fn from_human_readable(input: Vec<&str>) -> Self {
        let abi = JsonAbi::parse(input).expect("Abi: Invalid solidity function(s) format");
        let evts = convert_events(&abi.events);
        Self {
            abi,
            bytecode: None,
            events_logs: evts,
        }
    }

    /// Extract and decode logs from emitted events
    pub fn extract_logs(&self, logs: Vec<Log>) -> Vec<(String, DynSolValue)> {
        let mut results: Vec<(String, DynSolValue)> = Vec::new();
        for log in logs {
            for e in &self.events_logs {
                if let Some(p) = e.decode(&log.data) {
                    results.push(p);
                }
            }
        }
        results
    }

    /// Is there a function with the given name?
    pub fn has_function(&self, name: &str) -> bool {
        self.abi.functions.contains_key(name)
    }

    /// Does the ABI have a fallback?
    pub fn has_fallback(&self) -> bool {
        self.abi.fallback.is_some()
    }

    /// Does the ABI have a receive?
    pub fn has_receive(&self) -> bool {
        self.abi.receive.is_some()
    }

    /// Return the contract bytecode as a Vec
    pub fn bytecode(&self) -> Option<Vec<u8>> {
        self.bytecode.as_ref().map(|b| b.to_vec())
    }

    /// Encode the information needed to create a contract.  This will
    /// concatenate the contract bytecode with any arguments required by
    /// the constructor.  Note: `args` is a string of input arguments.  See
    /// `encode_function` for more information.
    pub fn encode_constructor(&self, args: &str) -> Result<(Vec<u8>, bool)> {
        let bytecode = match self.bytecode() {
            Some(b) => b,
            _ => bail!("Abi: Missing contract bytecode!"),
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
        let dynavalues = ty.coerce_str(args).map_err(|_| {
            anyhow!("Abi: Error coercing the arguments for the constructor. Check the input argument(s)")
        })?;
        let encoded_args = dynavalues.abi_encode_params();
        let is_payable = matches!(constructor.state_mutability, StateMutability::Payable);

        Ok(([bytecode, encoded_args].concat(), is_payable))
    }

    fn extract(funcs: &Function, args: &str) -> Result<DynSolValue> {
        let types = funcs
            .inputs
            .iter()
            .map(|i| i.resolve().unwrap())
            .collect::<Vec<_>>();
        let ty = DynSolType::Tuple(types);
        ty.coerce_str(args).map_err(|_| {
            anyhow!(
                "Abi: Error coercing the arguments for the function call. Check the input argument(s)"
            )
        })
    }

    /// Encode function information for use in a transaction. Note: `args` is a string
    /// of input parameters that are parsed by alloy `DynSolType`'s  and converted into
    /// `DynSolValue`s.   See [DynSolType.coerce_str()](https://docs.rs/alloy-dyn-abi/latest/alloy_dyn_abi/enum.DynSolType.html#method.coerce_str)
    ///  
    /// - `name` is the name of the function
    /// - `args` string of input arguments
    ///
    /// ## Example
    ///
    /// `"(1, hello, (0x11111111111111111111111111111, 5))"`
    ///
    /// is parsed into an alloy `DynSolValue` ...tuple, U256, etc...
    ///
    /// Returns a tuple with:
    /// - encoded function and args
    /// - whether the function is payable
    /// - and the output `DynSolType` that can be used to decode the result of the call
    pub fn encode_function(
        &self,
        name: &str,
        args: &str,
    ) -> anyhow::Result<(Vec<u8>, bool, Option<DynSolType>)> {
        let funcs = match self.abi.function(name) {
            Some(funcs) => funcs,
            _ => bail!("Abi: Function {} not found in the ABI!", name),
        };

        // find the first function that matches the input args
        for f in funcs {
            let result = Self::extract(f, args);
            let is_payable = matches!(f.state_mutability, StateMutability::Payable);
            if result.is_ok() {
                // Get the return type decoder, if any...
                let ty = match f.outputs.len() {
                    0 => None,
                    1 => f.outputs.first().unwrap().clone().resolve().ok(),
                    _ => {
                        let t = f
                            .outputs
                            .iter()
                            .map(|i| i.resolve().unwrap())
                            .collect::<Vec<_>>();
                        Some(DynSolType::Tuple(t))
                    }
                };

                let selector = f.selector().to_vec();
                let encoded_args = result.unwrap().abi_encode_params();
                let all = [selector, encoded_args].concat();

                return Ok((all, is_payable, ty));
            }
        }

        // if we get here, it means we didn't find a function that
        // matched the input arguments
        Err(anyhow::anyhow!(
            "Abi: Arguments to the function do not match what is expected"
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::{b256, bytes, Address, LogData};

    #[test]
    fn check_constructor_encoding() {
        let input = vec!["constructor()"];
        let mut abi = ContractAbi::from_human_readable(input);
        // short-circuit internal check...
        abi.bytecode = Some(b"hello".into());

        assert!(abi.encode_constructor("()").is_ok());
        assert!(abi.encode_constructor("(1234)").is_err());
    }

    #[test]
    fn encoding_function_decoder_types() {
        let tc = ContractAbi::from_human_readable(vec![
            "function a()",
            "function b() (uint256)",
            "function c() (bool, address, uint256)",
        ]);

        let (_, _, r1) = tc.encode_function("a", "()").unwrap();
        let (_, _, r2) = tc.encode_function("b", "()").unwrap();
        let (_, _, r3) = tc.encode_function("c", "()").unwrap();

        assert_eq!(None, r1);
        assert_eq!(Some(DynSolType::Uint(256)), r2);
        assert_eq!(
            Some(DynSolType::Tuple(vec![
                DynSolType::Bool,
                DynSolType::Address,
                DynSolType::Uint(256)
            ])),
            r3
        );
    }

    #[test]
    fn encoding_functions() {
        let hello_world = vec!["function hello(tuple(uint256, address, uint160)) (bool)"];
        let hw = ContractAbi::from_human_readable(hello_world);
        assert!(hw.has_function("hello"));

        let addy = Address::with_last_byte(24);

        assert!(hw.encode_function("bob", "()").is_err());
        assert!(hw.encode_function("hello", "(1,2").is_err());

        let (_, is_payable, dtype) = hw
            .encode_function("hello", &format!("(({}, {}, {}))", 10, addy.to_string(), 1))
            .unwrap();

        assert!(!is_payable);
        assert_eq!(dtype, Some(DynSolType::Bool));
    }

    #[test]
    fn encoding_overloaded_functions() {
        let overit = vec![
            "function one() (bool)",
            "function one(uint256)",
            "function one(address, (uint64, uint64)) (address)",
        ];
        let abi = ContractAbi::from_human_readable(overit);
        let addy = Address::with_last_byte(24);

        let (_, _, otype) = abi
            .encode_function("one", &format!("({},({},{}))", addy.to_string(), 10, 11))
            .unwrap();

        assert_eq!(Some(DynSolType::Address), otype);
    }

    #[test]
    fn test_flatten_event_structure() {
        // mint signature: 0x0f6798a560793a54c3bcfe86a93cde1e73087d944c0ea20544137d4121396885
        // burn signature: 0xcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5
        let sample = ContractAbi::from_human_readable(vec![
            "event Transfer(address indexed from,address indexed to,uint256 amount)",
            "event Transfer(address indexed from) anonymous",
            "event Mint(address indexed recip,uint256 amount)",
            "event Burn(address indexed recip,uint256 amount)",
        ]);

        assert_eq!(4, sample.events_logs.len());

        let transfer = LogData::new_unchecked(
            vec![
                b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
                b256!("000000000000000000000000c2e9f25be6257c210d7adf0d4cd6e3e881ba25f8"),
                b256!("0000000000000000000000002b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"),
            ],
            bytes!("0000000000000000000000000000000000000000000000000000000000000005"),
        );

        let burn = LogData::new_unchecked(
            vec![
                b256!("cc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5"),
                b256!("000000000000000000000000c2e9f25be6257c210d7adf0d4cd6e3e881ba25f8"),
            ],
            bytes!("0000000000000000000000000000000000000000000000000000000000000005"),
        );

        let log_address = Address::repeat_byte(14);
        let logs = vec![
            Log {
                address: log_address,
                data: transfer,
            },
            Log {
                address: log_address,
                data: burn,
            },
        ];

        let results = sample.extract_logs(logs);
        assert_eq!(2, results.len());

        //println!("{:?}", results);
    }
}
