
# Simular

[![pypi](https://img.shields.io/pypi/v/simular-evm.svg)](https://pypi.python.org/pypi/simular-evm)

A Python smart-contract API with a fast (embedded) Ethereum Virtual Machine (EVM). `Simular` creates a Python wrapper around production grade Rust based Ethereum APIs.

How is it different than Brownie, Ganache, Anvil?
- It's only an EVM, no blocks or mining
- No HTTP/JSON-RPC. You talk directly to the EVM (and it's fast)
- Full functionality: account transfers, contract interaction, etc...

The primary motivation for this work is to be able to model smart-contract interaction in an Agent Based Modeling environment like [Mesa](https://mesa.readthedocs.io/en/main/).

## Features
- `EVM`: run a local version with an in-memory database, or fork db state from a remote node.
- `Snapshot`: dump the current state of the EVM to json for future use in pre-populating EVM storage
- `ABI`: parse compiled Solidity json files or define a specific set of functions using `human-readable` notation
- `Contract`: high-level, user-friendy Python API

## Build from source
- You need `Rust` and `Python`, and optionally `Make`. We use `hatch` for Python project management, but it's not required
- Create a local Python virtual environment. Within that environment install Python dependencies
- Run `make build` or `hatch run maturin develop`
- See `simular/` for the main python api

## Getting Started
See [Simular Documentation](https://simular.readthedocs.io/en/latest/) for examples and API details.

## Standing on the shoulders of giants...
Thanks to the following projects for making this work possible!
- [pyO3](https://github.com/PyO3)
- [revm](https://github.com/bluealloy/revm)
- [alloy-rs](https://github.com/alloy-rs)
- [eth_utils/eth_abi](https://eth-utils.readthedocs.io/en/stable/) 
