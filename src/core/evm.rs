//!
//! An API to interact with an embedded Ethereum Virtual Machine.
//!
//! This is wrapper around [REVM](https://docs.rs/revm/latest/revm/index.html).  The implementation
//! is a simplfied version of [Foundry's Executor](https://github.com/foundry-rs/foundry)
//!

use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::decode_revert_reason;
use anyhow::{anyhow, bail, Result};
use revm::{
    db::{DatabaseCommit, DatabaseRef},
    primitives::{
        Account, AccountInfo, BlockEnv, Env, EnvWithHandlerCfg, ExecutionResult, HashMap as Map,
        Log, Output, ResultAndState, TransactTo, TxEnv,
    },
};

use crate::{core::snapshot::SnapShot, core::storage::CreateFork, core::storage::StorageBackend};

/// type alias for a `revm` hashmap of `Address` => `Account`
type StateChangeSet = Map<Address, Account>;

/// EVM that supports both in-memory and forked storage.
pub struct BaseEvm {
    backend: StorageBackend,
    env: EnvWithHandlerCfg,
}

/// Create an EVM with the in-memory database
impl Default for BaseEvm {
    fn default() -> Self {
        BaseEvm::new(None)
    }
}

impl BaseEvm {
    /// Create an instance of the EVM.  If fork is None it will use the in-memory database.
    /// Otherwise it will create a forked database.
    pub fn new(fork: Option<CreateFork>) -> Self {
        let env = EnvWithHandlerCfg::default();
        let backend = StorageBackend::new(fork);
        Self { env, backend }
    }

    /// Create an instance of the EVM and load it's state from the `SnapShot`.  This
    /// will use the in-memory database.
    pub fn new_from_snapshot(snap: SnapShot) -> Self {
        let env = EnvWithHandlerCfg::default();
        let mut backend = StorageBackend::default();
        backend.load_snapshot(snap);
        Self { env, backend }
    }

    /// Create an account for the given `user` with an optional balance (`amount`).
    /// This will overwrite an account if it already exists.
    pub fn create_account(&mut self, user: Address, amount: Option<U256>) -> Result<()> {
        let mut info = AccountInfo::default();
        if let Some(amnt) = amount {
            info.balance = amnt;
        }
        self.backend.insert_account_info(user, info);
        Ok(())
    }

    /// Return the balance for the `caller`'s account.
    pub fn get_balance(&mut self, caller: Address) -> Result<U256> {
        Ok(self
            .backend
            .basic_ref(caller)?
            .map(|acc| acc.balance)
            .unwrap_or_default())
    }

    /*
    /// Set the balance for the given `address` with the given `amount`
    pub fn set_balance(&mut self, address: Address, amount: U256) -> Result<&mut Self> {
        let mut account = self.backend.basic_ref(address)?.unwrap_or_default();
        account.balance = amount;

        self.backend.insert_account_info(address, account);
        Ok(self)
    }
    */

    /// Create a snapshot of the current database. This can be used to reload state.
    pub fn create_snapshot(&self) -> Result<SnapShot> {
        self.backend.create_snapshot()
    }

    /// Deploy a contract returning the contract's address.
    /// If `value` is specified, the constructor must be `payable`.
    pub fn deploy(&mut self, caller: Address, data: Vec<u8>, value: U256) -> Result<Address> {
        let mut env = self.build_env(Some(caller), TransactTo::create(), data.into(), value);
        let result = self.backend.run_transact(&mut env)?;
        let mut call_results = process_call_result(result)?;
        self.commit(&mut call_results);

        match call_results.address {
            Some(addr) => Ok(addr),
            _ => Err(anyhow!("deploy did not return an Address!")),
        }
    }

    /// Transfer `value` from `caller` -> `to`
    pub fn transfer(&mut self, caller: Address, to: Address, value: U256) -> Result<()> {
        let _ = self.transact_commit(caller, to, vec![], value)?;
        Ok(())
    }

