#![cfg(test)]

use super::*;
use fp_evm::{ExitReason, ExitSucceed};
use mock::{WeightPerGas, *};
use sp_core::{bytes::from_hex, U256};
use std::str::FromStr;

#[test]
fn should_create_contract() {
	// pragma solidity ^0.5.0;
	//
	// contract Test {
	//	 function multiply(uint a, uint b) public pure returns(uint) {
	// 	 	return a * b;
	// 	 }
	// }
	let contract = from_hex(
        "0x608060405234801561001057600080fd5b5060b88061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063165c4a1614602d575b600080fd5b606060048036036040811015604157600080fd5b8101908080359060200190929190803590602001909291905050506076565b6040518082815260200191505060405180910390f35b600081830290509291505056fea265627a7a723158201f3db7301354b88b310868daf4395a6ab6cd42d16b1d8e68cdf4fdd9d34fffbf64736f6c63430005110032"
	).unwrap();

	ExtBuilder::default().build().execute_with(|| {
        assert_eq!(crate::mock::Balances::free_balance(alice()), 1000000000000000);

        let caller = alice();

        let result = <Test as pallet_evm::Config>::Runner::create(
            caller,
            contract,
            U256::from("0"),
            1000000,
            Some(U256::from("1")),
            None,
            Some(U256::from("1")),
            vec![],
            true,
            true,
            <Test as pallet_evm::Config>::config()
        ).unwrap();
        assert_eq!(result.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));

        let contract_address = result.value;
        assert_eq!(contract_address, H160::from_str("5f8bd49cd9f0cb2bd5bb9d4320dfe9b61023249d").unwrap());

        // multiply(2, 3)
		let multiply = from_hex(
            "0x165c4a1600000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
		).unwrap();

        // call method `multiply`
        let result = <Test as pallet_evm::Config>::Runner::call(
            alice(),
            contract_address,
            multiply,
            U256::from("0"),
            1000000,
            Some(U256::from("1000000")),
            None,
            None,
            vec![],
            true,
            true,
            <Test as pallet_evm::Config>::config(),
        ).unwrap();
        assert_eq!(U256::from(result.value.as_slice()), 6.into());
    });
}

#[test]
fn configured_base_extrinsic_weight_is_evm_compatible() {
	let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
	let base_extrinsic = <Test as frame_system::Config>::BlockWeights::get()
		.get(frame_support::dispatch::DispatchClass::Normal)
		.base_extrinsic;
	assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
}
