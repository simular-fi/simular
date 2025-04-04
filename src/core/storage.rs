//!
//! Provides access to EVM storage
//!

use alloy_primitives::{Address, U256};
use anyhow::{anyhow, Result};
use revm::{
    interpreter::primitives::EnvWithHandlerCfg,
    primitives::{
        Account, AccountInfo, Bytecode, HashMap as Map, ResultAndState, B256, KECCAK_EMPTY,
    },
    Database, DatabaseCommit, DatabaseRef, EvmBuilder,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::{errors::DatabaseError, snapshot::SnapShot};
use crate::core::{fork::Fork, in_memory_db::MemDb};

/// Information related to creating a fork
#[derive(Clone, Debug)]
pub struct CreateFork {
    /// the url of the RPC endpoint
    pub url: String,
    /// optional block number of the fork.  If none, it will use the latest block.
    pub blocknumber: Option<u64>,
}

/*
impl CreateFork {
    /// Fork at the given URL and block number
    pub fn new(url: String, blocknumber: Option<u64>) -> Self {
        Self { url, blocknumber }
    }

    /// For at the given URL and use the latest block available
    pub fn latest_block(url: String) -> Self {
        Self {
            url,
            blocknumber: None,
        }
    }
}
*/

// Used by the EVM to access storage.  This can either be an in-memory only db or a forked db.
// The EVM delegates transact() and transact_commit to this module
//
// This is based heavily on Foundry's approach.
pub struct StorageBackend {
    mem_db: MemDb, // impl wrapper to handle DbErrors
    forkdb: Option<Fork>,
    pub block_number: u64, // used to record in the snapshot...
    pub timestamp: u64,
}

impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend::new(None)
    }
}

impl StorageBackend {
    pub fn new(fork: Option<CreateFork>) -> Self {
        if let Some(fork) = fork {
            let backend = Fork::new(&fork.url, fork.blocknumber);
            let block_number = backend.block_number;
            let timestamp = backend.timestamp;
            Self {
                mem_db: MemDb::default(),
                forkdb: Some(backend),
                block_number,
                timestamp,
            }
        } else {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("StorageBackend: failed to get unix epoch time")
                .as_secs();
            Self {
                mem_db: MemDb::default(),
                forkdb: None,
                block_number: 1,
                timestamp,
            }
        }
    }

    pub fn insert_account_info(&mut self, address: Address, info: AccountInfo) {
        if let Some(fork) = self.forkdb.as_mut() {
            fork.database_mut().insert_account_info(address, info)
        } else {
            // use mem...
            self.mem_db.db.insert_account_info(address, info)
        }
    }

    /*
    pub fn insert_account_storage(
        &mut self,
        address: Address,
        slot: U256,
        value: U256,
    ) -> Result<(), DatabaseError> {
        let ret = if let Some(fork) = self.forkdb.as_mut() {
            fork.database_mut()
                .insert_account_storage(address, slot, value)
        } else {
            self.mem_db.db.insert_account_storage(address, slot, value)
        };
        ret
    }

    pub fn replace_account_storage(
        &mut self,
        address: Address,
        storage: Map<U256, U256>,
    ) -> Result<(), DatabaseError> {
        if let Some(fork) = self.forkdb.as_mut() {
            fork.database_mut()
                .replace_account_storage(address, storage)
        } else {
            self.mem_db.db.replace_account_storage(address, storage)
        }
    }
    */

    pub fn run_transact(&mut self, env: &mut EnvWithHandlerCfg) -> Result<ResultAndState> {
        let mut evm = create_evm(self, env.clone());
        let res = evm
            .transact()
            .map_err(|e| anyhow!("backend failed while executing transaction:  {:?}", e))?;
        env.env = evm.context.evm.inner.env;

        Ok(res)
    }

    /// Create a snapshot of the current state, delegates
    /// to the current backend database.
    pub fn create_snapshot(&self) -> Result<SnapShot> {
        if let Some(fork) = self.forkdb.as_ref() {
            fork.create_snapshot(self.block_number, self.timestamp)
        } else {
            self.mem_db
                .create_snapshot(self.block_number, self.timestamp)
        }
    }

    /// Load a snapshot into an in-memory database
    pub fn load_snapshot(&mut self, snapshot: SnapShot) {
        self.block_number = snapshot.block_num;
        self.timestamp = snapshot.timestamp;

        for (addr, account) in snapshot.accounts.into_iter() {
            // note: this will populate both 'accounts' and 'contracts'
            self.mem_db.db.insert_account_info(
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
                self.mem_db
                    .db
                    .accounts
                    .entry(addr)
                    .or_default()
                    .storage
                    .insert(k, v);
            }
        }
    }

    /// See EVM update_block
    pub fn update_block_info(&mut self, interval: u64) {
        self.block_number += 1;
        self.timestamp += interval;
    }
}

impl DatabaseRef for StorageBackend {
    type Error = DatabaseError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(db) = self.forkdb.as_ref() {
            db.basic_ref(address)
        } else {
            Ok(self.mem_db.basic_ref(address)?)
        }
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(db) = self.forkdb.as_ref() {
            db.code_by_hash_ref(code_hash)
        } else {
            Ok(self.mem_db.code_by_hash_ref(code_hash)?)
        }
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Some(db) = self.forkdb.as_ref() {
            DatabaseRef::storage_ref(db, address, index)
        } else {
            Ok(DatabaseRef::storage_ref(&self.mem_db, address, index)?)
        }
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        if let Some(db) = self.forkdb.as_ref() {
            db.block_hash_ref(number)
        } else {
            Ok(self.mem_db.block_hash_ref(number)?)
        }
    }
}

impl Database for StorageBackend {
    type Error = DatabaseError;
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(db) = self.forkdb.as_mut() {
            db.basic(address)
        } else {
            Ok(self.mem_db.basic(address)?)
        }
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if let Some(db) = self.forkdb.as_mut() {
            db.code_by_hash(code_hash)
        } else {
            Ok(self.mem_db.code_by_hash(code_hash)?)
        }
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        if let Some(db) = self.forkdb.as_mut() {
            Database::storage(db, address, index)
        } else {
            Ok(Database::storage(&mut self.mem_db, address, index)?)
        }
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        if let Some(db) = self.forkdb.as_mut() {
            db.block_hash(number)
        } else {
            Ok(self.mem_db.block_hash(number)?)
        }
    }
}

impl DatabaseCommit for StorageBackend {
    fn commit(&mut self, changes: Map<Address, Account>) {
        if let Some(db) = self.forkdb.as_mut() {
            db.commit(changes)
        } else {
            self.mem_db.commit(changes)
        }
    }
}

fn create_evm<'a, DB: Database>(
    db: DB,
    env: revm::primitives::EnvWithHandlerCfg,
) -> revm::Evm<'a, (), DB> {
    EvmBuilder::default()
        .with_db(db)
        .with_env(env.env.clone())
        .build()
}
