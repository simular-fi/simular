import pytest
from eth_utils import to_wei
from eth_abi import decode

from simular import PyEvmLocal, PyAbi


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
    state = evm.dump_state()

    evm2 = PyEvmLocal()
    evm2.load_state(state)

    assert evm2.get_balance(bob) == one_ether
    assert evm2.get_balance(alice) == one_ether


def test_contract_raw_interaction(evm, bob, kitchen_sink_json):
    abi = PyAbi.load_from_json(kitchen_sink_json)
    bytecode = abi.bytecode()

    contract_address = evm.deploy(bob, bytecode, 0)
    (enc, _) = abi.encode_function_input("increment", ())

    with pytest.raises(BaseException):
        evm.transact(bob, "Ox01", enc, 0)

    evm.transact(bob, contract_address, enc, 0)

    (enc1, output_params) = abi.encode_function_input("value", ())
    (result, _) = evm.call(contract_address, enc1)
    decoded = decode(output_params, bytes(result))
    assert decoded[0] == 1

    r = evm.view_storage_slot(contract_address, 0)
    # NOTE: little endian bytes!
    slot_value = int.from_bytes(r, "little")
    assert slot_value == 1
