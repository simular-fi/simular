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
    pub accounts: BTreeMap<Address, SerializableAccountRecord>,
}
