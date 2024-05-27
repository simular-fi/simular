.. _evm:

EVM
===

.. contents:: :local:
    

.. py:module:: simular.PyEvm

A Python wrapper around an Embedded `Rust based Ethereum Virtual Machine <https://github.com/bluealloy/revm>`_.

Constructor
-----------

.. py:class:: PyEvm()

    Create and return an instance of the EVM that uses an ``in-memory`` database.

Example: 

.. code-block:: python

    >>> from simular import PyEvm
    >>> evm = PyEvm()


Methods
-------

.. py:staticmethod:: PyEvm.from_fork(url: str, blocknumber: int=None)

    Create and return an instance of the EVM that will pull state from a remote
    Ethereum node.

    :param url: the url (``https://...``) to a remote Ethereum node with JSON-RPC support
    :param blocknumber: (optional) the specific blocknumber to pull state at.  If ``None``, the latest block will be used. 
    :return: an instance of the EVM

Example:

.. code-block:: python

    >>> from simular import PyEvm
    >>> evm = PyEvm.from_fork('http://...', blocknumber=195653)


.. py:staticmethod:: PyEvm.from_snapshot(snapshot: str)

    Create and return an instance of the EVM from a previously created ``snapshot``.  
    See ``create_snapshot`` below.

    :param snapshot: a (str) serialized snapshot
    :return: an instance of the EVM

Example:

Assume ``hello_snap.json`` is a file from a saved snapshot.

.. code-block:: python

    >>> from simular import PyEvm

    # load the json file 
    >>> with open('hello_snap.json') as f:
    ...     snap = f.read()
    >>> evm = PyEvm.from_snapshot(snap)


.. py:method:: create_snapshot()

    Create a JSON formatted snapshot of the current state of the EVM.

    :return: (str) the serialized state

Example:

.. code-block:: python
    
    >>> evm = PyEvm()

    # do stuff with the EVM 

    >>> snap = evm.create_snapshot()

.. py:method:: create_account(address: str, balance = None)

    Create an account

    .. note:: 
        See ``utils.create_account``

    :param address: (str) a valid, hex-encoded Ethereum address
    :param balance: (int) and optional balance in ``wei`` to set for the account. If None, balance = 0


.. py:method:: get_balance(address: str)

    Get the balance of the given address 

    :param address: (str) a valid, hex-encoded Ethereum address
    :return: the balance in ``wei``


.. py:method:: transfer(caller: str, to: str, amount: int)

    Transfer ``amount`` in ``wei`` from ``caller -> to``

    :param caller: (str) a valid, hex-encoded Ethereum address
    :param to: (str) a valid, hex-encoded Ethereum address
    :param amount: (int)  the amount to transfer

    .. warning::
        This will fail if the ``caller`` does not have a sufficient balance to transfer

Example:

.. code-block:: python

    >>> from similar import PyEvm, create_account

    >>> evm = PyEvm()

    # create an account for Bob with 1 Ether
    >>> bob = create_account(evm, value=int(1e18))

    # create an account for Alice with no balance 
    >>> alice = create_account(evm)

    # transfer 1 Ether from Bob to Alice 
    >>> evm.transfer(bob, alice, int(1e18))

    # check Alice's balance 
    >>> evm.get_balance(alice)
    1000000000000000000


.. py:method:: advance_block(interval = None)

    This method provides the ability to simulate the mining of blocks. It will advance 
    `block.number` and `block.timestamp`.  
    
    It's not necessary to call this method. However, some contracts may have logic 
    that need this information.

    :param interval: (int) optional. set the time in seconds between blocks. Default is 12 seconds

Example:

.. code-block:: python

    >>> evm.advance_block()


