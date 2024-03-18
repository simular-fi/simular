from simular import PyEvmFork, PyEvm, contract_from_inline_abi

USDC_ADDRESS = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
PROV_URL = "https://eth-mainnet.g.alchemy.com/v2/ATkznwXpa9jClTcg2hLhZr_xEVqaY2Do"


def test_fork_with_usdc():
    evm = PyEvmFork(PROV_URL)
    usdc = contract_from_inline_abi(
        evm, ["function totalSupply() (uint256)", "function owner() (address)"]
    )
    # set the address
    usdc.at(USDC_ADDRESS)

    value = usdc.totalSupply.call()
    print(f"total USDC: {value/1e6}")
    assert value > 1000
    print("done")

    owner = usdc.owner.call()
    print(f"owner: {owner}")

    state = evm.dump_state()
    print(state)

    print("DONE....")

    # Reload state!
    evm2 = PyEvm()
    evm2.load_state(state)
    usdc2 = contract_from_inline_abi(
        evm2, ["function totalSupply() (uint256)", "function owner() (address)"]
    )
    usdc2.at(USDC_ADDRESS)
    value2 = usdc2.totalSupply.call()
    assert value == value2
    owner2 = usdc2.owner.call()
    assert owner == owner2
