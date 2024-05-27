.. _getstarted:

Getting Started
===============

Install 
-------

Simular is available on `PyPi <https://pypi.org/project/simular-evm/>`_
It requires Python ``>=3.11``.

.. code-block:: bash

    >>> pip install simular-evm

Here are a few examples of how to use simular.

Transfer Ether
--------------

Create 2 Ethereum accounts and transfer Ether between the accounts.

.. code-block:: python

    # import the EVM and a few utility function
    >>> from simular inport PyEvm, ether_to_wei, create_account

    # create an instance of the Evm
    >>> evm = PyEvm()

    # convert 1 ether to wei (1e18)
    >>> one_ether = ether_to_wei(1)

    # create a couple of accounts - one for Bob with and initial 
    # balance of 1 ether and one for Alice with no balance.
    >>> bob = create_account(evm, value=one_ether)
    >>> alice = create_account(evm)

    # Bob transfers 1 ether to alice
    >>> evm.transfer(bob, alice, one_ether)

    # check balances
    >>> assert int(1e18) == evm.get_balance(alice)
    >>> assert 0 == evm.get_balance(bob)


Deploy and interact with a contract
-----------------------------------

Load an ERC20 contract and mint tokens.

.. code-block:: python
    
    # import the EVM and a few utility functions
    from simular inport PyEvm, create_account, contract_from_abi_bytecode

    def deploy_and_mint():

        # Create an instance of the EVM
        evm = PyEvm()

        # Create accounts
        alice = create_account(evm)
        bob = create_account(evm)

        # Load the contract.
        # ABI and BYTECODE are the str versions of the ERC20 contract 
        # interface and compiled bytecode
        erc20 = contract_from_abi_bytecode(evm, ABI, BYTECODE)

        # Deploy the contract. Returns the contract's address
        # The contract's constructor takes 3 arguments:
        # name: MyCoin
        # symbol: MYC
        # decimals: 6
        # 
        # 'caller' (bob) is the one deploying the contract. This 
        # translates to 'msg.sender'. And in the case of this contract, 
        # bob will be the 'owner' 
        contract_address = erc20.deploy("MyCoin", "MYC", 6, caller=bob)
        print(contract_address)


        # Let's check to see if it worked...
        # Notice how the contract functions are added as attributes
        # to the contract object.   
        #
        # We use 'call' to make a read-only request
        assert erc20.name.call() == "MyCoin"
        assert erc20.decimals.call() == 6
        assert erc20.owner.call() == bob

        # Bob mints 10 tokens to alice.
        # Again, 'mint' is a contract function. It's 
        # automatically attached to the erc20 contract
        # object as an attribute.
        # 'transact' is a write call to the contract (it will change state).
        erc20.mint.transact(alice, 10, caller=bob)

        # check balances and supply
        assert 10 == erc20.balanceOf.call(alice)
        assert 10 == erc20.totalSupply.call()

        # Let's take a snapshot of the state of the EVM
        # and use it again later to pre-populate the EVM:
        snapshot = evm.create_snapshot()

        # and save it to a file
        with open('erc20snap.json', 'w') as f:
            f.write(snapshot)

        
        # ... later on, we can load this back into the EVM
        with open('erc20snap.json') as f: 
            snapback = f.read()

        # a new instance of the EVM
        evm2 = PyEvm.from_snapshot(snapback)

        # load the contract definitions
        erc20back = contract_from_abi_bytecode(evm2, erc20abi, erc20bin)

        # check the state was preserved in the snapshot
        assert 10 == erc20back.balanceOf.call(alice)
