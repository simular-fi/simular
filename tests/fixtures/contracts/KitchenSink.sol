// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.13;

struct InputStruct {
    uint256 x;
    uint256 y;
    address user;
}

contract KitchenSink {
    uint256 public value;
    uint256 public x;
    uint256 public y;
    address public user;

    // increment by 1
    function increment() public returns (uint256) {
        value += 1;
        return value;
    }

    // increment by 'input'
    function increment(uint256 input) public returns (uint256, uint256) {
        value += input;
        return (input, value);
    }

    // set values by input struct
    function setInput(InputStruct calldata input) public returns (address) {
        x = input.x;
        y = input.y;
        user = input.user;
        return user;
    }

    receive() external payable {}
}
