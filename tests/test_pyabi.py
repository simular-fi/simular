from simular import PyAbi


def test_load_from_parts(erc20abi, erc20bin):
    abi = PyAbi.load_from_parts(erc20abi, erc20bin)
    assert abi.has_function("mint")
    assert abi.has_function("burn")
    assert not abi.has_fallback()
    assert not abi.has_receive()

    assert ["string", "string", "uint8"] == abi.constructor_input_types()

    addy = "0xa32f31673577f7a717716d8b88d85a9e7bbb76d3"
    (sig1, output) = abi.encode_function_input("mint", (addy, 2))
    hexed = bytes.hex(bytes(sig1))
    og = "40c10f19000000000000000000000000a32f31673577f7a717716d8b88d85a9e7bbb76d30000000000000000000000000000000000000000000000000000000000000002"
    assert og == hexed
    assert output == []

    (sig2, output2) = abi.encode_function_input("name", ())
    hexed2 = bytes.hex(bytes(sig2))
    assert "06fdde03" == hexed2
    assert output2 == ["string"]

    assert len(abi.bytecode()) > 100


def test_load_from_human_readable():
    funcs = ["function mint(address, uint256)", "function name()(string)"]
    abi = PyAbi.load_from_human_readable(funcs)
    assert abi.has_function("mint")
    assert abi.has_function("name")

    addy = "0xa32f31673577f7a717716d8b88d85a9e7bbb76d3"
    (sig1, output) = abi.encode_function_input("mint", (addy, 2))
    hexed = bytes.hex(bytes(sig1))
    og = "40c10f19000000000000000000000000a32f31673577f7a717716d8b88d85a9e7bbb76d30000000000000000000000000000000000000000000000000000000000000002"
    assert og == hexed
    assert output == []

    (sig2, output2) = abi.encode_function_input("name", ())
    hexed2 = bytes.hex(bytes(sig2))
    assert "06fdde03" == hexed2
    assert output2 == ["string"]

    assert None == abi.bytecode()


def test_load_full_json_abi(kitchen_sink_json):
    abi = PyAbi.load_from_json(kitchen_sink_json)
    assert abi.has_function("increment")
    assert abi.has_function("setInput")
    assert abi.has_receive()
    assert not abi.constructor_input_types()

    inc_no_args = "d09de08a"
    (sig1, output1) = abi.encode_function_input("increment", ())
    hexed1 = bytes.hex(bytes(sig1))
    assert hexed1 == inc_no_args
    assert ["uint256"] == output1

    inc_with_args = (
        "7cf5dab00000000000000000000000000000000000000000000000000000000000000002"
    )
    (sig2, output2) = abi.encode_function_input("increment", (2,))
    hexed2 = bytes.hex(bytes(sig2))
    assert hexed2 == inc_with_args
    assert ["uint256", "uint256"] == output2

    with_struct = "fa6a38d200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000a32f31673577f7a717716d8b88d85a9e7bbb76d3"
    (sig3, output3) = abi.encode_function_input(
        "setInput", (2, 3, "0xa32f31673577f7a717716d8b88d85a9e7bbb76d3")
    )
    hexed3 = bytes.hex(bytes(sig3))
    assert with_struct == hexed3
    assert ["address"] == output3
