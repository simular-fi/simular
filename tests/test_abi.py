import pytest
from eth_utils import is_address, to_wei, is_address

from simular import PyEvm, create_many_accounts, create_account, Contract


def test_contract_deploy_no_args():
    with open("./tests/fixtures/counter.json") as f:
        counterabi = f.read()

    client = PyEvm()
    deployer = create_account(client, value=2)

    counter = Contract(client, counterabi)
    address = counter.deploy(deployer)

    assert is_address(address)
    assert address == counter.address

    val = counter.number.call()
    assert val == 0

    with pytest.raises(BaseException):
        # constructor doesn't accept args
        counter.deploy(deployer, args=(1, 2))


def test_simple_read_write_contract():
    with open("./tests/fixtures/counter.json") as f:
        counterabi = f.read()

    client = PyEvm()
    actors = create_many_accounts(client, 2, 2)
    deployer = actors[0]
    bob = actors[1]

    counter = Contract(client, counterabi)
    counter.deploy(deployer)

    # make 99 calls to the number
    for i in range(1, 100):
        counter.setNumber.transact(i, caller=bob)

    b = counter.number.call()
    assert b == 99

    with pytest.raises(BaseException):
        # can't call functions that don't exist
        counter.nope.call()


def test_contract_deploy_with_args():
    with open("./tests/fixtures/erc20.json") as f:
        ercabi = f.read()

    client = PyEvm()
    deployer = create_account(client, value=2)

    erc = Contract(client, ercabi)
    erc.deploy(deployer, args=("hello", "H", 6))

    assert is_address(erc.address)

    with pytest.raises(BaseException):
        # missing args
        erc.deploy(deployer)

    name = erc.name.call()
    assert name == "hello"

    sym = erc.symbol.call()
    assert sym == "H"
