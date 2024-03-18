use alloy_dyn_abi::{DynSolType, DynSolValue};
use anyhow::Result;
use ethers_providers::{Http, Provider};
use revm::{
    db::{CacheDB, EthersDB, InMemoryDB},
    primitives::{
        AccountInfo, Address, Bytecode, ExecutionResult, Log, Output, ResultAndState, TransactTo,
        TxEnv, KECCAK_EMPTY, U256,
    },
    ContextWithHandlerCfg, Database, DatabaseCommit, DatabaseRef, Evm, Handler,
};
use std::sync::Arc;

use crate::core::snapshot::{SerializableAccountRecord, SerializableState};

pub type ForkDb = CacheDB<EthersDB<Provider<Http>>>;

pub struct BaseEvm<DB: Database + DatabaseCommit> {
    state: Option<ContextWithHandlerCfg<(), DB>>,
}

impl BaseEvm<ForkDb> {
    pub fn create(url: &str) -> Self {
        let client =
            Provider::<Http>::try_from(url).expect("failed to load HTTP provider for forkdb");
        let client = Arc::new(client);
        let ethersdb = EthersDB::new(
            Arc::clone(&client), // public infura mainnet
            None,
        )
        .expect("failed to load ethersdb for forkdb");

        // Using EthersDb with CacheDB for a ForkDb
        let cache_db = CacheDB::new(ethersdb);
        let evm = Evm::builder().with_db(cache_db).build();
        Self {
            state: Some(evm.into_context_with_handler_cfg()),
        }
    }

    pub fn create_account(&mut self, caller: Address, amount: Option<U256>) -> Result<()> {
        let mut info = AccountInfo::default();
        if let Some(amnt) = amount {
            info.balance = amnt;
        }
        let mut evm = self.get_evm();
        evm.context.evm.db.insert_account_info(caller, info);
        self.state = Some(evm.into_context_with_handler_cfg());

        Ok(())
    }

    pub fn dump_state(&mut self) -> Result<SerializableState> {
        let mut evm = self.get_evm();
        // adapted from foundry-rs
        let accounts = evm
            .context
            .evm
            .db
            .accounts
            .clone()
            .into_iter()
            .map(|(k, v)| -> Result<(Address, SerializableAccountRecord)> {
                let code = if let Some(code) = v.info.code {
                    code
                } else {
                    evm.context.evm.db.code_by_hash(v.info.code_hash)?
                }
                .to_checked();
                Ok((
                    k,
                    SerializableAccountRecord {
                        nonce: v.info.nonce,
                        balance: v.info.balance,
                        code: code.original_bytes(),
                        storage: v.storage.into_iter().collect(),
                    },
                ))
            })
            .collect::<Result<_, _>>()?;

        self.state = Some(evm.into_context_with_handler_cfg());

        Ok(SerializableState {
            accounts,
            //contracts,
        })
    }
}

impl BaseEvm<InMemoryDB> {
    pub fn create() -> Self {
        let evm = Evm::builder().with_db(InMemoryDB::default()).build();
        Self {
            state: Some(evm.into_context_with_handler_cfg()),
        }
    }

    pub fn create_account(&mut self, caller: Address, amount: Option<U256>) -> Result<()> {
        let mut info = AccountInfo::default();
        if let Some(amnt) = amount {
            info.balance = amnt;
        }
        let mut evm = self.get_evm();
        evm.context.evm.db.insert_account_info(caller, info);
        self.state = Some(evm.into_context_with_handler_cfg());

        Ok(())
    }

    // only in memory db
    pub fn load_state(&mut self, cache: SerializableState) {
        let mut evm = self.get_evm();
        for (addr, account) in cache.accounts.into_iter() {
            // note: this will populate both 'accounts' and 'contracts'
            evm.context.evm.db.insert_account_info(
                addr,
                AccountInfo {
                    balance: account.balance,
                    nonce: account.nonce,
                    code_hash: KECCAK_EMPTY,
                    code: if account.code.0.is_empty() {
                        None
                    } else {
                        Some(
                            Bytecode::new_raw(alloy_primitives::Bytes(account.code.0)).to_checked(),
                        )
                    },
                },
            );

            // ... but we still need to load the account storage map
            for (k, v) in account.storage.into_iter() {
                evm.context
                    .evm
                    .db
                    .accounts
                    .entry(addr)
                    .or_default()
                    .storage
                    .insert(k, v);
            }
        }
        self.state = Some(evm.into_context_with_handler_cfg());
    }

