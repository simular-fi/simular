from secrets import token_hex
from eth_utils import to_wei, is_address
import typing

from . import PyEvmLocal, PyAbi, PyEvmFork, Contract


def generate_random_address() -> str:
    """
    Generate a random hex encoded account/wallet address

    Returns: the address
    """
    return "0x" + token_hex(20)


def create_account(
    evm: PyEvmLocal | PyEvmFork, address: str = None, value: int = 0
) -> str:
    """
    Create an account in the EVM.

    Parameters:
    - evm    : PyEvmLocal | PyEvmForm.  the EVM client
    - address: str  optional. if set it will be used for the account address.
                              Otherwise a random address will be generated.
    - value  : int  optional. create an initial balance for the account in ether

    Returns: the address
    """
    wei = to_wei(value, "ether")
    if not address:
        address = generate_random_address()
        evm.create_account(address, wei)
        return address

    if not is_address(address):
        raise Exception("'address' does not appear to be a valid Ethereum address")

    evm.create_account(address, wei)
    return address


def create_many_accounts(
    evm: PyEvmLocal | PyEvmFork, num: int, value: int = 0
) -> typing.List[str]:
    """
    Create many accounts in the EVM

    Parameters:
    - evm    : PyEvmLocal | PyEvmForm.  the EVM client
    - num    : int  the number of accounts to create
    - value  : int  optional. create an initial balance for each account in ether

    Returns a list of addresses
    """
    return [create_account(evm, value=value) for _ in range(num)]


def contract_from_raw_abi(evm: PyEvmLocal | PyEvmFork, raw_abi: str) -> Contract:
    """
    Create the contract given the full ABI as a str.
    """
    if not isinstance(raw_abi, str):
        raise Exception("expected a an un-parsed json file")
    abi = PyAbi.load_from_json(raw_abi)
    return Contract(evm, abi)


def contract_from_abi_bytecode(
    evm: PyEvmLocal | PyEvmFork, raw_abi: str, bytecode: bytes
) -> Contract:
    """
    Create a contract given the abi and bytcodes
    """
    if not isinstance(raw_abi, str):
        raise Exception("expected a an un-parsed json file")
    abi = PyAbi.load_from_parts(raw_abi, bytecode)
    return Contract(evm, abi)


def contract_from_inline_abi(
    evm: PyEvmLocal | PyEvmFork, abi: typing.List[str]
) -> Contract:
    """
    Create the contract using inline ABI.

    For example, to support a method call of `hello(address name) (uint256)`
    that takes an `address` as input and returns an `uint256` value, pass this
    function a list of str(s) - `["function hello(address name) (uint256)"]. It
    will then be available on the contract: `contract.hello('0x...').transact(...)`
    """
    abi = PyAbi.load_from_human_readable(abi)
    return Contract(evm, abi)
