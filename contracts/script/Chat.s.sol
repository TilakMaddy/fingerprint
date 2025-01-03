// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Chat} from "../src/Chat.sol";

contract CounterScript is Script {
    Chat public chat;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        for (uint256 i = 0; i < 20; ++i) {
            chat = new Chat();
            console.log(address(chat));
        }

        vm.stopBroadcast();
    }
}
