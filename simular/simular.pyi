from typing import Optional, Type

class PyEvm:
    def __new__(cls: Type["PyEvm"]) -> "PyEvm":
        """
        Create an instance of the Evm
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
    def from_full_json(cls: Type["PyAbi"]): ...
