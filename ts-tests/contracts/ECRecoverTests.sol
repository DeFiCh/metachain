// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

contract ECRecoverTests {
    function recover(bytes memory input) public returns (bytes memory) {
        address ecrecoverAddress = address(
            0x0000000000000000000000000000000000000001
        );
        (bool success, bytes memory returnData) = ecrecoverAddress.call(input);
        return returnData;
    }
}
