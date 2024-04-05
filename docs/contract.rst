.. _contract:

Contract
========

.. contents:: :local:
    

.. py:module:: simular.contract


Provides a wrapper around the core Rust libraries implementing the EVM and ABI parser. 
Solidity contract methods are extracted from the ABI and made available as attributes 
on the instance of a ``Contract``. 

For example, given the following Solidity contract:

.. code-block:: javascript

    contract HelloWorld {
        address public owner;
        uint256 public value;

        constructor() {
            owner = msg.sender
        }

        function addNum(uint256 num) public {
            value += num;
        }
    }

``Contract`` will parse the ABI and make all the functions available as attributes 
using Python's ``__getattr__``.  To execute the function you can use one of the following:

.. code-block:: python

    # Send a write transaction to the contract. This will change state in the EVM. 
    transact(*args, caller: str = None, value: int = 0)

    # Send a read transaction to the contract. This will NOT change state
    call(*args)

    # Like transact but it will NOT change state.
    simulate(*args, caller: str = None, value: int = 0)


Example...

.. code-block:: python

    >>> contract = Contract(evm, abi)

    # Return the value of 'owner' in the contract
    >>> contract.owner.call()

    # Add 3 to the contract's 'value'
    >>> contract.addNum.transact(3, caller=bob)

Format:

``contract.attributename.[call(...), transact(...), simulate(...)]``

Properties
----------

.. py:attribute:: evm

    The instance of the embedded EVM

.. py:attribute:: abi 

    An instance of the ABI parser

.. py:attribute:: address

    The address of the contract.  This is available for a deployed contract


Constructor
-----------

.. py:class:: Contract(evm: PyEvm, abi: PyAbi)

    Represents a contract and all the functions defined in the Solidity Contract. All calls 
    are translated into Ethereum transactions and sent to the EVM.

    .. note::

        The preferred way to create a contract is to use one of the ``contract_*`` 
        functions in :ref:`utils`

    :param evm: an instance of the EVM
    :type evm: PyEvm
    :param abi: an instance of the Abi parser 
    :type abi: PyAbi
    :return: an instance of the contract 

.. code-block:: python

    >>> evm = PyEvm()
    >>> contract = Contract(evm)


Methods
--------

.. py:method:: at(address: str)

    Set the address for the contract.  This is automatically done when using ``deploy``

    :param address: the address of the deployed contract


.. py:method:: deploy(*args, caller: str = None, value: int = 0)

    Deploy a contract to the EVM. Under the covers, it uses the ABI to encode 
    the constructor call to make a transaction.

    :param args: 0 or more arguments expected by the Contract's constructor 
    :param caller: the address making the deploy. this is `msg.sender`
    :param value: (optional) amount of `wei` to send to the contract. This will fail if the contracts constructor is not mark as ``payable``
    :return: the address of the deployed contract 
    :raises Exception: If ``caller`` is not provided OR ``caller`` is not a valid address

Example:

Assume the ``HelloWorld.json`` is the compiled Solidy ABI

.. code-block:: python

    # imports
    >>> from simular import PyEvm, contract_from_raw_abi, create_account

    # load the json file 
    >>> with open('HelloWorld.json') as f:
    ...     abi = f.read()

    # create an instance of the EVM
    >>> evm = PyEvm()

    # create an account to deploy the contract
    >>> bob = create_account()

    # create an instance of the contract from the abi
    >>> contract = contract_from_raw_abi(abi)

    # deploy the contract, returning it's address
    >>> contract.deploy(caller=bob)
    '0x0091410228bf6062ab28c949ba4172ee9144bfde'

.. py:method:: transact(*args, caller: str = None, value: int = 0)

    Execute a write transaction to the contract. This will change the state of the contract

    .. note:: 
        Remember this is method is appended to the end of the Solidity contract's function name:
        ``contract.[attribute name].transact(...)``

    :param args: 0 or more arguments expected by the Contract's function
    :param caller: (required) the address making the call. this is `msg.sender`
    :param value: (optional) amount of `wei` to send to the contract. This will fail if the contracts function is not mark as ``payable``
    :return: the result of the function call (if any)
    :raises Exception: If ``caller`` is not provided OR ``caller`` is not a valid address

.. py:method:: call(*args)

    Execute a read transaction to the contract. This will NOT change the state of the contract

    .. note:: 
        Remember this is method is appended to the end of the Solidity contract's function name:
        ``contract.[attribute name].call(...)``

    :param args: 0 or more arguments expected by the Contract's function
    :return: the result of the function call (if any)
    :raises Exception: If the contract does not have an address


.. py:method:: simulate(*args, caller: str = None, value: int = 0)

    Just like ``transact``. Except it will NOT change the state of the contract.  Can be
    used to test a ``transact``.

    .. note:: 
        Remember this is method is appended to the end of the Solidity contract's function name:
        ``contract.[attribute name].simulate(...)``

    :param args: 0 or more arguments expected by the Contract's function
    :param caller: (required) the address making the call. this is `msg.sender`
    :param value: (optional) amount of `wei` to send to the contract. This will fail if the contracts function is not mark as ``payable``
    :return: the result of the function call (if any)
    :raises Exception: If ``caller`` is not provided OR ``caller`` is not a valid address