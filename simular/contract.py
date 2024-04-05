"""
Wraps pyo3 code to provide a high-level contract API
"""

from eth_utils import is_address
import typing

from .simular import PyEvm, PyAbi


def convert_for_soltypes(args: typing.Tuple):
    if len(args) == 1:
        i = str(args[0]).replace("'", "")
        return f"({i})"
    return str(args).replace("'", "")


class Function:
    """
    Contains information needed to interact with a contract function
    """

    def __init__(self, evm: PyEvm, abi: PyAbi, contract_address: str, name: str):
        self.name = name
        self.evm = evm
        self.abi = abi
        self.contract_address = contract_address

    def call(self, *args):
        """
        Make a read-only call to the contract, returning any results. Solidity
        read-only calls are marked as `view` or `pure`. Does not commit any state
        changes to the Evm.

        - `args`: 0 or more expected arguments to the function

        Returns: the decoded result
        """
        if not self.contract_address:
            raise Exception("missing contract address. see at() method")

        stargs = convert_for_soltypes(args)
        result = self.evm.call(self.name, stargs, self.contract_address, self.abi)
        if len(result) == 1:
            return result[0]
        return result

    def simulate(self, *args, caller: str = None, value: int = 0):
        """
        Simulate a write call to the contract w/o changing state.
        """
        if not self.contract_address:
            raise Exception("missing contract address. see at() method")

        if not is_address(caller):
            raise Exception("caller is missing or is not a valid address")

        stargs = convert_for_soltypes(args)
        result = self.evm.simulate(
            self.name, stargs, caller, self.contract_address, value, self.abi
        )
        if len(result) == 1:
            return result[0]
        return result

    def transact(self, *args, caller: str = None, value: int = 0):
        """
        Make a write call to the contract changing the state of the Evm.
        - `args`: 0 or more expected arguments to the function
        - `caller`: the address of the caller. This translates to `msg.sender` in a Solidity
        - `value` : an optional amount of Ether to send with the value ... `msg.value`
        Returns: the decoded result
        """
        if not self.contract_address:
            raise Exception("missing contract address. see at() method")

        if not is_address(caller):
            raise Exception("caller is missing or is not a valid address")

        stargs = convert_for_soltypes(args)
        result = self.evm.transact(
            self.name, stargs, caller, self.contract_address, value, self.abi
        )
        if len(result) == 1:
            return result[0]
        return result


class Contract:
    def __init__(self, evm: PyEvm, abi: PyAbi):
        """
        Instantiate a contract from an ABI.

        Maps contract functions to this class.  Making function available
        as attributes on the Contract.

        See `utils.py` for helper functions to create a Contract
        """
        self.address = None
        self.evm = evm
        self.abi = abi

    def __getattr__(self, name: str) -> Function:
        """
        Make solidity contract methods available as method calls. For a given function name,
        return `Function`.

        For example, if the ABI has the contract function 'function hello(uint256)',
        you can invoke it by name: contract.hello.transact(10)
        """
        if self.abi.has_function(name):
            return Function(self.evm, self.abi, self.address, name)
        else:
            raise Exception(f"contract function: '{name}' not found!")

    def at(self, address: str) -> "Contract":
        """
        Set the contract address.

        .. note::
            this is automatically set when using deploy``

        Parameters
        ----------
        address: str
            the address of a deployed contract

        Returns
        -------
            self
        """
        self.address = address
        return self

    def deploy(self, *args, caller: str = None, value: int = 0) -> str:
        """
        Deploy the contract, returning it's deployed address
        - `caller`: the address of the requester...`msg.sender`
        - `args`: a list of args (if any)
        - `value`: optional amount of Ether for the contract
        Returns the address of the deployed contract
        """
        if not caller:
            raise Exception("Missing required 'caller' address")

        if not is_address(caller):
            raise Exception("'caller' is not a valid ethereum address")

        stargs = convert_for_soltypes(args)
        addr = self.evm.deploy(stargs, caller, value, self.abi)
        self.address = addr
        return addr
