.. _utils:

Utilities
=========

.. contents:: :local:
    
.. py:module:: simular.utils

Provides several ``helper`` functions to simplify working with ``accounts`` and ``Contract``

Functions
---------

.. py:function:: ether_to_wei(value: int)

    Convert Ether to Wei 

    :params value: the given value in Ether to convert 
    :return: the amount in Wei

.. py:function:: generate_random_address()

    Randomly generate an Ethereum address 

    :return: (str) the hex-encoded address

.. py:function:: create_account(evm: PyEvm, address: str = None,  value: int = 0)

    Create a new account in the Evm 

    :param evm: an instance of PyEvm
    :param address: (optional) a valid, hex-encoded Ethereum address
    :param balance: (optional) balance in ``wei`` to set for the account.
    :return: the hex-encoded address 

.. note::
    * If no address is provided, a random address will be created
    * If no value is provided, the account balance will be set to 0


.. py:function:: create_many_accounts(evm: PyEvm, num: int, value: int = 0)

    Just like ``create_account`` except it can create many accounts at once.

    :param evm: an instance of PyEvm
    :param num: the number of accounts to create 
    :param value: (optional) the initial balance in `wei` for each account
    :return: a list of account addresses


.. py:function:: contract_from_raw_abi(evm: PyEvm, raw_abi: str)

    Create an instance of ``Contract`` given the full ABI. A full ABI should include
    the `abi` and `bytecode`. This is usually a single json file from a compiled Solidity contract.

    :param evm: an instance of PyEvm
    :param raw_abi: the full abi contents
    :return: an instance of ``Contract`` based on the ABI and contract creation bytecode

.. note::
    Don't forget to ``deploy`` the contract to make it available in the EVM


.. py:function:: contract_from_abi_bytecode(evm: PyEvm, raw_abi: str, bytecode: bytes = None)

    Create an instance of ``Contract`` from the ABI and (optionally) the contract creation
    bytecode.  This is often used when you have the ABI and bytecode are not in the same file OR 
    when you just want to create a ``Contract`` using just the ABI to interact with an already
    deployed contract.

     :param evm: an instance of PyEvm
     :param raw_abi: the full abi contents
     :param bytecode: (optional) contract creation code

     :return: an instance of ``Contract``

.. note::
    Don't forget to ``deploy`` the contract to make it available in the EVM


.. py:function:: contract_from_inline_abi(evm: PyEvm, abi: typing.List[str])

    Create an instance of ``Contract`` from a user-friendly form of the ABI This is used 
    to interact with an already deployed contract.  See `Human-Friendly ABI <https://docs.ethers.org/v5/api/utils/abi/formats/#abi-formats--human-readable-abi>`_

    :param evm: an instance of PyEvm
    :param abi: a list of (str) describing the contract's functions
    :param bytecode: (optional) contract creation code

     :return: an instance of ``Contract``

.. warning::
    The contract must already be deployed. You will need to use ``Contract.at()`` to 
    set the address of the contract.

Example:

.. code-block:: python

    >>> evm = PyEvm()

    # specifies a single contract function 'hello' 
    # that takes no arguments and returns a number
    >>> abi = ["function hello()(uint256)"]

    >>> contract = contract_from_inline_abi(abi)
    >>> contract.at('deployed contracts address')

    # call it 
    >>> value = contract.hello.call()

