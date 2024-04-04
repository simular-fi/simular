import pytest
from pathlib import Path

from simular import PyEvm

PATH = Path(__file__).parent


@pytest.fixture
def bob():
    return "0xed6ff00ae6a64df0bf28e159c4a48311b931f458"


@pytest.fixture
def alice():
    return "0x0091410228bf6062ab28c949ba4172ee9144bfde"


@pytest.fixture
def evm():
    return PyEvm()


@pytest.fixture
def erc20abi():
    with open(f"{PATH}/./fixtures/erc20.abi") as f:
        ercabi = f.read()
    return ercabi


@pytest.fixture
def erc20bin():
    with open(f"{PATH}/./fixtures/erc20.bin") as f:
        ercbin = f.read()
    bits = bytes.fromhex(ercbin)
    return bits


@pytest.fixture
def kitchen_sink_json():
    with open(f"{PATH}/./fixtures/KitchenSink.json") as f:
        rawabi = f.read()
    return rawabi
