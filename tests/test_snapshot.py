from simular import contract_from_inline_abi

FN_SIGS = [
    "function masterMinter() (address)",
    "function isMinter(address) (bool)",
    "function minterAllowance(address) (uint256)",
    "function totalSupply() (uint256)",
    "function mint(address, uint256) (bool)",
    "function burn(uint256)",
]

USDC_CONTRACT = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
MM = "0xe982615d461dd5cd06575bbea87624fda4e3de17"
M1 = "0x7b03fa6dba062b143b27861b7181b4f8c9e4476b"
M2 = "0xdc5957ee508cfa6056316b34184926dcecfde58f"
M3 = "0x2864eef30c5a0e35793df921efa5ee85dd3d2e65"

ALLOWANCE = 10000000


def test_usdc_cached(evm, usdc_cache):
    evm.load_state(usdc_cache)

    usdc = contract_from_inline_abi(evm, FN_SIGS)
    usdc.at(USDC_CONTRACT)

    assert usdc.isMinter.call(M1)
    assert ALLOWANCE == usdc.minterAllowance.call(M1)

    assert usdc.mint.transact(M1, 5000000, caller=M1)
    assert 5000000 == usdc.totalSupply.call()
