# PyEvm API

There are 2 version of PyEvm:  `PyEvmLocal` and `PyEvmFork`. They primarily differ in how they populate the EVM with state information. 

`PyEvmLocal` uses a local in-memory datasource. It populates EVM state from user-defined interaction with the Evm (contracts, etc...).  

`PyEvmFork` works very much the same, but has the ability to pull EVM state from a remote node.  It can access on-chain state information from any available Ethereum node offering a json-rpc endpoint.

Both versions are a Python class wrapper of the [REVM](https://github.com/bluealloy/revm/tree/main) Rust library.

- [PyEvm API](#pyevm-api)
  - [Import](#import)
  - [Constructor](#constructor)
  - [Common methods](#common-methods)
    - [get\_balance](#get_balance)
    - [create\_account](#create_account)
    - [transfer](#transfer)
    - [deploy](#deploy)
    - [transact](#transact)
    - [call](#call)
    - [dump\_state](#dump_state)
    - [view\_storage\_slot](#view_storage_slot)
  - [PyEVMLocal](#pyevmlocal)
    - [load\_state](#load_state)

## Import
```python
from simular import PyEvmLocal, PyEvmFork
```

## Constructor
  
Create an instance of the EVM with in-memory storage.

**PyEvmLocal()**

Create an instance of the EVM with in-memory storage and the ability to pull state from a remote node.

**PyEvmFork(url: str)** : 

**Parameters:** 
- `url` : The HTTP address of an Ethereum json-rpc endpoint.  Services such as `Infura` and `Alchemy` provide access to json-rpc endpoints


## Common methods
Both versions share the following methods:

### get_balance
Get the balance of the given account.

```python
def get_balance(self, address: str) -> int
```

**Parameters**:
- address: the address of account

**Returns:** int: the balance 

Example:
```python
evm = PyEvmLocal()
bal = evm.get_balance('0x123...')
```

### create_account
Create an account in the EVM

```python
def create_account(self, address: str, amount: int | None)
```

**Parameters**:
- address: the address of the account to make
- amount: (optional) the value in Ether to fund the account
  
Example:
```python
evm = PyEvmLocal()
evm.create_account('0x111...', to_wei(2, 'ether'))
```

### transfer
Transfer Ether from one account to another.

```python
def transfer(self, caller: str, to: str, amount: int)
```

**Parameters**:
- caller: the sender (from)
- to: the recipient (to)
- amount: in Ether to transfer
  
Example:
```python
evm = PyEvmLocal()
evm.transfer('0x11...', '0x22...', to_wei(1, 'ether')
```

### deploy
Deploy a contract

```python
def deploy(self, from: str, bytecode: bytes, value: int) -> str
```

*This is usually not called directly as it requires properly formatting `bytecode`. See the [Contract](./contract.md) API as an easier way to deploy a contract.*

**Parameters**:
- address: the account deploying the contract (from)
- bytecode: property formatted bytes to deploy the contract in the EVM
- value: in Ether to transfer to the contract

**Returns:** str: the address of the deployed contract

Example:
```python
evm = PyEvmLocal()
evm.deploy('0x11..., b'320...', 0)
```

### transact
Make a write operation changing the state of the given contract.

```python
def transact(self, 
             caller: str, 
             to: str, 
             data: bytes, 
             value: int) -> (bytes, int)
```

*This is usually not called directly as it requires properly formatting `data`. See the [Contract](./contract.md) API as an easier way to use transact.*

**Parameters**:
- caller: from address (msg.sender)
- to: the address of the contract
- data: abi encoded function call
- value: in Ether to transfer to the contract

**Returns:** tuple: (encoded response, gas used)

Example:
```python
evm = PyEvmLocal()
evm.transact('0x11..', '0x22..', b'661..', 0)
```

### call
Make a read-only call to the contract

```python
def call(self, to: str, data: bytes) -> (bytes, uint)
```
*This is usually not called directly as it requires properly formatting `data`. See the [Contract](./contract.md) API as an easier way to use call.*

**Parameters**:
- to: the address of the contract
- data: abi encoded function call

**Returns:** tuple: (encoded response, gas used)

Example:
```python
evm = PyEvmLocal()
evm.call('0x11..', b'661..')
```

### dump_state
Export the current state (snapshot) of the EVM to a JSON encoded string

```python
def dump_state(self) -> str
```

**Returns:** str: JSON encoded str

Example:
```python
evm = PyEvmLocal()
state = evm.dump_state()
```

### view_storage_slot
View the storage slot of the given account.

```python
def view_storage_slot(self, address: str, index: int) -> bytes
```

**Parameters**:
- address: the address of the contract
- index: this index of the slot in the internal Map.


**Returns:** bytes: encoded value.  Decoding requires knowlege of the type stored in the given slot

Example:
```python
evm = PyEvmLocal()
value = evm.view_storage_slot('0x11...', 1)
```

## PyEVMLocal
`PyEVMLocal` has one additional method:

### load_state
Load state into the EVM from a snapshot. See [dump_state](#dump_state).

```python
def load_state(self, snapshot: str)
```
**Parameters**:
- snapshot: the json file produced by `dump_state()`

Example:
```python
evm = PyEvmLocal()

with open('snapshot.json') as f:
  snap = f.read()

evm.dump_state(snap)
```









