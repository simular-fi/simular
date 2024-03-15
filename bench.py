import time
from simular import PyEvm, Contract, create_account


def how_fast_with_no_overloads():
    with open("./tests/fixtures/counter.json") as f:
        counterabi = f.read()

    NUM_TX = 15000
    client = PyEvm()
    deployer = create_account(client, value=2)

    counter = Contract(client, counterabi)
    counter.deploy(deployer)

    start_time = time.perf_counter()

    for _ in range(0, NUM_TX):
        counter.increment.transact(caller=deployer)

    end_time = time.perf_counter()
    total_time = end_time - start_time

    val = counter.number.call()
    assert NUM_TX == val
    print(f"time: {total_time:.6f} second(s) for {NUM_TX} transactions")


if __name__ == "__main__":
    how_fast_with_no_overloads()
