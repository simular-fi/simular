use alloy_primitives::FixedBytes;
use hex;
use revm::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableAccountRecord {
    pub nonce: u64,
    pub balance: U256,
    pub code: Bytes,
    pub storage: BTreeMap<U256, U256>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SerializableState {
    /// The block number of the state
    ///
    /// Note: This is an Option for backwards compatibility: <https://github.com/foundry-rs/foundry/issues/5460>
    pub accounts: BTreeMap<Address, SerializableAccountRecord>,
    //pub contracts: BTreeMap<FixedBytes<32>, Bytes>,
}

// Db Account
//  - address
//  - AccountInfo
//  - account state (u8)
//  - storage Map<u256, u256>
//
// AccountInfo
//  - balance (u256)
//  - nonce (u64)
//  - codehash (bytes)
//  - code (bytes)
