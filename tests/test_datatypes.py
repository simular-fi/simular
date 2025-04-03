from simular import (
    contract_from_raw_abi,
    create_account,
)


def test_signed_ints(evm, alice, signed_ints_json):
    create_account(evm, alice)
    test_contract = contract_from_raw_abi(evm, signed_ints_json)
    assert test_contract.deploy(caller=alice)

    # values
    cases = [
        (-128, 127),
        (-549755813888, 549755813887),
        (-2361183241434822606848, 2361183241434822606847),
        (
            -43556142965880123323311949751266331066368,
            43556142965880123323311949751266331066367,
        ),
        (
            -3138550867693340381917894711603833208051177722232017256448,
            3138550867693340381917894711603833208051177722232017256447,
        ),
        (
            -57896044618658097711785492504343953926634992332820282019728792003956564819968,
            57896044618658097711785492504343953926634992332820282019728792003956564819967,
        ),
    ]

    for n, p in cases:
        assert n == test_contract.in_and_out.call(n)
        assert p == test_contract.in_and_out.call(p)
