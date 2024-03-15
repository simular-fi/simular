from secrets import token_hex
from eth_abi import encode, decode
from eth_utils import to_wei, is_address
import typing


from .simular import PyEvm, PyAbi


def generate_random_address() -> str:
    """
    Generate a random hex encoded account/wallet address
    """
    return "0x" + token_hex(20)


def create_account(evm: PyEvm, address: str = None, value: int = 0) -> str:
    """
    Create an account in the EVM.

    Parameters:
    - address: str  optional. If set it will be used for the account address.
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


def create_many_accounts(evm: PyEvm, num: int, value: int = 0) -> typing.List[str]:
    """
    Create many accounts in the EVM

    Parameters:
    - num    : int the number of accounts to create
    - value  : int  optional. create an initial balance for the account in ether

    Returns a list of addresses
    """
    return [create_account(evm, value=value) for _ in range(num)]


class Contract:
    def __init__(self, evm: PyEvm, raw_abi: str):
        """
        Instantiate a contract from an ABI parsed on the Rust side.

        Maps contract functions to this class, and automatically determines
        if a method call should be a contract transaction or a read-only call.
        """
        self.address = None
        self.evm = evm
        if isinstance(raw_abi, str):
            self.abi = PyAbi.load_from_json(raw_abi)
        else:
            raise Exception("Unrecognized abi format")

    def __getattr__(self, name: str):
        """
        Make solidity contract methods available as method calls.
        For example, if the ABI has the contract function 'function hello(uint256)',
        you can invoke it by name: contract.hello(10)
        """
        if self.abi.has_function(name):
            return Function(self.address, name, self.evm, self.abi)
        else:
            raise Exception(f"Contract function: '{name}' not found!")

    def at(self, address: str):
        """
        Set the contract address, if not already set on deploy
        """
        self.address = address
        return self

    def deploy(self, caller, args=[], value: int = 0):
        """
        Deploy the contract, returning it's deployed address
        """
        constructor_params = self.abi.constructor_input_types()
        bytecode = self.abi.bytecode()

        if not constructor_params and len(args) > 0:
            raise Exception("Constructor doesn't take any args")

        if constructor_params:
            if len(constructor_params) != len(args):
                raise Exception("wrong number of args for the constructor")
            bytecode += encode(constructor_params, args)

        addr = self.evm.deploy(caller, bytecode, value)
        self.address = addr
        return addr


def decode_output(params, rawbits):
    decoded = decode(params, rawbits)
    if len(decoded) == 1:
        return decoded[0]
    else:
        return decoded


class Function:
    """
    Callback to invoke contract functions
    """

    def __init__(self, contract_address, name, client, abi):
        self.name = name
        self.client = client
        self.abi = abi
        self.contract_address = contract_address

    def call(self, *args):
        if not self.contract_address:
            raise Exception("Missing contract address. see at() method")

        encoded, output_params = self.abi.encode_function_input(self.name, args)
        (bits, _) = self.client.call(self.contract_address, encoded)

        return decode_output(output_params, bytes(bits))

    def transact(self, *args, caller: str = None, value: int = 0):
        if not self.contract_address:
            raise Exception("Missing contract address. see at() method")

        if not is_address(caller):
            raise Exception("Caller is missing or not a valid address")

        encoded, output_params = self.abi.encode_function_input(self.name, args)
        (bits, _) = self.client.transact(caller, self.contract_address, encoded, value)
        return decode_output(output_params, bytes(bits))

    """
    def __call__(self, *args, **kwargs):
        if not self.contract_address:
            raise Exception("Missing contract address. see at() method")

        value = kwargs.get("value", 0)
        caller = kwargs.get("caller", None)

        encoded, output_params = self.abi.encode_function_input(self.name, args)


        if transact:
            if not caller:
                raise Exception("Missing caller/sender address")
            # it's a write call
            (bits, _) = self.client.transact(
                caller, self.contract_address, encoded, value
            )
        else:
            # it's a read call
            (bits, _) = self.client.call(self.contract_address, encoded)

        decoded = decode(output_params, bytes(bits))
        if len(decoded) == 1:
            return decoded[0]
        else:
            return decoded
    """
