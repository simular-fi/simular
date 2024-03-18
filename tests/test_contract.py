import pytest

from simular import (
    Contract,
    contract_from_abi_bytecode,
    create_account,
    PyEvmLocal,
    contract_from_inline_abi,
)


def test_contract_interface(evm, bob, alice, erc20abi, erc20bin):
    create_account(evm, alice, 0)
    create_account(evm, bob, 0)

    erc20 = contract_from_abi_bytecode(evm, erc20abi, erc20bin)

    erc20.deploy(bob, ("USD Coin", "USDC", 6))
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

    state = evm.dump_state()

    evm2 = PyEvmLocal()
    evm2.load_state(state)

    erc20again = contract_from_inline_abi(evm2, ["function totalSupply() (uint256)"])
    erc20again.at(contract_address)
    assert 10 == erc20again.totalSupply.call()