    /* TODO Remove?
    /// Same as `transact_commit`, but supports [alloy's sol types](https://docs.rs/alloy-sol-types/latest/alloy_sol_types/index.html).
    pub fn transact_commit_sol<T: SolCall>(
        &mut self,
        caller: Address,
        to: Address,
        args: T,
        value: U256,
    ) -> Result<<T as SolCall>::Return> {
        let data = args.abi_encode();
        let result = self.transact_commit(caller, to, data, value)?;
        T::abi_decode_returns(&result.result, true)
            .map_err(|e| anyhow!("transact commit sol error: {:?}", e))
    }
    */

    /// Write call to a contact.  Send a transaction where any state changes are persisted to the underlying database.
    pub fn transact_commit(
        &mut self,
        caller: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> Result<CallResult> {
        let mut env = self.build_env(Some(caller), TransactTo::call(to), data.into(), value);
        let result = self.backend.run_transact(&mut env)?;
        let mut call_results = process_call_result(result)?;
        self.commit(&mut call_results);

        Ok(call_results)
    }

    /* TODO remove
    /// Same as `transact_call` but supports [alloy's sol types](https://docs.rs/alloy-sol-types/latest/alloy_sol_types/index.html).
    pub fn transact_call_sol<T: SolCall>(
        &mut self,
        to: Address,
        args: T,
        value: U256,
    ) -> Result<<T as SolCall>::Return> {
        let data = args.abi_encode();
        let result = self.transact_call(to, data, value)?;
        T::abi_decode_returns(&result.result, true)
            .map_err(|e| anyhow!("transact call sol error: {:?}", e))
    }
    */

    /// Read call to a contract.  Send a transaction but any state changes are NOT persisted to the
    /// database.   
    pub fn transact_call(&mut self, to: Address, data: Vec<u8>, value: U256) -> Result<CallResult> {
        let mut env = self.build_env(None, TransactTo::call(to), data.into(), value);
        let result = self.backend.run_transact(&mut env)?;
        process_call_result(result)
    }

    /// Simulate a `transact_commit` without actually committing/changing state.
    pub fn simulate(
        &mut self,
        caller: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> Result<CallResult> {
        let mut env = self.build_env(Some(caller), TransactTo::call(to), data.into(), value);
        let result = self.backend.run_transact(&mut env)?;
        process_call_result(result)
    }

    /// Advance `block.number` and `block.timestamp`. Set `interval` to the
    /// amount of time in seconds you want to advance the timestamp. Block number
    /// will be automatically incremented.
    ///
    /// Must be manually called.
    pub fn update_block(&mut self, interval: u64) {
        self.backend.update_block_info(interval);
    }

    fn build_env(
        &self,
        caller: Option<Address>,
        transact_to: TransactTo,
        data: Bytes,
        value: U256,
    ) -> EnvWithHandlerCfg {
        let blkn = self.backend.block_number;
        let ts = self.backend.timestamp;

        let env = Env {
            cfg: self.env.cfg.clone(),
            block: BlockEnv {
                basefee: U256::ZERO,
                timestamp: U256::from(ts),
                number: U256::from(blkn),
                ..self.env.block.clone()
            },
            tx: TxEnv {
                caller: caller.unwrap_or(Address::ZERO),
                transact_to,
                data,
                value,
                gas_price: U256::ZERO,
                gas_priority_fee: None,
                ..self.env.tx.clone()
            },
        };

        EnvWithHandlerCfg::new_with_spec_id(Box::new(env), self.env.handler_cfg.spec_id)
    }

    fn commit(&mut self, result: &mut CallResult) {
        if let Some(changes) = &result.state_changeset {
            self.backend.commit(changes.clone());
        }
    }
}

/// Container for the results of a transaction
pub struct CallResult {
    /// The raw result of the call.
    pub result: Bytes,
    /// An address if the call is a TransactTo::create (deploy)
    pub address: Option<Address>,
    /// The gas used for the call
    pub gas_used: u64,
    /// Refunded gas
    pub gas_refunded: u64,
    /// The logs emitted during the call
    pub logs: Vec<Log>,
    /// Changes made to the database
    pub state_changeset: Option<StateChangeSet>,
}

fn process_call_result(result: ResultAndState) -> Result<CallResult> {
    let ResultAndState {
        result: exec_result,
        state: state_changeset,
    } = result;

    let (gas_refunded, gas_used, out, logs) = match exec_result {
        ExecutionResult::Success {
            gas_used,
            gas_refunded,
            output,
            logs,
            ..
        } => (gas_refunded, gas_used, output, logs),
        ExecutionResult::Revert { gas_used, output } => match decode_revert_reason(&output) {
            Some(reason) => bail!("Reverted: {:?}. Gas used: {:?}", reason, gas_used),
            _ => bail!("Reverted with no reason. Gas used: {:?}", gas_used),
        },
        ExecutionResult::Halt { reason, gas_used } => {
            bail!("Halted: {:?}. Gas used: {:?}", reason, gas_used)
        }
    };

    match out {
        Output::Call(result) => Ok(CallResult {
            result,
            gas_used,
            gas_refunded,
            logs,
            address: None,
            state_changeset: Some(state_changeset),
        }),
        Output::Create(data, address) => Ok(CallResult {
            result: data.clone(),
            address,
            gas_used,
            logs,
            gas_refunded,
            state_changeset: Some(state_changeset),
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::abi::ContractAbi;
    use crate::core::evm::BaseEvm;
    use alloy_dyn_abi::DynSolValue;
    use alloy_primitives::{Address, U256};

    const BYTECODE: &str = "608060405260405161032c38038061032c8339810160408190526100\
        229161003c565b600155600080546001600160a01b03191633179055610055565b6000602\
        0828403121561004e57600080fd5b5051919050565b6102c8806100646000396000f3fe60\
        80604052600436106100555760003560e01c80633fa4f2451461005a57806361fa423b146\
        100835780637cf5dab0146100b35780638da5cb5b146100e8578063d09de08a1461012057\
        8063d0e30db014610135575b600080fd5b34801561006657600080fd5b506100706001548\
        1565b6040519081526020015b60405180910390f35b34801561008f57600080fd5b506100\
        a361009e36600461020a565b610137565b604051901515815260200161007a565b3480156\
        100bf57600080fd5b506100d36100ce366004610222565b6101c8565b6040805192835260\
        208301919091520161007a565b3480156100f457600080fd5b50600054610108906001600\
        160a01b031681565b6040516001600160a01b03909116815260200161007a565b34801561\
        012c57600080fd5b506100706101ec565b005b600080546001600160a01b0316331461018\
        e5760405162461bcd60e51b81526020600482015260156024820152743737ba103a343290\
        31bab93932b73a1037bbb732b960591b604482015260640160405180910390fd5b61019b6\
        02083018361023b565b600080546001600160a01b0319166001600160a01b039290921691\
        90911790555060200135600190815590565b60008082600160008282546101dd919061026\
        b565b90915550506001549293915050565b6001805460009180836101ff828561026b565b\
        909155509092915050565b60006040828403121561021c57600080fd5b50919050565b600\
        06020828403121561023457600080fd5b5035919050565b60006020828403121561024d57\
        600080fd5b81356001600160a01b038116811461026457600080fd5b9392505050565b808\
        2018082111561028c57634e487b7160e01b600052601160045260246000fd5b9291505056\
        fea264697066735822122073a633ec59ee8e261bbdfefdc6d54f1d47dd6ccd6dcab4aa1eb\
        37b62d24b4c1b64736f6c63430008140033";

    #[test]
    fn balances() {
        let zero = U256::from(0);
        //let one_eth = U256::from(1e18);

        let mut evm = BaseEvm::default();
        let bob = Address::repeat_byte(23);

        evm.create_account(bob, None).unwrap();
        assert!(evm.get_balance(bob).unwrap() == zero);

        //evm.set_balance(bob, one_eth).unwrap();
        //assert!(evm.get_balance(bob).unwrap() == one_eth);
    }

    #[test]
    fn simple_transfers() {
        let one_eth = U256::from(1e18);
        //let addresses = generate_random_addresses(2);
        let bob = Address::repeat_byte(23);
        let alice = Address::repeat_byte(24);

        let mut evm = BaseEvm::new(None);
        evm.create_account(bob, Some(U256::from(2e18))).unwrap();
        evm.create_account(alice, None).unwrap();

        assert!(evm.transfer(alice, bob, one_eth).is_err()); // alice has nothing to transfer...yet
        assert!(evm.transfer(bob, alice, one_eth).is_ok());

        assert!(evm.get_balance(bob).unwrap() == one_eth);
        assert!(evm.get_balance(alice).unwrap() == one_eth);

        let s = evm.create_snapshot();
        println!("{:?}", s);
    }

    #[test]
    fn no_sol_test_contract() {
        let contract_bytecode = hex::decode(BYTECODE).expect("failed to decode bytecode");

        let zero = U256::from(0);
        let owner = Address::repeat_byte(12);
        let mut evm = BaseEvm::default();
        evm.create_account(owner, Some(U256::from(1e18))).unwrap();

        let mut test_contract_abi = ContractAbi::from_human_readable(vec![
            "constructor(uint256)",
            "function owner() (address)",
            "function value() (uint256)",
            "function increment() (uint256)",
            "function increment(uint256) (uint256, uint256)",
        ]);
        test_contract_abi.bytecode = Some(contract_bytecode.into());

        let (args, _) = test_contract_abi.encode_constructor("(1)").unwrap();
        let contract_address = evm.deploy(owner, args, U256::from(0)).unwrap();

        // Check owner call
        let (enc_owner_call, _, de1) = test_contract_abi.encode_function("owner", "()").unwrap();
        let o1 = evm
            .transact_call(contract_address, enc_owner_call, zero)
            .unwrap();
        assert!(DynSolValue::Address(owner) == de1.unwrap().abi_decode(&o1.result).unwrap());

        // do increment()
        let (enc_inc_0, _, de2) = test_contract_abi
            .encode_function("increment", "()")
            .unwrap();
        let o2 = evm
            .transact_commit(owner, contract_address, enc_inc_0, zero)
            .unwrap();
        assert!(
            DynSolValue::Uint(U256::from(1), 256) == de2.unwrap().abi_decode(&o2.result).unwrap()
        );

        // check the value
        let (enc_value_call, _, de3) = test_contract_abi.encode_function("value", "()").unwrap();
        let o3 = evm
            .transact_call(contract_address, enc_value_call, zero)
            .unwrap();
        assert!(
            DynSolValue::Uint(U256::from(2), 256) == de3.unwrap().abi_decode(&o3.result).unwrap()
        );

        // do increment(value)
        let (enc_inc_1, _, de4) = test_contract_abi
            .encode_function("increment", "(2)")
            .unwrap();
        let o4 = evm
            .transact_commit(owner, contract_address, enc_inc_1, zero)
            .unwrap();
        assert!(
            DynSolValue::Tuple(vec![
                DynSolValue::Uint(U256::from(2), 256),
                DynSolValue::Uint(U256::from(4), 256)
            ]) == de4.unwrap().abi_decode(&o4.result).unwrap()
        );

        // simulate increment
        let (enc_inc_sim, _, des) = test_contract_abi
            .encode_function("increment", "()")
            .unwrap();
        let os = evm
            .simulate(owner, contract_address, enc_inc_sim, zero)
            .unwrap();
        assert!(
            DynSolValue::Uint(U256::from(4), 256) == des.unwrap().abi_decode(&os.result).unwrap()
        );

        // make sure value didn't change from 'simulate'
        let (enc_value_call1, _, de5) = test_contract_abi.encode_function("value", "()").unwrap();
        let o5 = evm
            .transact_call(contract_address, enc_value_call1, zero)
            .unwrap();
        assert!(
            DynSolValue::Uint(U256::from(4), 256) == de5.unwrap().abi_decode(&o5.result).unwrap()
        );
    }
}
