from simular import PyAbi


def test_load_from_parts(erc20abi, erc20bin):
    abi = PyAbi.from_abi_bytecode(erc20abi, erc20bin)
    assert abi.has_function("mint")
    assert abi.has_function("burn")
    assert not abi.has_fallback()
    assert not abi.has_receive()

    addy = "0xa32f31673577f7a717716d8b88d85a9e7bbb76d3"
    (sig1, _, _) = abi.encode_function("mint", f"({addy}, 2)")
    hexed = bytes.hex(bytes(sig1))
    og = "40c10f19000000000000000000000000a32f31673577f7a717716d8b88d85a9e7bbb76d30000000000000000000000000000000000000000000000000000000000000002"
    assert og == hexed

    (sig2, _, _) = abi.encode_function("name", "()")
    hexed2 = bytes.hex(bytes(sig2))
    assert "06fdde03" == hexed2


def test_load_from_human_readable():
    funcs = ["function mint(address, uint256)", "function name()(string)"]
    abi = PyAbi.from_human_readable(funcs)
    assert abi.has_function("mint")
    assert abi.has_function("name")


def test_load_full_json_abi(kitchen_sink_json):
    abi = PyAbi.from_full_json(kitchen_sink_json)
    assert abi.has_function("increment")
    assert abi.has_function("setInput")
    assert abi.has_receive()
    assert (abi.bytecode(), False) == abi.encode_constructor("()")

    with_struct = "fa6a38d200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000a32f31673577f7a717716d8b88d85a9e7bbb76d3"
    (sig3, _, _) = abi.encode_function(
        "setInput", "((2, 3, 0xa32f31673577f7a717716d8b88d85a9e7bbb76d3))"
    )
    hexed3 = bytes.hex(bytes(sig3))
    assert with_struct == hexed3
