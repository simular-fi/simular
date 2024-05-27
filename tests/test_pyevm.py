import pytest
from eth_utils import to_wei
from eth_abi import decode

from simular import PyEvm, PyAbi, contract_from_raw_abi


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


def test_advance_block(evm, bob, block_meta_json):
    # simple contract that can return block.timestamp and number
    evm.create_account(bob)

    contract = contract_from_raw_abi(evm, block_meta_json)
    contract.deploy(caller=bob)

    ts1, bn1 = contract.getMeta.call()

    assert bn1 == 1  # start at block 1

    evm.advance_block()
    evm.advance_block()
    evm.advance_block()

    ts2, bn2 = contract.getMeta.call()

    assert bn2 == 4  # block advanced
    assert ts2 == ts1 + 36  # timestamp advanced
