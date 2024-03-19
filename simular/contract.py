from eth_abi import encode, decode
from eth_utils import is_address
import typing

from .simular import PyEvmLocal, PyAbi, PyEvmFork


class Function:
    """
    Callback to invoke contract functions. Used by Contract
    """

    def __init__(
        self,
        contract_address: str,
        name: str,
        client: PyEvmLocal | PyEvmLocal,
        abi: PyAbi,
    ):
        self.name = name
        self.client = client
        self.abi = abi
        self.contract_address = contract_address

    def call(self, *args):
        if not self.contract_address:
            raise Exception("missing contract address. see at() method")

        encoded, output_params = self.abi.encode_function_input(self.name, args)
        (bits, _) = self.client.call(self.contract_address, encoded)

        return self.__decode_output(output_params, bytes(bits))

    def transact(self, *args, caller: str = None, value: int = 0):
        if not self.contract_address:
            raise Exception("missing contract address. see at() method")

        if not is_address(caller):
            raise Exception("caller is missing or not a valid address")

        encoded, output_params = self.abi.encode_function_input(self.name, args)
        (bits, _) = self.client.transact(caller, self.contract_address, encoded, value)
        return self.__decode_output(output_params, bytes(bits))

    def __decode_output(self, params: typing.List[str], rawbits: bytes):
        decoded = decode(params, rawbits)
        if len(decoded) == 1:
            return decoded[0]
        else:
            return decoded


class Contract:
    def __init__(self, evm: PyEvmLocal | PyEvmFork, abi: PyAbi):
        """
        Instantiate a contract from an ABI parsed on the Rust side.

        Maps contract functions to this class.  Making function available
        as attributs on the Contract.
        """
        self.address = None
        self.evm = evm
        self.abi = abi

    def __getattr__(self, name: str) -> Function:
        """
        Make solidity contract methods available as method calls.
        For example, if the ABI has the contract function 'function hello(uint256)',
        you can invoke it by name: contract.hello.transact(10)
        """
        if self.abi.has_function(name):
            return Function(self.address, name, self.evm, self.abi)
        else:
            raise Exception(f"contract function: '{name}' not found!")

    def at(self, address: str) -> "Contract":
        """
        Set the contract address. Note: this is automatically set when using deploy
        """
        self.address = address
        return self

    def deploy(self, caller: str, args=[], value: int = 0) -> str:
        """
        Deploy the contract, returning it's deployed address
        """
        constructor_params = self.abi.constructor_input_types()
        bytecode = self.abi.bytecode()

        if not constructor_params and len(args) > 0:
            raise Exception("constructor doesn't take any args")

        if constructor_params:
            if len(constructor_params) != len(args):
                raise Exception("wrong number of args for the constructor")
            bytecode += encode(constructor_params, args)

        addr = self.evm.deploy(caller, bytecode, value)
        self.address = addr
        return addr
