.. _index:

simular
=======

**Simular** is a Python API wrapped around a production grade Ethereum Virtual Machine (EVM).  You can use to 
locally deploy and interact with smart contracts, create accounts, transfer Ether, and much more.

How is it different than Brownie, Ganache, Anvil?

- It's only an EVM.
- No HTTP/JSON-RPC. You talk directly to the EVM (and it's fast)
- Full functionality: account transfers, contract interaction, and more.

The primary motivation for this work is to have a lightweight, fast environment for simulating and modeling Ethereum applications.

Features
--------

- User-friendy Python API
- Run a local version with an in-memory database. Or copy (fork) state from a remote node.
- Parse Solidity ABI json files or define a specific set of functions using `human-readable` notation.
- Dump the current state of the EVM to json for future use in pre-populating EVM storage.


Standing on the shoulders of giants...
--------------------------------------

Thanks to the following projects for making this work possible!

- `pyO3 <https://github.com/PyO3>`_
- `revm <https://github.com/bluealloy/revm>`_
- `alloy-rs <https://github.com/alloy-rs>`_
- `eth_utils/eth_abi <https://eth-utils.readthedocs.io/en/stable/>`_ 


.. include:: toc.rst




