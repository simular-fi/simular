# Utility Functions

Utilities defines several 'helper' functions to create contracts and accounts.

- [Utility Functions](#utility-functions)
  - [Functions](#functions)
    - [generate\_random address](#generate_random-address)
    - [create\_account](#create_account)
    - [create\_many\_accounts](#create_many_accounts)
    - [contract\_from\_raw\_abi](#contract_from_raw_abi)
    - [contract\_from\_abi\_bytecode](#contract_from_abi_bytecode)
    - [contract\_from\_inline\_abi](#contract_from_inline_abi)

## Functions

### generate_random address
Create a random Ethereum address

```python
def generate_random_address() -> str
```
**Returns:** an address 

Example:
```python
address = generate_random_address()
```

### create_account
Create an account in the EVM

```python
def create_account(
                  evm: PyEvmLocal | PyEvmFork,
                  address: str = None
                  value: int = 0) -> str
```
**Parameters**:

- `evm`: PyEvmLocal | PyEvmForm.  the EVM client
- `address`: str  optional. if set it will be used for the account address.
                          Otherwise a random address will be generated.
- `value`  : int  optional. create an initial balance for the account in ether

**Returns:** the address

Example:
```python
evm = PyEvmLocal()
bob = create_address(evm)
```

### create_many_accounts
Create many accounts in the EVM. Address are randomly generated

```python
def create_many_account(
                  evm: PyEvmLocal | PyEvmFork,
                  num: int
                  value: int
                  value: int = 0) -> typing.List[str]
```
**Parameters**

- `evm`: PyEvmLocal | PyEvmForm.  the EVM client
- `num`: int. the number of accounts to create.
- `value`  : int  optional. create an initial balance for each account in ether

**Returns** a list of addresses

Example:
```python
evm = PyEvmLocal()
# create 2 accounts, each with a balance of 1 Ether
[bob, alice] = create_many_address(evm, 2, 1)
```

### contract_from_raw_abi
Create the contract given the full ABI. Full ABI should include
`abi` and `bytecode`. This is usually a single json file from a compiled Solidity contract.

```python
def contract_from_raw_abi(
                          evm: evm: PyEvmLocal | PyEvmFork
                          raw_abi: str) -> Contract
```
**Parameters**

- `evm`     : PyEvmLocal | PyEvmForm.  the EVM client
- `raw_abi` : abi file as un-parsed json

**Returns** an instance of Contract

Example:
```python
evm = PyEvmLocal()
with open('counter.json') as f:
    raw = f.read()

contract = contract_from_raw_abi(evm, raw)
```

### contract_from_abi_bytecode
Create a contract given the abi and bytecode.

```python
def contract_from_abi_bytecode(
                               evm: evm: PyEvmLocal | PyEvmFork
                               raw_abi: str, 
                               bytecode: bytes) -> Contract
)
```
**Parameters**

- `evm`     : PyEvmLocal | PyEvmForm.  the EVM client
- `raw_abi` : abi file as un-parsed json
- `bytecode`: bytes

**Returns** an instance of Contract

Example:
```python
evm = PyEvmLocal()
with open('counter.abi') as f:
    abi = f.read()

with open('counter.bin') as f:
    bytecode = f.read()

bits = bytes.fromhex(bytecode)
contract = contract_from_abi_bytecode(abi, bits)
```

### contract_from_inline_abi
Create the contract using inline ABI method definitions.

```python
def contract_from_inline_abi(
                             evm: evm: PyEvmLocal | PyEvmFork)
                             abi: typing.List[str]) -> Contract
```
Function are described in the format: `function NAME(PARAMETER TYPES) (RETURN TYPES)`

where:
- `NAME` if the function name
- `PARAMETER` TYPES are 0 or more solidity types of any arguments to the function
- `RETURN TYPES` are any expected returned solidity types.  If the function does not return anything, this is not needed.

Examples:

- "function hello(uint256,uint256)" is hello function the expects 2 int arguments and returns nothing
- "function hello()(uint256)" is a hello function with no arguments and return an int

**Parameters**

- `evm` : PyEvmLocal | PyEvmForm.  the EVM client
- `abi` : a list of strs

**Returns** an instance of Contract

Example:
```python
evm = PyEvmLocal()
abi = ['function hello()(uint256)', 'function world(string) (string)']
contract = contract_from_inline_abi(evm, abi)
```
