import time
from pathlib import Path

from simular import PyEvm, create_account, contract_from_raw_abi, ether_to_wei

PATH = Path(__file__).parent
NUM_TX = 150_000


def how_fast():
    with open(f"{PATH}/../tests/fixtures/KitchenSink.json") as f:
        abi = f.read()

    client = PyEvm()
    deployer = create_account(client, value=ether_to_wei(2))

    counter = contract_from_raw_abi(client, abi)
    counter.deploy(caller=deployer)

    start_time = time.perf_counter()

    for _ in range(0, NUM_TX):
        counter.increment.transact(caller=deployer)

    end_time = time.perf_counter()
    total_time = end_time - start_time

    val = counter.value.call()
    assert NUM_TX == val
    print(f"{NUM_TX} transactions in : {total_time:.6f} second(s)")


if __name__ == "__main__":
    how_fast()
