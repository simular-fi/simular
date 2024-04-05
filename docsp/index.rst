.. simular documentation master file, created by
   sphinx-quickstart on Thu Apr  4 09:27:23 2024.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.
.. _index:

simular
=======

**Simular** is a Python API you can use to deploy and interact with Ethereum 
smart contracts and an embedded Ethereum Virtual Machine (EVM). It creates a 
Python wrapper around production grade Rust based Ethereum APIs making it very fast.


How is it different than Brownie, Ganache, Anvil?

- It's only an EVM. It doesn't include blocks and mining
- No HTTP/JSON-RPC. You talk directly to the EVM (and it's fast)
- Full functionality: account transfers, contract interaction, and more.

The primary motivation for this work is to be able to model smart contract 
interaction in an Agent Based Modeling environment 
like `Mesa <https://mesa.readthedocs.io/en/main/>`_.

Features
--------

- **EVM**: run a local version with an in-memory database, or fork db state from a remote node.
- **ABI**: parse compiled Solidity json files or define a specific set of functions using `human-readable` notation
- **Snapshot**: dump the current state of the EVM to json for future use in pre-populating EVM storage
- **Contract/Utilities**: high-level, user-friendy Python API


Standing on the shoulders of giants...
--------------------------------------

Thanks to the following projects for making this work possible!

- `pyO3 <https://github.com/PyO3>`_
- `revm <https://github.com/bluealloy/revm>`_
- `alloy-rs <https://github.com/alloy-rs>`_
- `eth_utils/eth_abi <https://eth-utils.readthedocs.io/en/stable/>`_ 


.. toctree::
   :maxdepth: 2
   :caption: Contents:
   
   getstarted
   api



