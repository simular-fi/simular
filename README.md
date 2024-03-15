# Simular
A Python smart-contract API with a blazingly fast (embedded) Ethereum Virtual Machine (EVM).
`Simular` creates a Python wrapper around production grade Rust based Ethereum APIs.

How is this different than Brownie, Ganache, Anvil?
- It's only an EVM, no blocks or mining
- No HTTP/JSON-RPC. You talk directly to the EVM (and it's fast)
- Full functionality: account transfers, contract interaction, etc...

The primary motivation for this work is to be able to model smart-contract interaction in an Agent Based Modeling environment like [Mesa](https://mesa.readthedocs.io/en/main/).


## Get Started
- You need `Rust`, `Python/Poetry`. Will be available on PyPi soon.
- Run `make build`
- See `simular/__init__.py` for the main python api

## Example
Deploy and interact with the classic `counter` smart contract

```python

    from simular import PyEvm, create_many_accounts, create_account, Contract

    # load contract abi
    with open("./tests/fixtures/counter.json") as f:
        counterabi = f.read()
    
    # create the EVM client
    client = PyEvm()

    # Create 2 accounts and fund them with 2 ether
    [deployer, alice] = create_many_accounts(2, 2)

    # Create and instance of the contract and deploy it to the EVM
    counter = Contract(client, counterabi)
    address = counter.deploy(deployer)
    assert is_address(counter.address)

    # Contract functions are dynamically built from the ABI and
    # attached to the 'Contract.
    #
    # Call the 'setNumber' function from the contract
    # Alice is the 'from' address...setting the number to 10
    # 'transact' is a write operation to the EVM
    counter.setNumber.transact(10, caller=alice)

    # Now call the 'number' function in the contract to 
    # check the state of the contract
    # 'call' is a read operation to the EVM
    result = counter.number.call()
    assert result == 10
```

## Standing on the shoulders of giants
Thanks to the following projects for making this work easy!
- [pyO3](https://github.com/PyO3)
- [revm](https://github.com/bluealloy/revm)
- [alloy-rs](https://github.com/alloy-rs/core/tree/main)
- [eth_utils/eth_abi](https://eth-utils.readthedocs.io/en/stable/) 
