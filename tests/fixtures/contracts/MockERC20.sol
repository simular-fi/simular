// SPDX-License-Identifier: UNLICENSED
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
