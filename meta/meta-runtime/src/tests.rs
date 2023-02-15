#![cfg(test)]

use super::*;
use fp_evm::{ExitReason, ExitSucceed};
use mock::{WeightPerGas, *};
use sp_core::{bytes::from_hex, U256};
use std::str::FromStr;

#[test]
fn should_create_and_call_contract() {
	// pragma solidity ^0.8.17;
	//
	// contract Test {
	//	 function multiply(uint a, uint b) public pure returns(uint) {
	// 	 	return a * b;
	// 	 }
	// }
	let contract = from_hex(
        "0x608060405234801561001057600080fd5b506101c2806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063165c4a1614610030575b600080fd5b61004a600480360381019061004591906100b1565b610060565b6040516100579190610100565b60405180910390f35b6000818361006e919061014a565b905092915050565b600080fd5b6000819050919050565b61008e8161007b565b811461009957600080fd5b50565b6000813590506100ab81610085565b92915050565b600080604083850312156100c8576100c7610076565b5b60006100d68582860161009c565b92505060206100e78582860161009c565b9150509250929050565b6100fa8161007b565b82525050565b600060208201905061011560008301846100f1565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b60006101558261007b565b91506101608361007b565b925082820261016e8161007b565b915082820484148315176101855761018461011b565b5b509291505056fea2646970667358221220163d162157001b2d44ca3fe09b3fb1d484798c1b8c0eaa88fbd6c094029c674864736f6c63430008110033"
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
