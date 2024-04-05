.. _api:

API
===

.. py:module:: simular.contract

Provides a wrapper around PyEvm(./pyevm.md) and [PyAbi](./pyabi.md) and is the easiest 
way to interact with Contracts.  Solidity contract methods are extracted from the ABI 
and made available as attributes on the instance of the Contract. 

For example, if a Solidity contract defines the following methods:

.. code-block:: javascript

    function hello(address caller) public returns (bool){}
    function world(string name) view (string) 


.. code-block:: none

    pragma solidity ^0.8.13;

    import {ERC20} from "solmate/tokens/ERC20.sol";

    contract MockERC20 is ERC20 {
        address public owner;

        constructor(
            string memory _name,
            string memory _symbol,
            uint8 _decimals
        ) ERC20(_name, _symbol, _decimals) {
            owner = msg.sender;
        }

        function mint(address to, uint256 value) public virtual {
            require(msg.sender == owner, "not the owner");
            _mint(to, value);
        }

        function burn(address from, uint256 value) public virtual {
            require(msg.sender == owner, "not the owner");
            _burn(from, value);
        }
    }


they will be automatically available in the instance of the Contract like this:

This is an example. How does this look

.. note::

    This is a note


.. py:class:: simular.Contract(*args, evm: PyEvm)

    Main smart contract API 


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
