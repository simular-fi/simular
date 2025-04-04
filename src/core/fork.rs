//

use crate::core::{
    errors::DatabaseError,
    fork_backend::ForkBackend,
    snapshot::{SnapShot, SnapShotAccountRecord, SnapShotSource},
};
use alloy_primitives::U256;
use revm::db::{CacheDB, DatabaseRef};
use revm::primitives::Address;
use revm::primitives::{Account, AccountInfo, Bytecode, HashMap as Map, B256};
use revm::{Database, DatabaseCommit};

#[derive(Clone, Debug)]
pub struct Fork {
    pub db: CacheDB<ForkBackend>,
    pub block_number: u64,
    pub timestamp: u64,
}

impl Fork {
    pub fn new(url: &str, starting_block_number: Option<u64>) -> Self {
        let backend = ForkBackend::new(url, starting_block_number);
        let block_number = backend.block_number;
        let timestamp = backend.timestamp;
        Self {
            db: CacheDB::new(backend),
            block_number,
            timestamp,
        }
    }

    pub fn database(&self) -> &CacheDB<ForkBackend> {
        &self.db
    }

    pub fn database_mut(&mut self) -> &mut CacheDB<ForkBackend> {
        &mut self.db
    }

    pub fn create_snapshot(&self, block_num: u64, timestamp: u64) -> anyhow::Result<SnapShot> {
        let accounts = self
            .database()
            .accounts
            .clone()
            .into_iter()
            .map(
                |(k, v)| -> anyhow::Result<(Address, SnapShotAccountRecord)> {
                    let code = if let Some(code) = v.info.code {
                        code
                    } else {
                        self.database().code_by_hash_ref(v.info.code_hash)?
                    }
                    .to_checked();
                    Ok((
                        k,
                        SnapShotAccountRecord {
                            nonce: v.info.nonce,
                            balance: v.info.balance,
                            code: code.original_bytes(),
                            storage: v.storage.into_iter().collect(),
                        },
                    ))
                },
            )
            .collect::<Result<_, _>>()?;
        Ok(SnapShot {
            block_num,
            timestamp,
            source: SnapShotSource::Fork,
            accounts,
        })
    }
}

impl Database for Fork {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // Note: this will always return Some, since the `SharedBackend` will always load the
        // account, this differs from `<CacheDB as Database>::basic`, See also
        // [MemDb::ensure_loaded](crate::backend::MemDb::ensure_loaded)
        Database::basic(&mut self.db, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Database::code_by_hash(&mut self.db, code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Database::storage(&mut self.db, address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        Database::block_hash(&mut self.db, number)
    }
}

impl DatabaseRef for Fork {
    type Error = DatabaseError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.db.basic_ref(address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db.code_by_hash_ref(code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        DatabaseRef::storage_ref(&self.db, address, index)
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        self.db.block_hash_ref(number)
    }
}

impl DatabaseCommit for Fork {
    fn commit(&mut self, changes: Map<Address, Account>) {
        self.database_mut().commit(changes)
    }
}
