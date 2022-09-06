// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

contract Test {
    string public name = 'Meta';
    address public owner;
    uint256 count = 0;

    event echo(string message);

    constructor() {
        owner = msg.sender;
        emit echo('Hello, Meta');
    }

    modifier onlyOwner() {
        require(msg.sender == owner); // validate whether caller is the address of owner
        _; // if true continue process
    }

    function mul(uint256 a, uint256 b) public pure returns (uint256) {
        return a * b;
    }

    function max10(uint256 a) public pure returns (uint256) {
        if (a > 10) revert('Value must not be greater than 10.');
        return a;
    }

    function incr() public {
        count += 1;
    }

    function getCount() public view returns (uint256) {
        return count;
    }

    function setCount(uint256 _count) public onlyOwner {
        count = _count;
    }

    // environmental with global vars
    // https://docs.soliditylang.org/en/v0.8.16/units-and-global-variables.html
    function getBlockHash(uint256 number) public view returns (bytes32) {
        return blockhash(number);
    }

    function getCurrentBlock() public view returns (uint256) {
        return block.number;
    }

    function getGasLimit() public view returns (uint256) {
        return block.gaslimit;
    }
}
