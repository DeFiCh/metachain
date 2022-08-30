// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

contract Test {
    function multiply(uint256 a) public pure returns (uint256 d) {
        return a * 7;
    }

    function gasLimit() public view returns (uint256) {
        return block.gaslimit;
    }

    function currentBlock() public view returns (uint256) {
        return block.number;
    }

    function blockHash(uint256 number) public view returns (bytes32) {
        return blockhash(number);
    }
}
