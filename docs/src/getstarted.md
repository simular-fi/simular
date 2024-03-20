# Getting Started

Simular can be installed via PyPi (LINK TO THE PYPI Page).  It requires a Python version of `>=3.11`.

**Install:**
```bash
pip install simular
```
## Examples

Here are a few quick examples that demonstrate the API. You can find more details on the API in the **Reference Guide** section.

### Transfer Ether between accounts

In this example, we'll create 2 Ethereum accounts and show how to transfer Ether between the accounts.
```python

# We use this to convert Ether to Wei
from eth_utils import to_wei
# import the EVM engine and a helper function to create accounts
from simular import PyEvmLocal, create_account

# Create the EVM
evm = PyEvmLocal()

# Create 2 accounts in the EVM:

# Bob is gifted with an initial balance of 2 Ether
bob = create_account(evm, value=2)
# Alice has an account with a 0 balance
alice = create_account(evm)

# Confirm their initial balances
assert evm.get_balance(bob) == to_wei(2, "ether")
assert evm.get_balance(alice) == 0

# transfer 1 ether from Bob to Alice
evm.transfer(bob, alice, to_wei(1, "ether"))

# Check balance. Both have 1 Ether now
assert evm.get_balance(bob) == to_wei(1, "ether")
assert evm.get_balance(alice) == to_wei(1, "ether")
```

### Deploy and interact with a Contract
Here's how you can deploy and interact with a simple Contract

For this example, we'll use the following smart contract:
```javascript
// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

contract SimpleCounter {
    // Store a number value in EVM storage.
    // You can read this value through the
    // auto-generated method 'number()'
    uint256 public number;

    // Increment number by the given 'num'.
    // Returns true if num is > 0 else false
    function increment(uint256 num) public returns (bool) {
        if (num > 0) {
            number += num;
            return true;
        }
        return false;
    }
}
```

The contract defines 2 methods we can call: `increment` and `number`.  The compiled version of this contract will result in a JSON file that contains both the ABI definition of the functions and the compiled bytecode we'll use to deploy the contract to our EVM.  In the example below, we assume the JSON is stored in the file `counter.json`.

You can learn more about ABI and the JSON format here: [Solidity ABI](https://docs.soliditylang.org/en/latest/abi-spec.html#json).  Tools like [Foundry](https://book.getfoundry.sh/) automatically generate the JSON file when building the code.  

```python

# import the EVM engine and a 2 helper functions to 
# create accounts and create a Contract object from the JSON file
from simular import (
   PyEvmLocal, 
   create_account, 
   contract_from_raw_abi,
)

# load the contract information from the counter.json file
with open('counter.json') as f:
   abi = f.read()

# Create the EVM
evm = PyEvmLocal()

# Create an account to deploy the contract
bob = create_account(evm)

# parses the JSON file and creates a contract 
# object with the contract functions
counter = contract_from_raw_abi(evm, abi)

# deploy the contract. returns the address of the deployed contract
address = counter.deploy(caller=bob)

# interact with the contract 

# calls a write operation (transact) incrementing number by 1
assert counter.increment.transact(1, caller=bob)

# check the value (read) of number in the contract by using 'call'
assert 1 == counter.number.call()
```

See **Reference Guide** for more API details.