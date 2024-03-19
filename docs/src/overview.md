
**Simular** is a Python API you can use to deploy and interact with Ethereum smart contracts and an embedded Ethereum Virtual Machine (EVM). It creates a Python wrapper around production grade Rust based Ethereum APIs making it very fast.

How is it different than Brownie, Ganache, Anvil?
- It's only an EVM. It doesn't include blocks and mining
- No HTTP/JSON-RPC. You talk directly to the EVM (and it's fast)
- Full functionality: account transfers, contract interaction, and more.

The primary motivation for this work is to be able to model smart contract interaction in an Agent Based Modeling environment like [Mesa](https://mesa.readthedocs.io/en/main/).

## Features
- `EVM`: run a local version with an in-memory database, or fork db state from a remote node.
- `Snapshot`: dump the current state of the EVM to json for future use in pre-populating EVM storage
- `ABI`: parse compiled Solidity json files or define a specific set of functions using `human-readable` notation
- `Contract/Utilities`: high-level, user-friendy Python API


## Standing on the shoulders of giants...
Thanks to the following projects for making this work possible!
- [pyO3](https://github.com/PyO3)
- [revm](https://github.com/bluealloy/revm)
- [alloy-rs](https://github.com/alloy-rs)
- [eth_utils/eth_abi](https://eth-utils.readthedocs.io/en/stable/) 
