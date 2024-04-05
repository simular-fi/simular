.. _getstarted:

Getting Started
===============

Simular is available on `PyPi <https://pypi.org/project/simular-evm/>`_
It requires Python ``>=3.11``.

.. code-block:: bash

    >>> pip install simular-evm


Here are a few examples of how to use simular. You can find more details 
in the API section.


Transfer Ether
--------------

In this example, we'll create 2 Ethereum accounts and show how to 
transfer Ether between the accounts.

.. code-block:: python
    
    >>> from simular inport PyEvm, ether_to_wei, create_account

Import the EVM and a few utility function. Next create an instance of the EVM.

    >>> evm = PyEvm()

Next, create a couple of accounts.  One for Bob with and initial balance of 1 ether,
and one for Alice with no balance.

    >>> bob = create_account(evm, value=ether_to_wei(1))
    >>> alice = create_account(evm)


