import pytest
from eth_utils import to_wei
from eth_abi import decode

from simular import PyEvm, PyAbi


def test_create_account_and_balance(evm, bob):
    two_ether = to_wei(2, "ether")

    assert evm.get_balance(bob) == 0
    evm.create_account(bob, two_ether)
    assert evm.get_balance(bob) == two_ether


def test_transfer_and_dump_state(evm, bob, alice):
    one_ether = to_wei(1, "ether")
    two_ether = to_wei(2, "ether")

    evm.create_account(bob, two_ether)
    assert evm.get_balance(bob) == two_ether

    evm.transfer(bob, alice, one_ether)

    assert evm.get_balance(bob) == one_ether
    assert evm.get_balance(alice) == one_ether

    with pytest.raises(BaseException):
        # bob doesn't have enough...
        evm.transfer(bob, alice, two_ether)

    assert evm.get_balance(bob) == one_ether
    assert evm.get_balance(alice) == one_ether

    # dump and reload state...
    state = evm.create_snapshot()
    evm2 = PyEvm.from_snapshot(state)

    assert evm2.get_balance(bob) == one_ether
    assert evm2.get_balance(alice) == one_ether


def test_contract_raw_interaction(evm, bob, kitchen_sink_json):
    abi = PyAbi.from_full_json(kitchen_sink_json)
    bytecode = abi.bytecode()

    contract_address = evm.deploy("()", bob, 0, abi)
    (enc, _, _) = abi.encode_function("increment", "()")

    with pytest.raises(BaseException):
        evm.transact(bob, "Ox01", enc, 0)

    evm.transact("increment", "()", bob, contract_address, 0, abi)

    (enc1, _, _) = abi.encode_function("value", "()")
    assert [1] == evm.call("value", "()", contract_address, abi)
