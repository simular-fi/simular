//! The in memory DB
//! ADAPTED FROM Foundry-rs
//! https://github.com/foundry-rs/foundry/blob/master/crates/evm/core/src/backend/in_memory_db.rs
//!
use crate::core::{
    errors::DatabaseError,
    snapshot::{SnapShot, SnapShotAccountRecord, SnapShotSource},
};
use alloy_primitives::{Address, B256, U256};
use revm::{
    db::{CacheDB, DatabaseRef, EmptyDB},
    primitives::{Account, AccountInfo, Bytecode, HashMap as Map},
    Database, DatabaseCommit,
};

///
/// This acts like a wrapper type for [InMemoryDB] but is capable of creating/applying snapshots
#[derive(Debug)]
pub struct MemDb {
    pub db: CacheDB<EmptyDBWrapper>,
}

impl Default for MemDb {
    fn default() -> Self {
        Self {
            db: CacheDB::new(Default::default()),
        }
    }
}

impl MemDb {
    pub fn create_snapshot(&self, block_num: u64, timestamp: u64) -> anyhow::Result<SnapShot> {
        let accounts = self
            .db
            .accounts
            .clone()
            .into_iter()
            .map(
                |(k, v)| -> anyhow::Result<(Address, SnapShotAccountRecord)> {
                    let code = if let Some(code) = v.info.code {
                        code
                    } else {
                        self.db.code_by_hash_ref(v.info.code_hash)?
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
            source: SnapShotSource::Memory,
            accounts,
        })
    }
}

impl DatabaseRef for MemDb {
    type Error = DatabaseError;
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        DatabaseRef::basic_ref(&self.db, address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        DatabaseRef::code_by_hash_ref(&self.db, code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        DatabaseRef::storage_ref(&self.db, address, index)
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        DatabaseRef::block_hash_ref(&self.db, number)
    }
}

impl Database for MemDb {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // Note: this will always return `Some(AccountInfo)`, See `EmptyDBWrapper`
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

impl DatabaseCommit for MemDb {
    fn commit(&mut self, changes: Map<Address, Account>) {
        DatabaseCommit::commit(&mut self.db, changes)
    }
}

/// *documentation from foundry-rs*
///
/// An empty database that always returns default values when queried.
///
/// This is just a simple wrapper for `revm::EmptyDB` but implements `DatabaseError` instead, this
/// way we can unify all different `Database` impls
///
/// This will also _always_ return `Some(AccountInfo)`:
///
/// The [`Database`](revm::Database) implementation for `CacheDB` manages an `AccountState` for the
/// `DbAccount`, this will be set to `AccountState::NotExisting` if the account does not exist yet.
/// This is because there's a distinction between "non-existing" and "empty",
/// see <https://github.com/bluealloy/revm/blob/8f4348dc93022cffb3730d9db5d3ab1aad77676a/crates/revm/src/db/in_memory_db.rs#L81-L83>.
/// If an account is `NotExisting`, `Database::basic_ref` will always return `None` for the
/// requested `AccountInfo`.
///
/// To prevent this, we ensure that a missing account is never marked as `NotExisting` by always
/// returning `Some` with this type, which will then insert a default [`AccountInfo`] instead
/// of one marked as `AccountState::NotExisting`.
#[derive(Clone, Debug, Default)]
pub struct EmptyDBWrapper(EmptyDB);

impl DatabaseRef for EmptyDBWrapper {
    type Error = DatabaseError;

    fn basic_ref(&self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // Note: this will always return `Some(AccountInfo)`, for the reason explained above
        Ok(Some(AccountInfo::default()))
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(self.0.code_by_hash_ref(code_hash)?)
    }
    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self.0.storage_ref(address, index)?)
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(self.0.block_hash_ref(number)?)
    }
}
