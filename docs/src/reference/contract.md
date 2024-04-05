# Contract API
Provides a wrapper around [PyEvm](./pyevm.md) and [PyAbi](./pyabi.md) and is the easiest way to interact with Contracts.  Solidity contract methods are extracted from the ABI and made available as attributes on the instance of the Contract. 

For example, if a Solidity contract defines the following methods:
```javascript
function hello(address caller) public returns (bool){}

function world(string name) view (string) 
```
they will be automatically available in the instance of the Contract like this:

```python
# write call to the hello method
contract.hello.transact("0x11", caller="0x..")

# read call to world method
contract.world.call("dave")
```
Each method name is an attribute on the instance of Contract.  To invoke them, you need to append either:

```python
# this is a write/transaction
# where:
# - args: is 0 of more expected arguments to the method
# - caller: is the address of the account calling the method
# - value: is an optional value in Ether 
.transact(*args, caller: str, value: int=0)

# this is a read (view) call
# where:
# - args: is 0 of more expected arguments to the method
.call(*args)
```

Under the covers, a Contract knows how to properly encode all interactions with the EVM, and likewise decode any return values.

- [Contract API](#contract-api)
  - [Constructor](#constructor)
  - [Methods](#methods)
    - [at](#at)
    - [deploy](#deploy)

## Constructor
Create an instance of a Contract from an ABI.

> See [utilities](./utils.md) for the preferred way to create a Contract

```python
def __init__(self, evm: PyEvmLocal | PyEvmFork, abi: PyAbi)
```
**Parameters**

- `evm` an instance of one of the EVMs
- `abi` an instance of PyAbi

**Returns** self (Contract)

Example:

```python
evm = PyEvmLocal()
abi = PyAbi.load_from_json(...)
counter = Contract(evm, abi)
```

## Methods

### at
Set the contract address. Note: this is automatically set when using deploy

```python
def at(self, address: str) -> Contract
```

**Parameters**

- `address` the address of the Contract in the EVM

**Returns** self

Example:
```python
contract.at('0x11...')
```

### deploy
Deploy the contract, returning it's address

```python
def deploy(self, caller: str, args=[], value: int = 0) -> str
```
**Parameters**
- `caller`: the address of the requester...`msg.sender`
- `args`: a list of args expected by the Contract's constructor (if any)
- `value`: optional amount of Ether for the contract

**Returns** the address of the deployed contract

Example:
```python
# deploy a contract that's expecting 
# no constructor arguments and no initial balance
contract.deploy(caller='0x11...')
```