    // in both memory and fork
    pub fn dump_state(&mut self) -> Result<SerializableState> {
        let mut evm = self.get_evm();
        // adapted from foundry-rs
        let accounts = evm
            .context
            .evm
            .db
            .accounts
            .clone()
            .into_iter()
            .map(|(k, v)| -> Result<(Address, SerializableAccountRecord)> {
                let code = if let Some(code) = v.info.code {
                    code
                } else {
                    evm.context.evm.db.code_by_hash(v.info.code_hash)?
                }
                .to_checked();
                Ok((
                    k,
                    SerializableAccountRecord {
                        nonce: v.info.nonce,
                        balance: v.info.balance,
                        code: code.original_bytes(),
                        storage: v.storage.into_iter().collect(),
                    },
                ))
            })
            .collect::<Result<_, _>>()?;

        self.state = Some(evm.into_context_with_handler_cfg());

        Ok(SerializableState {
            accounts,
            //contracts,
        })
    }
}

impl<DB: Database + DatabaseCommit + DatabaseRef> BaseEvm<DB> {
    fn get_evm(&mut self) -> Evm<(), DB> {
        match self.state.take() {
            Some(st) => {
                let ContextWithHandlerCfg { context, cfg } = st;
                Evm {
                    context,
                    handler: Handler::new(cfg),
                }
            }
            _ => panic!("EVM state is None"),
        }
    }

    pub fn view_storage_slot(&mut self, addr: Address, slot: U256) -> Result<U256> {
        let evm = self.get_evm();
        let r = evm
            .context
            .evm
            .db
            .storage_ref(addr, slot)
            .map_err(|_| anyhow::anyhow!("error viewing storage slot"))?;

        self.state = Some(evm.into_context_with_handler_cfg());
        Ok(r)
    }

    /// Get the balance for the account
    pub fn get_balance(&mut self, caller: Address) -> Result<U256> {
        let evm = self.get_evm();
        let result = match evm.context.evm.db.basic_ref(caller) {
            Ok(Some(account)) => account.balance,
            _ => U256::ZERO,
        };

        self.state = Some(evm.into_context_with_handler_cfg());
        Ok(result)
    }

    /// Deploy a contract
    pub fn deploy(&mut self, caller: Address, bincode: Vec<u8>, value: U256) -> Result<Address> {
        let tx = TxEnv {
            caller,
            transact_to: TransactTo::create(),
            data: bincode.into(),
            value,
            ..Default::default()
        };

        let mut evm = self.get_evm();
        evm.context.evm.env.tx = tx;

        let r = evm.transact_commit();
        self.state = Some(evm.into_context_with_handler_cfg());
        match r {
            Ok(result) => {
                let (output, _gas, _logs) = process_execution_result(result)?;
                match output {
                    Output::Create(_, Some(address)) => Ok(address),
                    _ => Err(anyhow::anyhow!("Error on deploy: expected a create call")),
                }
            }
            _ => Err(anyhow::anyhow!("Error on deploy")),
        }

        /*
        let (output, _, _) = evm
            .transact_commit()
            .map_err(|_| anyhow::anyhow!("error on deploy"))
            .and_then(process_execution_result)?;

        self.state = Some(evm.into_context_with_handler_cfg());

        match output {
            Output::Create(_, Some(address)) => Ok(address),
            _ => anyhow::bail!("expected a create call"),
        }
        */
    }

    /// Transfer value between two accounts. If the 'to' address is a contract, the should contract
    /// should have a [receive' or 'fallback](https://docs.soliditylang.org/en/latest/contracts.html#special-functions)
    pub fn transfer(&mut self, caller: Address, to: Address, amount: U256) -> Result<u64> {
        let tx = TxEnv {
            caller,
            transact_to: TransactTo::Call(to),
            value: amount,
            ..Default::default()
        };

        let mut evm = self.get_evm();
        evm.context.evm.env.tx = tx;

        let r = evm.transact_commit();
        self.state = Some(evm.into_context_with_handler_cfg());
        match r {
            Ok(result) => {
                let (_b, gas, _logs) = process_result_with_value(result)?;
                Ok(gas)
            }
            _ => Err(anyhow::anyhow!("Error on transfer")),
        }

        /*
        let result = match evm.transact_commit() {
            Ok(result) => {
                let (_b, gas, _logs) = process_result_with_value(result)?;
                Ok(gas)
            }
            Err(_) => Err(anyhow::anyhow!("Error on transfer")),
        };

        self.state = Some(evm.into_context_with_handler_cfg());
        result
        */
    }

