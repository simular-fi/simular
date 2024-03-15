use alloy_dyn_abi::{DynSolType, DynSolValue};
use anyhow::Result;

use revm::{
    db::InMemoryDB,
    primitives::{
        AccountInfo, Address, ExecutionResult, Log, Output, ResultAndState, TransactTo, TxEnv, U256,
    },
    ContextWithHandlerCfg, Database, Evm, Handler,
};

pub struct BaseEvm {
    //evm: Evm<'a, (), InMemoryDB>,
    state: Option<ContextWithHandlerCfg<(), InMemoryDB>>,
}

impl Default for BaseEvm {
    fn default() -> Self {
        let evm = Evm::builder().with_db(InMemoryDB::default()).build();
        Self {
            state: Some(evm.into_context_with_handler_cfg()),
        }
    }
}

impl BaseEvm {
    fn get_evm(&mut self) -> Evm<(), InMemoryDB> {
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

    /// Get the balance for the account
    pub fn get_balance(&mut self, caller: Address) -> Result<U256> {
        let mut evm = self.get_evm();
        let result = match evm.context.evm.db.basic(caller) {
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

        let (output, _, _) = evm
            .transact_commit()
            .map_err(|e| anyhow::anyhow!("error on deploy: {:?}", e))
            .and_then(process_execution_result)?;

        self.state = Some(evm.into_context_with_handler_cfg());

        match output {
            Output::Create(_, Some(address)) => Ok(address),
            _ => anyhow::bail!("expected a create call"),
        }
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

        match evm.transact_commit() {
            Ok(result) => {
                let (_b, gas, _logs) = process_result_with_value(result)?;
                self.state = Some(evm.into_context_with_handler_cfg());
                Ok(gas)
            }
            Err(e) => Err(anyhow::anyhow!("Error on transfer: {:?}", e)),
        }
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

        match evm.transact_commit() {
            Ok(result) => {
                let (b, gas, _logs) = process_result_with_value(result)?;
                self.state = Some(evm.into_context_with_handler_cfg());
                Ok((b, gas))
            }
            _ => anyhow::bail!("Error on write"),
        }
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
        match evm.transact() {
            Ok(ResultAndState { result, .. }) => {
                let (r, gas, _) = process_result_with_value(result)?;
                self.state = Some(evm.into_context_with_handler_cfg());
                Ok((r, gas))
            }
            _ => anyhow::bail!("error on read"),
        }
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
    fn simple_transfer() {
        let one_eth = U256::from(1e18);
        let two_eth = U256::from(2e18);
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);

        let mut evm = BaseEvm::default();
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
    }

    #[test]
    fn transfer_fail() {
        let a = Address::repeat_byte(1);
        let b = Address::repeat_byte(2);

        let mut evm = BaseEvm::default();
        evm.create_account(a, None).unwrap();

        let a1 = evm.get_balance(a).unwrap();
        let b1 = evm.get_balance(b).unwrap();
        assert_eq!(U256::from(0), a1);
        assert_eq!(U256::from(0), b1);

        // nothing to xfer
        assert!(evm.transfer(a, b, U256::from(1e18)).is_err());
    }
}
