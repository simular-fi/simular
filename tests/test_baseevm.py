from simular import PyEvm, create_account, create_many_accounts
from eth_utils import to_wei


def test_evm_accounts():
    evm = PyEvm()
    actor = create_account(evm, value=2)
    assert evm.get_balance(actor) == to_wei(2, "ether")


def test_balance_and_transfer():
    transfer_amt = to_wei(1, "ether")
    evm = PyEvm()
    [bob, alice] = create_many_accounts(evm, 2, value=5)

    evm.transfer(bob, alice, transfer_amt)
    assert evm.get_balance(bob) == to_wei(4, "ether")
    assert evm.get_balance(alice) == to_wei(6, "ether")