    /// Send a write transaction `to` the given contract
    pub fn transact(
        &mut self,
        caller: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> Result<(Vec<u8>, u64)> {
        let tx = TxEnv {
            caller,
            transact_to: TransactTo::Call(to),
            data: data.into(),
            value,
            ..Default::default()
        };

        let mut evm = self.get_evm();
        evm.context.evm.env.tx = tx;

        let r = evm.transact_commit();
        self.state = Some(evm.into_context_with_handler_cfg());
        match r {
            Ok(result) => {
                let (b, gas, _logs) = process_result_with_value(result)?;
                Ok((b, gas))
            }
            _ => Err(anyhow::anyhow!("Error on transact")),
        }

        // TODO: because of state, need to handle errors better. bail! is returning early!
        /*
        let result = match evm.transact_commit() {
            Ok(result) => {
                self.state = Some(evm.into_context_with_handler_cfg());

                let (b, gas, _logs) = process_result_with_value(result)?;
                Ok((b, gas))
            }
            _ => {
                self.state = Some(evm.into_context_with_handler_cfg());

                // Can use bail again!!
                Err(anyhow::anyhow!("Error on write"))
            }
        };

        result
        */
    }

    /// Send a read-only (view) call `to` the given contract
    pub fn call(&mut self, to: Address, data: Vec<u8>) -> Result<(Vec<u8>, u64)> {
        let tx = TxEnv {
            transact_to: TransactTo::Call(to),
            data: data.into(),
            ..Default::default()
        };

        let mut evm = self.get_evm();
        evm.context.evm.env.tx = tx;

        let r = evm.transact();
        self.state = Some(evm.into_context_with_handler_cfg());
        match r {
            Ok(ResultAndState { result, .. }) => {
                let (r, gas, _) = process_result_with_value(result)?;
                Ok((r, gas))
            }
            _ => Err(anyhow::anyhow!("Error on call")),
        }

        /*
        let result = match evm.transact() {
            Ok(ResultAndState { result, .. }) => {
                let (r, gas, _) = process_result_with_value(result)?;
                Ok((r, gas))
            }
            _ => anyhow::bail!("error on read"),
        };

        self.state = Some(evm.into_context_with_handler_cfg());
        result
        */
    }
}

/// helper to extract results, also parses any revert message into a readable format
fn process_execution_result(result: ExecutionResult) -> Result<(Output, u64, Vec<Log>)> {
    match result {
        ExecutionResult::Success {
            output,
            gas_used,
            logs,
            ..
        } => Ok((output, gas_used, logs)),
        ExecutionResult::Revert { output, .. } => {
            let msg = parse_revert_message(output)?;
            anyhow::bail!("Call reverted. Reason: {:?}", msg)
        }
        ExecutionResult::Halt { reason, .. } => anyhow::bail!("Call halted. Reason: {:?}", reason),
    }
}

fn process_result_with_value(result: ExecutionResult) -> Result<(Vec<u8>, u64, Vec<Log>)> {
    let (output, gas_used, logs) = process_execution_result(result)?;
    let bits = match output {
        Output::Call(value) => value,
        _ => anyhow::bail!("Failed to process results of call: Expected call output"),
    };

    Ok((bits.to_vec(), gas_used, logs))
}

fn parse_revert_message(output: revm::primitives::Bytes) -> Result<String> {
    let ty = DynSolType::parse("string")?;
    let rd = ty.abi_decode_params(&output[4..])?;
    match rd {
        DynSolValue::String(v) => Ok(v),
        _ => anyhow::bail!("Revert: unable to parse revert message"),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bal_account() {
        let one_eth = U256::from(1e18);
        let bob = Address::repeat_byte(1);
        let mut evm = BaseEvm::<InMemoryDB>::create();
        assert_eq!(U256::from(0), evm.get_balance(bob).unwrap());

        evm.create_account(bob, Some(one_eth)).unwrap();

        assert_eq!(one_eth, evm.get_balance(bob).unwrap());
    }

    #[test]
    fn simple_transfer() {
        let one_eth = U256::from(1e18);
        let two_eth = U256::from(2e18);
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);

        let mut evm = BaseEvm::<InMemoryDB>::create();
        evm.create_account(a, Some(two_eth)).unwrap();

        let a1 = evm.get_balance(a).unwrap();
        let b1 = evm.get_balance(b).unwrap();
        assert_eq!(two_eth, a1);
        assert_eq!(U256::from(0), b1);

        // xfer 1 eth to b
        evm.transfer(a, b, U256::from(1e18)).unwrap();

        let a2 = evm.get_balance(a).unwrap();
        let b2 = evm.get_balance(b).unwrap();
        assert_eq!(one_eth, a2);
        assert_eq!(one_eth, b2);

        //let st = evm.dump_state().unwrap();
        //let r = serde_json::to_string(&st);
        //println!("{:?}", r);
    }

    #[test]
    fn transfer_fail() {
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);

        let mut evm = BaseEvm::<InMemoryDB>::create();
        // note: didn't fund account...
        evm.create_account(a, None).unwrap();

        let a1 = evm.get_balance(a).unwrap();
        let b1 = evm.get_balance(b).unwrap();
        assert_eq!(U256::from(0), a1);
        assert_eq!(U256::from(0), b1);

        // nothing to xfer, caller has no balance
        assert!(evm.transfer(a, b, U256::from(1e18)).is_err());
    }
}
