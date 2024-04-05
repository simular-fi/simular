.. _contract:

Contract
========

.. contents:: :local:
    

.. py:module:: simular.contract


Provides a wrapper around the core Rust libraries implementing the EVM and ABI parser. 
Solidity contract methods are extracted from the ABI and made available as attributes 
on the instance of the ``Contract``. 

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

.. py:class:: simular.Contract(*args, evm: PyEvm)

    Represents a contract and all the functions defined in the Solidity Contract. All calls 
    are translated into Ethereum transactions and sent to the EVM.

    .. note::

        The preferred way to create a contract is to use one of the ``contract_*`` 
        functions in :ref:`utils`


    * ``evm`` (`PyEvm`) is an instance of `PyEvm`
    * ``args`` are whatever

    `Returns`: nothing right now.


.. code-block:: python

    >>> evm = PyEvm()
    >>> contract = Contract(evm)


Methods
--------

.. py:staticmethod:: from_snapshot(snap: str)

    Create a new EVM from a snapshot

    :param str snap:  this is the arg
    :param str another:  this is the arg
    :raises Exception: thrown on bad address
    :returns the values:
   
    load it...

.. py:method:: at(address: str)

    Set the contract address
