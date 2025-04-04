use alloy_primitives::{Address, U256};
use anyhow::Result;
use ethers_core::types::{Block, BlockId, BlockNumber, TxHash, H160, H256, U64};
use ethers_providers::{Http, Middleware, Provider, ProviderError};
use revm::{
    primitives::{AccountInfo, Bytecode, B256, KECCAK_EMPTY},
    DatabaseRef,
};
use std::sync::Arc;
use tokio::runtime::{Builder, Handle, RuntimeFlavor};

use crate::core::errors::DatabaseError;

pub type HttpProvider = Provider<Http>;

#[derive(Clone, Debug)]
pub struct ForkBackend {
    provider: Arc<HttpProvider>,
    pub block_number: u64,
    pub timestamp: u64,
}

impl ForkBackend {
    pub fn new(url: &str, starting_block_number: Option<u64>) -> Self {
        let client =
            Provider::<Http>::try_from(url).expect("ForkBackend: failed to load HTTP provider");
        let provider = Arc::new(client);

        let blockid = if let Some(bn) = starting_block_number {
            BlockId::from(U64::from(bn))
        } else {
            BlockId::from(BlockNumber::Latest)
        };

        let blk = match Self::block_on(provider.get_block(blockid)) {
            Ok(Some(b)) => b,
            _ => panic!("ForkBackend: failed to load block information"),
        };

        let block_number = blk
            .number
            .expect("ForkBackend: Got 'pending' block number")
            .as_u64();
        let timestamp = blk.timestamp.as_u64();
        /*
        let block_number = if let Some(bn) = starting_block_number {
            bn
        } else {
            Self::block_on(provider.get_block_number())
                .expect("ForkBackend: failed to load latest blocknumber from remote")
                .as_u64()
        };
        */

        Self {
            provider,
            block_number,
            timestamp,
        }
    }

    // adapted from revm ethersdb
    #[inline]
    fn block_on<F>(f: F) -> F::Output
    where
        F: core::future::Future + Send,
        F::Output: Send,
    {
        match Handle::try_current() {
            Ok(handle) => match handle.runtime_flavor() {
                RuntimeFlavor::CurrentThread => std::thread::scope(move |s| {
                    s.spawn(move || {
                        Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(f)
                    })
                    .join()
                    .unwrap()
                }),
                _ => tokio::task::block_in_place(move || handle.block_on(f)),
            },
            Err(_) => Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(f),
        }
    }

    fn fetch_basic_from_fork(&self, address: Address) -> Result<AccountInfo, ProviderError> {
        let add = H160::from(address.0 .0);
        let bn: Option<BlockId> = Some(BlockId::from(self.block_number));

        let f = async {
            let nonce = self.provider.get_transaction_count(add, bn);
            let balance = self.provider.get_balance(add, bn);
            let code = self.provider.get_code(add, bn);
            tokio::join!(nonce, balance, code)
        };
        let (nonce, balance, code) = Self::block_on(f);

        let balance = U256::from_limbs(balance?.0);
        let nonce = nonce?.as_u64();
        let bytecode = Bytecode::new_raw(code?.0.into());
        let code_hash = bytecode.hash_slow();
        Ok(AccountInfo::new(balance, nonce, code_hash, bytecode))
    }

    fn fetch_storage_from_fork(
        &self,
        address: Address,
        index: U256,
    ) -> Result<U256, ProviderError> {
        let add = H160::from(address.0 .0);
        let bn: Option<BlockId> = Some(BlockId::from(self.block_number));

        let index = H256::from(index.to_be_bytes());
        let slot_value: H256 = Self::block_on(self.provider.get_storage_at(add, index, bn))?;
        Ok(U256::from_be_bytes(slot_value.to_fixed_bytes()))
    }

    fn fetch_blockhash_from_fork(&self, number: U256) -> Result<B256, ProviderError> {
        if number > U256::from(u64::MAX) {
            return Ok(KECCAK_EMPTY);
        }
        // We know number <= u64::MAX so unwrap is safe
        let number = U64::from(u64::try_from(number).unwrap());
        let block: Option<Block<TxHash>> =
            Self::block_on(self.provider.get_block(BlockId::from(number)))?;
        Ok(B256::new(block.unwrap().hash.unwrap().0))
    }
}

impl DatabaseRef for ForkBackend {
    type Error = DatabaseError;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        match self.fetch_basic_from_fork(address) {
            Ok(addr) => Ok(Some(addr)),
            Err(_err) => Err(DatabaseError::GetAccount(address)),
        }
    }

    fn code_by_hash_ref(&self, hash: B256) -> Result<Bytecode, Self::Error> {
        Err(DatabaseError::MissingCode(hash))
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.fetch_storage_from_fork(address, index)
            .map_err(|_err| DatabaseError::GetStorage(address, index))
    }

    fn block_hash_ref(&self, number: U256) -> Result<B256, Self::Error> {
        self.fetch_blockhash_from_fork(number)
            .map_err(|_err| DatabaseError::GetBlockHash(number))
    }
}
