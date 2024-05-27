from typing import Optional, Type, List, Tuple

class PyEvm:
    def __new__(cls: Type["PyEvm"]) -> "PyEvm":
        """
        Create an instance of the Evm using In-memory storage
        """

    @staticmethod
    def from_fork(
        cls: Type["PyEvm"], url: str, blocknumber: Optional[int] = None
    ) -> "PyEvm":
        """
        Create an EVM configured to use a remote node to load state data.

        - `url`: the URL of the remote node to connect to
        - `blockchain`: optional block to start.  Default is 'latest'
        """

    @staticmethod
    def from_snapshot(raw: str) -> "PyEvm":
        """
        Create an EVM loading state from a snapshot.

        - `raw`: the snapshot data
        """

    def create_snapshot(self) -> str:
        """
        Create a snapshot by saving EVM state to str.
        """

    def create_account(self, address: str, balance: Optional[int] = 0):
        """
        Create an account.

        - `address`: the address for the account
        - `balance`: optional amount of Wei to fund the account. Default = 0
        """

    def get_balance(self, user: str) -> int:
        """
        Return the balance of the given user. Where 'user' is the address.
        """

    def transfer(self, caller: str, to: str, amount: int):
        """
        Transfer an 'amount' of Wei/Eth from 'caller' -> 'to'

        - `caller`: sender
        - `to`: recipient
        - `amount`: amount to transfer
        """

    def deploy(self, args: str, caller: str, value: int, abi: PyAbi) -> str:
        """
        Deploy a contract. See `Contract` for the recommended way to use this.
        """

    def advance_block(self, interval: Optional[int] = 12):
        """
        Advance the block.number / block.timestamp.

        - `interval`: optional. block time interval in seconds. Default: 12
        """

class PyAbi:
    """
    Load, parse, and encode Solidity ABI information
    """

    @staticmethod
    def from_full_json(abi: str) -> "PyAbi":
        """
        Load from a file that contains both ABI and bytecode information.
        For example, the output from compiling a contract with Foundry

        - `abi`: the str version of the compiled output file
        """

    @staticmethod
    def from_abi_bytecode(abi: str, data: Optional[bytes]) -> "PyAbi":
        """
        Load the ABI and optionally the bytecode

        - `abi`: just the abi information
        - `data`: optionally the contract bytecode
        """

    @staticmethod
    def from_human_readable(values: List[str]) -> "PyAbi":
        """
        Load from a list of contract function definitions.

        - `values`: list of function definitions

        For example: values = [
            'function hello() returns (bool)',
            'function add(uint256, uint256).
        ]

        Would provide the ABI to encode the 'hello' and 'add' functions
        """

    def has_function(self, name: str) -> bool:
        """
        Does the contract have the function with the given name?

        - `name`: the function name
        """

    def has_fallback(self) -> bool:
        """
        Does the contract have a fallback?
        """

    def has_receive(self) -> bool:
        """
        Does the contract have a receive?
        """

    def bytecode(self) -> Optional[bytes]:
        """
        Return the bytecode for the contract or None
        """

    def encode_constructor(self, args: str) -> Tuple[bytes, bool]:
        """
        ABI encode contract constructor. This is a low-level call.
        See `Contract`
        """

    def encode_function(
        self, name: str, args: str
    ) -> Tuple[bytes, bool, "DynSolTypeWrapper"]:
        """
        ABI encode a function.  This is a low-level call.
        See `Contract`

        - `name`: name of the function
        - `args`: arguments to the function
        """

class DynSolTypeWrapper:
    def __new__(cls: Type["DynSolTypeWrapper"]) -> "DynSolTypeWrapper": ...
