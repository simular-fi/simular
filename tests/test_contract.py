import pytest

from simular import (
    contract_from_abi_bytecode,
    contract_from_raw_abi,
    create_account,
    PyEvm,
    contract_from_inline_abi,
    ether_to_wei,
)


def test_loading_contracts(evm):
    with pytest.raises(BaseException):
        contract_from_raw_abi(evm, "")

    with pytest.raises(BaseException):
        contract_from_raw_abi(evm, {})

    with pytest.raises(BaseException):
        contract_from_abi_bytecode(evm, "", b"")


def test_contract_interface(evm, bob, alice, erc20abi, erc20bin):
    create_account(evm, alice, 0)
    create_account(evm, bob, 0)

    erc20 = contract_from_abi_bytecode(evm, erc20abi, erc20bin)

    erc20.deploy("USD Coin", "USDC", 6, caller=bob)
    contract_address = erc20.address

    assert erc20.name.call() == "USD Coin"
    assert erc20.decimals.call() == 6
    assert erc20.owner.call() == bob

    erc20.mint.transact(alice, 10, caller=bob)

    assert 10 == erc20.balanceOf.call(alice)
    assert 10 == erc20.totalSupply.call()

    with pytest.raises(BaseException):
        # alice can't mint, she's not the owner!
        erc20.mint.transact(alice, 10, caller=alice)

    # Test state
    evm2 = PyEvm.from_snapshot(evm.create_snapshot())

    erc20again = contract_from_inline_abi(evm2, ["function totalSupply() (uint256)"])
    erc20again.at(contract_address)
    assert 10 == erc20again.totalSupply.call()


def test_deploy_and_test_kitchensink(evm, alice, kitchen_sink_json):
    create_account(evm, alice, ether_to_wei(2))
    a = contract_from_raw_abi(evm, kitchen_sink_json)

    # fail on value with a non-payable constructor
    with pytest.raises(BaseException):
        a.deploy(caller=alice, value=1)

    assert a.deploy(caller=alice)

    assert 1 == a.increment.transact(caller=alice)
    assert [2, 3] == a.increment.transact(2, caller=alice)
    assert 4 == a.increment.simulate(caller=alice)
    assert 3 == a.value.call()
    assert alice == a.setInput.transact((1, 2, alice), caller=alice)

    # receive
    one_ether = ether_to_wei(1)
    assert 0 == evm.get_balance(a.address)
    evm.transfer(alice, a.address, one_ether)
    assert one_ether == evm.get_balance(alice)
    assert one_ether == evm.get_balance(a.address)
