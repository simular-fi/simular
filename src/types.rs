use anyhow::Result;
use revm::{
    db::{CacheDB, EthersDB, InMemoryDB},
    primitives::{
        AccountInfo, Address, ExecutionResult, Log, Output, ResultAndState, TransactTo, TxEnv, U256,
    },
    ContextWithHandlerCfg, Database, DatabaseCommit, Evm, Handler,
};

pub trait SimEvm {
    fn create_account(&mut self, caller: Address, amount: Option<U256>) -> Result<()>;
}
