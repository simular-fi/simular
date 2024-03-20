# PyAbi API
Provides the ability to load and parse ABI files. It's primarily used to help extract the information needed to interact with smart contracts.  This is rarely used directly. See `Contracts` and `Utilities`.

- [PyAbi API](#pyabi-api)
  - [Import](#import)
  - [Static functions](#static-functions)
    - [load\_from\_json](#load_from_json)
    - [load\_from\_parts](#load_from_parts)
    - [load\_from\_human\_readable](#load_from_human_readable)
  - [Methods](#methods)
    - [has\_function](#has_function)
    - [has\_fallback](#has_fallback)
    - [has\_receive](#has_receive)
    - [bytecode](#bytecode)
    - [constructor\_input\_types](#constructor_input_types)
    - [encode\_function\_input](#encode_function_input)


## Import
```python
from simular import PyAbi
```
## Static functions
You can create an instance of `PyAbi` by using one of the following static methods:

### load_from_json
Create an instance by loading a JSON file from a compiled Solidity contract. Expects a JSON file the includes an `abi` and `bytecode` entry.

```python
def load_from_json(abi: str) -> self
```
**Parameters**:
- abi: an un-parsed json encoded file

**Returns:** an instance of PyAbi

Example:
```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)
```


### load_from_parts
Create an instance by loading a the json encoded abi information and contract bytecode.

```python
def load_from_parts(abi: str, bytecode: bytes) -> self
```
**Parameters**:
- abi: an un-parsed json encoded file with just the abi information
- bytecode: the contract bytecode

**Returns:** an instance of PyAbi

Example:
```python
with open('counter.abi') as f:
    abi = f.read()

with open('counter.bin') as f:
    bytecode = f.read()

bits = bytes.fromhex(bytecode)
abi = PyAbi.load_from_json(abi, bits)
```

### load_from_human_readable
Create an instance from a list of contract function descriptions

```python
def load_from_human_readable(items: typing.List[str]) -> self
```
**Parameters**:
- items: is a list of `function desciptions`

**Returns:** an instance of PyAbi

A `function description` is shorthand way of describing the function name, inputs, and outputs.  The format is the form: 

`function NAME(ARG TYPES) (RETURN TYPES)`
Where: 
- NAME: is the name of the function.
- ARG TYPES: 0 or more of the require Solidity input types
- RETURN TYPES: tuple of expected Solidity return types. This is not required if the function doesn't return anything.

For example: 

`'function hello() (uint256)'` is a solidity function named `hello` that takes no input arguments and returns an `uint256`


`'function hello(address, uint256)'` is a solidity function named `hello` that takes 2 arguments, an `address`, and  `uint256` and returns nothing.

Example:
```python 
abi = PyAbi.load_from_human_readable([
    'function hello() (uint256)', 
    'function hello(address, uint256)'])
```


## Methods

### has_function
Does the ABI include the function with the given name.

```python
def has_function(self, name: str) -> bool
```
**Parameters**:
- items: is a list of `function desciptions`

**Returns:** an instance of True | False

Example
```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

assert abi.has_function("increment")
```


### has_fallback
Does the Contract define a [fallback function](https://docs.soliditylang.org/en/latest/contracts.html#fallback-function)

```python
def has_fallback(self) -> bool
```
**Returns:** an instance of True | False

Example
```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

assert not abi.has_fallback()
```

### has_receive
Does the Contract define a [receive function](https://docs.soliditylang.org/en/latest/contracts.html#receive-ether-function)

```python
def has_receive(self) -> bool
```
**Returns:** an instance of True | False

Example
```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

assert not abi.has_receive()
```

### bytecode
Return the byte from the ABI

```python
def bytecode(self) -> bytes | None
```

**Returns:** bytes or **None** if the bytecode wasn't set

```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

abi.bytecode()
```

### constructor_input_types
Return a list (if any) of the exopected constructor arguments.

```python
def constructor_input_types(self) -> typing.List[str] | None
```
**Returns:** a list of the Solidity types expected as arguments to the constructor.  Or **None** if the constructor doesn't take any arguments.

```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

abi.constructor_input_types()
```



### encode_function_input
Encode the function with any given input to call on the EVM. See [Function Encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector-and-argument-encoding) for more details.

```python
def encode_function_input(self, 
                          name: str, 
                          args: typing.Tuple[Any]
) -> typing.Tuple(bytes, typing.List[str])
```
**Parameters**:
- name: of the contract method to encode
- args: 0 or more arguments to pass to the method

**Returns:** tuple: the encoded bytes and a list of the expected output types from calling the method

```python
with open('counter.json') as f:
    raw = f.read()
abi = PyAbi.load_from_json(raw)

abi.encode_function_input(self, "increment", (1,))
```




