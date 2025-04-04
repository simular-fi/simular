//!
//! Containers for serializing EVM state information
//!
use revm::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Source of the snapshop.  Either from a fork or the local in-memory database.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum SnapShotSource {
    Memory,
    #[default]
    Fork,
}

/// A single AccountRecord and it's associated storage. `SnapShot` stores
/// a map of Accounts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapShotAccountRecord {
    pub nonce: u64,
    pub balance: U256,
    pub code: Bytes,
    pub storage: BTreeMap<U256, U256>,
}

/// The high-level objects containing all the snapshot information.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SnapShot {
    pub source: SnapShotSource,
    pub block_num: u64,
    pub timestamp: u64,
    pub accounts: BTreeMap<Address, SnapShotAccountRecord>,
}
