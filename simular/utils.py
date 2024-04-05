"""
Helper functions
"""

from secrets import token_hex
from eth_utils import to_wei, is_address
import typing

from . import PyEvm, PyAbi, Contract


def ether_to_wei(value: int) -> int:
    """Convert ether to wei"""
    return to_wei(value, "ether")


def generate_random_address() -> str:
    """Generate a random hex encoded account/wallet address

    Returns: the address
    """
    return "0x" + token_hex(20)


def create_account(evm: PyEvm, address: str = None, value: int = 0) -> str:
    """
    Create an account

    - evm    : PyEvm.  the EVM client
    - address: str  optional. if set it will be used for the account address.
                              Otherwise a random address will be generated.
    - value  : int  optional. create an initial balance for the account in wei

    Returns: the address
    """
    if not isinstance(evm, PyEvm):
        raise Exception("'evm' should be an instance of either PyEvmLocal or PyEvmFork")

    if not address:
        address = generate_random_address()
        evm.create_account(address, value)
        return address

    if not is_address(address):
        raise Exception("'address' does not appear to be a valid Ethereum address")

    evm.create_account(address, value)
    return address


def create_many_accounts(evm: PyEvm, num: int, value: int = 0) -> typing.List[str]:
    """
    Create many accounts in the EVM
    - evm    : PyEvm.  the EVM client
    - num    : int  the number of accounts to create
    - value  : int  optional. create an initial balance for each account in wei

    Returns a list of addresses
    """
    return [create_account(evm, value=value) for _ in range(num)]


def contract_from_raw_abi(evm: PyEvm, raw_abi: str) -> Contract:
    """
    Create the contract given the full ABI. Full ABI should include
    `abi` and `bytecode`. This is usually a single json file from a compiled Solidity contract.

    - `evm`     : PyEvm.  the EVM client
    - `raw_abi` : abi file as un-parsed json
    Returns an instance of Contract
    """
    if not isinstance(evm, PyEvm):
        raise Exception("'evm' should be an instance of either PyEvmLocal or PyEvmFork")

    if not isinstance(raw_abi, str):
        raise Exception("expected a an un-parsed json file")

    abi = PyAbi.from_full_json(raw_abi)
    return Contract(evm, abi)


def contract_from_abi_bytecode(
    evm: PyEvm, raw_abi: str, bytecode: bytes = None
) -> Contract:
    """
    Create a contract given the abi and bytecode.

    - `evm`     : PyEvm  the EVM client
    - `raw_abi` : abi file as un-parsed json
    - `bytecode`: Optional bytes
    Returns an instance of Contract
    """
    if not isinstance(evm, PyEvm):
        raise Exception("'evm' should be an instance of either PyEvmLocal or PyEvmFork")

    if not isinstance(raw_abi, str):
        raise Exception("expected a an un-parsed json file")

    abi = PyAbi.from_abi_bytecode(raw_abi, bytecode)
    return Contract(evm, abi)


def contract_from_inline_abi(evm: PyEvm, abi: typing.List[str]) -> Contract:
    """
    Create the contract using inline ABI.
    - `evm` : PyEvm.  the EVM client
    - `abi` : a list of strings that describe the solidity functions of interest.
    Returns an instance of Contract

    Function are described in the format: 'function NAME(PARAMETER TYPES) (RETURN TYPES)'
    where:
    `NAME` if the function name
    `PARAMETER TYPES are 0 or more solidity types of any arguments to the function
    `RETURN TYPES are any expected returned solidity types.  If the function does not return
    anything, this is not needed.

    Examples:
    - "function hello(uint256,uint256)`: hello function the expects 2 int arguments and returns nothing
    - "function hello()(uint256)"`: hello function with no arguments and return an int

    abi = ['function hello()(uint256)', 'function world(string) (string)']

    """
    if not isinstance(evm, PyEvm):
        raise Exception("'evm' should be an instance of either PyEvmLocal or PyEvmFork")

    abi = PyAbi.from_human_readable(abi)
    return Contract(evm, abi)
