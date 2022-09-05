// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

contract Test {
    string public name = 'Meta';

    function mul(uint256 a, uint256 b) public pure returns (uint256) {
        return a * b;
    }

    function max10(uint256 a) public pure returns (uint256) {
        if (a > 10) revert('Value must not be greater than 10.');
        return a;
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

    function getMsgSender() public view returns (address) {
        return msg.sender;
    }
}
